use inflector::Inflector;
use proc_macro::TokenStream;
use syn::{parse, DeriveInput, Ident};

pub fn proc_macro_derive(input: TokenStream) -> TokenStream {
  let input = parse::<DeriveInput>(input).unwrap();
  let DeriveInput {
    ident, generics, ..
  } = input;
  let scope =
    Ident::from(format!("__driver_{}", ident.as_ref().to_snake_case()));
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let expanded = quote! {
    mod #scope {
      extern crate drone_core;

      use self::drone_core::drv::Resource;

      impl #impl_generics Resource for #ident #ty_generics #where_clause {
        type Source = Self;

        #[inline(always)]
        fn from_source(source: Self) -> Self {
          source
        }
      }
    }
  };
  expanded.into()
}
