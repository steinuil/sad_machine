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
