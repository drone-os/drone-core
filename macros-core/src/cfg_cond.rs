use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream, Result},
    Ident, LitStr, Token,
};

/// List of conditional compilation clauses.
#[derive(Default, Clone, Debug)]
pub struct CfgCond {
    /// CNF of clauses.
    clauses: Vec<Vec<(Ident, LitStr)>>,
    inverse: bool,
}

impl Parse for CfgCond {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut clauses = Vec::new();
        if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let input2;
            bracketed!(input2 in input);
            let ident = input2.parse::<Ident>()?;
            if ident != "cfg" {
                return Err(input2.error("Unsupported attribute"));
            }
            let input3;
            parenthesized!(input3 in input2);
            let ident = input3.parse::<Ident>()?;
            if ident == "any" {
                let input4;
                parenthesized!(input4 in input3);
                let mut last_comma = true;
                while last_comma && !input4.is_empty() {
                    let ident = input4.parse::<Ident>()?;
                    input4.parse::<Token![=]>()?;
                    clauses.push((ident, input4.parse()?));
                    last_comma = input4.parse::<Option<Token![,]>>()?.is_some();
                }
            } else {
                input3.parse::<Token![=]>()?;
                clauses.push((ident, input3.parse()?));
                if !input3.is_empty() {
                    return Err(input3.error("Unsupported attribute"));
                }
            }
        }
        Ok(Self {
            clauses: if clauses.is_empty() {
                vec![]
            } else {
                vec![clauses]
            },
            inverse: false,
        })
    }
}

impl CfgCond {
    /// Copies `rhs` clauses into `self`.
    ///
    /// # Panics
    ///
    /// If `rhs` or `self` is a result of [`CfgCondExt::transpose`].
    pub fn add_clause(&mut self, rhs: &Self) {
        assert!(!self.inverse);
        assert!(!rhs.inverse);
        self.clauses.append(&mut rhs.clauses.clone());
    }

    /// Returns a `TokenStream` for conditional compilation.
    pub fn attrs(&self) -> Option<TokenStream> {
        let Self { clauses, inverse } = self;
        let tokens = clauses
            .iter()
            .map(|clauses| {
                clauses
                    .iter()
                    .map(|(key, value)| quote!(#key = #value))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        if tokens.is_empty() {
            None
        } else if *inverse {
            Some(quote!(#[cfg(not(any(#(all(#(#tokens),*)),*)))]))
        } else {
            Some(quote!(#(#[cfg(any(#(#tokens),*))])*))
        }
    }

    /// Converts to DNF.
    fn to_dnf(&self) -> Vec<Vec<(Ident, LitStr)>> {
        assert!(!self.inverse);
        match self
            .clauses
            .iter()
            .map(Vec::as_slice)
            .collect::<Vec<_>>()
            .as_slice()
        {
            [] | [[]] | [[], []] => Vec::new(),
            [x] | [x, []] | [[], x] => x.iter().map(|x| vec![x.clone()]).collect(),
            [x, y] => {
                let mut dnf = Vec::new();
                for x in x.iter() {
                    for y in y.iter() {
                        dnf.push(vec![x.clone(), y.clone()]);
                    }
                }
                dnf
            }
            x => panic!("{} clauses of CNF is unsupported", x.len()),
        }
    }
}

/// [`CfgCond`] helper extension trait for slices.
pub trait CfgCondExt<T: Clone> {
    /// Converts a sequence of `T` into a sequence of combinations of `T` for
    /// each possible condition.
    fn transpose(self) -> Vec<(CfgCond, Vec<T>)>;
}

impl<T: Clone> CfgCondExt<T> for &[(CfgCond, T)] {
    fn transpose(self) -> Vec<(CfgCond, Vec<T>)> {
        let mut map: HashMap<_, Vec<_>> = HashMap::new();
        let mut default = Vec::new();
        for (clauses, item) in self {
            let clauses = clauses.to_dnf();
            if clauses.is_empty() {
                default.push(item.clone());
            } else {
                for cond in clauses {
                    map.entry(cond).or_default().push(item.clone());
                }
            }
        }
        let mut result = Vec::new();
        for (clauses, mut items) in map {
            let clauses = CfgCond {
                clauses: vec![clauses],
                inverse: false,
            };
            items.append(&mut default.clone());
            result.push((clauses, items));
        }
        let clauses = result.iter().flat_map(|(x, _)| x.clauses.clone()).collect();
        let clauses = CfgCond {
            clauses,
            inverse: true,
        };
        result.push((clauses, default));
        result
    }
}
