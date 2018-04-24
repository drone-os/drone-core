use drone_macros_core::emit_err;
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::synom::Synom;
use syn::{parse, Data, DeriveInput, Field, Fields, GenericArgument, Ident,
          Index, PathArguments, Type};

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
  let def_site = Span::def_site();
  let input = parse::<DeriveInput>(input).unwrap();
  let input_span = input.span();
  let DeriveInput {
    attrs,
    ident,
    generics,
    data,
    ..
  } = input;
  let scope = Ident::new(
    &format!("__driver_{}", ident.as_ref().to_snake_case()),
    def_site,
  );
  let var = quote_spanned!(def_site => self);
  let zero_index = Index::from(0);
  let access = quote!(#var.#zero_index);
  let driver = attrs.into_iter().find(|attr| {
    if_chain! {
      if attr.path.leading_colon.is_none();
      if attr.path.segments.len() <= 1;
      if let Some(x) = attr.path.segments.iter().next();
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
    if x.unnamed.len() <= 1;
    if let Some(Field { ty, .. }) = x.unnamed.into_iter().next();
    then {
      ty
    } else {
      return emit_err(
        input_span,
        "Driver can be derived only from a tuple struct with one field",
      );
    }
  };
  let option = if_chain! {
    if let &Type::Path(ref x) = &res;
    if x.qself.is_none();
    if x.path.leading_colon.is_none();
    if x.path.segments.len() == 1;
    if let Some(x) = x.path.segments.iter().next();
    if x.ident == "Option";
    if let PathArguments::AngleBracketed(ref x) = x.arguments;
    if x.args.len() == 1;
    if let Some(GenericArgument::Type(x)) = x.args.iter().next();
    then { Some(x) } else { None }
  };
  let mut impl_tokens = Vec::new();
  if !forward {
    if let Some(res) = option {
      impl_tokens.push(quote_spanned! { def_site =>
        type Resource = #res;

        #[inline(always)]
        fn new(source: <Self::Resource as Resource>::Source) -> Self {
          #ident(Some(<Self::Resource as Resource>::from_source(source)))
        }

        #[inline(always)]
        fn free(self) -> Self::Resource {
          #access.unwrap()
        }
      });
    } else {
      impl_tokens.push(quote_spanned! { def_site =>
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
    }
  } else {
    if let Some(res) = option {
      impl_tokens.push(quote_spanned! { def_site =>
        type Resource = <#res as Driver>::Resource;

        #[inline(always)]
        fn new(source: <Self::Resource as Resource>::Source) -> Self {
          #ident(Some(<#res as Driver>::new(source)))
        }

        #[inline(always)]
        fn free(self) -> Self::Resource {
          Driver::free(#access).unwrap()
        }
      });
    } else {
      impl_tokens.push(quote_spanned! { def_site =>
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
  }

  let expanded = quote_spanned! { def_site =>
    mod #scope {
      extern crate core;
      extern crate drone_core;

      #[allow(unused_imports)]
      use self::core::option::Option::*;
      use self::drone_core::drv::{Driver, Resource};

      impl #impl_generics Driver for #ident #ty_generics #where_clause {
        #(#impl_tokens)*
      }
    }
  };
  expanded.into()
}
