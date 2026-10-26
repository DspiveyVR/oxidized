use crate::debugging::debugger::*;
use crate::static_analysis::data_structures::*;
use color_print::cprintln;
use std::println;
use std::vec;

pub type TCFnResult = Result<(ConcreteTyping, Type), OxidizedError>;
// FIXME: For now i'll ignore the continuation typing
type TC0ParamFn = dyn FnOnce() -> StepResult;
type TC1ParamFn = dyn FnOnce(ConcreteTyping) -> StepResult;
type TCPopVarsFn = dyn FnOnce(TCFnResult) -> StepResult;
type TC2ParamFn = dyn FnOnce(ConcreteTyping, Type) -> StepResult;
type TC3ParamFn = dyn FnOnce(ConcreteTyping, Type, Type) -> StepResult;

// impl FnResult {
//     fn get_tc(self) -> TCFnResult {
//         match self {
//             FnResult::TCResult(res) => res,
//             FnResult::OCResult(_) => panic!("Not TC"),
//         }
//     }
// }

#[derive(Clone)]
pub struct TCEnvStateCapture {
    pub(crate) stack_typing: ConcreteTyping,
    pub(crate) continuation_typing: ConcreteTyping,
}

pub struct TExprState {
    e_fn: Option<Box<TC0ParamFn>>,
    env_state: TCEnvStateCapture,
    method_name: String,
}

pub struct TLetrgnState {
    e_fn: Option<Box<TC0ParamFn>>,
    pop_vars_fn: Option<Box<TCPopVarsFn>>,
    env_state: TCEnvStateCapture,
    pub(crate) e_result: Option<TCFnResult>,
    method_name: String,
}

pub struct TSeqState {
    e1_fn: Option<Box<TC0ParamFn>>,
    e2_fn: Option<Box<TC1ParamFn>>,
    env_state: TCEnvStateCapture,
    pub(crate) e1_result: Option<TCFnResult>,
    method_name: String,
}

pub struct TLetState {
    e1_fn: Option<Box<TC0ParamFn>>,
    binding_fn: Option<Box<TC2ParamFn>>,
    e2_fn: Option<Box<TC1ParamFn>>,
    gc_fn: Option<Box<TC3ParamFn>>,
    env_state: TCEnvStateCapture,
    pub(crate) e1_result: Option<TCFnResult>,
    pub(crate) binding_result: Option<TCFnResult>,
    method_name: String,
}

pub struct TAssignState {
    e_fn: Option<Box<TC0ParamFn>>,
    binding_fn: Option<Box<TC2ParamFn>>,
    env_state: TCEnvStateCapture,
    method_name: String,
}

impl DebuggerState for TExprState {
    fn step_into(mut self: Box<Self>) -> StepResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e_fn()</yellow>");
        }
        let mut e_return = (self.e_fn.take().unwrap())()?;
        e_return.yielded_state = Some(self);
        Ok(e_return)
    }

    fn step_over(self: Box<Self>) -> StepResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> FnEnvelope {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn handle_result(self: Box<Self>, result: FnEnvelope) -> Result<FnReturn, OxidizedError> {
        match result {
            FnEnvelope::TCResult(result) => {
                if DEBUG_OUTPUT {
                    cprintln!(
                        "<red>[[handle_result]]: \n<white>({}, {})</white>\n{}</red>",
                        result.clone()?.0,
                        result.clone()?.1,
                        self
                    );
                }

                Ok(FnReturn::FnResult(FnEnvelope::TCResult(result)))
            }
            FnEnvelope::OCResult(ocfn_return) => todo!(),
        }
    }

    fn get_state_capture(&self) -> StateCapture {
        StateCapture::TCEnvStateCapture(self.env_state.clone())
    }

    fn get_state_name(&self) -> String {
        "Expr".to_string()
    }

    fn get_method_name(&self) -> String {
        self.method_name.clone()
    }
}

impl TExprState {
    pub fn new_obj<TC0ParamFn: FnOnce() -> StepResult + 'static>(
        e_fn: TC0ParamFn,
        env_state: TCEnvStateCapture,
        method_name: String,
    ) -> StepResult {
        Ok(StepEnvelope {
            yielded_state: None,
            result: FnReturn::DebuggerState(Box::new(Self {
                e_fn: Some(Box::new(e_fn)),
                env_state,
                method_name,
            })),
        })
    }
}

