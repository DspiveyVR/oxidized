use std::vec::IntoIter;

use crate::{
    ast::OwnershipQualifier,
    debugging::debugger::*,
    static_analysis::data_structures::{ConcreteTyping, Loan, OxidizedError, Place, Type, VecSet},
};
use color_print::cprintln;

pub type OCResult = Result<OCFnReturn, OxidizedError>;

pub struct ODerefState {
    step1_fn: Option<Box<OC0ParamFn>>,
    recursive_check_fn: Option<Box<OC0ParamFn>>,
    step2_fn: Option<Box<OC0ParamFn>>,
    r_loans: IntoIter<Loan>,
    state_capture: ODerefStateCapture,
    method_name: String,
}

#[derive(Clone)]
pub struct OCFnReturn {
    pub loans: VecSet<Loan>,
    pub reborrow_exclusion_list: VecSet<Place>,
}

#[derive(Clone)]
pub struct OSafePlaceStateCapture {
    pub stack_typing: ConcreteTyping,
    pub continuation_typing: ConcreteTyping,
    pub place: Place,
    pub ownership_qualifier: OwnershipQualifier,
    pub reborrow_exclusion_list: VecSet<Place>,
}

#[derive(Clone)]
pub struct ODerefStateCapture {
    pub stack_typing: ConcreteTyping,
    pub continuation_typing: ConcreteTyping,
    pub ref_type: Type,
    pub place: Place,
    pub ownership_qualifier: OwnershipQualifier,
    pub reborrow_exclusion_list: VecSet<Place>,
}

type OC0ParamFn = dyn FnOnce() -> StepResult;

pub struct OSafePlaceState {
    check_fn: Option<Box<OC0ParamFn>>,
    state_capture: OSafePlaceStateCapture,
    method_name: String,
}

impl DebuggerState for OSafePlaceState {
    fn step_into(mut self: Box<Self>) -> StepResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e_fn()</yellow>");
        }
        let mut e_return = (self.check_fn.take().unwrap())()?;
        e_return.yielded_state = Some(self);
        Ok(e_return)
    }

    fn step_over(self: Box<Self>) -> StepResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> GenericFnResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn handle_result(self: Box<Self>, _: FnEnvelope) -> Result<EvalReturn, OxidizedError> {
        panic!("Handle result is not defined for OSafePlaceState");
    }

    fn get_state_capture(&self) -> StateCapture {
        StateCapture::OSafePlaceStateCapture(self.state_capture.clone())
    }

    fn get_state_name(&self) -> String {
        "SafePlace".to_string()
    }

    fn get_method_name(&self) -> String {
        self.method_name.clone()
    }
}

impl OSafePlaceState {
    pub fn new_obj<OC0ParamFn: FnOnce() -> StepResult + 'static>(
        check_fn: OC0ParamFn,
        state_capture: OSafePlaceStateCapture,
        method_name: String,
    ) -> StepResult {
        Ok(StepEnvelope {
            yielded_state: None,
            return_val: EvalReturn::DebuggerState(Box::new(Self {
                check_fn: Some(Box::new(check_fn)),
                state_capture,
                method_name,
            })),
        })
    }
}
