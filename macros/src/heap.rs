use drone_macros_core::compile_error;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
  bracketed,
  parse::{Parse, ParseStream, Result},
  parse_macro_input,
  punctuated::Punctuated,
  Attribute, ExprPath, Ident, LitInt, Token, Visibility,
};

struct Heap {
  heap_attrs: Vec<Attribute>,
  heap_vis: Visibility,
  heap_ident: Ident,
  alloc_hook: Option<ExprPath>,
  dealloc_hook: Option<ExprPath>,
  size: LitInt,
  pools: Vec<Pool>,
}

struct Pool {
  size: LitInt,
  capacity: LitInt,
}

impl Parse for Heap {
  fn parse(input: ParseStream) -> Result<Self> {
    let heap_attrs = input.call(Attribute::parse_outer)?;
    let heap_vis = input.parse()?;
    input.parse::<Token![struct]>()?;
    let heap_ident = input.parse()?;
    input.parse::<Token![;]>()?;
    let (alloc_hook, dealloc_hook) = if input.peek(Token![extern]) {
      input.parse::<Token![extern]>()?;
      input.parse::<Token![fn]>()?;
      let alloc_hook = input.parse()?;
      input.parse::<Token![;]>()?;
      input.parse::<Token![extern]>()?;
      input.parse::<Token![fn]>()?;
      let dealloc_hook = input.parse()?;
      input.parse::<Token![;]>()?;
      (Some(alloc_hook), Some(dealloc_hook))
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
      heap_attrs,
      heap_vis,
      heap_ident,
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
  let Heap {
    heap_attrs,
    heap_vis,
    heap_ident,
    alloc_hook,
    dealloc_hook,
    size,
    mut pools,
  } = parse_macro_input!(input as Heap);
  pools.sort_by_key(|pool| pool.size.value());
  let free = pools
    .iter()
    .map(Pool::length)
    .fold(size.value() as i64, |a, e| a - e as i64);
  if free != 0 {
    compile_error!(
      "`pools` not matches `size`. Difference is {}. Consider setting `size` \
       to {}.",
      -free,
      size.value() as i64 - free
    );
  }
  let (mut pools_tokens, mut offset, pools_len) = (Vec::new(), 0, pools.len());
  for pool in &pools {
    let &Pool {
      ref size,
      ref capacity,
    } = pool;
    pools_tokens.push(quote! {
      ::drone_core::heap::Pool::new(#offset, #size, #capacity)
    });
    offset += pool.length();
  }
  let mut hook_tokens = Vec::new();
  if let Some(path) = alloc_hook {
    hook_tokens.push(quote! {
      #[inline(always)]
      fn alloc_hook(
        layout: ::core::alloc::Layout,
        pool: &::drone_core::heap::Pool,
      ) {
        #path(layout, pool)
      }
    });
  }
  if let Some(path) = dealloc_hook {
    hook_tokens.push(quote! {
      #[inline(always)]
      fn dealloc_hook(
        layout: ::core::alloc::Layout,
        pool: &::drone_core::heap::Pool,
      ) {
        #path(layout, pool)
      }
    });
  }

  let expanded = quote! {
    #(#heap_attrs)*
    #heap_vis struct #heap_ident {
      pools: [::drone_core::heap::Pool; #pools_len],
    }

    impl #heap_ident {
      /// Creates a new heap.
      pub const fn new() -> Self {
        Self {
          pools: [#(#pools_tokens),*],
        }
      }
    }

    impl ::drone_core::heap::Allocator for #heap_ident {
      const POOL_COUNT: usize = #pools_len;

      #[inline(always)]
      unsafe fn get_pool_unchecked<I>(&self, index: I) -> &I::Output
      where
        I: ::core::slice::SliceIndex<[::drone_core::heap::Pool]>,
      {
        self.pools.get_unchecked(index)
      }

      #[inline(always)]
      unsafe fn get_pool_unchecked_mut<I>(
        &mut self, index: I
      ) -> &mut I::Output
      where
        I: ::core::slice::SliceIndex<[::drone_core::heap::Pool]>,
      {
        self.pools.get_unchecked_mut(index)
      }

      #(#hook_tokens)*
    }

    unsafe impl ::core::alloc::Alloc for #heap_ident {
      unsafe fn alloc(
        &mut self,
        layout: ::core::alloc::Layout,
      ) -> Result<::core::ptr::NonNull<u8>, ::core::alloc::AllocErr> {
        ::drone_core::heap::Allocator::alloc(self, layout)
      }

      unsafe fn dealloc(
        &mut self,
        ptr: ::core::ptr::NonNull<u8>,
        layout: ::core::alloc::Layout,
      ) {
        ::drone_core::heap::Allocator::dealloc(self, ptr, layout)
      }

      fn usable_size(&self, layout: &::core::alloc::Layout) -> (usize, usize) {
        unsafe { ::drone_core::heap::Allocator::usable_size(self, layout) }
      }

      unsafe fn realloc(
        &mut self,
        ptr: ::core::ptr::NonNull<u8>,
        layout: ::core::alloc::Layout,
        new_size: usize,
      ) -> Result<::core::ptr::NonNull<u8>, ::core::alloc::AllocErr> {
        ::drone_core::heap::Allocator::realloc(self, ptr, layout, new_size)
      }

      unsafe fn alloc_excess(
        &mut self,
        layout: ::core::alloc::Layout,
      ) -> Result<::core::alloc::Excess, ::core::alloc::AllocErr> {
        ::drone_core::heap::Allocator::alloc(self, layout)
      }

      unsafe fn realloc_excess(
        &mut self,
        ptr: ::core::ptr::NonNull<u8>,
        layout: ::core::alloc::Layout,
        new_size: usize,
      ) -> Result<::core::alloc::Excess, ::core::alloc::AllocErr> {
        ::drone_core::heap::Allocator::realloc(self, ptr, layout, new_size)
      }

      unsafe fn grow_in_place(
        &mut self,
        ptr: ::core::ptr::NonNull<u8>,
        layout: ::core::alloc::Layout,
        new_size: usize,
      ) -> Result<(), ::core::alloc::CannotReallocInPlace> {
        ::drone_core::heap::Allocator::grow_in_place(
          self,
          ptr,
          layout,
          new_size,
        )
      }

      unsafe fn shrink_in_place(
        &mut self,
        ptr: ::core::ptr::NonNull<u8>,
        layout: ::core::alloc::Layout,
        new_size: usize,
      ) -> Result<(), ::core::alloc::CannotReallocInPlace> {
        ::drone_core::heap::Allocator::shrink_in_place(
          self,
          ptr,
          layout,
          new_size,
        )
      }
    }

    unsafe impl ::core::alloc::GlobalAlloc for #heap_ident {
      unsafe fn alloc(&self, layout: ::core::alloc::Layout) -> *mut u8 {
        ::drone_core::heap::Allocator::alloc(self, layout)
          .map(::core::ptr::NonNull::as_ptr).unwrap_or(::core::ptr::null_mut())
      }

      unsafe fn dealloc(&self, ptr: *mut u8, layout: ::core::alloc::Layout) {
        ::drone_core::heap::Allocator::dealloc(
          self,
          ::core::ptr::NonNull::new_unchecked(ptr),
          layout,
        )
      }

      unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: ::core::alloc::Layout,
        new_size: usize,
      ) -> *mut u8 {
        ::drone_core::heap::Allocator::realloc(
          self,
          ::core::ptr::NonNull::new_unchecked(ptr),
          layout,
          new_size,
        ).map(::core::ptr::NonNull::as_ptr).unwrap_or(::core::ptr::null_mut())
      }
    }
  };
  expanded.into()
}
