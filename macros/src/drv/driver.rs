use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{
  Data, DeriveInput, Field, Fields, GenericArgument, Ident, PathArguments, Type,
};

#[derive(Default)]
struct Driver {
  forward: bool,
}

impl Parse for Driver {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    parenthesized!(content in input);
    match content.parse::<Option<Ident>>()? {
      Some(ref ident) if ident == "forward" => Ok(Self { forward: true }),
      Some(_) => Err(input.error("invalid attribute")),
      None => Ok(Self { forward: false }),
    }
  }
}

pub fn proc_macro_derive(input: TokenStream) -> TokenStream {
  let def_site = Span::def_site();
  let input = parse_macro_input!(input as DeriveInput);
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
    Some(attr) => {
      let input = attr.tts.into();
      parse_macro_input!(input as Driver)
    }
    None => Driver::default(),
  };
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  let fields = if_chain! {
    if let Data::Struct(ref x) = &data;
    if let Fields::Unnamed(ref x) = x.fields;
    let mut fields = x.unnamed.iter();
    if let Some(Field { ty: ref res, .. }) = fields.next();
    then {
      if let Some(Field { ty: ref data, .. }) = fields.next() {
        if fields.next().is_none() &&
          is_wrapper(data, "PhantomData").is_some() {
          Some((res, Some(data)))
        } else {
          None
        }
      } else {
        Some((res, None))
      }
    } else {
      None
    }
  };
  let (mut res, data) = if let Some((res, data)) = fields {
    (res, data)
  } else {
    return Error::new(input_span, "Unsupported sequence of fields")
      .to_compile_error()
      .into();
  };
  let ref_cell = is_wrapper(res, "RefCell");
  if let Some(ref_cell) = ref_cell {
    res = ref_cell;
  }
  let option = is_wrapper(res, "Option");
  if let Some(option) = option {
    res = option;
  }

  let mut res_def = quote!(#res);
  let mut free_def = quote!(self.0);
  let mut new_def = if !forward {
    quote!(<#res as #rt::Resource>::from_source(source))
  } else {
    quote!(<#res as #rt::Driver>::new(source))
  };
  if option.is_some() {
    new_def = quote!(#rt::Some(#new_def));
  }
  if ref_cell.is_some() {
    new_def = quote!(#rt::RefCell::new(#new_def));
    free_def = quote!(#free_def.into_inner());
  }
  if option.is_some() {
    free_def = quote!(#free_def.unwrap());
  }
  if forward {
    res_def = quote!(<#res_def as #rt::Driver>::Resource);
    free_def = quote!(#rt::Driver::free(#free_def));
  }

  let data = if data.is_some() {
    quote!(PhantomData)
  } else {
    quote!()
  };

  let expanded = quote! {
    mod #rt {
      extern crate core;
      extern crate drone_core;

      pub use self::core::option::Option::*;
      pub use self::core::cell::RefCell;
      pub use self::drone_core::drv::{Driver, Resource};
    }

    impl #impl_generics #rt::Driver for #ident #ty_generics #where_clause {
      type Resource = #res_def;

      #[inline]
      fn new(source: <Self::Resource as #rt::Resource>::Source) -> Self {
        #ident(#new_def, #data)
      }

      #[inline]
      fn free(self) -> Self::Resource {
        #free_def
      }
    }
  };
  expanded.into()
}

fn is_wrapper<'a>(res: &'a Type, wrapper: &str) -> Option<&'a Type> {
  if_chain! {
    if let &Type::Path(ref x) = res;
    if x.qself.is_none();
    if x.path.leading_colon.is_none();
    if x.path.segments.len() == 1;
    if let Some(x) = x.path.segments.iter().next();
    if x.ident == wrapper;
    if let PathArguments::AngleBracketed(ref x) = x.arguments;
    if x.args.len() == 1;
    if let Some(GenericArgument::Type(x)) = x.args.iter().next();
    then {
      Some(x)
    } else {
      None
    }
  }
}
