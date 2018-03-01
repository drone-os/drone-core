use drone_macros_core::{emit_err, NewStatic, NewStruct};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Ident, LitInt};
use syn::punctuated::Punctuated;
use syn::synom::Synom;

struct Heap {
  heap: NewStruct,
  alloc: NewStatic,
  size: LitInt,
  pools: Vec<Pool>,
}

struct Pool {
  size: LitInt,
  capacity: LitInt,
}

impl Synom for Heap {
  named!(parse -> Self, do_parse!(
    heap: syn!(NewStruct) >>
    alloc: syn!(NewStatic) >>

    ident: syn!(Ident) >>
    switch!(value!(ident.as_ref()),
      "size" => value!(()) |
      _ => reject!()
    ) >>
    punct!(=) >>
    size: syn!(LitInt) >>
    punct!(;) >>

    ident: syn!(Ident) >>
    switch!(value!(ident.as_ref()),
      "pools" => value!(()) |
      _ => reject!()
    ) >>
    punct!(=) >>
    pools: map!(
      brackets!(call!(Punctuated::<Pool, Token![,]>::parse_terminated)),
      |x| x.1.into_iter().collect()
    ) >>
    punct!(;) >>

    (Heap { heap, alloc, size, pools })
  ));
}

impl Synom for Pool {
  named!(parse -> Self, do_parse!(
    brackets: brackets!(do_parse!(
      size: syn!(LitInt) >>
      punct!(;) >>
      capacity: syn!(LitInt) >>
      (Pool { size, capacity })
    )) >>
    (brackets.1)
  ));
}

impl Pool {
  fn length(&self) -> usize {
    self.size.value() as usize * self.capacity.value() as usize
  }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let call_site = Span::call_site();
  let Heap {
    heap:
      NewStruct {
        attrs: heap_attrs,
        vis: heap_vis,
        ident: heap_ident,
      },
    alloc:
      NewStatic {
        attrs: alloc_attrs,
        vis: alloc_vis,
        ident: alloc_ident,
      },
    size,
    mut pools,
  } = try_parse!(call_site, input);
  let rt = Ident::from("__heap_rt");
  pools.sort_by_key(|pool| pool.size.value());
  let free = pools
    .iter()
    .map(Pool::length)
    .fold(size.value() as i64, |a, e| a - e as i64);
  if free != 0 {
    return emit_err(
      call_site,
      &format!(
        "`pools` not matches `size`. Difference is {}. Consider setting \
         `size` to {}.",
        -free,
        size.value() as i64 - free
      ),
    );
  }
  let (mut pools_tokens, mut offset, pools_len) = (Vec::new(), 0, pools.len());
  for pool in &pools {
    let &Pool {
      ref size,
      ref capacity,
    } = pool;
    pools_tokens.push(quote! {
      #rt::Pool::new(#offset, #size, #capacity)
    });
    offset += pool.length();
  }

  let expanded = quote! {
    mod #rt {
      extern crate alloc;
      extern crate core;
      extern crate drone_core;

      pub use self::alloc::allocator::{Alloc, AllocErr, CannotReallocInPlace,
                                       Excess, Layout};
      pub use self::core::slice::SliceIndex;
      pub use self::drone_core::heap::{Allocator, Pool};
    }

    #(#heap_attrs)*
    #heap_vis struct #heap_ident {
      pools: [#rt::Pool; #pools_len],
    }

    #(#alloc_attrs)*
    #alloc_vis static mut #alloc_ident: #heap_ident = #heap_ident {
      pools: [#(#pools_tokens),*],
    };

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
      unsafe fn get_pool_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
      where
        I: #rt::SliceIndex<[#rt::Pool]>,
      {
        self.pools.get_unchecked_mut(index)
      }
    }

    unsafe impl<'a> #rt::Alloc for &'a #heap_ident {
      unsafe fn alloc(
        &mut self,
        layout: #rt::Layout,
      ) -> Result<*mut u8, #rt::AllocErr> {
        #rt::Allocator::alloc(*self, layout)
      }

      unsafe fn dealloc(&mut self, ptr: *mut u8, layout: #rt::Layout) {
        #rt::Allocator::dealloc(*self, ptr, layout)
      }

      #[inline(always)]
      fn usable_size(&self, layout: &#rt::Layout) -> (usize, usize) {
        unsafe { #rt::Allocator::usable_size(*self, layout) }
      }

      unsafe fn realloc(
        &mut self,
        ptr: *mut u8,
        layout: #rt::Layout,
        new_layout: #rt::Layout
      ) -> Result<*mut u8, #rt::AllocErr> {
        #rt::Allocator::realloc(*self, ptr, layout, new_layout)
      }

      unsafe fn alloc_excess(
        &mut self,
        layout: #rt::Layout,
      ) -> Result<#rt::Excess, #rt::AllocErr> {
        #rt::Allocator::alloc_excess(*self, layout)
      }

      unsafe fn realloc_excess(
        &mut self,
        ptr: *mut u8,
        layout: #rt::Layout,
        new_layout: #rt::Layout
      ) -> Result<#rt::Excess, #rt::AllocErr> {
        #rt::Allocator::realloc_excess(*self, ptr, layout, new_layout)
      }

      unsafe fn grow_in_place(
        &mut self,
        ptr: *mut u8,
        layout: #rt::Layout,
        new_layout: #rt::Layout
      ) -> Result<(), #rt::CannotReallocInPlace> {
        #rt::Allocator::grow_in_place(*self, ptr, layout, new_layout)
      }

      unsafe fn shrink_in_place(
        &mut self,
        ptr: *mut u8,
        layout: #rt::Layout,
        new_layout: #rt::Layout
      ) -> Result<(), #rt::CannotReallocInPlace> {
        #rt::Allocator::shrink_in_place(*self, ptr, layout, new_layout)
      }
    }
  };
  expanded.into()
}
