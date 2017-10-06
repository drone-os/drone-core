use proc_macro::TokenStream;
use quote;
use std::{fmt, vec};
use syn;

struct Pool {
  size: u32,
  capacity: u32,
}

pub(crate) fn heap(input: TokenStream) -> TokenStream {
  let input = syn::parse_token_trees(&input.to_string()).unwrap();
  let mut input = input.into_iter();
  let mut attributes = Vec::new();
  let mut pools = Vec::new();
  let mut size = 0;
  while let Some(token) = input.next() {
    match token {
      syn::TokenTree::Token(token) => match token {
        syn::Token::DocComment(string) => parse_doc(&string, &mut attributes),
        syn::Token::Pound => parse_attr(&mut input, &mut attributes),
        syn::Token::Ident(ident) => if ident == "size" {
          parse_size(&mut input, &mut size);
        } else if ident == "pools" {
          parse_pools(&mut input, &mut pools);
        } else {
          panic!("Invalid ident: {}", ident);
        },
        token => panic!("Invalid root token: {:?}", token),
      },
      token => panic!("Invalid root token tree: {:?}", token),
    }
  }

  normalize_pools(&mut pools, size);
  let mut pool_start = Vec::new();
  let mut pool_size = Vec::new();
  let mut pool_capacity = Vec::new();
  let mut offset = 0;
  let pool_count = pools.len();
  for pool in &pools {
    pool_start.push(offset.as_usize_lit());
    pool_size.push(pool.size.as_usize_lit());
    pool_capacity.push(pool.capacity.as_usize_lit());
    offset += pool.length();
  }

  let output = quote! {
    use ::alloc::allocator::{Alloc, AllocErr, CannotReallocInPlace, Excess,
                             Layout};
    use ::core::slice::SliceIndex;
    use ::drone::heap::{Allocator, Pool};

    /// The heap allocator.
    pub struct Heap {
      pools: [Pool<u8>; #pool_count],
    }

    #(#attributes)*
    pub static mut ALLOC: Heap = Heap {
      pools: [
        #(
          Pool::new(#pool_start, #pool_size, #pool_capacity),
        )*
      ],
    };

    impl Allocator for Heap {
      const POOL_COUNT: usize = #pool_count;

      #[inline]
      unsafe fn get_pool_unchecked<I>(&self, index: I) -> &I::Output
      where
        I: SliceIndex<[Pool<u8>]>,
      {
        self.pools.get_unchecked(index)
      }

      #[inline]
      unsafe fn get_pool_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
      where
        I: SliceIndex<[Pool<u8>]>,
      {
        self.pools.get_unchecked_mut(index)
      }
    }

    unsafe impl<'a> Alloc for &'a Heap {
      unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        (**self).alloc(layout)
      }

      unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        (**self).dealloc(ptr, layout)
      }

      #[inline]
      fn usable_size(&self, layout: &Layout) -> (usize, usize) {
        unsafe { (**self).usable_size(layout) }
      }

      unsafe fn realloc(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
        new_layout: Layout
      ) -> Result<*mut u8, AllocErr> {
        (**self).realloc(ptr, layout, new_layout)
      }

      unsafe fn alloc_excess(
        &mut self,
        layout: Layout,
      ) -> Result<Excess, AllocErr> {
        (**self).alloc_excess(layout)
      }

      unsafe fn realloc_excess(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
        new_layout: Layout
      ) -> Result<Excess, AllocErr> {
        (**self).realloc_excess(ptr, layout, new_layout)
      }

      unsafe fn grow_in_place(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
        new_layout: Layout
      ) -> Result<(), CannotReallocInPlace> {
        (**self).grow_in_place(ptr, layout, new_layout)
      }

      unsafe fn shrink_in_place(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
        new_layout: Layout
      ) -> Result<(), CannotReallocInPlace> {
        (**self).shrink_in_place(ptr, layout, new_layout)
      }
    }

    /// Initializes the heap.
    ///
    /// # Safety
    ///
    /// * Must be called exactly once and before using the allocator.
    /// * `start` must have the word-size alignment.
    #[inline]
    pub unsafe fn heap_init(start: &mut usize) {
      ALLOC.init(start)
    }
  };
  output.parse().unwrap()
}

fn normalize_pools(pools: &mut Vec<Pool>, size: u32) {
  pools.sort_by_key(|pool| pool.size);
  let free = pools
    .iter()
    .map(Pool::length)
    .fold(size as i64, |a, e| a - e as i64);
  if free != 0 {
    panic!(
      "`pools` not matches `size`. Difference is {}. Consider setting `size` \
       to {}.",
      -free,
      size as i64 - free
    );
  }
}

