use drone_macros2_core::{ExternStatic, NewStruct};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse, Attribute, Expr, Ident, Type, Visibility};
use syn::synom::Synom;

struct ThreadLocal {
  thread_local: NewStruct,
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

impl Synom for ThreadLocal {
  named!(parse -> Self, do_parse!(
    thread_local: syn!(NewStruct) >>
    array: syn!(ExternStatic) >>
    fields: many0!(syn!(Field)) >>
    (ThreadLocal { thread_local, array, fields })
  ));
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let call_site = Span::call_site();
  let input = parse::<ThreadLocal>(input).unwrap();
  let thread_local_attrs = input.thread_local.attrs;
  let thread_local_vis = input.thread_local.vis;
  let thread_local_ident = input.thread_local.ident;
  let array_ident = input.array.ident;
  let new_ident = Ident::new("new", call_site);
  let mut field_tokens = Vec::new();
  let mut field_ctor_tokens = Vec::new();
  for field in input.fields {
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
    mod rt {
      extern crate drone_core;

      pub use self::drone_core::fiber::Chain;
      pub use self::drone_core::thread::{TaskCell, Thread};
    }

    #(#thread_local_attrs)*
    #thread_local_vis struct #thread_local_ident {
      fibers: rt::Chain,
      task: rt::TaskCell,
      preempted: usize,
      #(#field_tokens,)*
    }

    impl #thread_local_ident {
      /// Creates a new blank thread.
      #[inline(always)]
      pub const fn #new_ident(_index: usize) -> Self {
        Self {
          fibers: rt::Chain::new(),
          task: rt::TaskCell::new(),
          preempted: 0,
          #(#field_ctor_tokens,)*
        }
      }
    }

    impl rt::Thread for #thread_local_ident {
      #[inline(always)]
      fn all() -> *mut [Self] {
        unsafe { &mut #array_ident }
      }

      #[inline(always)]
      fn fibers(&self) -> &rt::Chain {
        &self.fibers
      }

      #[inline(always)]
      fn fibers_mut(&mut self) -> &mut rt::Chain {
        &mut self.fibers
      }

      #[inline(always)]
      fn task(&self) -> &rt::TaskCell {
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
