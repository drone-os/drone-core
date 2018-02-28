use drone_macros2_core::{ExternStatic, NewStruct};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Attribute, Expr, Ident, Type, Visibility};
use syn::synom::Synom;

struct Thread {
  thread: NewStruct,
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

impl Synom for Thread {
  named!(parse -> Self, do_parse!(
    thread: syn!(NewStruct) >>
    array: syn!(ExternStatic) >>
    fields: many0!(syn!(Field)) >>
    (Thread { thread, array, fields })
  ));
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let call_site = Span::call_site();
  let Thread {
    thread:
      NewStruct {
        attrs: thread_attrs,
        vis: thread_vis,
        ident: thread_ident,
      },
    array: ExternStatic { ident: array_ident },
    fields,
  } = try_parse!(call_site, input);
  let rt = Ident::from("__thread_rt");
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

      pub use self::drone_core::fiber::Chain;
      pub use self::drone_core::thread::{TaskCell, Thread};
    }

    #(#thread_attrs)*
    #thread_vis struct #thread_ident {
      fibers: #rt::Chain,
      task: #rt::TaskCell,
      preempted: usize,
      #(#field_tokens,)*
    }

    impl #thread_ident {
      /// Creates a new blank thread.
      #[inline(always)]
      pub const fn #new_ident(_index: usize) -> Self {
        Self {
          fibers: #rt::Chain::new(),
          task: #rt::TaskCell::new(),
          preempted: 0,
          #(#field_ctor_tokens,)*
        }
      }
    }

    impl #rt::Thread for #thread_ident {
      #[inline(always)]
      fn all() -> *mut [Self] {
        unsafe { &mut #array_ident }
      }

      #[inline(always)]
      fn fibers(&self) -> &#rt::Chain {
        &self.fibers
      }

      #[inline(always)]
      fn fibers_mut(&mut self) -> &mut #rt::Chain {
        &mut self.fibers
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
