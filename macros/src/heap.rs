use drone_config::{Layout, LAYOUT_CONFIG};
use drone_macros_core::parse_error;
use heck::ToShoutySnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::{format_ident, quote};
use std::iter;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Attribute, Ident, LitInt, LitStr, Token, Visibility};

struct Input {
    layout: Ident,
    metadata: Metadata,
    instance: Instance,
    trace_stream: Option<LitInt>,
}

struct Metadata {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

struct Instance {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut layout = None;
        let mut metadata = None;
        let mut instance = None;
        let mut trace_stream = None;
        while !input.is_empty() {
            let attrs = input.call(Attribute::parse_outer)?;
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![=>]>()?;
            if attrs.is_empty() && ident == "layout" {
                if layout.is_none() {
                    layout = Some(input.parse()?);
                } else {
                    return Err(input.error("multiple `layout` specifications"));
                }
            } else if ident == "metadata" {
                if metadata.is_none() {
                    metadata = Some(Metadata::parse(input, attrs)?);
                } else {
                    return Err(input.error("multiple `metadata` specifications"));
                }
            } else if ident == "instance" {
                if instance.is_none() {
                    instance = Some(Instance::parse(input, attrs)?);
                } else {
                    return Err(input.error("multiple `instance` specifications"));
                }
            } else if attrs.is_empty() && ident == "enable_trace_stream" {
                if trace_stream.is_none() {
                    trace_stream = Some(input.parse()?);
                } else {
                    return Err(input.error("multiple `trace_stream` specifications"));
                }
            } else {
                return Err(input.error(format!("unknown key: `{}`", ident)));
            }
            if !input.is_empty() {
                input.parse::<Token![;]>()?;
            }
        }
        Ok(Self {
            layout: layout.ok_or_else(|| input.error("missing `layout` specification"))?,
            metadata: metadata.ok_or_else(|| input.error("missing `metadata` specification"))?,
            instance: instance.ok_or_else(|| input.error("missing `instance` specification"))?,
            trace_stream,
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

impl Instance {
    fn parse(input: ParseStream<'_>, attrs: Vec<Attribute>) -> Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;
        Ok(Self { attrs, vis, ident })
    }
}

#[allow(clippy::too_many_lines)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { layout: heap_layout, metadata, instance, trace_stream } = parse_macro_input!(input);
    let Metadata { attrs: metadata_attrs, vis: metadata_vis, ident: metadata_ident } = &metadata;
    let Instance { attrs: instance_attrs, vis: instance_vis, ident: instance_ident } = &instance;
    let layout = match Layout::read_from_cargo() {
        Ok(layout) => layout,
        Err(err) => parse_error!("{err:#?}"),
    };

    let pools = match layout.heap.get(&heap_layout.to_string()) {
        Some(heap) => &heap.pools,
        None => parse_error!("Couldn't find heap.{heap_layout} in {LAYOUT_CONFIG}"),
    };
    let heap_layout_shouty_snk = heap_layout.to_string().to_shouty_snake_case();
    let heap_rt_load = format_ident!("HEAP_{}_RT_LOAD", heap_layout_shouty_snk);
    let heap_rt_base = format_ident!("HEAP_{}_RT_BASE", heap_layout_shouty_snk);
    let heap_rt_end = format_ident!("HEAP_{}_RT_END", heap_layout_shouty_snk);
    let section = LitStr::new(&format!(".heap_{heap_layout}_rt"), Span::call_site());
    let pools_len = pools.len();
    let pools_tokens = iter::repeat(quote! {
        // Actual parameters will be set by drone-ld.
        ::drone_core::heap::Pool::new(0, 0, 0),
    })
    .take(pools_len)
    .collect::<Vec<_>>();

    let drone_alloc = def_drone_alloc(&metadata, trace_stream, pools_len);
    let core_alloc = def_core_alloc(&metadata);
    let global_alloc = instance_attrs
        .clone()
        .into_iter()
        .any(|attr| {
            fn any_global_alloc(stream: TokenStream2) -> bool {
                stream.into_iter().any(|tt| match tt {
                    TokenTree2::Group(group) => any_global_alloc(group.stream()),
                    TokenTree2::Ident(ident) => ident == "global_allocator",
                    _ => false,
                })
            }
            attr.path.get_ident().map_or(false, |ident| ident == "global_allocator")
                || any_global_alloc(attr.tokens)
        })
        .then(|| def_global_alloc(&metadata));

    quote! {
        #(#metadata_attrs)*
        #metadata_vis struct #metadata_ident {
            pools: [::drone_core::heap::Pool; #pools_len],
        }

        #(#instance_attrs)*
        #[link_section = #section]
        #instance_vis static #instance_ident: #metadata_ident = #metadata_ident::new();

        impl #metadata_ident {
            /// Creates a instance of this new heap metadata.
            pub const fn new() -> Self {
                Self {
                    pools: [
                        #(#pools_tokens)*
                    ],
                }
            }

            /// Initializes this heap metadata.
            ///
            /// This function **must** be called as early as possible.
            ///
            /// # Safety
            ///
            /// This function reverts the state of the heap.
            pub unsafe fn init() {
                extern "C" {
                    static #heap_rt_load: ::core::cell::UnsafeCell<usize>;
                    static #heap_rt_base: ::core::cell::UnsafeCell<usize>;
                    static #heap_rt_end: ::core::cell::UnsafeCell<usize>;
                }
                unsafe {
                    ::drone_core::mem::data_section_init(
                        &#heap_rt_load,
                        &#heap_rt_base,
                        &#heap_rt_end,
                    );
                }
            }
        }

        #drone_alloc
        #core_alloc
        #global_alloc
    }
    .into()
}

fn def_drone_alloc(
    metadata: &Metadata,
    trace_stream: Option<LitInt>,
    pools_len: usize,
) -> TokenStream2 {
    let Metadata { ident: metadata_ident, .. } = metadata;
    let trace_stream = if let Some(trace_stream) = trace_stream {
        quote!(::core::option::Option::Some(#trace_stream))
    } else {
        quote!(::core::option::Option::None)
    };
    quote! {
        impl ::drone_core::heap::Allocator for #metadata_ident {
            const POOLS_COUNT: usize = #pools_len;
            const TRACE_STREAM: ::core::option::Option<u8> = #trace_stream;

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

fn def_core_alloc(metadata: &Metadata) -> TokenStream2 {
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
                ::drone_core::heap::allocate(self, layout)
            }

            fn allocate_zeroed(
                &self,
                layout: ::core::alloc::Layout,
            ) -> ::core::result::Result<
                ::core::ptr::NonNull<[u8]>,
                ::core::alloc::AllocError,
            > {
                ::drone_core::heap::allocate_zeroed(self, layout)
            }

            unsafe fn deallocate(
                &self,
                ptr: ::core::ptr::NonNull<u8>,
                layout: ::core::alloc::Layout,
            ) {
                ::drone_core::heap::deallocate(self, ptr, layout)
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
                ::drone_core::heap::allocate(self, layout)
                    .map(|ptr| ptr.as_mut_ptr())
                    .unwrap_or(::core::ptr::null_mut())
            }

            unsafe fn dealloc(&self, ptr: *mut u8, layout: ::core::alloc::Layout) {
                ::drone_core::heap::deallocate(
                    self,
                    ::core::ptr::NonNull::new_unchecked(ptr),
                    layout,
                )
            }
        }
    }
}
