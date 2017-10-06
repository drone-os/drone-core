use proc_macro::TokenStream;
use syn;

pub(crate) fn reg(input: TokenStream) -> TokenStream {
  let input = syn::parse_token_trees(&input.to_string()).unwrap();
  let mut input = input.into_iter();
  let mut attributes = Vec::new();
  let address = loop {
    match input.next() {
      Some(syn::TokenTree::Token(syn::Token::DocComment(string))) => {
        let string = string.trim_left_matches("//!");
        attributes.push(quote!(#[doc = #string]));
      }
      Some(syn::TokenTree::Token(syn::Token::Pound)) => match input.next() {
        Some(syn::TokenTree::Token(syn::Token::Not)) => match input.next() {
          Some(syn::TokenTree::Delimited(delimited)) => {
            attributes.push(quote!(# #delimited))
          }
          token => panic!("Invalid tokens after `#!`: {:?}", token),
        },
        token => panic!("Invalid tokens after `#`: {:?}", token),
      },
      Some(
        syn::TokenTree::Token(
          syn::Token::Literal(syn::Lit::Int(address, syn::IntTy::Unsuffixed)),
        ),
      ) => {
        break syn::Lit::Int(address, syn::IntTy::Usize);
      }
      token => panic!("Invalid token: {:?}", token),
    }
  };
  let value_attributes = attributes.clone();
  let raw = match input.next() {
    Some(
      syn::TokenTree::Token(
        syn::Token::Literal(syn::Lit::Int(raw, syn::IntTy::Unsuffixed)),
      ),
    ) => syn::Ident::new(format!("u{}", raw)),
    token => panic!("Invalid tokens after {:?}: {:?}", address, token),
  };
  let reg = match input.next() {
    Some(syn::TokenTree::Token(syn::Token::Ident(reg))) => reg,
    token => panic!("Invalid tokens after {}: {:?}", raw, token),
  };
  let value = syn::Ident::new(format!("{}Val", reg));
  let trait_name = input
    .map(|token| match token {
      syn::TokenTree::Token(syn::Token::Ident(name)) => name,
      token => panic!("Trait name expected, got {:?}", token),
    })
    .collect::<Vec<_>>();
  let trait_reg = trait_name.iter().map(|_| reg.clone()).collect::<Vec<_>>();

  let output = quote! {
    #(#attributes)*
    pub struct #reg<T: RegFlavor> {
      flavor: ::core::marker::PhantomData<T>,
    }

    #(#value_attributes)*
    #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct #value {
      value: #raw,
    }

    impl<T: RegFlavor> Reg<T> for #reg<T> {
      type Value = #value;

      const ADDRESS: usize = #address;

      #[inline]
      unsafe fn bind() -> Self {
        let flavor = ::core::marker::PhantomData;
        Self { flavor }
      }
    }

    impl RegValue for #value {
      type Raw = #raw;

      #[inline]
      fn new(value: #raw) -> Self {
        Self { value }
      }

      #[inline]
      fn raw(&self) -> #raw {
        self.value
      }

      #[inline]
      fn raw_mut(&mut self) -> &mut #raw {
        &mut self.value
      }
    }

    #[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
    impl Clone for #reg<::drone::reg::Ar> {
      #[inline]
      fn clone(&self) -> Self {
        Self { ..*self }
      }
    }

    impl Copy for #reg<::drone::reg::Ar> {}

    #(
      impl<T: RegFlavor> #trait_name<T> for #trait_reg<T> {}
    )*
  };
  output.parse().unwrap()
}
