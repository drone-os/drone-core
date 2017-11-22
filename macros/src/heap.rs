use failure::{err_msg, Error};
use proc_macro::TokenStream;
use quote;
use quote::Tokens;
use std::{fmt, vec};
use syn::{parse_token_trees, DelimToken, Delimited, IntTy, Lit, Token,
          TokenTree};

struct Pool {
  size: u32,
  capacity: u32,
}

pub(crate) fn heap(input: TokenStream) -> Result<Tokens, Error> {
  let input = parse_token_trees(&input.to_string()).map_err(err_msg)?;
  let mut input = input.into_iter();
  let mut attributes = Vec::new();
  let mut pools = Vec::new();
  let mut size = 0;
  while let Some(token) = input.next() {
    match token {
      TokenTree::Token(token) => match token {
        Token::DocComment(ref string) if string.starts_with("//!") => {
          parse_doc(&string, &mut attributes)
        }
        Token::Pound => parse_attr(&mut input, &mut attributes)?,
        Token::Ident(ident) => if ident == "size" {
          parse_size(&mut input, &mut size)?;
        } else if ident == "pools" {
          parse_pools(&mut input, &mut pools)?;
        } else {
          Err(format_err!("Invalid ident: {}", ident))?;
        },
        token => Err(format_err!("Invalid root token: {:?}", token))?,
      },
      token => Err(format_err!("Invalid root token tree: {:?}", token))?,
    }
  }

  normalize_pools(&mut pools, size)?;
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

  Ok(quote! {
    use ::alloc::allocator::{Alloc, AllocErr, CannotReallocInPlace, Excess,
                             Layout};
    use ::core::slice::SliceIndex;
    use ::drone::heap::{Allocator, Pool};

    #[doc(hidden)]
    pub struct Heap {
      pools: [Pool; #pool_count],
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

      #[inline(always)]
      unsafe fn get_pool_unchecked<I>(&self, index: I) -> &I::Output
      where
        I: SliceIndex<[Pool]>,
      {
        self.pools.get_unchecked(index)
      }

      #[inline(always)]
      unsafe fn get_pool_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
      where
        I: SliceIndex<[Pool]>,
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

      #[inline(always)]
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

    /// Initializes the allocator.
    ///
    /// See [`Allocator::init()`] for more details.
    ///
    /// [`Allocator::init()`]:
    /// ../../drone/heap/allocator/trait.Allocator.html#method.init
    #[inline(always)]
    pub unsafe fn init(start: &mut usize) {
      ALLOC.init(start)
    }
  })
}

fn normalize_pools(pools: &mut Vec<Pool>, size: u32) -> Result<(), Error> {
  pools.sort_by_key(|pool| pool.size);
  let free = pools
    .iter()
    .map(Pool::length)
    .fold(size as i64, |a, e| a - e as i64);
  if free != 0 {
    Err(format_err!(
      "`pools` not matches `size`. Difference is {}. Consider setting `size` \
       to {}.",
      -free,
      size as i64 - free
    ))?;
  }
  Ok(())
}

fn parse_doc(input: &str, attributes: &mut Vec<quote::Tokens>) {
  let string = input.trim_left_matches("//!");
  attributes.push(quote!(#[doc = #string]));
}

fn parse_attr(
  input: &mut vec::IntoIter<TokenTree>,
  attributes: &mut Vec<quote::Tokens>,
) -> Result<(), Error> {
  match input.next() {
    Some(TokenTree::Token(Token::Not)) => match input.next() {
      Some(TokenTree::Delimited(delimited)) => {
        attributes.push(quote!(# #delimited))
      }
      token => Err(format_err!("Invalid tokens after `#!`: {:?}", token))?,
    },
    token => Err(format_err!("Invalid tokens after `#`: {:?}", token))?,
  }
  Ok(())
}

fn parse_size(
  input: &mut vec::IntoIter<TokenTree>,
  size: &mut u32,
) -> Result<(), Error> {
  match input.next() {
    Some(TokenTree::Token(Token::Eq)) => match input.next() {
      Some(
        TokenTree::Token(Token::Literal(Lit::Int(int, IntTy::Unsuffixed))),
      ) => {
        if int > u32::max_value() as u64 {
          Err(format_err!("Invalid size: {}", int))?;
        }
        *size = int as u32;
        match input.next() {
          Some(TokenTree::Token(Token::Semi)) => (),
          token => Err(format_err!(
            "Invalid tokens after `size = {}`: {:?}",
            int,
            token
          ))?,
        }
      }
      token => Err(format_err!("Invalid tokens after `size =`: {:?}", token))?,
    },
    token => Err(format_err!("Invalid tokens after `size`: {:?}", token))?,
  }
  Ok(())
}

fn parse_pools(
  input: &mut vec::IntoIter<TokenTree>,
  pools: &mut Vec<Pool>,
) -> Result<(), Error> {
  match input.next() {
    Some(TokenTree::Token(Token::Eq)) => (),
    token => Err(format_err!("Invalid tokens after `pools`: {:?}", token))?,
  }
  match input.next() {
    Some(TokenTree::Delimited(Delimited {
      delim: DelimToken::Bracket,
      tts: pools_tokens,
    })) => {
      let mut pools_tokens = pools_tokens.into_iter();
      while let Some(token) = pools_tokens.next() {
        match token {
          TokenTree::Delimited(Delimited {
            delim: DelimToken::Bracket,
            tts: pool_tokens,
          }) => {
            let mut pool_tokens = pool_tokens.into_iter();
            let size = match pool_tokens.next() {
              Some(TokenTree::Token(
                Token::Literal(Lit::Int(size, IntTy::Unsuffixed)),
              )) => size,
              token => Err(format_err!(
                "Invalid tokens after `pools = [... [`: {:?}",
                token
              ))?,
            };
            match pool_tokens.next() {
              Some(TokenTree::Token(Token::Semi)) => (),
              token => Err(format_err!(
                "Invalid tokens after `pools = [... [{}`: {:?}",
                size,
                token
              ))?,
            }
            let capacity = match pool_tokens.next() {
              Some(TokenTree::Token(
                Token::Literal(Lit::Int(capacity, IntTy::Unsuffixed)),
              )) => capacity,
              token => Err(format_err!(
                "Invalid tokens after `pools = [... [{};`: {:?}",
                size,
                token
              ))?,
            };
            match pool_tokens.next() {
              Some(token) => Err(format_err!(
                "Invalid tokens after `pools = [... [{}; {}`: {:?}",
                size,
                capacity,
                token
              ))?,
              None => (),
            }
            if size == 0 || size > u32::max_value() as u64 {
              Err(format_err!("Invalid pool size: {}", size))?;
            }
            if capacity == 0 || capacity > u32::max_value() as u64 {
              Err(format_err!("Invalid pool capacity: {}", capacity))?;
            }
            pools.push(Pool {
              size: size as u32,
              capacity: capacity as u32,
            });
            match pools_tokens.next() {
              Some(TokenTree::Token(Token::Comma)) | None => (),
              token => Err(format_err!(
                "Invalid tokens after `pools = [... {}`: {:?}",
                pools.last().unwrap(),
                token
              ))?,
            }
          }
          token => if let Some(pool) = pools.last() {
            Err(format_err!(
              "Invalid tokens after `pools = [... {}`: {:?}",
              pool,
              token
            ))?;
          } else {
            Err(format_err!("Invalid tokens after `pools = [`: {:?}", token))?;
          },
        }
      }
      match input.next() {
        Some(TokenTree::Token(Token::Semi)) => (),
        token => Err(format_err!(
          "Invalid tokens after `pools = [...]`: {:?}",
          token
        ))?,
      }
    }
    token => Err(format_err!("Invalid tokens after `pools =`: {:?}", token))?,
  }
  Ok(())
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
  fn as_usize_lit(self) -> Token;
}

impl IntExt for u32 {
  fn as_usize_lit(self) -> Token {
    Token::Literal(Lit::Int(self as u64, IntTy::Usize))
  }
}
