use crate::debugging::debugger::*;
use crate::static_analysis::data_structures::*;
use color_print::cprintln;
use std::fmt::Display;
use std::println;
use std::vec;

pub enum TCReturn {
    TCResult(TCResult),
    TCState(Box<dyn TCState>),
}
pub type TCReturnResult = Result<(Option<Box<dyn TCState>>, TCReturn), OxidizedError>;
pub type TCResult = Result<(ConcreteTyping, Type), OxidizedError>;
// FIXME: For now i'll ignore the continuation typing
type TCFn = dyn FnOnce() -> TCReturnResult;
type TCConsumerFn = dyn FnOnce(ConcreteTyping) -> TCReturnResult;
type TCPopVarsFn = dyn FnOnce(TCResult) -> TCReturnResult;
type TC2ConsumerFn = dyn FnOnce(ConcreteTyping, Type) -> TCReturnResult;
type TC3ConsumerFn = dyn FnOnce(ConcreteTyping, Type, Type) -> TCReturnResult;

// FIXME: DebuggerState should be replaced with this trait and therefore moved into that file. 
// I think this can be used for both TC and OC. No enums needed. The return types can just be changed 
// so they're compatible with both TC and OC.
pub trait TCState: Display {
    fn step_into(self: Box<Self>) -> TCReturnResult;

    fn step_over(self: Box<Self>) -> TCReturnResult;

    fn evaluate(self: Box<Self>) -> TCResult;

    // If a step method on a state returns a result, the state machine will call this method on the next
    // state on the stack so it can assign its result properties and/or call other functions like gc_fn.
    // NOTE: Results passed in here seem to consistently have unit return types so i'm not sure if the return types could
    // be incorrect coming from some typing rules or if they aren't passed through states properly. May be a non issue.
    fn handle_result(self: Box<Self>, result: TCResult) -> Result<TCReturn, OxidizedError>;

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping);

    fn get_state_name(&self) -> String;

    fn get_method_name(&self) -> String;
}

pub struct TExprState {
    e_fn: Option<Box<TCFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    method_name: String,
}

pub struct TLetrgnState {
    e_fn: Option<Box<TCFn>>,
    pop_vars_fn: Option<Box<TCPopVarsFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    pub(crate) e_result: Option<TCResult>,
    method_name: String,
}

pub struct TSeqState {
    e1_fn: Option<Box<TCFn>>,
    e2_fn: Option<Box<TCConsumerFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    pub(crate) e1_result: Option<TCResult>,
    method_name: String,
}

pub struct TLetState {
    e1_fn: Option<Box<TCFn>>,
    binding_fn: Option<Box<TC2ConsumerFn>>,
    e2_fn: Option<Box<TCConsumerFn>>,
    gc_fn: Option<Box<TC3ConsumerFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    pub(crate) e1_result: Option<TCResult>,
    pub(crate) binding_result: Option<TCResult>,
    method_name: String,
}

pub struct TAssignState {
    e_fn: Option<Box<TCFn>>,
    binding_fn: Option<Box<TC2ConsumerFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    method_name: String,
}

impl TCState for TExprState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e_fn()</yellow>");
        }
        let mut e_return = (self.e_fn.take().unwrap())()?;
        e_return.0 = Some(self);
        Ok(e_return)
    }

    fn step_over(self: Box<Self>) -> TCReturnResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn handle_result(self: Box<Self>, result: TCResult) -> Result<TCReturn, OxidizedError> {
        if DEBUG_OUTPUT {
            cprintln!(
                "<red>[[handle_result]]: \n<white>({}, {})</white>\n{}</red>",
                result.clone()?.0,
                result.clone()?.1,
                self
            );
        }
        Ok(TCReturn::TCResult(result))
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn get_state_name(&self) -> String {
        "Expr".to_string()
    }

    fn get_method_name(&self) -> String {
        self.method_name.clone()
    }
}

impl TExprState {
    pub fn new_obj<TCFn: FnOnce() -> TCReturnResult + 'static>(
        e_fn: TCFn,
        env_state: (ConcreteTyping, ConcreteTyping),
        method_name: String,
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e_fn: Some(Box::new(e_fn)),
                env_state,
                method_name,
            })),
        ))
    }
}

