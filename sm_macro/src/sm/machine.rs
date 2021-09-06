use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident,
};

use crate::sm::{
    initial_state::InitialStates,
    state::{State, States},
    state_transition::StateTransitions,
    transition::Transitions,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Machines(Vec<Machine>);

impl Parse for Machines {
    /// example machines tokens:
    ///
    /// ```text
    /// TurnStile { ... }
    /// Lock { ... }
    /// MyStateMachine { ... }
    /// ```
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut machines: Vec<Machine> = Vec::new();

        while !input.is_empty() {
            // `TurnStile { ... }`
            //  ^^^^^^^^^^^^^^^^^
            let machine = Machine::parse(input)?;
            machines.push(machine);
        }

        Ok(Machines(machines))
    }
}

impl ToTokens for Machines {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for machine in &self.0 {
            machine.to_tokens(tokens);
        }
    }
}

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

        let initial_states = &self.initial_states;

        let states = &self.states();

        let state_transitions = StateTransitions {
            states,
            transitions: &self.transitions,
        };

        tokens.extend(quote! {
            mod #name {
                #machine_enum
                #states
                #initial_states
                #state_transitions
            }
        });
    }
}

#[cfg(test)]
mod machines_tests {
    use super::*;
    use crate::sm::{event::Event, initial_state::InitialState, transition::Transition};
    use proc_macro2::TokenStream;
    use syn::{self, parse_quote};

    #[test]
    fn test_machine_parse() {
        let left: Machine = syn::parse2(quote! {
           turn_stile {
               InitialStates { Locked, Unlocked }

               Coin { Locked => Unlocked }
               Push { Unlocked => Locked }
           }
        })
        .unwrap();

        let right = Machine {
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

        assert_eq!(left, right);
    }

    #[test]
    fn test_machine_to_tokens() {
        let machine = Machine {
            name: parse_quote! { turn_stile },
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
            mod turn_stile {
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
                pub enum State {
                    Unlocked(UnlockedState),
                    Locked(LockedState)
                }

                pub fn unlocked() -> State {
                    State::Unlocked(UnlockedState::FromInit)
                }

                pub fn locked() -> State {
                    State::Locked(LockedState::FromInit)
                }

                impl UnlockedState {
                    pub fn push(&self) -> State {
                        State::Locked(LockedState::FromPush)
                    }
                }
            }
        };

        let mut right = TokenStream::new();
        machine.to_tokens(&mut right);

        assert_eq!(format!("{}", left), format!("{}", right))
    }

    #[test]
    fn test_machines_parse() {
        let left: Machines = syn::parse2(quote! {
           turn_stile {
               InitialStates { Locked, Unlocked }

               Coin { Locked => Unlocked }
               Push { Unlocked => Locked }
           }

           Lock {
               InitialStates { Locked, Unlocked }

               TurnKey {
                   Locked => Unlocked
                   Unlocked => Locked
                }
           }
        })
        .unwrap();

        let right = Machines(vec![
            Machine {
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
            },
            Machine {
                name: parse_quote! { Lock },
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
                            name: parse_quote! { TurnKey },
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
                            name: parse_quote! { TurnKey },
                        },
                        from: State {
                            name: parse_quote! { Unlocked },
                        },
                        to: State {
                            name: parse_quote! { Locked },
                        },
                    },
                ]),
            },
        ]);

        assert_eq!(left, right);
    }

    #[test]
    fn test_machines_to_tokens() {
        let machines = Machines(vec![
            Machine {
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
            },
            Machine {
                name: parse_quote! { lock },
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
                            name: parse_quote! { TurnKey },
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
                            name: parse_quote! { TurnKey },
                        },
                        from: State {
                            name: parse_quote! { Unlocked },
                        },
                        to: State {
                            name: parse_quote! { Locked },
                        },
                    },
                ]),
            },
        ]);

        let left = quote! {
            mod turn_stile {
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

                #[derive(Debug, Clone, PartialEq, Eq)]
                pub enum State {
                    Locked(LockedState),
                    Unlocked(UnlockedState)
                }

                pub fn locked() -> State {
                    State::Locked(LockedState::FromInit)
                }

                pub fn unlocked() -> State {
                    State::Unlocked(UnlockedState::FromInit)
                }

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
            }

            mod lock {
                #[derive(Debug, Clone, PartialEq, Eq)]
                pub enum LockedState {
                    FromTurnKey,
                    FromInit
                }

                #[derive(Debug, Clone, PartialEq, Eq)]
                pub enum UnlockedState {
                    FromTurnKey,
                    FromInit
                }

                #[derive(Debug, Clone, PartialEq, Eq)]
                pub enum State {
                    Locked(LockedState),
                    Unlocked(UnlockedState)
                }

                pub fn locked() -> State {
                    State::Locked(LockedState::FromInit)
                }

                pub fn unlocked() -> State {
                    State::Unlocked(UnlockedState::FromInit)
                }

                impl LockedState {
                    pub fn turn_key(&self) -> State {
                        State::Unlocked(UnlockedState::FromTurnKey)
                    }
                }

                impl UnlockedState {
                    pub fn turn_key(&self) -> State {
                        State::Locked(LockedState::FromTurnKey)
                    }
                }
            }
        };

        let mut right = TokenStream::new();
        machines.to_tokens(&mut right);

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
    use crate::sm::{event::Event, initial_state::InitialState, transition::Transition};
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
