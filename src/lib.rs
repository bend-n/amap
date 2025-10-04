use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Error, Expr, Lit, Pat, PatConst, Stmt, Token,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
};

#[derive(Clone)]
struct Index {
    indices: Vec<Expr>,
    value: Expr,
}
fn indices(index: &Pat) -> syn::Result<Vec<Expr>> {
    match index {
        Pat::Lit(v) => match &v.lit {
            Lit::Int(_) => Ok(vec![v.clone().into()]),
            _ => Err(Error::new_spanned(v, "must be numeric literal"))?,
        },
        Pat::Or(v) => v.cases.iter().map(indices).flatten_ok().collect(),
        Pat::Range(r) => {
            let s = r.span();
            let r = r.clone();
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

            match r.limits {
                syn::RangeLimits::Closed(..) => Ok((begin..=end)
                    .map(|x: usize| syn::parse::<Expr>(x.to_token_stream().into()).unwrap())
                    .collect()),
                syn::RangeLimits::HalfOpen(..) => Ok((begin..end)
                    .map(|x: usize| syn::parse::<Expr>(x.to_token_stream().into()).unwrap())
                    .collect()),
            }
        }
        Pat::Const(PatConst { block, .. }) => {
            Ok(vec![if let [Stmt::Expr(x, None)] = &block.stmts[..] {
                x.clone()
            } else {
                Expr::Block(syn::ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: block.clone(),
                })
            }])
        }
        _ => Err(Error::new(
            index.span(),
            "pattern must be literal(5) | or(5 | 4) | range(4..5) | const { .. }",
        ))?,
    }
}

impl Parse for Index {
    fn parse(input: ParseStream<'_>) -> parse::Result<Index> {
        let index = Pat::parse_multi(input)?;
        let indices = indices(&index)?;
        input.parse::<Token![=>]>()?;
        Ok(Index {
            indices,
            value: input.parse()?,
        })
    }
}

struct Map(Punctuated<Index, Token![,]>);
impl Parse for Map {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed = Punctuated::<Index, Token![,]>::parse_terminated(input)?;
        if parsed.is_empty() {
            return Err(input.error("no keys"));
        }
        Ok(Map(parsed))
    }
}

impl Map {
    fn into(self, d: TokenStream, f: impl Fn(&Expr) -> TokenStream + Copy) -> TokenStream {
        let map = self
            .0
            .into_iter()
            .zip(1..)
            .flat_map(|(Index { indices, value }, i)| {
                indices.into_iter().map(move |x| {
                    let s = format!(
                        "duplicate / overlapping key @ pattern `{}` (#{i})",
                        x.to_token_stream()
                            .to_string()
                            .replace('{', "{{")
                            .replace('}', "}}")
                    );
                    let value = f(&value);
                    quote! {{
                        let (index, value) = { let (__ඞඞ, __set) = ((), ()); (#x, #value) };
                        assert!(!__set[index], #s);
                        __set[index] = true;
                        __ඞඞ[index] = value;
                    }}
                })
            });
        quote! {{
            let mut __ඞඞ = [#d; _];
            const fn steal<const N:usize, T>(_: &[T; N]) -> [bool; N] { [false; N] }
            let mut __set = steal(&__ඞඞ);
            #(#map)*
            __ඞඞ
        }}
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
///     44 => Y::B,
/// };
/// assert_eq!(X[44].as_ref().unwrap(), &Y::B);
/// ```
#[proc_macro]
pub fn amap(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as Map)
        .into(quote! { const { None } }, |x| quote! { Some(#x)})
        .into()
}

#[proc_macro]
/// This method uses default instead of Option<T>. Nightly required for use in const.
/// ```
/// # use amap::amap_d;
/// let x: [u8; 42] = amap_d! {
///     4 => 2,
///     16..25 => 4,
/// };
/// assert_eq!(x[17], 4);
/// ```
pub fn amap_d(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as Map)
        .into(
            quote! { ::core::default::Default::default() },
            |x| quote! { #x },
        )
        .into()
}