impl DebuggerState for TLetrgnState {
    fn step_into(mut self: Box<Self>) -> StepResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e_fn()</yellow>");
        }
        let mut e_return = (self.e_fn.take().unwrap())()?;
        e_return.yielded_state = Some(self);
        Ok(e_return)
    }

    fn step_over(self: Box<Self>) -> StepResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> FnEnvelope {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn handle_result(mut self: Box<Self>, result: FnEnvelope) -> Result<FnReturn, OxidizedError> {
        match result {
            FnEnvelope::TCResult(result) => {
                if DEBUG_OUTPUT {
                    cprintln!(
                        "<red>[[handle_result]]: \n<white>({}, {})</white>\n{}</red>",
                        result.clone()?.0,
                        result.clone()?.1,
                        self
                    );
                    cprintln!(
                        "<yellow>[called]: pop_vars_fn(({}, {}))</yellow>",
                        result.clone()?.0,
                        result.clone()?.1
                    );
                }
                let pop_var_val = (self.pop_vars_fn.take().unwrap())(result)?;
                Ok(pop_var_val.result)
            }
            FnEnvelope::OCResult(ocfn_return) => todo!(),
        }
    }

    fn get_state_capture(&self) -> StateCapture {
        StateCapture::TCEnvStateCapture(self.env_state.clone())
    }

    fn get_state_name(&self) -> String {
        "Letrgn".to_string()
    }

    fn get_method_name(&self) -> String {
        self.method_name.clone()
    }
}

impl TLetrgnState {
    pub fn new_obj<
        TC0ParamFn: FnOnce() -> StepResult + 'static,
        TCPopVarsFn: FnOnce(TCFnResult) -> StepResult + 'static,
    >(
        e_fn: TC0ParamFn,
        pop_vars_fn: TCPopVarsFn,
        env_state: TCEnvStateCapture,
        method_name: String,
    ) -> StepResult {
        Ok(StepEnvelope {
            yielded_state: None,
            result: FnReturn::DebuggerState(Box::new(Self {
                e_fn: Some(Box::new(e_fn)),
                pop_vars_fn: Some(Box::new(pop_vars_fn)),
                env_state,
                e_result: None,
                method_name,
            })),
        })
    }
}

impl DebuggerState for TSeqState {
    fn step_into(mut self: Box<Self>) -> StepResult {
        if let Some(_) = self.e1_result {
            return self.step_over();
        }
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e1_fn()</yellow>");
        }
        let mut e1_result = (self.e1_fn.take().unwrap())()?;
        e1_result.yielded_state = Some(self);
        Ok(e1_result)
    }

    fn evaluate(self: Box<Self>) -> FnEnvelope {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn handle_result(mut self: Box<Self>, result: FnEnvelope) -> Result<FnReturn, OxidizedError> {
        match result {
            FnEnvelope::TCResult(result) => {
                if DEBUG_OUTPUT {
                    cprintln!(
                        "<red>[[handle_result]]: \n<white>({}, {})</white>\n{}</red>",
                        result.clone()?.0,
                        result.clone()?.1,
                        self
                    );
                }

                if let None = self.e1_result {
                    self.e1_result = Some(result.clone());
                    Ok(FnReturn::DebuggerState(self))
                } else {
                    Ok(FnReturn::FnResult(FnEnvelope::TCResult(result)))
                }
            }
            FnEnvelope::OCResult(ocfn_return) => todo!(),
        }
    }

    fn get_state_capture(&self) -> StateCapture {
        StateCapture::TCEnvStateCapture(self.env_state.clone())
    }

    fn step_over(mut self: Box<Self>) -> StepResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_over]]\n{}</red>", self);
        }
        if let Some(result) = self.e1_result.clone() {
            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: e2_fn({})</yellow>", result.clone()?.0,);
            }
            let mut e2_return = (self.e2_fn.take().unwrap())(result?.0)?;
            e2_return.yielded_state = Some(self);
            Ok(e2_return)
        } else {
            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: e1_fn()</yellow>");
            }
            let e1_return = (self.e1_fn.take().unwrap())()?;
            match e1_return {
                StepEnvelope {
                    result: FnReturn::FnResult(result),
                    ..
                } => {
                    let e1_result = result;
                    match e1_result {
                        FnEnvelope::TCResult(e1_result) => {
                            self.e1_result = Some(e1_result.clone());
                            let e1_value = e1_result?;
                            if DEBUG_OUTPUT {
                                cprintln!(
                                    "<yellow>[called]: e2_fn({})</yellow>",
                                    e1_value.clone().0,
                                );
                            }
                            let mut e2_return = (self.e2_fn.take().unwrap())(e1_value.0)?;
                            e2_return.yielded_state = Some(self);
                            Ok(e2_return)
                        }
                        FnEnvelope::OCResult(ocfn_return) => todo!(),
                    }
                }
                StepEnvelope {
                    result: FnReturn::DebuggerState(state),
                    ..
                } => {
                    let e1_result = state.evaluate();
                    match e1_result {
                        FnEnvelope::TCResult(e1_result) => {
                            self.e1_result = Some(e1_result.clone());
                            let e1_value = e1_result?;

                            if DEBUG_OUTPUT {
                                cprintln!(
                                    "<yellow>[called]: e2_fn({})</yellow>",
                                    e1_value.clone().0,
                                );
                            }
                            let mut e2_return = (self.e2_fn.take().unwrap())(e1_value.0)?;
                            e2_return.yielded_state = Some(self);
                            Ok(e2_return)
                        }
                        FnEnvelope::OCResult(ocfn_return) => todo!(),
                    }
                }
            }
        }
    }

    fn get_state_name(&self) -> String {
        "Seq".to_string()
    }

    fn get_method_name(&self) -> String {
        self.method_name.clone()
    }
}

