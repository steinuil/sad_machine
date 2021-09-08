use convert_case::Casing;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    token::Comma,
    Token,
};

use crate::{event::Event, state::State};

#[derive(Debug, PartialEq)]
pub(crate) struct Transitions(pub Vec<Transition>);

impl Parse for Transitions {
    /// example transitions tokens:
    ///
    /// ```text
    /// Push { ... }
    /// Coin { ... }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut transitions: Vec<Transition> = Vec::new();
        while !input.is_empty() {
            // `Coin { Locked, Unlocked => Unlocked }`
            //  ^^^^
            let event = Event::parse(input)?;

            // `Coin { Locked, Unlocked => Unlocked }`
            //         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
            let block_transition;
            braced!(block_transition in input);

            while !block_transition.is_empty() {
                let mut from_states: Vec<State> = Vec::new();

                // `Coin { Locked, Unlocked => Unlocked }`
                //                          ^^
                while !block_transition.peek(Token![=>]) {
                    // `Coin { Locked, Unlocked => Unlocked }`
                    //               ^
                    if block_transition.peek(Token![,]) {
                        let _: Comma = block_transition.parse()?;
                        continue;
                    }

                    // `Coin { Locked, Unlocked => Unlocked }`
                    //         ^^^^^^  ^^^^^^^^
                    from_states.push(State::parse(&block_transition)?);
                }

                // `Coin { Locked, Unlocked => Unlocked }`
                //                          ^^
                let _: Token![=>] = block_transition.parse()?;

                // `Coin { Locked, Unlocked => Unlocked }`
                //                             ^^^^^^^^
                let to = State::parse(&block_transition)?;

                for from in from_states {
                    let event = event.clone();
                    let to = to.clone();

                    transitions.push(Transition { event, from, to })
                }
            }
        }

        Ok(Transitions(transitions))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Transition {
    pub event: Event,
    pub from: State,
    pub to: State,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TransitionFns {
    pub enum_name: Ident,
    pub transitions: Vec<Transition>,
}

impl Transitions {
    pub fn to_fns(&self, enum_name: &Ident) -> TransitionFns {
        TransitionFns {
            enum_name: enum_name.clone(),
            transitions: self.0.clone(),
        }
    }
}

impl ToTokens for TransitionFns {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for t in &self.transitions {
            let event_fn = Ident::new(
                &t.event.name.to_string().to_case(convert_case::Case::Snake),
                t.event.name.span(),
            );

            let to_enum = &t.to.name.clone();

            let to_struct = Ident::new(&format!("{}State", t.to.name), t.to.name.span());

            let event_enum = Ident::new(&format!("From{}", t.event.name), t.event.name.span());

            let enum_name = &self.enum_name;

            tokens.extend(quote! {
                pub fn #event_fn(&self) -> #enum_name {
                    #enum_name::#to_enum(#to_struct::#event_enum)
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_transitions_parse() {
        let left: Transitions = syn::parse2(quote! {
            Push { Locked, Unlocked => Locked }
            Coin { Locked, Unlocked => Unlocked }
        })
        .unwrap();

        let right = Transitions(vec![
            Transition {
                event: Event {
                    name: parse_quote! { Push },
                },
                from: State {
                    name: parse_quote! { Locked },
                },
                to: State {
                    name: parse_quote! { Locked },
                },
            },
            Transition {
                event: Event {
                    name: parse_quote! { Push },
                },
                from: State {
                    name: parse_quote! { Unlocked },
                },
                to: State {
                    name: parse_quote! { Locked },
                },
            },
            Transition {
                event: Event {
                    name: parse_quote! { Coin },
                },
                from: State {
                    name: parse_quote! { Locked },
                },
                to: State {
                    name: parse_quote! { Unlocked },
                },
            },
            Transition {
                event: Event {
                    name: parse_quote! { Coin },
                },
                from: State {
                    name: parse_quote! { Unlocked },
                },
                to: State {
                    name: parse_quote! { Unlocked },
                },
            },
        ]);

        assert_eq!(left, right);
    }

    #[test]
    fn test_transitions_to_tokens() {
        let transitions = Transitions(vec![
            Transition {
                event: Event {
                    name: parse_quote! { Push },
                },
                from: State {
                    name: parse_quote! { Locked },
                },
                to: State {
                    name: parse_quote! { Locked },
                },
            },
            Transition {
                event: Event {
                    name: parse_quote! { Push },
                },
                from: State {
                    name: parse_quote! { Unlocked },
                },
                to: State {
                    name: parse_quote! { Locked },
                },
            },
            Transition {
                event: Event {
                    name: parse_quote! { Coin },
                },
                from: State {
                    name: parse_quote! { Locked },
                },
                to: State {
                    name: parse_quote! { Unlocked },
                },
            },
            Transition {
                event: Event {
                    name: parse_quote! { Coin },
                },
                from: State {
                    name: parse_quote! { Unlocked },
                },
                to: State {
                    name: parse_quote! { Unlocked },
                },
            },
        ])
        .to_fns(&parse_quote! { TurnStile });

        let left = quote! {
            pub fn push(&self) -> TurnStile {
                TurnStile::Locked(LockedState::FromPush)
            }

            pub fn push(&self) -> TurnStile {
                TurnStile::Locked(LockedState::FromPush)
            }

            pub fn coin(&self) -> TurnStile {
                TurnStile::Unlocked(UnlockedState::FromCoin)
            }

            pub fn coin(&self) -> TurnStile {
                TurnStile::Unlocked(UnlockedState::FromCoin)
            }
        };

        let mut right = TokenStream::new();
        transitions.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
