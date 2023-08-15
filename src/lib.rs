use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Error, Expr, LitInt, Token,
};

#[derive(Clone)]
struct Index {
    index: usize,
    value: Expr,
}

impl std::fmt::Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl Parse for Index {
    fn parse(input: ParseStream<'_>) -> parse::Result<Index> {
        let index = input.parse::<LitInt>()?;
        let index = index.base10_parse()?;
        input.parse::<Token![=>]>()?;
        let value = input.parse()?;
        Ok(Index { index, value })
    }
}

struct Map(Vec<Option<Index>>);
impl Parse for Map {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed = Punctuated::<Index, Token![,]>::parse_terminated(input)?;
        let mut all = parsed.into_iter().collect::<Vec<_>>();
        if all.len() == 0 {
            return Err(input.error("no keys"));
        }
        all.sort_unstable_by(|a, b| a.index.cmp(&b.index));
        let max = all[all.len() - 1].index;
        let mut out: Vec<Option<Index>> = vec![None; max + 1];
        for Index { value, index } in all {
            let o = out.get_mut(index).unwrap();
            match o {
                Some(_) => {
                    // err.combine(Error::new_spanned(&v.value, "other duplicate key"));
                    return Err(Error::new_spanned(&value, "duplicate keys"));
                }
                None => *o = Some(Index { value, index }),
            }
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
///    A,
///    B,
///    C,
/// }
/// static X: [Option<Y>; 46] = amap! {
///     2 => Y::A,
///     5 => Y::C,
///     45 => Y::B,
/// };
/// assert_eq!(X[45].as_ref().unwrap(), &Y::B);
/// ```
#[proc_macro]
pub fn amap(input: TokenStream) -> TokenStream {
    let map = parse_macro_input!(input as Map);
    let map = map.0.iter().map(|index| {
        if let Some(index) = index {
            let v = &index.value;
            quote!(Some(#v))
        } else {
            quote!(None)
        }
    });
    quote! {
        [#(#map), *]
    }
    .into()
}
