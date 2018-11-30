use drone_macros_core::{ExternFn, NewStruct};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Ident, LitInt};

struct Heap {
  heap: NewStruct,
  alloc_hook: Option<ExternFn>,
  dealloc_hook: Option<ExternFn>,
  size: LitInt,
  pools: Vec<Pool>,
}

struct Pool {
  size: LitInt,
  capacity: LitInt,
}

impl Parse for Heap {
  fn parse(input: ParseStream) -> Result<Self> {
    let heap = input.parse()?;
    let (alloc_hook, dealloc_hook) = if input.peek(Token![extern]) {
      (Some(input.parse()?), Some(input.parse()?))
    } else {
      (None, None)
    };
    let mut size = None;
    let mut pools = Vec::new();
    while !input.is_empty() {
      let ident = input.parse::<Ident>()?;
      input.parse::<Token![=]>()?;
      if ident == "size" {
        if size.is_some() {
          return Err(input.error("`size` is already defined"));
        }
        size = Some(input.parse()?);
      } else if ident == "pools" {
        if !pools.is_empty() {
          return Err(input.error("`pools` is already defined"));
        }
        let content;
        bracketed!(content in input);
        pools = content
          .call(Punctuated::<Pool, Token![,]>::parse_terminated)?
          .into_iter()
          .collect();
      } else {
        return Err(input.error("invalid option"));
      }
      input.parse::<Token![;]>()?;
    }
    Ok(Self {
      heap,
      alloc_hook,
      dealloc_hook,
      size: size.ok_or_else(|| input.error("`size` is not defined"))?,
      pools,
    })
  }
}

impl Parse for Pool {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    bracketed!(content in input);
    let size = content.parse()?;
    content.parse::<Token![;]>()?;
    let capacity = content.parse()?;
    Ok(Self { size, capacity })
  }
}

impl Pool {
  fn length(&self) -> usize {
    self.size.value() as usize * self.capacity.value() as usize
  }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let (def_site, call_site) = (Span::def_site(), Span::call_site());
  let Heap {
    heap:
      NewStruct {
        attrs: heap_attrs,
        vis: heap_vis,
        ident: heap_ident,
      },
    alloc_hook,
    dealloc_hook,
    size,
    mut pools,
  } = parse_macro_input!(input as Heap);
  let rt = Ident::new("__heap_rt", def_site);
  pools.sort_by_key(|pool| pool.size.value());
  let free = pools
    .iter()
    .map(Pool::length)
    .fold(size.value() as i64, |a, e| a - e as i64);
  if free != 0 {
    return Error::new(
      call_site,
      &format!(
        "`pools` not matches `size`. Difference is {}. Consider setting \
         `size` to {}.",
        -free,
        size.value() as i64 - free
      ),
    )
    .to_compile_error()
    .into();
  }
  let (mut pools_tokens, mut offset, pools_len) = (Vec::new(), 0, pools.len());
  for pool in &pools {
    let &Pool {
      ref size,
      ref capacity,
    } = pool;
    pools_tokens.push(quote!(#rt::Pool::new(#offset, #size, #capacity)));
    offset += pool.length();
  }
  let mut hook_tokens = Vec::new();
  if let Some(ExternFn { path }) = alloc_hook {
    hook_tokens.push(quote! {
      #[inline(always)]
      fn alloc_hook(layout: #rt::Layout, pool: &#rt::Pool) {
        #path(layout, pool)
      }
    });
  }
  if let Some(ExternFn { path }) = dealloc_hook {
    hook_tokens.push(quote! {
      #[inline(always)]
      fn dealloc_hook(layout: #rt::Layout, pool: &#rt::Pool) {
        #path(layout, pool)
      }
    });
  }

  let expanded = quote! {
    mod #rt {
      extern crate core;
      extern crate drone_core;

      pub use self::core::alloc::{Alloc, AllocErr, CannotReallocInPlace, Excess,
                                  GlobalAlloc, Layout};
      pub use self::core::ptr::{self, NonNull};
      pub use self::core::slice::SliceIndex;
      pub use self::drone_core::heap::{Allocator, Pool};
    }

    #(#heap_attrs)*
    #heap_vis struct #heap_ident {
      pools: [#rt::Pool; #pools_len],
    }

    impl #heap_ident {
      /// Creates a new heap.
      pub const fn new() -> Self {
        Self {
          pools: [#(#pools_tokens),*],
        }
      }
    }

    impl #rt::Allocator for #heap_ident {
      const POOL_COUNT: usize = #pools_len;

      #[inline(always)]
      unsafe fn get_pool_unchecked<I>(&self, index: I) -> &I::Output
      where
        I: #rt::SliceIndex<[#rt::Pool]>,
      {
        self.pools.get_unchecked(index)
      }

      #[inline(always)]
      unsafe fn get_pool_unchecked_mut<I>(
        &mut self, index: I
      ) -> &mut I::Output
      where
        I: #rt::SliceIndex<[#rt::Pool]>,
      {
        self.pools.get_unchecked_mut(index)
      }

      #(#hook_tokens)*
    }

    unsafe impl #rt::Alloc for #heap_ident {
      unsafe fn alloc(
        &mut self,
        layout: #rt::Layout,
      ) -> Result<#rt::NonNull<u8>, #rt::AllocErr> {
        #rt::Allocator::alloc(self, layout)
      }

