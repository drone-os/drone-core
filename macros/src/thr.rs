use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Expr, ExprPath, Ident, Type, Visibility};

struct Thr {
  thr_attrs: Vec<Attribute>,
  thr_vis: Visibility,
  thr_ident: Ident,
  local_attrs: Vec<Attribute>,
  local_vis: Visibility,
  local_ident: Ident,
  sv: Option<ExprPath>,
  array: ExprPath,
  fields: Vec<Field>,
}

struct Field {
  attrs: Vec<Attribute>,
  shared: bool,
  ident: Ident,
  ty: Type,
  init: Expr,
}

impl Parse for Thr {
  fn parse(input: ParseStream) -> Result<Self> {
    let thr_attrs = input.call(Attribute::parse_outer)?;
    let thr_vis = input.parse()?;
    input.parse::<Token![struct]>()?;
    let thr_ident = input.parse()?;
    input.parse::<Token![;]>()?;
    let local_attrs = input.call(Attribute::parse_outer)?;
    let local_vis = input.parse()?;
    input.parse::<Token![struct]>()?;
    let local_ident = input.parse()?;
    input.parse::<Token![;]>()?;
    let sv = if input.peek(Token![extern]) && input.peek2(Token![struct]) {
      input.parse::<Token![extern]>()?;
      input.parse::<Token![struct]>()?;
      let sv = input.parse()?;
      input.parse::<Token![;]>()?;
      Some(sv)
    } else {
      None
    };
    input.parse::<Token![extern]>()?;
    input.parse::<Token![static]>()?;
    let array = input.parse()?;
    input.parse::<Token![;]>()?;
    let mut fields = Vec::new();
    while !input.is_empty() {
      fields.push(input.parse()?);
    }
    Ok(Self {
      thr_attrs,
      thr_vis,
      thr_ident,
      local_attrs,
      local_vis,
      local_ident,
      sv,
      array,
      fields,
    })
  }
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

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let Thr {
    thr_attrs,
    thr_vis,
    thr_ident,
    local_attrs,
    local_vis,
    local_ident,
    sv,
    array,
    fields,
  } = parse_macro_input!(input as Thr);
  let local = new_def_ident!("Local");
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
  thr_tokens.push(quote! {
    fib_chain: ::drone_core::fib::Chain
  });
  thr_ctor_tokens.push(quote! {
    fib_chain: ::drone_core::fib::Chain::new()
  });
  local_tokens.push(quote! {
    task: ::drone_core::thr::TaskCell
  });
  local_tokens.push(quote! {
    preempted: ::drone_core::thr::PreemptedCell
  });
  local_ctor_tokens.push(quote! {
    task: ::drone_core::thr::TaskCell::new()
  });
  local_ctor_tokens.push(quote! {
    preempted: ::drone_core::thr::PreemptedCell::new()
  });
  let sv_ty = if let Some(path) = sv {
    quote!(#path)
  } else {
    quote!(::drone_core::sv::SvNone)
  };

  let expanded = quote! {
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

    impl ::drone_core::thr::Thread for #thr_ident {
      type Local = #local_ident;
      type Sv = #sv_ty;

      #[inline(always)]
      fn first() -> *const Self {
        unsafe { #array.as_ptr() }
      }

      #[inline(always)]
      fn fib_chain(&self) -> &::drone_core::fib::Chain {
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

    impl ::drone_core::thr::ThreadLocal for #local_ident {
      #[inline(always)]
      fn task(&self) -> &::drone_core::thr::TaskCell {
        &self.task
      }

      #[inline(always)]
      fn preempted(&self) -> &::drone_core::thr::PreemptedCell {
        &self.preempted
      }
    }

    unsafe impl Sync for #local {}
  };
  expanded.into()
}
