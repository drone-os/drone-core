use drone_config::Config;
use drone_macros_core::compile_error;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Ident, LitInt, Token, Visibility,
};

struct Input {
    heap_attrs: Vec<Attribute>,
    heap_vis: Visibility,
    heap_ident: Ident,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let heap_attrs = input.call(Attribute::parse_outer)?;
        let heap_vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let heap_ident = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self { heap_attrs, heap_vis, heap_ident })
    }
}

#[allow(clippy::too_many_lines)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { heap_attrs, heap_vis, heap_ident } = parse_macro_input!(input);
    let config = match Config::read_from_cargo_manifest_dir() {
        Ok(config) => config,
        Err(err) => compile_error!("{}: {}", drone_config::CONFIG_NAME, err),
    };
    let mut pools = config.heap.pools;
    pools.sort_by_key(|pool| pool.block);
    let mut pools_tokens = Vec::new();
    let mut pointer = config.memory.ram.origin + config.memory.ram.size - config.heap.size;
    for pool in &pools {
        let block = LitInt::new(&pool.block.to_string(), Span::call_site());
        let capacity = LitInt::new(&pool.capacity.to_string(), Span::call_site());
        let address = LitInt::new(&pointer.to_string(), Span::call_site());
        pools_tokens.push(quote! {
            ::drone_core::heap::Pool::new(#address, #block, #capacity)
        });
        pointer += pool.block * pool.capacity;
    }
    let pools_len = pools.len();

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

            #[inline]
            unsafe fn get_pool_unchecked<I>(&self, index: I) -> &I::Output
            where
                I: ::core::slice::SliceIndex<[::drone_core::heap::Pool]>,
            {
                self.pools.get_unchecked(index)
            }
        }

        unsafe impl ::core::alloc::AllocRef for #heap_ident {
            fn alloc(
                &mut self,
                layout: ::core::alloc::Layout,
            ) -> Result<(::core::ptr::NonNull<u8>, usize), ::core::alloc::AllocErr> {
                ::drone_core::heap::alloc(self, layout)
            }

            unsafe fn dealloc(
                &mut self,
                ptr: ::core::ptr::NonNull<u8>,
                layout: ::core::alloc::Layout,
            ) {
                ::drone_core::heap::dealloc(self, ptr, layout)
            }

            unsafe fn grow_in_place(
                &mut self,
                ptr: ::core::ptr::NonNull<u8>,
                layout: ::core::alloc::Layout,
                new_size: usize
            ) -> Result<usize, ::core::alloc::CannotReallocInPlace> {
                ::drone_core::heap::grow_in_place(self, ptr, layout, new_size)
            }

            unsafe fn shrink_in_place(
                &mut self,
                ptr: ::core::ptr::NonNull<u8>,
                layout: ::core::alloc::Layout,
                new_size: usize
            ) -> Result<usize, ::core::alloc::CannotReallocInPlace> {
                ::drone_core::heap::shrink_in_place(self, ptr, layout, new_size)
            }
        }

        unsafe impl ::core::alloc::GlobalAlloc for #heap_ident {
            unsafe fn alloc(&self, layout: ::core::alloc::Layout) -> *mut u8 {
                ::drone_core::heap::alloc(self, layout)
                    .map(|(ptr, _size)| ptr.as_ptr())
                    .unwrap_or(::core::ptr::null_mut())
            }

            unsafe fn dealloc(&self, ptr: *mut u8, layout: ::core::alloc::Layout) {
                ::drone_core::heap::dealloc(
                    self,
                    ::core::ptr::NonNull::new_unchecked(ptr),
                    layout,
                )
            }
        }
    };
    expanded.into()
}