impl TSeqState {
    pub fn new_obj<
        TC0ParamFn: FnOnce() -> StepResult + 'static,
        TC1ParamFn: FnOnce(ConcreteTyping) -> StepResult + 'static,
    >(
        e1_fn: TC0ParamFn,
        e2_fn: TC1ParamFn,
        env_state: TCEnvStateCapture,
        method_name: String,
    ) -> StepResult {
        Ok(StepEnvelope {
            yielded_state: None,
            result: FnReturn::DebuggerState(Box::new(Self {
                e1_fn: Some(Box::new(e1_fn)),
                e2_fn: Some(Box::new(e2_fn)),
                env_state,
                e1_result: None,
                method_name,
            })),
        })
    }
}

impl DebuggerState for TAssignState {
    fn step_into(mut self: Box<Self>) -> StepResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e_fn()</yellow>");
        }
        let mut e_result = (self.e_fn.take().unwrap())()?;
        e_result.yielded_state = Some(self);
        Ok(e_result)
    }

    fn step_over(self: Box<Self>) -> StepResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> FnEnvelope {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn get_state_capture(&self) -> StateCapture {
        StateCapture::TCEnvStateCapture(self.env_state.clone())
    }

    fn handle_result(mut self: Box<Self>, result: FnEnvelope) -> Result<FnReturn, OxidizedError> {
        match result {
            FnEnvelope::TCResult(result) => {
                if DEBUG_OUTPUT {
                    cprintln!(
                        "<red>[[handle_result]]: \n<white>({}, {})</white>\n{}</red>",
                        result.clone()?.0,
                        result.clone()?.1,
                        self
                    );
                }

                let result_val = result?;

                if DEBUG_OUTPUT {
                    cprintln!(
                        "<yellow>[called]: binding_fn({}, {})</yellow>",
                        result_val.clone().0,
                        result_val.clone().1
                    );
                }
                let binding_return_val =
                    (self.binding_fn.take().unwrap())(result_val.0, result_val.1)?;
                if let FnReturn::FnResult(result) = &binding_return_val.result {
                    Ok(FnReturn::FnResult(result.clone()))
                } else {
                    panic!("Unexpected match");
                }
            }
            FnEnvelope::OCResult(ocfn_return) => todo!(),
        }
    }

    fn get_state_name(&self) -> String {
        "Assign".to_string()
    }

    fn get_method_name(&self) -> String {
        self.method_name.clone()
    }
}

