use crate::{
    ast::OwnershipQualifier,
    debugging::debugger::*,
    static_analysis::data_structures::{ConcreteTyping, Loan, OxidizedError, Place, Type, VecSet},
};
use color_print::cprintln;
use std::vec::IntoIter;

pub type OCResult = Result<OCFnReturn, OxidizedError>;

#[derive(Clone)]
pub struct OCFnReturn {
    pub loans: VecSet<Loan>,
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

pub struct ODerefState {
    step1_fn: Option<Box<OC0ParamFn>>,
    recursive_check_fn: Option<Box<OCDerefExclParamFn>>,
    step2_fn: Option<Box<OCDerefExclParamFn>>,
    r_loans: Option<IntoIter<Loan>>,
    new_loans: Option<VecSet<Loan>>,
    state_capture: ODerefStateCapture,
    method_name: String,
}

impl DebuggerState for ODerefState {
    fn step_into(self: Box<Self>) -> StepResult {
        if let Some(_) = self.step1_fn {
            if DEBUG_OUTPUT {
                cprintln!("<red>[[step_into]]\n{}</red>", self);
                cprintln!("<yellow>[called]: step1_fn()</yellow>");
            }

            let mut s1_return = (self.step1_fn.take().unwrap())()?;
            s1_return.yielded_state = Some(self);
            // Ok(e_return)
        } else if let Some(_) = self.recursive_check_fn {
            todo!()
        } else {
            self.step_over()
        }
    }

    fn step_over(self: Box<Self>) -> StepResult {
        todo!()
    }

    fn evaluate(self: Box<Self>) -> GenericFnResult {
        todo!()
    }

    fn handle_result(self: Box<Self>, result: FnEnvelope) -> Result<EvalReturn, OxidizedError> {
        todo!()
    }

    fn get_state_capture(&self) -> StateCapture {
        todo!()
    }

    fn get_state_name(&self) -> String {
        todo!()
    }

    fn get_method_name(&self) -> String {
        todo!()
    }
}

impl ODerefState {
    pub fn new_obj<
        OC0ParamFn: FnOnce() -> StepResult + 'static,
        OCDerefExclParamFn: FnOnce(VecSet<Place>) -> StepResult + 'static,
    >(
        step1_fn: OC0ParamFn,
        recursive_check_fn: OCDerefExclParamFn,
        step2_fn: OCDerefExclParamFn,
        state_capture: ODerefStateCapture,
        method_name: String,
    ) -> StepResult {
        Ok(StepEnvelope {
            yielded_state: None,
            return_val: EvalReturn::DebuggerState(Box::new(Self {
                step1_fn: Some(Box::new(step1_fn)),
                recursive_check_fn: Some(Box::new(recursive_check_fn)),
                step2_fn: Some(Box::new(step2_fn)),
                state_capture,
                method_name,
            })),
        })
    }
}

#[derive(Clone)]
pub struct OSafePlaceStateCapture {
    pub stack_typing: ConcreteTyping,
    pub continuation_typing: ConcreteTyping,
    pub place: Place,
    pub ownership_qualifier: OwnershipQualifier,
    pub reborrow_exclusion_list: VecSet<Place>,
}

type OC0ParamFn = dyn FnOnce() -> StepResult;
type OCDerefExclParamFn = dyn FnOnce(VecSet<Place>) -> StepResult;

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

    // TODO: See if this can be eliminated easily or if it's simpler to just leave it. Probably don't want to make a whole new trait
    // just for this type.
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
