use drone_macros_core::{ExternStatic, ExternStruct, NewStruct};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Expr, Ident, Type};

struct Thr {
  thr: NewStruct,
  local: NewStruct,
  sv: ExternStruct,
  array: ExternStatic,
  fields: Vec<Field>,
}

struct Field {
  attrs: Vec<Attribute>,
  shared: bool,
  ident: Ident,
  ty: Type,
  init: Expr,
}

impl Parse for Field {
  fn parse(input: ParseStream) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let shared = input.parse::<Option<Token![pub]>>()?.is_some();
    let ident = input.parse()?;
    input.parse::<Token![:]>()?;
    let ty = input.parse()?;
    input.parse::<Token![=]>()?;
    let init = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self {
      attrs,
      shared,
      ident,
      ty,
      init,
    })
  }
}

impl Parse for Thr {
  fn parse(input: ParseStream) -> Result<Self> {
    let thr = input.parse()?;
    let local = input.parse()?;
    let sv = input.parse()?;
    let array = input.parse()?;
    let mut fields = Vec::new();
    while !input.is_empty() {
      fields.push(input.parse()?);
    }
    Ok(Self {
      thr,
      local,
      sv,
      array,
      fields,
    })
  }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let def_site = Span::def_site();
  let Thr {
    thr:
      NewStruct {
        attrs: thr_attrs,
        vis: thr_vis,
        ident: thr_ident,
      },
    local:
      NewStruct {
        attrs: local_attrs,
        vis: local_vis,
        ident: local_ident,
      },
    sv: ExternStruct { ident: sv_ident },
    array: ExternStatic { ident: array_ident },
    fields,
  } = parse_macro_input!(input as Thr);
  let rt = Ident::new("__thr_rt", def_site);
  let local = Ident::new("Local", def_site);
  let mut thr_tokens = Vec::new();
  let mut thr_ctor_tokens = Vec::new();
  let mut local_tokens = Vec::new();
  let mut local_ctor_tokens = Vec::new();
  for field in fields {
    let Field {
      attrs,
      shared,
      ident,
      ty,
      init,
    } = field;
    let tokens = quote!(#(#attrs)* pub #ident: #ty);
    let ctor_tokens = quote!(#ident: #init);
    if shared {
      thr_tokens.push(tokens);
      thr_ctor_tokens.push(ctor_tokens);
    } else {
      local_tokens.push(tokens);
      local_ctor_tokens.push(ctor_tokens);
    }
  }
  thr_tokens.push(quote!(fib_chain: #rt::Chain));
  thr_ctor_tokens.push(quote!(fib_chain: #rt::Chain::new()));
  local_tokens.push(quote!(task: #rt::TaskCell));
  local_tokens.push(quote!(preempted: #rt::PreemptedCell));
  local_ctor_tokens.push(quote!(task: #rt::TaskCell::new()));
  local_ctor_tokens.push(quote!(preempted: #rt::PreemptedCell::new()));

  let expanded = quote! {
    mod #rt {
      extern crate drone_core;

      pub use self::drone_core::fib::Chain;
      pub use self::drone_core::thr::{PreemptedCell, TaskCell, Thread,
                                      ThreadLocal};
    }

    #(#thr_attrs)*
    #thr_vis struct #thr_ident {
      local: #local,
      #(#thr_tokens),*
    }

    #(#local_attrs)*
    #local_vis struct #local_ident {
      #(#local_tokens),*
    }

    struct #local(#local_ident);

    impl #thr_ident {
      /// Creates a new thread.
      pub const fn new(_index: usize) -> Self {
        Self {
          local: #local(#local_ident::new()),
          #(#thr_ctor_tokens),*
        }
      }
    }

    impl #rt::Thread for #thr_ident {
      type Local = #local_ident;
      type Sv = #sv_ident;

      #[inline(always)]
      fn first() -> *const Self {
        unsafe { #array_ident.as_ptr() }
      }

      #[inline(always)]
      fn fib_chain(&self) -> &#rt::Chain {
        &self.fib_chain
      }

      #[inline(always)]
      unsafe fn get_local(&self) -> &#local_ident {
        &self.local.0
      }
    }

    impl #local_ident {
      const fn new() -> Self {
        Self { #(#local_ctor_tokens,)* }
      }
    }

    impl #rt::ThreadLocal for #local_ident {
      #[inline(always)]
      fn task(&self) -> &#rt::TaskCell {
        &self.task
      }

      #[inline(always)]
      fn preempted(&self) -> &#rt::PreemptedCell {
        &self.preempted
      }
    }

    unsafe impl Sync for #local {}
  };
  expanded.into()
}
