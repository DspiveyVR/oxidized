use crate::debugging::debugger::*;
use std::fmt::Display;

impl Display for DebuggerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebuggerState::TCState(tcstate) => write!(f, "{}", tcstate),
        }
    }
}
