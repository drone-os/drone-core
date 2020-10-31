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
        let mut array = None;
        let mut thr = None;
        let mut local = None;
        while !input.is_empty() {
            let attrs = input.call(Attribute::parse_outer)?;
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![=>]>()?;
            if attrs.is_empty() && ident == "array" {
                if array.is_none() {
                    array = Some(input.parse()?);
                } else {
                    return Err(input.error("multiple `array` specifications"));
                }
            } else if ident == "thread" {
                if thr.is_none() {
                    thr = Some(Thr::parse(input, attrs)?);
                } else {
                    return Err(input.error("multiple `thread` specifications"));
                }
            } else if ident == "local" {
                if local.is_none() {
                    local = Some(Local::parse(input, attrs)?);
                } else {
                    return Err(input.error("multiple `local` specifications"));
                }
            } else {
                return Err(input.error(format!("unknown key: `{}`", ident)));
            }
            if !input.is_empty() {
                input.parse::<Token![;]>()?;
            }
        }
        Ok(Self {
            array: array.ok_or_else(|| input.error("missing `array` specification"))?,
            thr: thr.ok_or_else(|| input.error("missing `thread` specification"))?,
            local: local.ok_or_else(|| input.error("missing `local` specification"))?,
        })
    }
}

impl Parse for Array {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let path = input.parse()?;
        Ok(Self { path })
    }
}

impl Thr {
    fn parse(input: ParseStream<'_>, attrs: Vec<Attribute>) -> Result<Self> {
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

impl Local {
    fn parse(input: ParseStream<'_>, attrs: Vec<Attribute>) -> Result<Self> {
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
    let Local { ident: local_ident, .. } = &local;
    let local_wrapper = format_ident!("{}Wrapper", local_ident);
    let def_thr = def_thr(&array, &thr, &local, &local_wrapper);
    let def_local = def_local(&local, &local_wrapper);
    let expanded = quote! {
        #def_thr
        #def_local
    };
    expanded.into()
}

fn def_thr(array: &Array, thr: &Thr, local: &Local, local_wrapper: &Ident) -> TokenStream2 {
    let Array { path: array } = array;
    let Thr { vis: thr_vis, attrs: thr_attrs, ident: thr_ident, fields: thr_fields } = thr;
    let Local { ident: local_ident, .. } = local;
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
            local: #local_wrapper,
            #(#thr_tokens,)*
        }

        impl #thr_ident {
            /// Creates a new thread object with given `index`.
            pub const fn new(index: usize) -> Self {
                Self {
                    fib_chain: ::drone_core::fib::Chain::new(),
                    local: #local_wrapper(#local_ident::new(index)),
                    #(#thr_ctor_tokens,)*
                }
            }
        }

        impl ::drone_core::thr::Thread for #thr_ident {
            type Local = #local_ident;

            #[inline]
            fn first() -> *const Self {
                unsafe { #array.as_ptr() }
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

fn def_local(local: &Local, local_wrapper: &Ident) -> TokenStream2 {
    let Local { vis: local_vis, attrs: local_attrs, ident: local_ident, fields: local_fields } =
        &local;
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

        struct #local_wrapper(#local_ident);

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

        unsafe impl Sync for #local_wrapper {}
    }
}
