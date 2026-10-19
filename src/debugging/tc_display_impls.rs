use crate::debugging::tc_state::*;
use std::fmt::Display;

impl Display for TExprState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TExprState:{} {{ state: ({}, {}) }}",
            self.get_method_name(),
            self.get_env_state().0,
            self.get_env_state().1
        )
    }
}
impl Display for TLetrgnState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TLetrgnState:{} {{ state: ({}, {}), e_result: {} }}",
            self.get_method_name(),
            self.get_env_state().0,
            self.get_env_state().1,
            match &self.e_result {
                Some(Ok(tup)) => format!("Some(({}, {}))", tup.0, tup.1),
                Some(Err(err)) => format!("Some({})", err),
                _ => "None".to_string(),
            }
        )
    }
}
impl Display for TSeqState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TSeqState:{} {{ state: ({}, {}), e1_result: {} }}",
            self.get_method_name(),
            self.get_env_state().0,
            self.get_env_state().1,
            match &self.e1_result {
                Some(Ok(tup)) => format!("Some(({}, {}))", tup.0, tup.1),
                Some(Err(err)) => format!("Some({})", err),
                _ => "None".to_string(),
            }
        )
    }
}
impl Display for TAssignState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TAssignState:{} {{ state: ({}, {})}}",
            self.get_method_name(),
            self.get_env_state().0,
            self.get_env_state().1,
        )
    }
}

impl Display for TLetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TLetState:{} {{ state: ({}, {}), e1_result: {}, binding_result: {} }}",
            self.get_method_name(),
            self.get_env_state().0,
            self.get_env_state().1,
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
    }
}