impl TCState for TLetrgnState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e_fn()</yellow>");
        }
        let mut e_return = (self.e_fn.take().unwrap())()?;
        e_return.0 = Some(self);
        Ok(e_return)
    }

    fn step_over(self: Box<Self>) -> TCReturnResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn handle_result(mut self: Box<Self>, result: TCResult) -> Result<TCReturn, OxidizedError> {
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
        Ok(pop_var_val.1)
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
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
        TCFn: FnOnce() -> TCReturnResult + 'static,
        TCPopVarsFn: FnOnce(TCResult) -> TCReturnResult + 'static,
    >(
        e_fn: TCFn,
        pop_vars_fn: TCPopVarsFn,
        env_state: (ConcreteTyping, ConcreteTyping),
        method_name: String,
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e_fn: Some(Box::new(e_fn)),
                pop_vars_fn: Some(Box::new(pop_vars_fn)),
                env_state,
                e_result: None,
                method_name,
            })),
        ))
    }
}

impl TCState for TSeqState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        if let Some(_) = self.e1_result {
            return self.step_over();
        }
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e1_fn()</yellow>");
        }
        let mut e1_result = (self.e1_fn.take().unwrap())()?;
        e1_result.0 = Some(self);
        Ok(e1_result)
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn handle_result(mut self: Box<Self>, result: TCResult) -> Result<TCReturn, OxidizedError> {
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
            Ok(TCReturn::TCState(self))
        } else {
            Ok(TCReturn::TCResult(result))
        }
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn step_over(mut self: Box<Self>) -> TCReturnResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_over]]\n{}</red>", self);
        }
        if let Some(result) = self.e1_result.clone() {
            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: e2_fn({})</yellow>", result.clone()?.0,);
            }
            let mut e2_return = (self.e2_fn.take().unwrap())(result?.0)?;
            e2_return.0 = Some(self);
            Ok(e2_return)
        } else {
            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: e1_fn()</yellow>");
            }
            let e1_return = (self.e1_fn.take().unwrap())()?;
            match e1_return {
                (_, TCReturn::TCResult(result)) => {
                    let e1_result = result;
                    self.e1_result = Some(e1_result.clone());
                    let e1_value = e1_result?;
                    if DEBUG_OUTPUT {
                        cprintln!("<yellow>[called]: e2_fn({})</yellow>", e1_value.clone().0,);
                    }
                    let mut e2_return = (self.e2_fn.take().unwrap())(e1_value.0)?;
                    e2_return.0 = Some(self);
                    Ok(e2_return)
                }
                (_, TCReturn::TCState(state)) => {
                    let e1_result = state.evaluate();
                    self.e1_result = Some(e1_result.clone());
                    let e1_value = e1_result?;

                    if DEBUG_OUTPUT {
                        cprintln!("<yellow>[called]: e2_fn({})</yellow>", e1_value.clone().0,);
                    }
                    let mut e2_return = (self.e2_fn.take().unwrap())(e1_value.0)?;
                    e2_return.0 = Some(self);
                    Ok(e2_return)
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
        TCFn: FnOnce() -> TCReturnResult + 'static,
        TCConsumerFn: FnOnce(ConcreteTyping) -> TCReturnResult + 'static,
    >(
        e1_fn: TCFn,
        e2_fn: TCConsumerFn,
        env_state: (ConcreteTyping, ConcreteTyping),
        method_name: String,
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e1_fn: Some(Box::new(e1_fn)),
                e2_fn: Some(Box::new(e2_fn)),
                env_state,
                e1_result: None,
                method_name,
            })),
        ))
    }
}

impl TCState for TAssignState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e_fn()</yellow>");
        }
        let mut e_result = (self.e_fn.take().unwrap())()?;
        e_result.0 = Some(self);
        Ok(e_result)
    }

    fn step_over(self: Box<Self>) -> TCReturnResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn handle_result(mut self: Box<Self>, result: TCResult) -> Result<TCReturn, OxidizedError> {
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
        let binding_return_val = (self.binding_fn.take().unwrap())(result_val.0, result_val.1)?;
        if let TCReturn::TCResult(result) = &binding_return_val.1 {
            Ok(TCReturn::TCResult(result.clone()))
        } else {
            panic!("Unexpected match");
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
        TCFn: FnOnce() -> TCReturnResult + 'static,
        TC2ConsumerFn: FnOnce(ConcreteTyping, Type) -> TCReturnResult + 'static,
    >(
        e_fn: TCFn,
        binding_fn: TC2ConsumerFn,
        env_state: (ConcreteTyping, ConcreteTyping),
        method_name: String,
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e_fn: Some(Box::new(e_fn)),
                binding_fn: Some(Box::new(binding_fn)),
                env_state,
                method_name,
            })),
        ))
    }
}

