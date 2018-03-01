use drone_macros2_core::{ExternStatic, NewStruct};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Attribute, Expr, Ident, Type, Visibility};
use syn::synom::Synom;

struct Thr {
  thr: NewStruct,
  array: ExternStatic,
  fields: Vec<Field>,
}

struct Field {
  attrs: Vec<Attribute>,
  vis: Visibility,
  ident: Ident,
  ty: Type,
  init: Expr,
}

impl Synom for Field {
  named!(parse -> Self, do_parse!(
    attrs: many0!(Attribute::parse_outer) >>
    vis: syn!(Visibility) >>
    ident: syn!(Ident) >>
    punct!(:) >>
    ty: syn!(Type) >>
    punct!(=) >>
    init: syn!(Expr) >>
    punct!(;) >>
    (Field { attrs, vis, ident, ty, init })
  ));
}

impl Synom for Thr {
  named!(parse -> Self, do_parse!(
    thr: syn!(NewStruct) >>
    array: syn!(ExternStatic) >>
    fields: many0!(syn!(Field)) >>
    (Thr { thr, array, fields })
  ));
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let call_site = Span::call_site();
  let Thr {
    thr:
      NewStruct {
        attrs: thr_attrs,
        vis: thr_vis,
        ident: thr_ident,
      },
    array: ExternStatic { ident: array_ident },
    fields,
  } = try_parse!(call_site, input);
  let rt = Ident::from("__thr_rt");
  let new_ident = Ident::new("new", call_site);
  let mut field_tokens = Vec::new();
  let mut field_ctor_tokens = Vec::new();
  for field in fields {
    let Field {
      attrs,
      vis,
      ident,
      ty,
      init,
    } = field;
    field_tokens.push(quote!(#(#attrs)* #vis #ident: #ty));
    field_ctor_tokens.push(quote!(#ident: #init));
  }

  let expanded = quote! {
    mod #rt {
      extern crate drone_core;

      pub use self::drone_core::fib::Chain;
      pub use self::drone_core::thr::{TaskCell, Thread};
    }

    #(#thr_attrs)*
    #thr_vis struct #thr_ident {
      fib_chain: #rt::Chain,
      task: #rt::TaskCell,
      preempted: usize,
      #(#field_tokens,)*
    }

    impl #thr_ident {
      /// Creates a new thread.
      #[inline(always)]
      pub const fn #new_ident(_index: usize) -> Self {
        Self {
          fib_chain: #rt::Chain::new(),
          task: #rt::TaskCell::new(),
          preempted: 0,
          #(#field_ctor_tokens,)*
        }
      }
    }

    impl #rt::Thread for #thr_ident {
      #[inline(always)]
      fn all() -> *mut [Self] {
        unsafe { &mut #array_ident }
      }

      #[inline(always)]
      fn fib_chain(&self) -> &#rt::Chain {
        &self.fib_chain
      }

      #[inline(always)]
      fn fib_chain_mut(&mut self) -> &mut #rt::Chain {
        &mut self.fib_chain
      }

      #[inline(always)]
      fn task(&self) -> &#rt::TaskCell {
        &self.task
      }

      #[inline(always)]
      fn preempted(&mut self) -> &mut usize {
        &mut self.preempted
      }
    }
  };
  expanded.into()
}
