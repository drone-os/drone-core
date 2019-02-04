use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{
  bracketed, parenthesized,
  parse::{Parse, ParseStream, Result},
  Ident, LitStr, Token,
};

/// List of features for conditional compilation.
#[derive(Default, Clone, Debug)]
pub struct CfgFeatures {
  /// CNF of features.
  features: Vec<Vec<LitStr>>,
  inverse: bool,
}

impl Parse for CfgFeatures {
  fn parse(input: ParseStream<'_>) -> Result<Self> {
    let mut features = Vec::new();
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
          if ident != "feature" {
            return Err(input4.error("Unsupported attribute"));
          }
          input4.parse::<Token![=]>()?;
          features.push(input4.parse()?);
          last_comma = input4.parse::<Option<Token![,]>>()?.is_some();
        }
      } else {
        if ident != "feature" {
          return Err(input3.error("Unsupported attribute"));
        }
        input3.parse::<Token![=]>()?;
        features.push(input3.parse()?);
        if !input3.is_empty() {
          return Err(input3.error("Unsupported attribute"));
        }
      }
    }
    Ok(Self {
      features: if features.is_empty() {
        vec![]
      } else {
        vec![features]
      },
      inverse: false,
    })
  }
}

impl CfgFeatures {
  /// Copies `rhs` clauses into `self`.
  ///
  /// # Panics
  ///
  /// If `rhs` or `self` is a result of
  /// [`transpose`](CfgFeaturesExt::transpose).
  pub fn add_clause(&mut self, rhs: &Self) {
    assert!(!self.inverse);
    assert!(!rhs.inverse);
    self.features.append(&mut rhs.features.clone());
  }

  /// Returns a `TokenStream` for conditional compilation.
  pub fn attrs(&self) -> Option<TokenStream> {
    let Self { features, inverse } = self;
    if features.is_empty() {
      None
    } else if *inverse {
      Some(quote! {
        #[cfg(not(any(
          #(
            all(#(feature = #features),*)
          ),*
        )))]
      })
    } else {
      Some(quote! {
        #(
          #[cfg(any(
            #(feature = #features),*
          ))]
        )*
      })
    }
  }

  /// Converts to DNF.
  fn to_dnf(&self) -> Vec<Vec<LitStr>> {
    assert!(!self.inverse);
    match self
      .features
      .iter()
      .map(|x| x.as_slice())
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

/// [`CfgFeatures`] helper extension trait for slices.
pub trait CfgFeaturesExt<T: Clone> {
  /// Converts a sequence of `T` into a sequence of combinations of `T` for
  /// each possible feature.
  fn transpose(self) -> Vec<(CfgFeatures, Vec<T>)>;
}

impl<T: Clone> CfgFeaturesExt<T> for &[(CfgFeatures, T)] {
  fn transpose(self) -> Vec<(CfgFeatures, Vec<T>)> {
    let mut map: HashMap<Vec<LitStr>, Vec<T>> = HashMap::new();
    let mut default = Vec::new();
    for (features, item) in self {
      let features = features.to_dnf();
      if features.is_empty() {
        default.push(item.clone());
      } else {
        for feature in features {
          map.entry(feature).or_default().push(item.clone());
        }
      }
    }
    let mut result = Vec::new();
    for (features, mut items) in map {
      let features = CfgFeatures {
        features: vec![features],
        inverse: false,
      };
      items.append(&mut default.clone());
      result.push((features, items));
    }
    let features = result
      .iter()
      .flat_map(|(x, _)| x.features.clone())
      .collect();
    let features = CfgFeatures {
      features,
      inverse: true,
    };
    result.push((features, default));
    result
  }
}
