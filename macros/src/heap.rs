use drone_config::Config;
use drone_macros_core::parse_error;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Ident, LitBool, LitInt, Token, Visibility,
};

struct Input {
    heap: Heap,
    global: Option<LitBool>,
}

struct Heap {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut heap = None;
        let mut global = None;
        while !input.is_empty() {
            let attrs = input.call(Attribute::parse_outer)?;
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![=>]>()?;
            if ident == "heap" {
                if heap.is_none() {
                    heap = Some(Heap::parse(input, attrs)?);
                } else {
                    return Err(input.error("multiple `heap` specifications"));
                }
            } else if attrs.is_empty() && ident == "global" {
                if global.is_none() {
                    global = Some(input.parse()?);
                } else {
                    return Err(input.error("multiple `global` specifications"));
                }
            } else {
                return Err(input.error(format!("unknown key: `{}`", ident)));
            }
            if !input.is_empty() {
                input.parse::<Token![;]>()?;
            }
        }
        Ok(Self { heap: heap.ok_or_else(|| input.error("missing `heap` specification"))?, global })
    }
}

impl Heap {
    fn parse(input: ParseStream<'_>, attrs: Vec<Attribute>) -> Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;
        Ok(Self { attrs, vis, ident })
    }
}

#[allow(clippy::too_many_lines)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { heap, global } = parse_macro_input!(input);
    let Heap { attrs: heap_attrs, vis: heap_vis, ident: heap_ident } = &heap;
    let config = match Config::read_from_cargo_manifest_dir() {
        Ok(config) => config,
        Err(err) => parse_error!("{}: {}", drone_config::CONFIG_NAME, err),
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

    let global_alloc = match global {
        Some(LitBool { value, .. }) if value => Some(def_global_alloc(&heap)),
        _ => None,
    };
    let allocator = def_allocator(&heap, pools_len);
    let alloc_ref = def_alloc_ref(&heap);

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

        #allocator
        #alloc_ref
        #global_alloc
    };
    expanded.into()
}

fn def_allocator(heap: &Heap, pools_len: usize) -> TokenStream2 {
    let Heap { ident: heap_ident, .. } = heap;
    quote! {
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
    }
}

fn def_alloc_ref(heap: &Heap) -> TokenStream2 {
    let Heap { ident: heap_ident, .. } = heap;
    quote! {
        unsafe impl ::core::alloc::AllocRef for #heap_ident {
            fn alloc(
                &self,
                layout: ::core::alloc::Layout,
            ) -> ::core::result::Result<
                ::core::ptr::NonNull<[u8]>,
                ::core::alloc::AllocError,
            > {
                ::drone_core::heap::alloc(self, layout)
            }

            fn alloc_zeroed(
                &self,
                layout: ::core::alloc::Layout,
            ) -> ::core::result::Result<
                ::core::ptr::NonNull<[u8]>,
                ::core::alloc::AllocError,
            > {
                ::drone_core::heap::alloc_zeroed(self, layout)
            }

            unsafe fn dealloc(
                &self,
                ptr: ::core::ptr::NonNull<u8>,
                layout: ::core::alloc::Layout,
            ) {
                ::drone_core::heap::dealloc(self, ptr, layout)
            }

            unsafe fn grow(
                &self,
                ptr: ::core::ptr::NonNull<u8>,
                old_layout: ::core::alloc::Layout,
                new_layout: ::core::alloc::Layout,
            ) -> ::core::result::Result<
                ::core::ptr::NonNull<[u8]>,
                ::core::alloc::AllocError,
            > {
                ::drone_core::heap::grow(self, ptr, old_layout, new_layout)
            }

            unsafe fn grow_zeroed(
                &self,
                ptr: ::core::ptr::NonNull<u8>,
                old_layout: ::core::alloc::Layout,
                new_layout: ::core::alloc::Layout,
            ) -> ::core::result::Result<
                ::core::ptr::NonNull<[u8]>,
                ::core::alloc::AllocError,
            > {
                ::drone_core::heap::grow_zeroed(self, ptr, old_layout, new_layout)
            }

            unsafe fn shrink(
                &self,
                ptr: ::core::ptr::NonNull<u8>,
                old_layout: ::core::alloc::Layout,
                new_layout: ::core::alloc::Layout,
            ) -> ::core::result::Result<
                ::core::ptr::NonNull<[u8]>,
                ::core::alloc::AllocError,
            > {
                ::drone_core::heap::shrink(self, ptr, old_layout, new_layout)
            }
        }
    }
}

fn def_global_alloc(heap: &Heap) -> TokenStream2 {
    let Heap { ident: heap_ident, .. } = heap;
    quote! {
        unsafe impl ::core::alloc::GlobalAlloc for #heap_ident {
            unsafe fn alloc(&self, layout: ::core::alloc::Layout) -> *mut u8 {
                ::drone_core::heap::alloc(self, layout)
                    .map(|ptr| ptr.as_mut_ptr())
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
    }
}
