use failure::{err_msg, Error};
use proc_macro::TokenStream;
use quote::Tokens;
use std::mem;
use syn::{parse_expr, parse_item, parse_token_trees, ItemKind, Stmt};

pub(crate) fn async_future(
  args: TokenStream,
  input: TokenStream,
) -> Result<Tokens, Error> {
  let args = parse_token_trees(&args.to_string()).map_err(err_msg)?;
  let mut item = parse_item(&input.to_string()).map_err(err_msg)?;

  if !args.is_empty() {
    Err(err_msg("#[async_future] attribute takes no arguments"))?;
  }

  if let ItemKind::Fn(_, _, _, _, _, ref mut block) = item.node {
    let stmts = mem::replace(&mut block.stmts, Vec::new());
    let expr = quote! {
      AsyncFuture::new(move || {
        #(#stmts)*
      })
    };
    let expr = parse_expr(expr.as_str()).map_err(err_msg)?;
    block.stmts = vec![Stmt::Expr(Box::new(expr))];
  } else {
    Err(err_msg("#[async_future] can only be applied to functions"))?
  }

  Ok(quote!(#item))
}
