use crate::debugging::debugger::*;
use crate::debugging::tc_state::*;
use std::fmt::Display;

impl Display for TExprState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let StateCapture::TCEnvStateCapture(env) = self.get_state_capture() {
            write!(
                f,
                "TExprState:{} {{ state: ({}, {}) }}",
                self.get_method_name(),
                env.stack_typing,
                env.continuation_typing
            )
        } else {
            panic!("Unexpected match")
        }
    }
}

impl Display for TLetrgnState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let StateCapture::TCEnvStateCapture(env) = self.get_state_capture() {
            write!(
                f,
                "TLetrgnState:{} {{ state: ({}, {}), e_result: {} }}",
                self.get_method_name(),
                env.stack_typing,
                env.continuation_typing,
                match &self.e_result {
                    Some(Ok(tup)) => format!("Some(({}, {}))", tup.0, tup.1),
                    Some(Err(err)) => format!("Some({})", err),
                    _ => "None".to_string(),
                }
            )
        } else {
            panic!("Unexpected match")
        }
    }
}

impl Display for TSeqState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let StateCapture::TCEnvStateCapture(env) = self.get_state_capture() {
            write!(
                f,
                "TSeqState:{} {{ state: ({}, {}), e1_result: {} }}",
                self.get_method_name(),
                env.stack_typing,
                env.continuation_typing,
                match &self.e1_result {
                    Some(Ok(tup)) => format!("Some(({}, {}))", tup.0, tup.1),
                    Some(Err(err)) => format!("Some({})", err),
                    _ => "None".to_string(),
                }
            )
        } else {
            panic!("Unexpected match")
        }
    }
}

impl Display for TAssignState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let StateCapture::TCEnvStateCapture(env) = self.get_state_capture() {
            write!(
                f,
                "TAssignState:{} {{ state: ({}, {})}}",
                self.get_method_name(),
                env.stack_typing,
                env.continuation_typing,
            )
        } else {
            panic!("Unexpected match")
        }
    }
}

impl Display for TLetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let StateCapture::TCEnvStateCapture(env) = self.get_state_capture() {
            write!(
                f,
                "TLetState:{} {{ state: ({}, {}), e1_result: {}, binding_result: {} }}",
                self.get_method_name(),
                env.stack_typing,
                env.continuation_typing,
                match &self.e1_result {
                    Some(Ok(tup)) => format!("Some(({}, {}))", tup.0, tup.1),
                    Some(Err(err)) => format!("Some({})", err),
                    _ => "None".to_string(),
                },
                match &self.binding_result {
                    Some(Ok(tup)) => format!("Some(({}, {}))", tup.0, tup.1),
                    Some(Err(err)) => format!("Some({})", err),
                    _ => "None".to_string(),
                }
            )
        } else {
            panic!("Unexpected match")
        }
    }
}
