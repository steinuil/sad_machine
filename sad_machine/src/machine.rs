use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident,
};

use crate::{
    initial_state::InitialStates,
    state::{State, States},
    state_transition::StateTransitions,
    transition::Transitions,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Machine {
    pub name: Ident,
    pub initial_states: InitialStates,
    pub transitions: Transitions,
}

impl Machine {
    fn states(&self) -> States {
        let mut states: Vec<State> = Vec::new();

        for t in &self.transitions.0 {
            if !states.iter().any(|s| s.name == t.from.name) {
                states.push(t.from.clone());
            }

            if !states.iter().any(|s| s.name == t.to.name) {
                states.push(t.to.clone());
            }
        }

        for i in &self.initial_states.0 {
            if !states.iter().any(|s| s.name == i.name) {
                states.push(State {
                    name: i.name.clone(),
                });
            }
        }

        States(states)
    }
}

impl Parse for Machine {
    /// example machine tokens:
    ///
    /// ```text
    /// TurnStile {
    ///     InitialStates { ... }
    ///
    ///     Push { ... }
    ///     Coin { ... }
    /// }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // `TurnStile { ... }`
        //  ^^^^^^^^^
        let name: Ident = input.parse()?;

        // `TurnStile { ... }`
        //              ^^^
        let block_machine;
        braced!(block_machine in input);

        // `InitialStates { ... }`
        //  ^^^^^^^^^^^^^^^^^^^^^
        let initial_states = InitialStates::parse(&block_machine)?;

        // `Push { ... }`
        //  ^^^^^^^^^^^^
        let transitions = Transitions::parse(&block_machine)?;

        Ok(Machine {
            name,
            initial_states,
            transitions,
        })
    }
}

impl ToTokens for Machine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;

        let machine_enum = MachineEnum { machine: &self };

        let states = &self.states();

        let initial_states = &self.initial_states.to_fn(name);

        let state_transitions = StateTransitions {
            enum_name: name,
            states,
            transitions: &self.transitions,
        };

        tokens.extend(quote! {
            #machine_enum

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum #name {
                #states
            }

            impl #name {
                #initial_states
            }

            #state_transitions
        });
    }
}

#[cfg(test)]
mod machines_tests {
    use super::*;
    use crate::{event::Event, initial_state::InitialState, transition::Transition};
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_machine_parse() {
        let left: Machine = syn::parse2(quote! {
           TurnStile {
               InitialStates { Locked, Unlocked }

               Coin { Locked => Unlocked }
               Push { Unlocked => Locked }
           }
        })
        .unwrap();

        let right = Machine {
            name: parse_quote! { TurnStile },
            initial_states: InitialStates(vec![
                InitialState {
                    name: parse_quote! { Locked },
                },
                InitialState {
                    name: parse_quote! { Unlocked },
                },
            ]),
            transitions: Transitions(vec![
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

        assert_eq!(left, right);
    }

    #[test]
    fn test_machine_to_tokens() {
        let machine = Machine {
            name: parse_quote! { TurnStile },
            initial_states: InitialStates(vec![
                InitialState {
                    name: parse_quote! { Unlocked },
                },
                InitialState {
                    name: parse_quote! { Locked },
                },
            ]),
            transitions: Transitions(vec![Transition {
                event: Event {
                    name: parse_quote! { Push },
                },
                from: State {
                    name: parse_quote! { Unlocked },
                },
                to: State {
                    name: parse_quote! { Locked },
                },
            }]),
        };

        let left = quote! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum UnlockedState {
                FromInit
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum LockedState {
                FromPush,
                FromInit
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum TurnStile {
                Unlocked(UnlockedState),
                Locked(LockedState)
            }

            impl TurnStile {
                pub fn unlocked() -> TurnStile {
                    TurnStile::Unlocked(UnlockedState::FromInit)
                }

                pub fn locked() -> TurnStile {
                    TurnStile::Locked(LockedState::FromInit)
                }
            }

            impl UnlockedState {
                pub fn push(&self) -> TurnStile {
                    TurnStile::Locked(LockedState::FromPush)
                }
            }
        };

        let mut right = TokenStream::new();
        machine.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}

#[derive(Debug)]
#[allow(single_use_lifetimes)]
struct MachineEnum<'a> {
    machine: &'a Machine,
}

#[allow(single_use_lifetimes)]
impl<'a> ToTokens for MachineEnum<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for s in &self.machine.states() {
            let state_enum = Ident::new(&format!("{}State", s.name), Span::call_site());

            let mut events = self
                .machine
                .transitions
                .0
                .iter()
                .filter_map(|t| {
                    if t.to.name.to_string() == s.name.to_string() {
                        let event = Ident::new(&format!("From{}", t.event.name), Span::call_site());
                        Some(event)
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();

            if self
                .machine
                .initial_states
                .0
                .iter()
                .any(|is| is.name.to_string() == s.name.to_string())
            {
                events.push(Ident::new(&"FromInit", Span::call_site()));
            }

            let state_enum = &state_enum;
            let events = &events;

            tokens.extend(quote! {
                #[derive(Debug, Clone, PartialEq, Eq)]
                pub enum #state_enum {
                    #(#events),*
                }
            });
        }
    }
}

#[cfg(test)]
mod machine_enum_tests {
    use super::*;
    use crate::{event::Event, initial_state::InitialState, transition::Transition};
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_machine_enum_to_tokens() {
        let machine = Machine {
            name: parse_quote! { turn_stile },
            initial_states: InitialStates(vec![
                InitialState {
                    name: parse_quote! { Locked },
                },
                InitialState {
                    name: parse_quote! { Unlocked },
                },
            ]),
            transitions: Transitions(vec![
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

        let machine_enum = MachineEnum { machine: &machine };

        let left = quote! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum LockedState {
                FromPush,
                FromInit
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum UnlockedState {
                FromCoin,
                FromInit
            }
        };

        let mut right = TokenStream::new();
        machine_enum.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }
}
