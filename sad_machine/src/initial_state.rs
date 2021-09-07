use convert_case::Casing;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct InitialStates(pub Vec<InitialState>);

impl Parse for InitialStates {
    /// example initial states tokens:
    ///
    /// ```text
    /// InitialStates { Locked, Unlocked }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut initial_states: Vec<InitialState> = Vec::new();

        // `InitialStates { ... }`
        //  ^^^^^^^^^^^^^
        let block_name: Ident = input.parse()?;

        if block_name != "InitialStates" {
            return Err(input.error("expected `InitialStates { ... }` block"));
        }

        // `InitialStates { ... }`
        //                  ^^^
        let block_initial_states;
        braced!(block_initial_states in input);

        // `InitialStates { Locked, Unlocked }`
        //                  ^^^^^^  ^^^^^^^^
        let punctuated_initial_states: Punctuated<Ident, Token![,]> =
            block_initial_states.parse_terminated(Ident::parse)?;

        for name in punctuated_initial_states {
            initial_states.push(InitialState { name });
        }

        Ok(InitialStates(initial_states))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InitialState {
    pub name: Ident,
}

impl Parse for InitialState {
    /// example initial state tokens:
    ///
    /// ```text
    /// Locked
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name = input.parse()?;

        Ok(InitialState { name })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InitialStateFns {
    pub enum_name: Ident,
    pub initial_states: Vec<InitialState>,
}

impl InitialStates {
    pub fn to_fn(&self, enum_name: &Ident) -> InitialStateFns {
        InitialStateFns {
            enum_name: enum_name.clone(),
            initial_states: self.0.clone(),
        }
    }
}

impl ToTokens for InitialStateFns {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for s in &self.initial_states {
            let fn_name = Ident::new(
                &s.name.to_string().to_case(convert_case::Case::Snake),
                s.name.span(),
            );
            let variant_name = &s.name;
            let struct_name = Ident::new(&format!("{}State", &s.name), Span::call_site());

            let enum_name = &self.enum_name;

            tokens.extend(quote! {
                pub fn #fn_name() -> #enum_name {
                    #enum_name::#variant_name(#struct_name::FromInit)
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::{parse2, parse_quote};

    #[test]
    fn test_initial_state_parse() {
        let left: InitialState = parse2(quote! { Unlocked }).unwrap();
        let right = InitialState {
            name: parse_quote! { Unlocked },
        };

        assert_eq!(left, right);
    }

    #[test]
    fn test_initial_states_parse() {
        let left: InitialStates = parse2(quote! {
            InitialStates { Locked, Unlocked }
        })
        .unwrap();

        let right = InitialStates(vec![
            InitialState {
                name: parse_quote! { Locked },
            },
            InitialState {
                name: parse_quote! { Unlocked },
            },
        ]);

        assert_eq!(left, right);
    }

    #[test]
    fn test_initial_states_to_tokens() {
        let initial_states = InitialStates(vec![
            InitialState {
                name: parse_quote! { Locked },
            },
            InitialState {
                name: parse_quote! { Unlocked },
            },
        ])
        .to_fn(&parse_quote! { Door });

        let left = quote! {
            pub fn locked() -> Door {
                Door::Locked(LockedState::FromInit)
            }

            pub fn unlocked() -> Door {
                Door::Unlocked(UnlockedState::FromInit)
            }
        };

        let mut right = TokenStream::new();
        initial_states.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
