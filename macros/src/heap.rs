use drone_macros_core::{emit_err2, NewStruct};
use proc_macro2::{Span, TokenStream};
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use syn::{Ident, LitInt};

struct Heap {
  heap: NewStruct,
  size: LitInt,
  pools: Vec<Pool>,
}

struct Pool {
  size: LitInt,
  capacity: LitInt,
}

#[cfg_attr(feature = "cargo-clippy", allow(redundant_closure))]
impl Synom for Heap {
  named!(parse -> Self, do_parse!(
    heap: syn!(NewStruct) >>

    ident: syn!(Ident) >>
    switch!(value!(ident.to_string().as_ref()),
      "size" => value!(()) |
      _ => reject!()
    ) >>
    punct!(=) >>
    size: syn!(LitInt) >>
    punct!(;) >>

    ident: syn!(Ident) >>
    switch!(value!(ident.to_string().as_ref()),
      "pools" => value!(()) |
      _ => reject!()
    ) >>
    punct!(=) >>
    pools: map!(
      brackets!(call!(Punctuated::<Pool, Token![,]>::parse_terminated)),
      |x| x.1.into_iter().collect()
    ) >>
    punct!(;) >>

    (Heap { heap, size, pools })
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
  let (def_site, call_site) = (Span::def_site(), Span::call_site());
  let Heap {
    heap:
      NewStruct {
        attrs: heap_attrs,
        vis: heap_vis,
        ident: heap_ident,
      },
    size,
    mut pools,
  } = try_parse2!(call_site, input);
  let scope = Ident::new("__HEAP_RT", def_site);
  let def_new = Ident::new("new", call_site);
  pools.sort_by_key(|pool| pool.size.value());
  let free = pools
    .iter()
    .map(Pool::length)
    .fold(size.value() as i64, |a, e| a - e as i64);
  if free != 0 {
    return emit_err2(
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
    pools_tokens.push(quote_spanned! { def_site =>
      Pool::new(#offset, #size, #capacity)
    });
    offset += pool.length();
  }

  quote_spanned! { def_site =>
    #(#heap_attrs)*
    #heap_vis struct #heap_ident {
      pools: [extern::drone_core::heap::Pool; #pools_len],
    }

    const #scope: () = {
      use extern::core::alloc::{Alloc, AllocErr, CannotReallocInPlace, Excess,
                                GlobalAlloc, Layout};
      use extern::core::ptr::{self, NonNull};
      use extern::core::slice::SliceIndex;
      use extern::drone_core::heap::{Allocator, Pool};

      impl #heap_ident {
        /// Creates a new heap.
        pub const fn #def_new() -> Self {
          Self {
            pools: [#(#pools_tokens),*],
          }
        }
      }

      impl Allocator for #heap_ident {
        const POOL_COUNT: usize = #pools_len;

        #[inline(always)]
        unsafe fn get_pool_unchecked<I>(&self, index: I) -> &I::Output
        where
          I: SliceIndex<[Pool]>,
        {
          self.pools.get_unchecked(index)
        }

        #[inline(always)]
        unsafe fn get_pool_unchecked_mut<I>(
          &mut self, index: I
        ) -> &mut I::Output
        where
          I: SliceIndex<[Pool]>,
        {
          self.pools.get_unchecked_mut(index)
        }
      }

      unsafe impl Alloc for #heap_ident {
        unsafe fn alloc(
          &mut self,
          layout: Layout,
        ) -> Result<NonNull<u8>, AllocErr> {
          Allocator::alloc(self, layout)
        }

        unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
          Allocator::dealloc(self, ptr, layout)
        }

        #[inline(always)]
        fn usable_size(&self, layout: &Layout) -> (usize, usize) {
          unsafe { Allocator::usable_size(self, layout) }
        }

        unsafe fn realloc(
          &mut self,
          ptr: NonNull<u8>,
          layout: Layout,
          new_size: usize,
        ) -> Result<NonNull<u8>, AllocErr> {
          Allocator::realloc(self, ptr, layout, new_size)
        }

        unsafe fn alloc_excess(
          &mut self,
          layout: Layout,
        ) -> Result<Excess, AllocErr> {
          Allocator::alloc_excess(self, layout)
        }

        unsafe fn realloc_excess(
          &mut self,
          ptr: NonNull<u8>,
          layout: Layout,
          new_size: usize,
        ) -> Result<Excess, AllocErr> {
          Allocator::realloc_excess(self, ptr, layout, new_size)
        }

        unsafe fn grow_in_place(
          &mut self,
          ptr: NonNull<u8>,
          layout: Layout,
          new_size: usize,
        ) -> Result<(), CannotReallocInPlace> {
          Allocator::grow_in_place(self, ptr, layout, new_size)
        }

        unsafe fn shrink_in_place(
          &mut self,
          ptr: NonNull<u8>,
          layout: Layout,
          new_size: usize,
        ) -> Result<(), CannotReallocInPlace> {
          Allocator::shrink_in_place(self, ptr, layout, new_size)
        }
      }

      unsafe impl GlobalAlloc for #heap_ident {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
          Allocator::alloc(self, layout)
            .map(NonNull::as_ptr).unwrap_or(ptr::null_mut())
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
          Allocator::dealloc(self, NonNull::new_unchecked(ptr), layout)
        }

        unsafe fn realloc(
          &self,
          ptr: *mut u8,
          layout: Layout,
          new_size: usize,
        ) -> *mut u8 {
          Allocator::realloc(
            self,
            NonNull::new_unchecked(ptr),
            layout,
            new_size,
          ).map(NonNull::as_ptr).unwrap_or(ptr::null_mut())
        }
      }
    };
  }
}