fn parse_doc(input: &str, attributes: &mut Vec<quote::Tokens>) {
  let string = input.trim_left_matches("//!");
  attributes.push(quote!(#[doc = #string]));
}

fn parse_attr(
  input: &mut vec::IntoIter<syn::TokenTree>,
  attributes: &mut Vec<quote::Tokens>,
) {
  match input.next() {
    Some(syn::TokenTree::Token(syn::Token::Not)) => match input.next() {
      Some(syn::TokenTree::Delimited(delimited)) => {
        attributes.push(quote!(# #delimited))
      }
      token => panic!("Invalid tokens after `#!`: {:?}", token),
    },
    token => panic!("Invalid tokens after `#`: {:?}", token),
  }
}

fn parse_size(input: &mut vec::IntoIter<syn::TokenTree>, size: &mut u32) {
  match input.next() {
    Some(syn::TokenTree::Token(syn::Token::Eq)) => match input.next() {
      Some(
        syn::TokenTree::Token(
          syn::Token::Literal(syn::Lit::Int(int, syn::IntTy::Unsuffixed)),
        ),
      ) => {
        if int > u32::max_value() as u64 {
          panic!("Invalid size: {}", int);
        }
        *size = int as u32;
        match input.next() {
          Some(syn::TokenTree::Token(syn::Token::Semi)) => (),
          token => panic!("Invalid tokens after `size = {}`: {:?}", int, token),
        }
      }
      token => panic!("Invalid tokens after `size =`: {:?}", token),
    },
    token => panic!("Invalid tokens after `size`: {:?}", token),
  }
}

fn parse_pools(
  input: &mut vec::IntoIter<syn::TokenTree>,
  pools: &mut Vec<Pool>,
) {
  match input.next() {
    Some(syn::TokenTree::Token(syn::Token::Eq)) => (),
    token => panic!("Invalid tokens after `pools`: {:?}", token),
  }
  match input.next() {
    Some(
      syn::TokenTree::Delimited(syn::Delimited {
        delim: syn::DelimToken::Bracket,
        tts: pools_tokens,
      }),
    ) => {
      let mut pools_tokens = pools_tokens.into_iter();
      while let Some(token) = pools_tokens.next() {
        match token {
          syn::TokenTree::Delimited(syn::Delimited {
            delim: syn::DelimToken::Bracket,
            tts: pool_tokens,
          }) => {
            let mut pool_tokens = pool_tokens.into_iter();
            let size = match pool_tokens.next() {
              Some(
                syn::TokenTree::Token(
                  syn::Token::Literal(
                    syn::Lit::Int(size, syn::IntTy::Unsuffixed),
                  ),
                ),
              ) => size,
              token => {
                panic!("Invalid tokens after `pools = [... [`: {:?}", token)
              }
            };
            match pool_tokens.next() {
              Some(syn::TokenTree::Token(syn::Token::Semi)) => (),
              token => panic!(
                "Invalid tokens after `pools = [... [{}`: {:?}",
                size,
                token
              ),
            }
            let capacity = match pool_tokens.next() {
              Some(
                syn::TokenTree::Token(
                  syn::Token::Literal(
                    syn::Lit::Int(capacity, syn::IntTy::Unsuffixed),
                  ),
                ),
              ) => capacity,
              token => panic!(
                "Invalid tokens after `pools = [... [{};`: {:?}",
                size,
                token
              ),
            };
            match pool_tokens.next() {
              Some(token) => panic!(
                "Invalid tokens after `pools = [... [{}; {}`: {:?}",
                size,
                capacity,
                token
              ),
              None => (),
            }
            if size == 0 || size > u32::max_value() as u64 {
              panic!("Invalid pool size: {}", size);
            }
            if capacity == 0 || capacity > u32::max_value() as u64 {
              panic!("Invalid pool capacity: {}", capacity);
            }
            pools.push(Pool {
              size: size as u32,
              capacity: capacity as u32,
            });
            match pools_tokens.next() {
              Some(syn::TokenTree::Token(syn::Token::Comma)) | None => (),
              token => panic!(
                "Invalid tokens after `pools = [... {}`: {:?}",
                pools.last().unwrap(),
                token
              ),
            }
          }
          token => if let Some(pool) = pools.last() {
            panic!("Invalid tokens after `pools = [... {}`: {:?}", pool, token)
          } else {
            panic!("Invalid tokens after `pools = [`: {:?}", token)
          },
        }
      }
      match input.next() {
        Some(syn::TokenTree::Token(syn::Token::Semi)) => (),
        token => panic!("Invalid tokens after `pools = [...]`: {:?}", token),
      }
    }
    token => panic!("Invalid tokens after `pools =`: {:?}", token),
  }
}

impl fmt::Display for Pool {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[{}; {}]", self.size, self.capacity)
  }
}

impl Pool {
  fn length(&self) -> u32 {
    self.size * self.capacity
  }
}

trait IntExt {
  fn as_usize_lit(self) -> syn::Token;
}

impl IntExt for u32 {
  fn as_usize_lit(self) -> syn::Token {
    syn::Token::Literal(syn::Lit::Int(self as u64, syn::IntTy::Usize))
  }
}
