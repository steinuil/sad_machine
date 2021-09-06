use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::sm::{
    state::States,
    transition::{Transition, Transitions},
};

#[derive(Debug, PartialEq)]
#[allow(single_use_lifetimes)]
pub(crate) struct StateTransitions<'a> {
    pub states: &'a States,
    pub transitions: &'a Transitions,
}

#[allow(single_use_lifetimes)]
impl<'a> ToTokens for StateTransitions<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for s in self.states {
            let struct_name = Ident::new(&format!("{}State", s.name), Span::call_site());

            let transitions = self
                .transitions
                .0
                .iter()
                .filter(|t| t.from.name.to_string() == s.name.to_string())
                .collect::<Vec<&Transition>>();

            if transitions.is_empty() {
                continue;
            }

            tokens.extend(quote! {
                impl #struct_name {
                    #(#transitions)*
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sm::{event::Event, state::State};

    use super::*;
    use syn::parse_quote;

    #[test]
    fn state_transition_tokens() {
        let state_transitions = StateTransitions {
            states: &States(vec![parse_quote!(Locked), parse_quote!(Unlocked)]),
            transitions: &Transitions(vec![
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
                        name: parse_quote! { Push },
                    },
                    from: State {
                        name: parse_quote! { Unlocked },
                    },
                    to: State {
                        name: parse_quote! { Locked },
                    },
                },
            ]),
        };

        let left = quote! {
            impl LockedState {
                pub fn coin(&self) -> State {
                    State::Unlocked(UnlockedState::FromCoin)
                }
            }

            impl UnlockedState {
                pub fn push(&self) -> State {
                    State::Locked(LockedState::FromPush)
                }
            }
        };

        let mut right = TokenStream::new();
        state_transitions.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
