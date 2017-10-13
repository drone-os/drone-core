use proc_macro::TokenStream;
use syn;

pub(crate) fn thread_local(input: TokenStream) -> TokenStream {
  let input = syn::parse_token_trees(&input.to_string()).unwrap();
  let mut input = input.into_iter();
  let mut attributes = Vec::new();
  let mut field_visiblity = Vec::new();
  let mut field_attributes = Vec::new();
  let mut field_name = Vec::new();
  let mut field_type = Vec::new();
  let mut field_init = Vec::new();
  'outer: loop {
    let mut public = false;
    let mut inner_attributes = Vec::new();
    loop {
      match input.next() {
        Some(syn::TokenTree::Token(syn::Token::DocComment(string))) => {
          if string.starts_with("//!") {
            let string = string.trim_left_matches("//!");
            attributes.push(quote!(#[doc = #string]));
          } else {
            let string = string.trim_left_matches("///");
            inner_attributes.push(quote!(#[doc = #string]));
          }
        }
        Some(syn::TokenTree::Token(syn::Token::Pound)) => match input.next() {
          Some(syn::TokenTree::Token(syn::Token::Not)) => match input.next() {
            Some(syn::TokenTree::Delimited(delimited)) => {
              attributes.push(quote!(# #delimited))
            }
            token => panic!("Invalid tokens after `#!`: {:?}", token),
          },
          Some(syn::TokenTree::Delimited(delimited)) => {
            inner_attributes.push(quote!(# #delimited))
          }
          token => panic!("Invalid tokens after `#`: {:?}", token),
        },
        Some(syn::TokenTree::Token(syn::Token::Ident(ref ident)))
          if ident == "pub" =>
        {
          public = true;
        }
        Some(syn::TokenTree::Token(syn::Token::Ident(name))) => {
          match input.next() {
            Some(syn::TokenTree::Token(syn::Token::Colon)) => (),
            token => panic!("Invalid token after `{}`: {:?}", name, token),
          }
          let mut ty = Vec::new();
          loop {
            match input.next() {
              Some(syn::TokenTree::Token(syn::Token::Eq)) => break,
              Some(syn::TokenTree::Token(token)) => ty.push(token),
              token => {
                panic!("Invalid token after `{}: {:?}`: {:?}", name, ty, token)
              }
            }
          }
          let init = match input.next() {
            Some(
              syn::TokenTree::Delimited(syn::Delimited {
                delim: syn::DelimToken::Brace,
                tts,
              }),
            ) => tts,
            token => {
              panic!("Invalid token after `{}: {:?} =`: {:?}", name, ty, token)
            }
          };
          field_visiblity.push(if public {
            Some(syn::Ident::new("pub"))
          } else {
            None
          });
          field_attributes.push(inner_attributes);
          field_name.push(name);
          field_type.push(ty);
          field_init.push(init);
          break;
        }
        None => break 'outer,
        token => panic!("Invalid token: {:?}", token),
      }
    }
  }
  let field_name2 = field_name.clone();

  let output = quote! {
    use core::cell::Cell;
    use core::ptr;
    use drone::collections::LinkedList;
    use drone::thread::{Routine, Thread};

    #(#attributes)*
    pub struct ThreadLocal {
      list: LinkedList<Routine>,
      preempted_id: usize,
      task: Cell<*mut u8>,
      #(
        #(#field_attributes)*
        #field_visiblity #field_name: #(#field_type)*,
      )*
    }

    impl ThreadLocal {
      /// Creates a blank `ThreadLocal`.
      #[allow(dead_code)]
      pub const fn new(_id: usize) -> Self {
        Self {
          list: LinkedList::new(),
          preempted_id: 0,
          task: Cell::new(ptr::null_mut()),
          #(
            #field_name2: { #(#field_init)* },
          )*
        }
      }
    }

    impl Thread for ThreadLocal {
      #[inline]
      unsafe fn get_unchecked(id: usize) -> &'static Self {
        THREADS.get_unchecked(id)
      }

      #[inline]
      fn list(&self) -> &LinkedList<Routine> {
        &self.list
      }

      #[inline]
      fn list_mut(&mut self) -> &mut LinkedList<Routine> {
        &mut self.list
      }

      #[inline]
      fn preempted_id(&self) -> usize {
        self.preempted_id
      }

      #[inline]
      unsafe fn set_preempted_id(&mut self, id: usize) {
        self.preempted_id = id;
      }

      #[inline]
      fn task(&self) -> *mut u8 {
        self.task.get()
      }

      #[inline]
      unsafe fn set_task(&self, task: *mut u8) {
        self.task.set(task);
      }
    }
  };
  output.parse().unwrap()
}
