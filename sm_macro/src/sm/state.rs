use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::vec::IntoIter;
use syn::{
    parse::{Parse, ParseStream, Result},
    Ident,
};

#[derive(Debug, PartialEq)]
pub(crate) struct States(pub Vec<State>);

impl ToTokens for States {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let states = &self.0;

        tokens.extend(quote! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum State {
                #(#states),*
            }
        })
    }
}

#[allow(single_use_lifetimes)] // TODO: how to fix this?
impl<'a> IntoIterator for &'a States {
    type IntoIter = IntoIter<State>;
    type Item = State;

    fn into_iter(self) -> Self::IntoIter {
        self.0.clone().into_iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct State {
    pub name: Ident,
}

impl Parse for State {
    /// example state tokens:
    ///
    /// ```text
    /// Locked
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name = input.parse()?;

        Ok(State { name })
    }
}

impl ToTokens for State {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let struct_name = Ident::new(&format!("{}State", self.name), self.name.span());

        tokens.extend(quote! {
            #name(#struct_name)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_state_parse() {
        let left: State = syn::parse2(quote! { Unlocked }).unwrap();
        let right = State {
            name: parse_quote! { Unlocked },
        };

        assert_eq!(left, right);
    }

    #[test]
    fn test_state_to_tokens() {
        let state = State {
            name: parse_quote! { Unlocked },
        };

        let left = quote! {
            Unlocked(UnlockedState)
        };

        let mut right = TokenStream::new();
        state.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }

    #[test]
    fn test_states_to_tokens() {
        let states = States(vec![
            State {
                name: parse_quote! { Locked },
            },
            State {
                name: parse_quote! { Unlocked },
            },
        ]);

        let left = quote! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum State {
                Locked(LockedState),
                Unlocked(UnlockedState)
            }
        };

        let mut right = TokenStream::new();
        states.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
