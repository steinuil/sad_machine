# Sad Machine

`sad_machine` provides a macro to declaratively define a state machine
and the transitions between states. It's focused on providing a nice API
for applications that deal with event loops and use a state machine to keep
track of their state.

`sad_machine` is a fork of the [`sm`](https://github.com/rustic-games/sm)
library which removes the traits and keeps only the macro, and redesigns the
generated code to be more enum-friendly.

## Usage

`sad_machine` exposes only one macro, `state_machine!`. A quick example:

```rust
use sad_machine::state_machine;

state_machine! {
    Lock {
        InitialStates { Locked, Unlocked }

        TurnKey {
            Locked => Unlocked
            Unlocked => Locked
        }

        BreakKeyhole {
            Locked, Unlocked => Broken
        }

        Repair {
            Broken => Locked
        }
    }
}

fn main() {
    let mut lock = Lock::locked();

    loop {
        match lock {
            Lock::Locked(m @ LockedState::FromInit) => lock = m.turn_key(),
            Lock::Unlocked(m) => lock = m.turn_key(),
            Lock::Locked(m) => lock = m.break_keyhole(),
            Lock::Broken(_) => break,
        }
    }

    assert_eq!(lock, Lock::Broken(BrokenState::FromBreakKeyhole));
}
```

In this example, the macro generated:

- An enum called `Lock` containing all states of the enum.
- An enum for each state containing the name of the event that triggered
  the transition. For the `Unlocked` state, the enum is called `UnlockedState`
  and contains the two cases `FromInit, FromTurnKey`.
- Two initialization functions: `Lock::locked()` and `Lock::unlocked()`,
  mirroring the states defined in `InitialStates`.
- Transition methods for the state enums. For the `Broken` state,
  a `.repair()` method is generated which mirrors the `Repair` event.

A few differences from `sm`'s API:

- The generated code is not wrapped in a module, and all enums and functions
  are `pub`.
- Initial states are encoded as functions on the state enum.
- Transitions are encoded as methods on the object contained inside
  the cases of the state enum.
- The initial state and transition functions all return the state enum.
- The cases of the state enum contain the name of the event that triggered
  the transition. Each state has its own enum for this purpose.
- The transitions do not consume the original state machine.
- You can only define one state machine per macro instantiation.

### Descriptive Example

The below example explains step-by-step how to create a new state machine
using the provided macro, and then how to use the created machine in your
code.

#### Declaring a new State Machine

First, we import the macro from the crate:

```rust
use sad_machine::state_machine;
```

Next, we initiate the macro declaration:

```rust
state_machine! {
```

Then, provide a name for the machine, and declare a list of allowed initial
states:

```rust
    Lock {
        InitialStates { Locked, Unlocked }
```

Finally, we declare one or more events and the associated transitions:

```rust
        TurnKey {
            Locked => Unlocked
            Unlocked => Locked
        }

        BreakKeyhole {
            Locked, Unlocked => Broken
        }
    }
}
```

And we're done. We've defined our state machine structure, and the valid
transitions, and can now use this state machine in our code.

#### Using your State Machine

You can initialise the machine as follows:

```rust
let sm = Lock::locked();
```

We've initialised our machine in the `Locked` state. The `sm` is as an enum
covering all possible states of the state machine, and each state contains
the name of the event that triggered it. A full pattern match on the state
enum looks like this:

```rust
match lock {
    Lock::Locked(LockedState::FromInit) => ..,
    Lock::Locked(LockedState::FromTurnKey) => ..,
    Lock::Locked(LockedState::FromRepair) => ..,
    Lock::Unlocked(UnlockedState::FromInit) => ..,
    Lock::Unlocked(UnlockedState::FromTurnKey) => ..,
    Lock::Broken(BrokenState::FromBreakKeyhole) => ..,
}
```

To transition this machine to the `Unlocked` state, we send the `turn_key`
method on the LockedState object:

```rust
let lock = match lock {
    Lock::Locked(locked) => locked.turn_key(),
    _ => panic!("wrong state"),
}
```

## Caveat emptor, or why you might not want to use this crate

1. The state machine **does not consume the previous state** when performing
   a transition, as opposed to `sm`'s behavior, so be careful when operating in
   a concurrent context.

2. The API doesn't prevent you from constructing a state that is not one of the
   initial states, due to Rust's lack of private constructors for enums.

3. In the example above, the transition `TurnKey` is defined for two states,
   but the API does not allow you to pattern match on both cases that define that
   transition and then call the `turn_key` method in a single match case. An example:

    ```rust
    // This code will not compile!
    fn toggle_lock(lock: Lock) -> Lock {
        match lock {
            Lock::Locked(state) | Lock::Unlocked(state) => {
                do_stuff();
                state.turn_key()
            }
            _ => panic!("wrong state"),
        }
    }
    ```

    Since the previous state is not consumed, you can work around this by returning
    the next state from the match and doing the work you need to do outside of it.

    ```rust
    fn toggle_lock(lock: Lock) -> Lock {
        let new_state = match lock {
            Lock::Locked(locked) => locked.turn_key(),
            Lock::Unlocked(unlocked) => unlocked.turn_key(),
            _ => panic!("wrong state"),
        };

        do_stuff();
        new_state
    }
    ```

    This can get cumbersome if you have a function that needs to do several things
    inside of the match case where some states are equivalent. For example:

    ```rust
    fn advance(lock: Lock) -> Lock {
        match lock {
            Lock::Locked(state) => {
                do_stuff1();
                state.turn_key();
            }
            Lock::Unlocked(state) => {
                do_stuff1();
                state.turn_key();
            }
            Lock::Broken => {
                do_stuff2();
                lock
            }
        }
    }
    ```

## Why fork

Some of the design choices that `sm` makes conflict with my use case.

I was using the library in an event loop where:

1. The state is stored as its enum `Variant` representation, and can only
   advance by one step in a single loop
2. Multiple events can trigger a state change to a certain state, but I
   don't particularly care about the event that triggered the stage change

`sm` seems to have different design goals:

1. The `transition` method returns a `Machine` type and not an enum, which
   forces me to call `.as_enum()` on its result every time to store it as
   `Variant`, but makes it easy to trigger multiple state transitions in
   a single piece of code
2. The cases of the `Variant` enum also include the name of the event that
   triggered the state change, which led me to duplicate code in multiple
   branches for each state that had multiple entry points

`sad_machine`'s API focuses on the state enum rather than on concrete states.
The transition methods it generates return the enum, which makes it harder to
trigger multiple transitions in the same piece of code, but on the other hand
it removes the cruft of calling `.as_enum()` on the result, and its state enum
does not encode the event name in the name of its cases, but rather carries it
inside itself.

This forks keeps `sm`'s parser for the DSL to define the state machine and
changes the generated code.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

This was the license of the original crate and I'd rather not change it.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
