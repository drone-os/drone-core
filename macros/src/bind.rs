use proc_macro::TokenStream;
use syn;

pub(crate) fn bind(input: TokenStream) -> TokenStream {
  let input = syn::parse_token_trees(&input.to_string()).unwrap();
  let mut input = input.into_iter().fuse();
  let mut names = Vec::new();
  let mut regs = Vec::new();
  'outer: loop {
    match input.next() {
      Some(syn::TokenTree::Token(syn::Token::Ident(name))) => {
        match input.next() {
          Some(syn::TokenTree::Token(syn::Token::Colon)) => (),
          token => panic!("Invalid token after `{}`: {:?}", name, token),
        }
        let mut reg = Vec::new();
        loop {
          match input.next() {
            Some(syn::TokenTree::Token(token @ syn::Token::Ident(_))) |
            Some(syn::TokenTree::Token(token @ syn::Token::ModSep)) |
            Some(syn::TokenTree::Token(token @ syn::Token::Lt)) |
            Some(syn::TokenTree::Token(token @ syn::Token::Gt)) => {
              reg.push(token)
            }
            Some(syn::TokenTree::Token(syn::Token::Comma)) | None => break,
            token => {
              panic!("Invalid token after `{}: {:?}`: {:?}", name, reg, token)
            }
          }
        }
        names.push(name);
        regs.push(reg);
      }
      None => break,
      token => panic!("Invalid token: {:?}", token),
    }
  }

  let output = quote! {
    #(
      let #names = unsafe {
        type Register = #(#regs)*;
        Register::bind()
      };
    )*
  };
  output.parse().unwrap()
}
