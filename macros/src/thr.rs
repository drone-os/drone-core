use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Expr, ExprPath, Ident, Token, Type, Visibility,
};

struct Input {
    array: ExprPath,
    thr_attrs: Vec<Attribute>,
    thr_vis: Visibility,
    thr_ident: Ident,
    thr_fields: Vec<Field>,
    local_attrs: Vec<Attribute>,
    local_vis: Visibility,
    local_ident: Ident,
    local_fields: Vec<Field>,
}

struct Field {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    ty: Type,
    init: Expr,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        input.parse::<Token![use]>()?;
        let array = input.parse()?;
        input.parse::<Token![;]>()?;
        let thr_attrs = input.call(Attribute::parse_outer)?;
        let thr_vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let thr_ident = input.parse()?;
        let thr_content;
        braced!(thr_content in input);
        let mut thr_fields = Vec::new();
        while !thr_content.is_empty() {
            thr_fields.push(thr_content.parse()?);
        }
        let local_attrs = input.call(Attribute::parse_outer)?;
        let local_vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let local_ident = input.parse()?;
        let local_content;
        braced!(local_content in input);
        let mut local_fields = Vec::new();
        while !local_content.is_empty() {
            local_fields.push(local_content.parse()?);
        }
        Ok(Self {
            array,
            thr_attrs,
            thr_vis,
            thr_ident,
            thr_fields,
            local_attrs,
            local_vis,
            local_ident,
            local_fields,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        input.parse::<Token![=]>()?;
        let init = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self { attrs, vis, ident, ty, init })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input {
        array,
        thr_attrs,
        thr_vis,
        thr_ident,
        thr_fields,
        local_attrs,
        local_vis,
        local_ident,
        local_fields,
    } = parse_macro_input!(input);
    let local = format_ident!("Local");
    let mut thr_tokens = Vec::new();
    let mut thr_ctor_tokens = Vec::new();
    let mut local_tokens = Vec::new();
    let mut local_ctor_tokens = Vec::new();
    for Field { attrs, vis, ident, ty, init } in thr_fields {
        thr_tokens.push(quote!(#(#attrs)* #vis #ident: #ty));
        thr_ctor_tokens.push(quote!(#ident: #init));
    }
    for Field { attrs, vis, ident, ty, init } in local_fields {
        local_tokens.push(quote!(#(#attrs)* #vis #ident: #ty));
        local_ctor_tokens.push(quote!(#ident: #init));
    }

    let expanded = quote! {
        mod __thr {
            use super::*;

            #(#thr_attrs)*
            pub struct #thr_ident {
                fib_chain: ::drone_core::fib::Chain,
                local: #local,
                #(#thr_tokens,)*
            }

            #(#local_attrs)*
            pub struct #local_ident {
                task: ::drone_core::thr::TaskCell,
                preempted: ::drone_core::thr::PreemptedCell,
                #(#local_tokens,)*
            }

            struct #local(#local_ident);

            impl #thr_ident {
                /// Creates a new thread object with given `index`.
                pub const fn new(index: usize) -> Self {
                    Self {
                        fib_chain: ::drone_core::fib::Chain::new(),
                        local: #local(#local_ident {
                            task: ::drone_core::thr::TaskCell::new(),
                            preempted: ::drone_core::thr::PreemptedCell::new(),
                            #(#local_ctor_tokens,)*
                        }),
                        #(#thr_ctor_tokens,)*
                    }
                }
            }

            impl ::drone_core::thr::Thread for #thr_ident {
                type Local = #local_ident;

                #[inline]
                fn first() -> *const Self {
                    unsafe { super::#array.as_ptr() }
                }

                #[inline]
                fn fib_chain(&self) -> &::drone_core::fib::Chain {
                    &self.fib_chain
                }

                #[inline]
                unsafe fn local(&self) -> &#local_ident {
                    &self.local.0
                }
            }

            impl ::drone_core::thr::ThreadLocal for #local_ident {
                #[inline]
                fn task(&self) -> &::drone_core::thr::TaskCell {
                    &self.task
                }

                #[inline]
                fn preempted(&self) -> &::drone_core::thr::PreemptedCell {
                    &self.preempted
                }
            }

            unsafe impl Sync for #local {}
        }

        #thr_vis use self::__thr::#thr_ident;
        #local_vis use self::__thr::#local_ident;
    };
    expanded.into()
}
