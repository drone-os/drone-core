use drone_config::{Layout, LAYOUT_CONFIG};
use drone_macros_core::parse_error;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Attribute, Ident, LitBool, LitStr, Token, Visibility};

struct Input {
    layout: Ident,
    metadata: Metadata,
    instance: Instance,
    global: bool,
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
        let mut global = None;
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
            } else if attrs.is_empty() && ident == "global" {
                if global.is_none() {
                    global = Some(input.parse::<LitBool>()?.value);
                } else {
                    return Err(input.error("multiple `global` specifications"));
                }
            } else {
                return Err(input.error(format!("unknown key: `{ident}`")));
            }
            if !input.is_empty() {
                input.parse::<Token![;]>()?;
            }
        }
        Ok(Self {
            layout: layout.ok_or_else(|| input.error("missing `layout` specification"))?,
            metadata: metadata.ok_or_else(|| input.error("missing `metadata` specification"))?,
            instance: instance.ok_or_else(|| input.error("missing `instance` specification"))?,
            global: global.unwrap_or(false),
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
    let Input { layout: stream_layout, metadata, instance, global } = parse_macro_input!(input);
    let Metadata { attrs: metadata_attrs, vis: metadata_vis, ident: metadata_ident } = &metadata;
    let Instance { attrs: instance_attrs, vis: instance_vis, ident: instance_ident } = &instance;
    let layout = match Layout::read_from_cargo() {
        Ok(layout) => layout,
        Err(err) => parse_error!("{err:#?}"),
    };
    let stream = match layout
        .stream
        .as_ref()
        .and_then(|stream| stream.sections.get(&stream_layout.to_string()))
    {
        Some(stream) => stream,
        None => {
            parse_error!("Couldn't find stream.{stream_layout} in {LAYOUT_CONFIG}");
        }
    };
    let buffer_size = stream.size;
    let init_primary = stream.init_primary.unwrap_or(false);
    let init_ident =
        if init_primary { format_ident!("init_primary") } else { format_ident!("init") };
    let section = LitStr::new(&format!(".stream_{stream_layout}_rt"), Span::call_site());
    let global = global.then(|| def_global(&instance));

    quote! {
        #(#metadata_attrs)*
        #[repr(C)]
        #metadata_vis struct #metadata_ident {
            /// Drone Stream runtime structure.
            pub runtime: ::drone_core::_rt::drone_stream::Runtime,
        }

        #(#instance_attrs)*
        #[link_section = #section]
        #instance_vis static #instance_ident: ::core::cell::SyncUnsafeCell<#metadata_ident> =
            ::core::cell::SyncUnsafeCell::new(#metadata_ident::zeroed());

        impl #metadata_ident {
            /// Creates a new zeroed Drone Stream runtime.
            #[must_use]
            #[inline]
            pub const fn zeroed() -> Self {
                Self { runtime: ::drone_core::_rt::drone_stream::Runtime::zeroed() }
            }

            /// Initializes this Drone Stream runtime.
            ///
            /// # Safety
            ///
            /// This function may corrupt any on-going transmissions.
            #[inline]
            pub unsafe fn #init_ident() {
                unsafe {
                    ::drone_core::stream::init(
                        ::core::ptr::addr_of_mut!((*#instance_ident.get()).runtime),
                        #buffer_size,
                        #init_primary,
                    );
                }
            }
        }

        #global
    }
    .into()
}

fn def_global(instance: &Instance) -> TokenStream2 {
    let Instance { ident: instance_ident, .. } = instance;
    quote! {
        #[no_mangle]
        extern "C" fn drone_stream_runtime() -> *mut ::drone_core::_rt::drone_stream::Runtime {
            unsafe { ::core::ptr::addr_of_mut!((*#instance_ident.get()).runtime) }
        }
    }
}
