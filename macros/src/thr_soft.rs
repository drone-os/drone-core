use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parse_macro_input, Attribute, ExprPath, Ident, Token, Visibility};

struct Input {
    thr: Thr,
    local: Local,
    index: Index,
    threads: Threads,
    resume: Option<ExprPath>,
    set_pending: Option<ExprPath>,
}

struct Thr {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    tokens: TokenStream2,
}

struct Local {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    tokens: TokenStream2,
}

struct Index {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

struct Threads {
    tokens: TokenStream2,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut thr = None;
        let mut local = None;
        let mut index = None;
        let mut threads = None;
        let mut resume = None;
        let mut set_pending = None;
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
            } else if attrs.is_empty() && ident == "set_pending" {
                if set_pending.is_none() {
                    set_pending = Some(input.parse()?);
                } else {
                    return Err(input.error("multiple `set_pending` specifications"));
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
            set_pending,
        })
    }
}

impl Thr {
    fn parse(input: ParseStream<'_>, attrs: Vec<Attribute>) -> Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;
        let input2;
        braced!(input2 in input);
        let tokens = input2.parse()?;
        Ok(Self { attrs, vis, ident, tokens })
    }
}

impl Local {
    fn parse(input: ParseStream<'_>, attrs: Vec<Attribute>) -> Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;
        let input2;
        braced!(input2 in input);
        let tokens = input2.parse()?;
        Ok(Self { attrs, vis, ident, tokens })
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
        let tokens = input2.parse()?;
        Ok(Self { tokens })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { thr, local, index, threads, resume, set_pending } = parse_macro_input!(input);
    let def_pool = def_pool(&thr, &local, &index, &threads, resume.as_ref());
    let def_soft = def_soft(&thr, set_pending.as_ref());

    quote! {
        #def_pool
        #def_soft
    }
    .into()
}

fn def_pool(
    thr: &Thr,
    local: &Local,
    index: &Index,
    threads: &Threads,
    resume: Option<&ExprPath>,
) -> TokenStream2 {
    let Thr { attrs: thr_attrs, vis: thr_vis, ident: thr_ident, tokens: thr_tokens } = thr;
    let Local { attrs: local_attrs, vis: local_vis, ident: local_ident, tokens: local_tokens } =
        local;
    let Index { attrs: index_attrs, vis: index_vis, ident: index_ident } = index;
    let Threads { tokens: threads_tokens } = threads;
    let resume = resume.into_iter();

    quote! {
        ::drone_core::thr::pool! {
            #(#thr_attrs)*
            thread => #thr_vis #thr_ident {
                priority: ::core::sync::atomic::AtomicU8 = ::core::sync::atomic::AtomicU8::new(0);
                #thr_tokens
            };

            #(#local_attrs)*
            local => #local_vis #local_ident {
                #local_tokens
            };

            #(#index_attrs)*
            index => #index_vis #index_ident;

            threads => {
                #threads_tokens
            };

            #(resume => #resume;)*
        }
    }
}

fn def_soft(thr: &Thr, set_pending: Option<&ExprPath>) -> TokenStream2 {
    let Thr { ident: thr_ident, .. } = thr;
    let set_pending = set_pending.map(|set_pending| {
        quote! {
            #[inline]
            unsafe fn set_pending(thr_idx: u16) {
                unsafe { #set_pending(thr_idx) };
            }
        }
    });

    quote! {
        unsafe impl ::drone_core::thr::SoftThread for #thr_ident {
            #[inline]
            fn pending() -> *const ::core::sync::atomic::AtomicU32 {
                #[allow(clippy::declare_interior_mutable_const)]
                const VALUE: ::core::sync::atomic::AtomicU32 =
                    ::core::sync::atomic::AtomicU32::new(0);
                const COUNT: usize = ::drone_core::thr::pending_size::<#thr_ident>();
                static PENDING: [::core::sync::atomic::AtomicU32; COUNT] = [VALUE; COUNT];
                PENDING.as_ptr()
            }

            #[inline]
            fn pending_priority() -> *const ::core::sync::atomic::AtomicU8 {
                static PENDING_PRIORITY: ::core::sync::atomic::AtomicU8 =
                    ::core::sync::atomic::AtomicU8::new(0);
                &PENDING_PRIORITY
            }

            #[inline]
            fn priority(&self) -> *const ::core::sync::atomic::AtomicU8 {
                &self.priority
            }

            #set_pending
        }
    }
}
