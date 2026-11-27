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
    recursive_check_fn: Option<Box<OCDerefRecursiveCheckFn>>,
    step2_fn: Option<Box<OCDerefStep2Fn>>,
    r_loans: Option<IntoIter<Loan>>,
    new_loans: Option<VecSet<Loan>>,
    new_excl_list: Option<VecSet<Place>>,
    state_capture: ODerefStateCapture,
    method_name: String,
}

impl DebuggerState for ODerefState {
    fn step_into(mut self: Box<Self>) -> StepResult {
        if let Some(_) = self.step1_fn {
            if DEBUG_OUTPUT {
                cprintln!("<red>[[step_into]]\n{}</red>", self);
                cprintln!("<yellow>[called]: step1_fn()</yellow>");
            }

            let mut s1_return = (self.step1_fn.take().unwrap())()?;
            s1_return.yielded_state = Some(self);
            Ok(s1_return)
        } else if let Some(_) = self.recursive_check_fn {
            let r_loans = self.r_loans.as_mut().unwrap();
            let new_loans = self.new_loans.as_ref().unwrap();
            let new_excl_list = self.new_excl_list.as_ref().unwrap();
            if let Some(loan) = r_loans.next() {
                if DEBUG_OUTPUT {
                    cprintln!("<red>[[step_into]]\n{}</red>", self);
                    cprintln!("<yellow>[called]: recursive_check_fn()</yellow>");
                }

                let mut check_return =
                    (self.recursive_check_fn.as_ref().unwrap())(loan, new_excl_list)?;
                check_return.yielded_state = Some(self);
                Ok(check_return)
            } else {
                if DEBUG_OUTPUT {
                    cprintln!("<red>[[step_into]]\n{}</red>", self);
                    cprintln!("<yellow>[done checking loans in r]</yellow>");
                }

                self.recursive_check_fn = None;
                let return_val =
                    EvalReturn::new_ocfn_result(new_loans.clone(), new_excl_list.clone());
                let check_return = StepEnvelope {
                    yielded_state: Some(self),
                    return_val,
                };
                Ok(check_return)
            }
        } else {
            self.step_over()
        }
    }

    fn step_over(mut self: Box<Self>) -> StepResult {
        let unwrap_fn_envelope =
            |env: FnEnvelope| -> Result<(VecSet<Loan>, VecSet<Place>), OxidizedError> {
                match env {
                    FnEnvelope::OCResult(fn_return) => {
                        let fn_return = fn_return?;
                        Ok((fn_return.loans, fn_return.reborrow_exclusion_list))
                    }
                    FnEnvelope::TCResult(_) => panic!("Unexpected match"),
                }
            };
        let extract_eval_return = |eval_return: EvalReturn| match eval_return {
            EvalReturn::FnResult(env) => unwrap_fn_envelope(env?),
            EvalReturn::DebuggerState(state) => {
                let env = state.evaluate()?;
                unwrap_fn_envelope(env)
            }
        };

        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_over]]\n{}</red>", self);
        }
        if let Some(_) = self.step1_fn {
            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: step1_fn()</yellow>");
            }
            let s1_return = (self.step1_fn.take().unwrap())()?;

            let (r_loans, new_excl_list) = extract_eval_return(s1_return.return_val)?;
            let mut new_loans = VecSet::new_empty();

            let check_fn = self.recursive_check_fn.as_ref().unwrap();
            for loan in r_loans {
                if DEBUG_OUTPUT {
                    cprintln!("<yellow>[called]: recursive_check_fn()</yellow>");
                }
                let check_return = check_fn(loan, &new_excl_list)?;
                new_loans = extract_eval_return(check_return.return_val)?.0;
            }
            self.recursive_check_fn = None;
            self.new_loans = Some(new_loans);

            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: step2_fn()</yellow>");
            }
            let mut s2_return = (self.step2_fn.take().unwrap())(new_excl_list)?;
            s2_return.yielded_state = Some(self);
            Ok(s2_return)
        } else if let Some(_) = self.recursive_check_fn {
            let r_loans = self.r_loans.clone().unwrap();
            let new_excl_list = self.new_excl_list.clone().unwrap();

            let mut new_loans = VecSet::new_empty();

            let check_fn = self.recursive_check_fn.as_ref().unwrap();
            for loan in r_loans {
                if DEBUG_OUTPUT {
                    cprintln!("<yellow>[called]: recursive_check_fn()</yellow>");
                }
                let check_return = check_fn(loan, &new_excl_list)?;
                new_loans = extract_eval_return(check_return.return_val)?.0;
            }
            self.recursive_check_fn = None;
            self.new_loans = Some(new_loans);

            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: step2_fn()</yellow>");
            }
            let mut s2_return = (self.step2_fn.take().unwrap())(new_excl_list)?;
            s2_return.yielded_state = Some(self);
            Ok(s2_return)
        } else {
            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: step2_fn()</yellow>");
            }

            let mut s2_return =
                (self.step2_fn.take().unwrap())(self.new_excl_list.clone().unwrap())?;
            s2_return.yielded_state = Some(self);
            Ok(s2_return)
        }
    }

    fn evaluate(self: Box<Self>) -> GenericFnResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn handle_result(self: Box<Self>, result: FnEnvelope) -> Result<EvalReturn, OxidizedError> {
        todo!()
    }

    fn get_state_capture(&self) -> StateCapture {
        StateCapture::ODerefStateCapture(self.state_capture.clone())
    }

    fn get_state_name(&self) -> String {
        "ODerefState".to_string()
    }

    fn get_method_name(&self) -> String {
        self.method_name.clone()
    }
}

impl ODerefState {
    pub fn new_obj<
        OC0ParamFn: FnOnce() -> StepResult + 'static,
        OCDerefRecursiveCheckFn: Fn(Loan, &VecSet<Place>) -> StepResult + 'static,
        OCDerefStep2Fn: FnOnce(VecSet<Place>) -> StepResult + 'static,
    >(
        step1_fn: OC0ParamFn,
        recursive_check_fn: OCDerefRecursiveCheckFn,
        step2_fn: OCDerefStep2Fn,
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
                new_excl_list: None,
                new_loans: None,
                r_loans: None,
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
type OCDerefRecursiveCheckFn = dyn Fn(Loan, &VecSet<Place>) -> StepResult;
type OCDerefStep2Fn = dyn FnOnce(VecSet<Place>) -> StepResult;

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
