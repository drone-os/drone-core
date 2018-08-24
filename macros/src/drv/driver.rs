use drone_macros_core::emit_err2;
use inflector::Inflector;
use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;
use syn::synom::Synom;
use syn::{
  parse2, Data, DeriveInput, Field, Fields, GenericArgument, Ident,
  PathArguments, Type,
};

#[derive(Default)]
struct Driver {
  forward: bool,
}

impl Synom for Driver {
  named!(parse -> Self, do_parse!(
    parens: parens!(do_parse!(
      forward: option!(do_parse!(
        ident: syn!(Ident) >>
        value: switch!(value!(ident.to_string().as_ref()),
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
  let input = parse2::<DeriveInput>(input).unwrap();
  let input_span = input.span();
  let DeriveInput {
    attrs,
    ident,
    generics,
    data,
    ..
  } = input;
  let rt = Ident::new(
    &format!("__driver_rt_{}", ident.to_string().to_snake_case()),
    def_site,
  );
  let driver = attrs.into_iter().find(|attr| {
    if_chain! {
      if attr.path.leading_colon.is_none();
      if attr.path.segments.len() <= 1;
      if let Some(x) = attr.path.segments.iter().next();
      if let PathArguments::None = x.arguments;
      then { x.ident == "driver" } else { false }
    }
  });
  let Driver { forward } = match driver {
    Some(attr) => try_parse2!(attr.span(), attr.tts),
    None => Driver::default(),
  };
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  let mut res = if_chain! {
    if let Data::Struct(ref x) = &data;
    if let Fields::Unnamed(ref x) = x.fields;
    if x.unnamed.len() <= 1;
    if let Some(Field { ref ty, .. }) = x.unnamed.iter().next();
    then {
      ty
    } else {
      return emit_err2(
        input_span,
        "Driver can be derived only from a tuple struct with one field",
      );
    }
  };
  let ref_cell = parse_wrapper("RefCell", &mut res);
  let option = parse_wrapper("Option", &mut res);

  let mut res_def = quote!(#res);
  let mut free_def = quote!(self.0);
  let mut new_def = if !forward {
    quote!(<#res as #rt::Resource>::from_source(source))
  } else {
    quote!(<#res as #rt::Driver>::new(source))
  };
  if option {
    new_def = quote!(#rt::Some(#new_def));
  }
  if ref_cell {
    new_def = quote!(#rt::RefCell::new(#new_def));
    free_def = quote!(#free_def.into_inner());
  }
  if option {
    free_def = quote!(#free_def.unwrap());
  }
  if forward {
    res_def = quote!(<#res_def as #rt::Driver>::Resource);
    free_def = quote!(#rt::Driver::free(#free_def));
  }

  quote! {
    mod #rt {
      extern crate core;
      extern crate drone_core;

      pub use self::core::option::Option::*;
      pub use self::core::cell::RefCell;
      pub use self::drone_core::drv::{Driver, Resource};
    }

    impl #impl_generics #rt::Driver for #ident #ty_generics #where_clause {
      type Resource = #res_def;

      #[inline(always)]
      fn new(source: <Self::Resource as #rt::Resource>::Source) -> Self {
        #ident(#new_def)
      }

      #[inline(always)]
      fn free(self) -> Self::Resource {
        #free_def
      }
    }
  }
}

fn parse_wrapper(wrapper: &str, res: &mut &Type) -> bool {
  if_chain! {
    if let &Type::Path(ref x) = *res;
    if x.qself.is_none();
    if x.path.leading_colon.is_none();
    if x.path.segments.len() == 1;
    if let Some(x) = x.path.segments.iter().next();
    if x.ident == wrapper;
    if let PathArguments::AngleBracketed(ref x) = x.arguments;
    if x.args.len() == 1;
    if let Some(GenericArgument::Type(x)) = x.args.iter().next();
    then {
      *res = x;
      true
    } else {
      false
    }
  }
}
