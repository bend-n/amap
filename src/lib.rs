use std::collections::{hash_map::Entry, HashMap};

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Expr, Lit, Pat, Token,
};

#[derive(Clone)]
struct Index {
    indices: Vec<usize>,
    value: Expr,
}

impl Parse for Index {
    fn parse(input: ParseStream<'_>) -> parse::Result<Index> {
        let index = Pat::parse_multi(input)?;
        match index {
            Pat::Lit(v) => match v.lit {
                Lit::Int(v) => {
                    input.parse::<Token![=>]>()?;
                    Ok(Index {
                        indices: vec![v.base10_parse()?],
                        value: input.parse()?,
                    })
                }
                _ => Err(Error::new_spanned(v, "must be numeric literal"))?,
            },
            Pat::Or(v) => {
                let mut index = Vec::with_capacity(v.cases.len());
                for p in v.cases {
                    match p {
                        Pat::Lit(v) => match v.lit {
                            Lit::Int(v) => index.push(v.base10_parse()?),
                            _ => Err(Error::new_spanned(v, "must be numeric literal"))?,
                        },
                        _ => Err(Error::new_spanned(
                            p,
                            "pattern must include only literal ints",
                        ))?,
                    }
                }
                input.parse::<Token![=>]>()?;
                Ok(Index {
                    indices: index,
                    value: input.parse()?,
                })
            }
            Pat::Range(r) => {
                let s = r.span();
                let begin = match *r.start.ok_or(Error::new(s, "range must be bounded"))? {
                    Expr::Lit(v) => match v.lit {
                        Lit::Int(v) => v.base10_parse()?,
                        _ => Err(Error::new_spanned(
                            v,
                            "range start bound must be integer literal",
                        ))?,
                    },
                    e => Err(Error::new_spanned(
                        e,
                        "range start bound must include only literal ints",
                    ))?,
                };
                let end = match *r.end.ok_or(Error::new(s, "range must be bounded"))? {
                    Expr::Lit(v) => match v.lit {
                        Lit::Int(v) => v.base10_parse()?,
                        _ => Err(Error::new_spanned(
                            v,
                            "range end bound must be integer literal",
                        ))?,
                    },
                    e => Err(Error::new_spanned(
                        e,
                        "range end bound must include only literal ints",
                    ))?,
                };
                input.parse::<Token![=>]>()?;
                match r.limits {
                    syn::RangeLimits::Closed(..) => Ok(Index {
                        indices: (begin..=end).collect(),
                        value: input.parse()?,
                    }),
                    syn::RangeLimits::HalfOpen(..) => Ok(Index {
                        indices: (begin..end).collect(),
                        value: input.parse()?,
                    }),
                }
            }
            _ => Err(input.error("pattern must be literal(5) | or(5 | 4) | range(4..5)"))?,
        }
    }
}

struct Map(Vec<Option<Expr>>);
impl Parse for Map {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed = Punctuated::<Index, Token![,]>::parse_terminated(input)?;
        if parsed.is_empty() {
            return Err(input.error("no keys"));
        }
        let mut flat = HashMap::new();
        let mut largest = 0;
        for Index { value, indices } in parsed.into_iter() {
            for index in indices {
                if index > largest {
                    largest = index;
                }
                match flat.entry(index) {
                    Entry::Occupied(_) => Err(input.error("duplicate key"))?,
                    Entry::Vacant(v) => v.insert(value.clone()),
                };
            }
        }
        let mut out = vec![None; largest + 1];
        for (index, expr) in flat.into_iter() {
            out[index] = Some(expr)
        }
        Ok(Map(out))
    }
}

/// Easily make a `[Option<T>; N]`
///
/// ```
/// # use amap::amap;
/// #[derive(Debug, PartialEq)]
/// enum Y {
///     A,
///     B,
///     C,
///     D,
/// }
/// static X: [Option<Y>; 46] = amap! {
///     2..=25 => Y::A,
///     26 | 32 => Y::C,
///     27..32 => Y::D,
///     45 => Y::B,
/// };
/// assert_eq!(X[45].as_ref().unwrap(), &Y::B);
/// ```
#[proc_macro]
pub fn amap(input: TokenStream) -> TokenStream {
    let map = parse_macro_input!(input as Map);
    let map = map.0.iter().map(|index| match index {
        Some(v) => quote!(::core::option::Option::Some(#v)),
        None => quote!(::core::option::Option::None),
    });
    quote! {
        [#(#map), *]
    }
    .into()
}
