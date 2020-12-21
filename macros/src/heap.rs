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
    config: Ident,
    metadata: Metadata,
    trace_port: Option<LitInt>,
    global: Option<LitBool>,
}

struct Metadata {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut config = None;
        let mut metadata = None;
        let mut trace_port = None;
        let mut global = None;
        while !input.is_empty() {
            let attrs = input.call(Attribute::parse_outer)?;
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![=>]>()?;
            if attrs.is_empty() && ident == "config" {
                if config.is_none() {
                    config = Some(input.parse()?);
                } else {
                    return Err(input.error("multiple `config` specifications"));
                }
            } else if ident == "metadata" {
                if metadata.is_none() {
                    metadata = Some(Metadata::parse(input, attrs)?);
                } else {
                    return Err(input.error("multiple `metadata` specifications"));
                }
            } else if attrs.is_empty() && ident == "trace_port" {
                if trace_port.is_none() {
                    trace_port = Some(input.parse()?);
                } else {
                    return Err(input.error("multiple `trace_port` specifications"));
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
        Ok(Self {
            config: config.ok_or_else(|| input.error("missing `config` specification"))?,
            metadata: metadata.ok_or_else(|| input.error("missing `metadata` specification"))?,
            trace_port,
            global,
        })
    }
}

impl Metadata {
    fn parse(input: ParseStream<'_>, attrs: Vec<Attribute>) -> Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;
        Ok(Self { attrs, vis, ident })
    }
}

#[allow(clippy::too_many_lines)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { config: heap_config, metadata, trace_port, global } = parse_macro_input!(input);
    let Metadata { attrs: metadata_attrs, vis: metadata_vis, ident: metadata_ident } = &metadata;
    let mut config = match Config::read_from_cargo_manifest_dir() {
        Ok(config) => config,
        Err(err) => parse_error!("{}: {}", drone_config::CONFIG_NAME, err),
    };

    let (mut pointer, pools) = if heap_config == "main" {
        (
            config.memory.ram.origin + config.memory.ram.size - config.heap.main.size,
            &mut config.heap.main.pools,
        )
    } else {
        match config.heap.extra.get_mut(&heap_config.to_string()) {
            Some(heap) => (heap.origin, &mut heap.block.pools),
            None => {
                parse_error!(
                    "Missing `{}` heap configuration in {}",
                    heap_config,
                    drone_config::CONFIG_NAME
                )
            }
        }
    };

    pools.sort_by_key(|pool| pool.block);
    let mut pools_tokens = Vec::new();
    for pool in pools.iter() {
        let block = LitInt::new(&pool.block.to_string(), Span::call_site());
        let capacity = LitInt::new(&pool.capacity.to_string(), Span::call_site());
        let address = LitInt::new(&pointer.to_string(), Span::call_site());
        pools_tokens.push(quote! {
            ::drone_core::heap::Pool::new(#address, #block, #capacity)
        });
        pointer += pool.block * pool.capacity;
    }
    let pools_len = pools.len();

    let allocator = def_allocator(&metadata, trace_port, pools_len);
    let alloc_ref = def_alloc_ref(&metadata);
    let global_alloc = match global {
        Some(LitBool { value, .. }) if value => Some(def_global_alloc(&metadata)),
        _ => None,
    };

    let expanded = quote! {
        #(#metadata_attrs)*
        #metadata_vis struct #metadata_ident {
            pools: [::drone_core::heap::Pool; #pools_len],
        }

        impl #metadata_ident {
            /// Creates a new metadata.
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

fn def_allocator(
    metadata: &Metadata,
    trace_port: Option<LitInt>,
    pools_len: usize,
) -> TokenStream2 {
    let Metadata { ident: metadata_ident, .. } = metadata;
    let trace_port =
        if let Some(trace_port) = trace_port { quote!(Some(#trace_port)) } else { quote!(None) };
    quote! {
        impl ::drone_core::heap::Allocator for #metadata_ident {
            const POOL_COUNT: usize = #pools_len;
            const TRACE_PORT: Option<u8> = #trace_port;

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

fn def_alloc_ref(metadata: &Metadata) -> TokenStream2 {
    let Metadata { ident: metadata_ident, .. } = metadata;
    quote! {
        unsafe impl ::core::alloc::Allocator for #metadata_ident {
            fn allocate(
                &self,
                layout: ::core::alloc::Layout,
            ) -> ::core::result::Result<
                ::core::ptr::NonNull<[u8]>,
                ::core::alloc::AllocError,
            > {
                ::drone_core::heap::alloc(self, layout)
            }

            unsafe fn deallocate(
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

fn def_global_alloc(metadata: &Metadata) -> TokenStream2 {
    let Metadata { ident: metadata_ident, .. } = metadata;
    quote! {
        unsafe impl ::core::alloc::GlobalAlloc for #metadata_ident {
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
