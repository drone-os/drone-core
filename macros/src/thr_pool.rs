use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::{
    braced, parse_macro_input, Attribute, Expr, ExprPath, Ident, LitInt, Token, Type, Visibility,
};

struct Input {
    thr: Thr,
    local: Local,
    index: Index,
    threads: Threads,
    resume: Option<ExprPath>,
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

struct Index {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

struct Threads {
    threads: Vec<Thread>,
}

struct Thread {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut thr = None;
        let mut local = None;
        let mut index = None;
        let mut threads = None;
        let mut resume = None;
        while !input.is_empty() {
            let attrs = input.call(Attribute::parse_outer)?;
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![=>]>()?;
            if ident == "thread" {
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
            } else if ident == "index" {
                if index.is_none() {
                    index = Some(Index::parse(input, attrs)?);
                } else {
                    return Err(input.error("multiple `index` specifications"));
                }
            } else if attrs.is_empty() && ident == "threads" {
                if threads.is_none() {
                    threads = Some(input.parse()?);
                } else {
                    return Err(input.error("multiple `threads` specifications"));
                }
            } else if attrs.is_empty() && ident == "resume" {
                if resume.is_none() {
                    resume = Some(input.parse()?);
                } else {
                    return Err(input.error("multiple `resume` specifications"));
                }
            } else {
                return Err(input.error(format!("unknown key: `{}`", ident)));
            }
            if !input.is_empty() {
                input.parse::<Token![;]>()?;
            }
        }
        Ok(Self {
            thr: thr.ok_or_else(|| input.error("missing `thread` specification"))?,
            local: local.ok_or_else(|| input.error("missing `local` specification"))?,
            index: index.ok_or_else(|| input.error("missing `index` specification"))?,
            threads: threads.ok_or_else(|| input.error("missing `threads` specification"))?,
            resume,
        })
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

impl Index {
    fn parse(input: ParseStream<'_>, attrs: Vec<Attribute>) -> Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;
        Ok(Self { attrs, vis, ident })
    }
}

impl Parse for Threads {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let input2;
        braced!(input2 in input);
        let mut threads = Vec::new();
        while !input2.is_empty() {
            let attrs = input2.call(Attribute::parse_outer)?;
            let vis = input2.parse()?;
            let ident = input2.parse()?;
            threads.push(Thread { attrs, vis, ident });
            if !input2.is_empty() {
                input2.parse::<Token![;]>()?;
            }
        }
        Ok(Self { threads })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { thr, local, index, threads, resume } = parse_macro_input!(input);
    let Threads { threads } = threads;
    let def_thr = def_thr(&thr, &threads, &local, resume.as_ref());
    let def_local = def_local(&local);
    let def_index = def_index(&thr, &index, &threads);
    quote! {
        #def_thr
        #def_local
        #def_index
    }
    .into()
}

fn def_thr(
    thr: &Thr,
    threads: &[Thread],
    local: &Local,
    resume: Option<&ExprPath>,
) -> TokenStream2 {
    let Thr { vis: thr_vis, attrs: thr_attrs, ident: thr_ident, fields: thr_fields } = thr;
    let Local { ident: local_ident, .. } = local;
    let resume = resume.map(|resume| {
        quote! {
            #[inline]
            unsafe fn resume(&self) {
                unsafe { #resume(self) };
            }
        }
    });
    let count = LitInt::new(&format!("{}_u16", threads.len()), Span::call_site());
    let mut threads_tokens = Vec::new();
    for idx in 0..threads.len() {
        let idx = LitInt::new(&format!("{}_u16", idx), Span::call_site());
        threads_tokens.push(quote! {
            #thr_ident::new(#idx)
        });
    }
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
            local: ::drone_core::thr::LocalOpaque<Self>,
            #(#thr_tokens,)*
        }

        impl #thr_ident {
            /// Creates a new thread object with given `index`.
            pub const fn new(index: u16) -> Self {
                Self {
                    fib_chain: ::drone_core::fib::Chain::new(),
                    local: ::drone_core::thr::LocalOpaque::new(#local_ident::new(index)),
                    #(#thr_ctor_tokens,)*
                }
            }
        }

        unsafe impl ::drone_core::thr::Thread for #thr_ident {
            type Local = #local_ident;

            const COUNT: u16 = #count;

            #[inline]
            fn pool() -> *const Self {
                static THREADS: [#thr_ident; #count as usize] = [#(#threads_tokens),*];
                THREADS.as_ptr()
            }

            #[inline]
            fn current() -> *const ::core::sync::atomic::AtomicU16 {
                static CURRENT: ::core::sync::atomic::AtomicU16 =
                    ::core::sync::atomic::AtomicU16::new(0);
                &CURRENT
            }

            #[inline]
            fn fib_chain(&self) -> &::drone_core::fib::Chain {
                &self.fib_chain
            }

            #[inline]
            fn local_opaque(&self) -> &::drone_core::thr::LocalOpaque<Self> {
                &self.local
            }

            #resume
        }
    }
}

fn def_local(local: &Local) -> TokenStream2 {
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
            #(#local_tokens,)*
        }

        impl #local_ident {
            const fn new(index: u16) -> Self {
                Self {
                    #(#local_ctor_tokens,)*
                }
            }
        }
    }
}

fn def_index(thr: &Thr, index: &Index, threads: &[Thread]) -> TokenStream2 {
    let Index { attrs: index_attrs, vis: index_vis, ident: index_ident } = index;
    let mut tokens = Vec::new();
    let mut index_tokens = Vec::new();
    let mut index_ctor_tokens = Vec::new();
    for (idx, thread) in threads.iter().enumerate() {
        let thr_token = def_thr_token(thr, idx, thread);
        tokens.push(thr_token.0);
        index_tokens.push(thr_token.1);
        index_ctor_tokens.push(thr_token.2);
    }
    quote! {
        #(#index_attrs)*
        #index_vis struct #index_ident {
            #(#index_tokens),*
        }

        unsafe impl ::drone_core::token::Token for #index_ident {
            #[inline]
            unsafe fn take() -> Self {
                Self {
                    #(#index_ctor_tokens),*
                }
            }
        }

        #(#tokens)*
    }
}

fn def_thr_token(
    thr: &Thr,
    idx: usize,
    thread: &Thread,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let Thr { ident: thr_ident, .. } = thr;
    let Thread { attrs, vis, ident } = thread;
    let mut tokens = Vec::new();
    let field_ident = format_ident!("{}", ident);
    let struct_ident = format_ident!("{}", ident.to_string().to_upper_camel_case());
    let idx = LitInt::new(&format!("{}_u16", idx), Span::call_site());
    tokens.push(quote! {
        #(#attrs)*
        #[derive(Clone, Copy)]
        #vis struct #struct_ident {
            __priv: (),
        }

        unsafe impl ::drone_core::token::Token for #struct_ident {
            #[inline]
            unsafe fn take() -> Self {
                #struct_ident {
                    __priv: (),
                }
            }
        }

        unsafe impl ::drone_core::thr::ThrToken for #struct_ident {
            type Thread = #thr_ident;

            const THR_IDX: u16 = #idx;
        }
    });
    (
        quote!(#(#tokens)*),
        quote! {
            #(#attrs)*
            #vis #field_ident: #struct_ident
        },
        quote! {
            #field_ident: ::drone_core::token::Token::take()
        },
    )
}