impl TAssignState {
    pub fn new_obj<
        TC0ParamFn: FnOnce() -> StepResult + 'static,
        TC2ParamFn: FnOnce(ConcreteTyping, Type) -> StepResult + 'static,
    >(
        e_fn: TC0ParamFn,
        binding_fn: TC2ParamFn,
        env_state: TCEnvStateCapture,
        method_name: String,
    ) -> StepResult {
        Ok(StepEnvelope {
            yielded_state: None,
            result: FnReturn::DebuggerState(Box::new(Self {
                e_fn: Some(Box::new(e_fn)),
                binding_fn: Some(Box::new(binding_fn)),
                env_state,
                method_name,
            })),
        })
    }
}

impl DebuggerState for TLetState {
    fn step_into(mut self: Box<Self>) -> StepResult {
        if let Some(_) = self.e1_result {
            return self.step_over();
        }
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e1_fn()</yellow>");
        }
        let mut e1_result = (self.e1_fn.take().unwrap())()?;
        e1_result.yielded_state = Some(self);
        Ok(e1_result)
    }

    fn evaluate(self: Box<Self>) -> FnEnvelope {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn get_state_capture(&self) -> StateCapture {
        StateCapture::TCEnvStateCapture(self.env_state.clone())
    }

    fn step_over(mut self: Box<Self>) -> StepResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_over]]\n{}</red>", self);
        }
        if let Some(result) = self.binding_result.clone() {
            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: e2_fn({})</yellow>", result.clone()?.0,);
            }
            let mut e2_return = (self.e2_fn.take().unwrap())(result?.0)?;
            e2_return.yielded_state = Some(self);
            Ok(e2_return)
        } else {
            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: e1_fn()</yellow>");
            }
            let e1_return = (self.e1_fn.take().unwrap())()?;
            match e1_return {
                StepEnvelope {
                    result: FnReturn::FnResult(result),
                    ..
                } => match result {
                    FnEnvelope::TCResult(result) => {
                        self.e1_result = Some(result.clone());
                        let e1_value = result?;
                        if DEBUG_OUTPUT {
                            cprintln!(
                                "<yellow>[called]: binding_fn({}, {})</yellow>",
                                e1_value.clone().0,
                                e1_value.clone().1
                            );
                        }
                        let binding_return =
                            (self.binding_fn.take().unwrap())(e1_value.0, e1_value.1)?;
                        match binding_return {
                            StepEnvelope {
                                result: FnReturn::FnResult(result),
                                ..
                            } => match result {
                                FnEnvelope::TCResult(result) => {
                                    let binding_value = result?;
                                    if DEBUG_OUTPUT {
                                        cprintln!(
                                            "<yellow>[called]: e2_fn({})</yellow>",
                                            binding_value.clone().0,
                                        );
                                    }
                                    let mut e2_return =
                                        (self.e2_fn.take().unwrap())(binding_value.0)?;
                                    e2_return.yielded_state = Some(self);
                                    Ok(e2_return)
                                }
                                FnEnvelope::OCResult(ocfn_return) => todo!(),
                            },
                            StepEnvelope {
                                result: FnReturn::DebuggerState(_),
                                ..
                            } => {
                                panic!("Unexpected match");
                            }
                        }
                    }
                    FnEnvelope::OCResult(ocfn_return) => todo!(),
                },
                StepEnvelope {
                    result: FnReturn::DebuggerState(state),
                    ..
                } => {
                    let e1_result = state.evaluate();
                    match e1_result {
                        FnEnvelope::TCResult(e1_result) => {
                            self.e1_result = Some(e1_result.clone());
                            let e1_value = e1_result?;
                            if DEBUG_OUTPUT {
                                cprintln!(
                                    "<yellow>[called]: binding_fn({}, {})</yellow>",
                                    e1_value.clone().0,
                                    e1_value.clone().1
                                );
                            }
                            let binding_return =
                                (self.binding_fn.take().unwrap())(e1_value.0, e1_value.1)?;
                            match binding_return {
                                StepEnvelope {
                                    result: FnReturn::FnResult(result),
                                    ..
                                } => match result {
                                    FnEnvelope::TCResult(result) => {
                                        self.binding_result = Some(result.clone());
                                        let binding_value = result?;
                                        if DEBUG_OUTPUT {
                                            cprintln!(
                                                "<yellow>[called]: e2_fn({})</yellow>",
                                                binding_value.clone().0,
                                            );
                                        }
                                        let mut e2_return =
                                            (self.e2_fn.take().unwrap())(binding_value.0)?;
                                        e2_return.yielded_state = Some(self);
                                        Ok(e2_return)
                                    }
                                    FnEnvelope::OCResult(ocfn_return) => todo!(),
                                },
                                StepEnvelope {
                                    result: FnReturn::DebuggerState(_),
                                    ..
                                } => {
                                    panic!("Unexpected match");
                                }
                            }
                        }
                        FnEnvelope::OCResult(ocfn_return) => todo!(),
                    }
                }
            }
        }
    }

    fn handle_result(mut self: Box<Self>, result: FnEnvelope) -> Result<FnReturn, OxidizedError> {
        match result {
            FnEnvelope::TCResult(result) => {
                if DEBUG_OUTPUT {
                    cprintln!(
                        "<red>[[handle_result]]: \n<white>({}, {})</white>\n{}</red>",
                        result.clone()?.0,
                        result.clone()?.1,
                        self
                    );
                }
                if let Some(binding_result) = self.binding_result.clone() {
                    let result_val = result?;
                    let binding_result_val = binding_result?;

                    if DEBUG_OUTPUT {
                        cprintln!(
                            "<yellow>[called]: gc_fn({}, {}, {})</yellow>",
                            result_val.clone().0,
                            result_val.clone().1,
                            binding_result_val.clone().1
                        );
                    }
                    // It is possible that the type originally bound was reassigned before the gc_fn was called.
                    let current_bound_type = result_val.0.clone().lookup_type(Place {
                        variable: binding_result_val.1.variable.unwrap(),
                        path: vec![],
                    })?;
                    let gc_return_val = (self.gc_fn.take().unwrap())(
                        result_val.0,
                        result_val.1,
                        current_bound_type,
                    )?;
                    Ok(gc_return_val.result)
                } else {
                    self.e1_result = Some(result.clone());
                    let result_val = result?;

                    if DEBUG_OUTPUT {
                        cprintln!(
                            "<yellow>[called]: binding_fn({}, {})</yellow>",
                            result_val.clone().0,
                            result_val.clone().1
                        );
                    }
                    let binding_return_val =
                        (self.binding_fn.take().unwrap())(result_val.0, result_val.1)?;
                    if let FnReturn::FnResult(result) = &binding_return_val.result {
                        match result {
                            FnEnvelope::TCResult(result) => {
                                self.binding_result = Some(result.clone())
                            },
                            FnEnvelope::OCResult(ocfn_return) => todo!(),
                        }
                    } else {
                        panic!("Unexpected match");
                    }
                    Ok(FnReturn::DebuggerState(self))
                }
            }
            FnEnvelope::OCResult(ocfn_return) => todo!(),
        }
    }

    fn get_state_name(&self) -> String {
        "Let".to_string()
    }

    fn get_method_name(&self) -> String {
        self.method_name.clone()
    }
}

impl TLetState {
    pub fn new_obj<
        TC0ParamFn: FnOnce() -> StepResult + 'static,
        TC1ParamFn: FnOnce(ConcreteTyping) -> StepResult + 'static,
        TC2ParamFn: FnOnce(ConcreteTyping, Type) -> StepResult + 'static,
        TC3ParamFn: FnOnce(ConcreteTyping, Type, Type) -> StepResult + 'static,
    >(
        e1_fn: TC0ParamFn,
        binding_fn: TC2ParamFn,
        e2_fn: TC1ParamFn,
        gc_fn: TC3ParamFn,
        env_state: TCEnvStateCapture,
        method_name: String,
    ) -> StepResult {
        Ok(StepEnvelope {
            yielded_state: None,
            result: FnReturn::DebuggerState(Box::new(Self {
                e1_fn: Some(Box::new(e1_fn)),
                binding_fn: Some(Box::new(binding_fn)),
                e2_fn: Some(Box::new(e2_fn)),
                gc_fn: Some(Box::new(gc_fn)),
                env_state,
                e1_result: None,
                binding_result: None,
                method_name,
            })),
        })
    }
}