impl TCState for TLetState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        if let Some(_) = self.e1_result {
            return self.step_over();
        }
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e1_fn()</yellow>");
        }
        let mut e1_result = (self.e1_fn.take().unwrap())()?;
        e1_result.0 = Some(self);
        Ok(e1_result)
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut debugger = Debugger::new(self);
        debugger._evaluate()
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn step_over(mut self: Box<Self>) -> TCReturnResult {
        if DEBUG_OUTPUT {
            cprintln!("<red>[[step_over]]\n{}</red>", self);
        }
        if let Some(result) = self.binding_result.clone() {
            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: e2_fn({})</yellow>", result.clone()?.0,);
            }
            let mut e2_return = (self.e2_fn.take().unwrap())(result?.0)?;
            e2_return.0 = Some(self);
            Ok(e2_return)
        } else {
            if DEBUG_OUTPUT {
                cprintln!("<yellow>[called]: e1_fn()</yellow>");
            }
            let e1_return = (self.e1_fn.take().unwrap())()?;
            match e1_return {
                (_, TCReturn::TCResult(result)) => {
                    self.e1_result = Some(result.clone());
                    let e1_value = result?;
                    if DEBUG_OUTPUT {
                        cprintln!(
                            "<yellow>[called]: binding_fn({}, {})</yellow>",
                            e1_value.clone().0,
                            e1_value.clone().1
                        );
                    }
                    let binding_return = (self.binding_fn.take().unwrap())(e1_value.0, e1_value.1)?;
                    match binding_return {
                        (_, TCReturn::TCResult(result)) => {
                            let binding_value = result?;
                            if DEBUG_OUTPUT {
                                cprintln!(
                                    "<yellow>[called]: e2_fn({})</yellow>",
                                    binding_value.clone().0,
                                );
                            }
                            let mut e2_return = (self.e2_fn.take().unwrap())(binding_value.0)?;
                            e2_return.0 = Some(self);
                            Ok(e2_return)
                        }
                        (_, TCReturn::TCState(_)) => {
                            panic!("Unexpected match");
                        }
                    }
                }
                (_, TCReturn::TCState(state)) => {
                    let e1_result = state.evaluate();
                    self.e1_result = Some(e1_result.clone());
                    let e1_value = e1_result?;
                    if DEBUG_OUTPUT {
                        cprintln!(
                            "<yellow>[called]: binding_fn({}, {})</yellow>",
                            e1_value.clone().0,
                            e1_value.clone().1
                        );
                    }
                    let binding_return = (self.binding_fn.take().unwrap())(e1_value.0, e1_value.1)?;
                    match binding_return {
                        (_, TCReturn::TCResult(result)) => {
                            self.binding_result = Some(result.clone());
                            let binding_value = result?;
                            if DEBUG_OUTPUT {
                                cprintln!(
                                    "<yellow>[called]: e2_fn({})</yellow>",
                                    binding_value.clone().0,
                                );
                            }
                            let mut e2_return = (self.e2_fn.take().unwrap())(binding_value.0)?;
                            e2_return.0 = Some(self);
                            Ok(e2_return)
                        }
                        (_, TCReturn::TCState(_)) => {
                            panic!("Unexpected match");
                        }
                    }
                }
            }
        }
    }

    fn handle_result(mut self: Box<Self>, result: TCResult) -> Result<TCReturn, OxidizedError> {
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
            let gc_return_val =
                (self.gc_fn.take().unwrap())(result_val.0, result_val.1, current_bound_type)?;
            Ok(gc_return_val.1)
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
            let binding_return_val = (self.binding_fn.take().unwrap())(result_val.0, result_val.1)?;
            if let TCReturn::TCResult(result) = &binding_return_val.1 {
                self.binding_result = Some(result.clone())
            } else {
                panic!("Unexpected match");
            }
            Ok(TCReturn::TCState(self))
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
        TCFn: FnOnce() -> TCReturnResult + 'static,
        TCConsumerFn: FnOnce(ConcreteTyping) -> TCReturnResult + 'static,
        TC2ConsumerFn: FnOnce(ConcreteTyping, Type) -> TCReturnResult + 'static,
        TC3ConsumerFn: FnOnce(ConcreteTyping, Type, Type) -> TCReturnResult + 'static,
    >(
        e1_fn: TCFn,
        binding_fn: TC2ConsumerFn,
        e2_fn: TCConsumerFn,
        gc_fn: TC3ConsumerFn,
        env_state: (ConcreteTyping, ConcreteTyping),
        method_name: String,
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e1_fn: Some(Box::new(e1_fn)),
                binding_fn: Some(Box::new(binding_fn)),
                e2_fn: Some(Box::new(e2_fn)),
                gc_fn: Some(Box::new(gc_fn)),
                env_state,
                e1_result: None,
                binding_result: None,
                method_name,
            })),
        ))
    }
}