      unsafe fn dealloc(&mut self, ptr: #rt::NonNull<u8>, layout: #rt::Layout) {
        #rt::Allocator::dealloc(self, ptr, layout)
      }

      fn usable_size(&self, layout: &#rt::Layout) -> (usize, usize) {
        unsafe { #rt::Allocator::usable_size(self, layout) }
      }

      unsafe fn realloc(
        &mut self,
        ptr: #rt::NonNull<u8>,
        layout: #rt::Layout,
        new_size: usize,
      ) -> Result<#rt::NonNull<u8>, #rt::AllocErr> {
        #rt::Allocator::realloc(self, ptr, layout, new_size)
      }

      unsafe fn alloc_excess(
        &mut self,
        layout: #rt::Layout,
      ) -> Result<#rt::Excess, #rt::AllocErr> {
        #rt::Allocator::alloc(self, layout)
      }

      unsafe fn realloc_excess(
        &mut self,
        ptr: #rt::NonNull<u8>,
        layout: #rt::Layout,
        new_size: usize,
      ) -> Result<#rt::Excess, #rt::AllocErr> {
        #rt::Allocator::realloc(self, ptr, layout, new_size)
      }

      unsafe fn grow_in_place(
        &mut self,
        ptr: #rt::NonNull<u8>,
        layout: #rt::Layout,
        new_size: usize,
      ) -> Result<(), #rt::CannotReallocInPlace> {
        #rt::Allocator::grow_in_place(self, ptr, layout, new_size)
      }

      unsafe fn shrink_in_place(
        &mut self,
        ptr: #rt::NonNull<u8>,
        layout: #rt::Layout,
        new_size: usize,
      ) -> Result<(), #rt::CannotReallocInPlace> {
        #rt::Allocator::shrink_in_place(self, ptr, layout, new_size)
      }
    }

    unsafe impl #rt::GlobalAlloc for #heap_ident {
      unsafe fn alloc(&self, layout: #rt::Layout) -> *mut u8 {
        #rt::Allocator::alloc(self, layout)
          .map(#rt::NonNull::as_ptr).unwrap_or(#rt::ptr::null_mut())
      }

      unsafe fn dealloc(&self, ptr: *mut u8, layout: #rt::Layout) {
        #rt::Allocator::dealloc(self, #rt::NonNull::new_unchecked(ptr), layout)
      }

      unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: #rt::Layout,
        new_size: usize,
      ) -> *mut u8 {
        #rt::Allocator::realloc(
          self,
          #rt::NonNull::new_unchecked(ptr),
          layout,
          new_size,
        ).map(#rt::NonNull::as_ptr).unwrap_or(#rt::ptr::null_mut())
      }
    }
  };
  expanded.into()
}
