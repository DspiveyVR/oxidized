
// // pub struct ODerefState {
// //     check_fn: Option<Box<TCFn>>,
// //     env_state: (ConcreteTyping, ConcreteTyping),
// //     method_name: String,
// // }

// /*

//     fn o_safe_place(
//         &self,
//         continuation_typing: &Self,
//         place: Place,
//         ownership_qualifier: OwnershipQualifier,
//         reborrow_exclusion_list: &VecSet<Place>,
//     ) -> Result<VecSet<Loan>, OxidizedError> {


// */

use crate::{ast::OwnershipQualifier, debugging::debugger::StepResult, static_analysis::data_structures::{ConcreteTyping, Loan, OxidizedError, Place, VecSet}};

pub type OCResult = Result<OCFnReturn, OxidizedError>;

#[derive(Clone)]
pub struct OCFnReturn {
    loans: VecSet<Loan>,
    reborrow_exclusion_list: VecSet<Place>
}

#[derive(Clone)]
pub struct OSafePlaceStateCapture {
    stack_typing: ConcreteTyping,
    continuation_typing: ConcreteTyping,
    place: Place,
    ownership_qualifier: OwnershipQualifier,
    reborrow_exclusion_list: VecSet<Place>
}

type OC0ParamFn = dyn FnOnce() -> StepResult;

pub struct OSafePlaceState {
    check_fn: Option<Box<OC0ParamFn>>,
    state_capture: OSafePlaceStateCapture,
    method_name: String,
}