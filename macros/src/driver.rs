use drone_macros_core::emit_err;
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse, Data, DeriveInput, Fields, Ident, Index, PathArguments};
use syn::punctuated::Pair;
use syn::spanned::Spanned;
use syn::synom::Synom;

#[derive(Default)]
struct Driver {
  forward: bool,
}

impl Synom for Driver {
  named!(parse -> Self, do_parse!(
    parens: parens!(do_parse!(
      forward: option!(do_parse!(
        ident: syn!(Ident) >>
        value: switch!(value!(ident.as_ref()),
          "forward" => value!(true) |
          _ => reject!()
        ) >>
        (value)
      )) >>
      (Driver {
        forward: forward.unwrap_or(false),
      })
    )) >>
    (parens.1)
  ));
}

pub fn proc_macro_derive(input: TokenStream) -> TokenStream {
  let call_site = Span::call_site();
  let input = parse::<DeriveInput>(input).unwrap();
  let input_span = input.span();
  let DeriveInput {
    attrs,
    ident,
    generics,
    data,
    ..
  } = input;
  let scope =
    Ident::from(format!("__driver_{}", ident.as_ref().to_snake_case()));
  let var = quote!(self);
  let zero_index = Index {
    index: 0,
    span: call_site,
  };
  let access = quote_spanned!(call_site => #var.#zero_index);
  let driver = attrs.into_iter().find(|attr| {
    if_chain! {
      if attr.path.leading_colon.is_none();
      if let Some(Pair::End(x)) = attr.path.segments.first();
      if let PathArguments::None = x.arguments;
      if x.ident == "driver";
      then { true } else { false }
    }
  });
  let Driver { forward } = match driver {
    Some(attr) => try_parse2!(attr.span(), attr.tts),
    None => Driver::default(),
  };
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  let res = if_chain! {
    if let Data::Struct(x) = data;
    if let Fields::Unnamed(mut x) = x.fields;
    if let Some(Pair::End(x)) = x.unnamed.pop();
    then {
      x
    } else {
      return emit_err(
        input_span,
        "Driver can be derived only from a tuple struct with one field",
      );
    }
  };
  let mut impl_tokens = Vec::new();
  if !forward {
    impl_tokens.push(quote! {
      type Resource = #res;

      #[inline(always)]
      fn new(source: <Self::Resource as Resource>::Source) -> Self {
        #ident(<Self::Resource as Resource>::from_source(source))
      }

      #[inline(always)]
      fn free(self) -> Self::Resource {
        #access
      }
    });
  } else {
    impl_tokens.push(quote! {
      type Resource = <#res as Driver>::Resource;

      #[inline(always)]
      fn new(source: <Self::Resource as Resource>::Source) -> Self {
        #ident(<#res as Driver>::new(source))
      }

      #[inline(always)]
      fn free(self) -> Self::Resource {
        Driver::free(#access)
      }
    });
  }

  let expanded = quote! {
    mod #scope {
      extern crate drone_core;

      use self::drone_core::drv::{Driver, Resource};

      impl #impl_generics Driver for #ident #ty_generics #where_clause {
        #(#impl_tokens)*
      }
    }
  };
  expanded.into()
}
