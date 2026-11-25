use crate::debugging::debugger::*;
use crate::debugging::oc_state::*;
use std::fmt::Display;

// #[derive(Clone)]
// pub struct OSafePlaceStateCapture {
//     pub stack_typing: ConcreteTyping,
//     pub continuation_typing: ConcreteTyping,
//     pub place: Place,
//     pub ownership_qualifier: OwnershipQualifier,
//     pub reborrow_exclusion_list: VecSet<Place>,
// }

impl Display for OSafePlaceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let StateCapture::OSafePlaceStateCapture(state) = self.get_state_capture() {
            write!(
                f,
                "OSafePlaceState:{} {{ Γ:{} Θ:{}\n(⊢ {} {}) }}",
                self.get_method_name(),
                state.stack_typing,
                state.continuation_typing,
                state.ownership_qualifier.to_string(),
                state.reborrow_exclusion_list
            )
        } else {
            panic!("Unexpected match")
        }
    }
}
