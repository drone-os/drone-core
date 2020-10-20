use drone_macros_core::parse_ident;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Expr, ExprPath, Ident, Token, Type, Visibility,
};

struct Input {
    array: Array,
    thr: Thr,
    local: Local,
}

struct Array {
    path: ExprPath,
}

struct Thr {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    fields: Vec<Field>,
}

struct Local {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    fields: Vec<Field>,
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
        let array = input.parse()?;
        input.parse::<Token![;]>()?;
        let thr = input.parse()?;
        input.parse::<Token![;]>()?;
        let local = input.parse()?;
        if !input.is_empty() {
            input.parse::<Token![;]>()?;
        }
        Ok(Self { array, thr, local })
    }
}

impl Parse for Array {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        parse_ident!(input, "array");
        input.parse::<Token![=>]>()?;
        let path = input.parse()?;
        Ok(Self { path })
    }
}

impl Parse for Thr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        parse_ident!(input, "thread");
        input.parse::<Token![=>]>()?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut fields = Vec::new();
        while !content.is_empty() {
            fields.push(content.parse()?);
            if !content.is_empty() {
                content.parse::<Token![;]>()?;
            }
        }
        Ok(Self { attrs, vis, ident, fields })
    }
}

impl Parse for Local {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        parse_ident!(input, "local");
        input.parse::<Token![=>]>()?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut fields = Vec::new();
        while !content.is_empty() {
            fields.push(content.parse()?);
            if !content.is_empty() {
                content.parse::<Token![;]>()?;
            }
        }
        Ok(Self { attrs, vis, ident, fields })
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
        Ok(Self { attrs, vis, ident, ty, init })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { array, thr, local } = parse_macro_input!(input);
    let def_thr = def_thr(&array, &thr, &local);
    let def_local = def_local(&local);
    let expanded = quote! {
        mod __thr {
            #[allow(unused_imports)]
            use super::*;
            #def_thr
            #def_local
        }
        #[allow(unused_imports)]
        pub use self::__thr::*;
    };
    expanded.into()
}

fn def_thr(array: &Array, thr: &Thr, local: &Local) -> TokenStream2 {
    let Array { path: array } = array;
    let Thr { vis: thr_vis, attrs: thr_attrs, ident: thr_ident, fields: thr_fields } = thr;
    let Local { ident: local_ident, .. } = local;
    let local = format_ident!("Local");
    let mut thr_tokens = Vec::new();
    let mut thr_ctor_tokens = Vec::new();
    for Field { attrs, vis, ident, ty, init } in thr_fields {
        thr_tokens.push(quote!(#(#attrs)* #vis #ident: #ty));
        thr_ctor_tokens.push(quote!(#ident: #init));
    }
    quote! {
        #(#thr_attrs)*
        #thr_vis struct #thr_ident {
            fib_chain: ::drone_core::fib::Chain,
            local: #local,
            #(#thr_tokens,)*
        }

        impl #thr_ident {
            /// Creates a new thread object with given `index`.
            pub const fn new(index: usize) -> Self {
                Self {
                    fib_chain: ::drone_core::fib::Chain::new(),
                    local: #local(#local_ident::new(index)),
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
    }
}

fn def_local(local: &Local) -> TokenStream2 {
    let Local { vis: local_vis, attrs: local_attrs, ident: local_ident, fields: local_fields } =
        &local;
    let local = format_ident!("Local");
    let mut local_tokens = Vec::new();
    let mut local_ctor_tokens = Vec::new();
    for Field { attrs, vis, ident, ty, init } in local_fields {
        local_tokens.push(quote!(#(#attrs)* #vis #ident: #ty));
        local_ctor_tokens.push(quote!(#ident: #init));
    }
    quote! {
        #(#local_attrs)*
        #local_vis struct #local_ident {
            preempted: ::drone_core::thr::PreemptedCell,
            #(#local_tokens,)*
        }

        struct #local(#local_ident);

        impl #local_ident {
            const fn new(index: usize) -> Self {
                Self {
                    preempted: ::drone_core::thr::PreemptedCell::new(),
                    #(#local_ctor_tokens,)*
                }
            }
        }

        impl ::drone_core::thr::ThreadLocal for #local_ident {
            #[inline]
            fn preempted(&self) -> &::drone_core::thr::PreemptedCell {
                &self.preempted
            }
        }

        unsafe impl Sync for #local {}
    }
}
