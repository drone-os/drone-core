use drone_config::Config;
use drone_macros_core::compile_error;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, IntSuffix, LitInt,
};

struct BmpUartBaudrate;

impl Parse for BmpUartBaudrate {
    fn parse(_input: ParseStream<'_>) -> Result<Self> {
        Ok(Self)
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let BmpUartBaudrate = parse_macro_input!(input as BmpUartBaudrate);
    let config = match Config::read_from_cargo_manifest_dir() {
        Ok(config) => config,
        Err(err) => compile_error!("{}", err),
    };
    match config.bmp() {
        Ok(bmp) => {
            let value = LitInt::new(
                u64::from(bmp.uart_baudrate),
                IntSuffix::Usize,
                Span::call_site(),
            );
            quote!(#value).into()
        }
        Err(err) => {
            compile_error!("{}", err);
        }
    }
}
