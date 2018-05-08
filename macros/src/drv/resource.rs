use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse, DeriveInput, Ident};

pub fn proc_macro_derive(input: TokenStream) -> TokenStream {
  let def_site = Span::def_site();
  let input = parse::<DeriveInput>(input).unwrap();
  let DeriveInput {
    ident, generics, ..
  } = input;
  let rt = Ident::new(
    &format!("__resource_{}", ident.as_ref().to_snake_case()),
    def_site,
  );
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let expanded = quote! {
    mod #rt {
      extern crate drone_core;
      pub use self::drone_core::drv::Resource;
    }

    impl #impl_generics #rt::Resource for #ident #ty_generics #where_clause {
      type Source = Self;

      #[inline(always)]
      fn from_source(source: Self) -> Self {
        source
      }
    }
  };
  expanded.into()
}
