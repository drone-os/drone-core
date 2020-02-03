use drone_config::Config;
use drone_macros_core::compile_error;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, ExprPath, Ident, LitInt, Token, Visibility,
};

struct Heap {
    heap_attrs: Vec<Attribute>,
    heap_vis: Visibility,
    heap_ident: Ident,
    hooks: Option<[(Vec<Attribute>, ExprPath); 4]>,
}

impl Parse for Heap {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let heap_attrs = input.call(Attribute::parse_outer)?;
        let heap_vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let heap_ident = input.parse()?;
        input.parse::<Token![;]>()?;
        let hooks = if input.is_empty() {
            None
        } else {
            let alloc_attrs = input.call(Attribute::parse_outer)?;
            input.parse::<Token![use]>()?;
            let alloc_path = input.parse()?;
            input.parse::<Token![;]>()?;
            let dealloc_attrs = input.call(Attribute::parse_outer)?;
            input.parse::<Token![use]>()?;
            let dealloc_path = input.parse()?;
            input.parse::<Token![;]>()?;
            let grow_in_place_attrs = input.call(Attribute::parse_outer)?;
            input.parse::<Token![use]>()?;
            let grow_in_place_path = input.parse()?;
            input.parse::<Token![;]>()?;
            let shrink_in_place_attrs = input.call(Attribute::parse_outer)?;
            input.parse::<Token![use]>()?;
            let shrink_in_place_path = input.parse()?;
            input.parse::<Token![;]>()?;
            Some([
                (alloc_attrs, alloc_path),
                (dealloc_attrs, dealloc_path),
                (grow_in_place_attrs, grow_in_place_path),
                (shrink_in_place_attrs, shrink_in_place_path),
            ])
        };
        Ok(Self { heap_attrs, heap_vis, heap_ident, hooks })
    }
}

#[allow(clippy::too_many_lines)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Heap { heap_attrs, heap_vis, heap_ident, hooks } = parse_macro_input!(input as Heap);
    let config = match Config::read_from_cargo_manifest_dir() {
        Ok(config) => config,
        Err(err) => compile_error!("Drone.toml: {}", err),
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
    let hook_tokens = if let Some(hooks) = hooks {
        let [(alloc_attrs, alloc_path), (dealloc_attrs, dealloc_path), (grow_in_place_attrs, grow_in_place_path), (shrink_in_place_attrs, shrink_in_place_path)] =
            hooks;
        vec![quote! {
            #(#alloc_attrs)*
            #[inline]
            fn alloc_hook(
                layout: ::core::alloc::Layout,
                pool: &::drone_core::heap::Pool,
            ) {
                #alloc_path(layout, pool)
            }

            #(#dealloc_attrs)*
            #[inline]
            fn dealloc_hook(
                layout: ::core::alloc::Layout,
                pool: &::drone_core::heap::Pool,
            ) {
                #dealloc_path(layout, pool)
            }

            #(#grow_in_place_attrs)*
            #[inline]
            fn grow_in_place_hook(
                layout: ::core::alloc::Layout,
                new_size: usize,
            ) {
                #grow_in_place_path(layout, new_size)
            }

            #(#shrink_in_place_attrs)*
            #[inline]
            fn shrink_in_place_hook(
                layout: ::core::alloc::Layout,
                new_size: usize,
            ) {
                #shrink_in_place_path(layout, new_size)
            }
        }]
    } else {
        vec![]
    };
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

            #[inline]
            unsafe fn get_pool_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
            where
                I: ::core::slice::SliceIndex<[::drone_core::heap::Pool]>,
            {
                self.pools.get_unchecked_mut(index)
            }

            #(#hook_tokens)*
        }

        unsafe impl ::core::alloc::AllocRef for #heap_ident {
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
                ::drone_core::heap::Allocator::grow_in_place(self, ptr, layout, new_size)
            }

            unsafe fn shrink_in_place(
                &mut self,
                ptr: ::core::ptr::NonNull<u8>,
                layout: ::core::alloc::Layout,
                new_size: usize,
            ) -> Result<(), ::core::alloc::CannotReallocInPlace> {
                ::drone_core::heap::Allocator::shrink_in_place(self, ptr, layout, new_size)
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
