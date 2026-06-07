use std::fmt::Display;
use super::data_structures::*;
use super::auxiliary::*;

// FIXME: figure out where to put err_if_dead calls
pub(crate) enum TCReturn {
    TCResult(TCResult),
    TCState(Box<dyn TCState>),
}
pub(crate) type TCReturnResult = Result<(Option<Box<dyn TCState>>, TCReturn), OxidizedError>;
pub(crate) type TCResult = Result<(ConcreteTyping, Type), OxidizedError>;
// FIXME: For now i'll ignore the continuation typing
type TCFn = dyn FnOnce() -> TCReturnResult;
type TCConsumerFn = dyn FnOnce(ConcreteTyping) -> TCReturnResult;
type TCPopVarsFn = dyn FnOnce(TCResult) -> TCReturnResult;
type TC2ConsumerFn = dyn FnOnce(ConcreteTyping, Type) -> TCReturnResult;
type TC3ConsumerFn = dyn FnOnce(ConcreteTyping, Type, Type) -> TCReturnResult;

pub(crate) struct TCStateMachine {
    pub(crate) states: Vec<Box<dyn TCState>>,
}

pub(crate) trait TCState: Display {
    fn step_into(self: Box<Self>) -> TCReturnResult;

    fn step_over(self: Box<Self>) -> TCReturnResult;

    fn evaluate(self: Box<Self>) -> TCResult;

    fn handle_result(
        self: Box<Self>,
        result: TCResult,
    ) -> Result<(Box<dyn TCState>, Option<TCReturn>), OxidizedError>;

    fn is_complete(&self) -> bool;

    // If a state contains a function intended to be called at the end of its evaluation
    // such as gc_fn for TLetState, the state must remain on the stack until it is fully evaluated.
    fn should_remain_on_stack(&self) -> bool {
        false
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping);
}

pub(crate) struct TExprState {
    e_fn: Option<Box<TCFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    is_complete: bool,
}

impl Display for TExprState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TExprState {{ state: ({}, {}) }}",
            self.get_env_state().0,
            self.get_env_state().1
        )
    }
}

pub(crate) struct TLetrgnState {
    e_fn: Option<Box<TCFn>>,
    pop_vars_fn: Option<Box<TCPopVarsFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    e_result: Option<TCResult>,
    is_complete: bool,
}

impl Display for TLetrgnState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TLetrgnState {{ state: ({}, {}), e_result: {} }}",
            self.get_env_state().0,
            self.get_env_state().1,
            if let Some(_) = self.e_result {
                "Some(_)"
            } else {
                "None"
            }
        )
    }
}

pub(crate) struct TSeqState {
    e1_fn: Option<Box<TCFn>>,
    e2_fn: Option<Box<TCConsumerFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    e1_result: Option<TCResult>,
    is_complete: bool,
}

impl Display for TSeqState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TSeqState {{ state: ({}, {}), e1_result: {} }}",
            self.get_env_state().0,
            self.get_env_state().1,
            if let Some(_) = self.e1_result {
                "Some(_)"
            } else {
                "None"
            }
        )
    }
}

pub(crate) struct TLetState {
    e1_fn: Option<Box<TCFn>>,
    binding_fn: Option<Box<TC2ConsumerFn>>,
    e2_fn: Option<Box<TCConsumerFn>>,
    gc_fn: Option<Box<TC3ConsumerFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    bound_type: Option<Type>,
    e1_result: Option<TCResult>,
    e2_result: Option<TCResult>,
    binding_result: Option<TCResult>,
    is_complete: bool,
}

impl Display for TLetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TLetState {{ state: ({}, {}), e1_result: {}, e2_result: {}, bound_type: {}, binding_result: {} }}",
            self.get_env_state().0,
            self.get_env_state().1,
            if let Some(_) = self.e1_result {
                "Some(_)"
            } else {
                "None"
            },
            if let Some(_) = self.e2_result {
                "Some(_)"
            } else {
                "None"
            },
            if let Some(_) = self.bound_type {
                "Some(_)"
            } else {
                "None"
            },
            if let Some(_) = self.binding_result {
                "Some(_)"
            } else {
                "None"
            }
        )
    }
}

// Maybe is complete should be something like "can call exit function"

impl TCState for TExprState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        let mut e_return = (self.e_fn.take().unwrap())()?;
        if let (_, TCReturn::TCResult(_)) = e_return {
            self.is_complete = true;
        }
        e_return.0 = Some(self);
        Ok(e_return)
    }

    fn step_over(self: Box<Self>) -> TCReturnResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        let mut state_machine = TCStateMachine::new(self);
        state_machine.evaluate()
    }

    fn handle_result(
        self: Box<Self>,
        _: TCResult,
    ) -> Result<(Box<dyn TCState>, Option<TCReturn>), OxidizedError> {
        Ok((self, None))
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }
}

impl TExprState {
    pub(crate) fn new_obj<TCFn: FnOnce() -> TCReturnResult + 'static>(
        e_fn: TCFn,
        env_state: (ConcreteTyping, ConcreteTyping),
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e_fn: Some(Box::new(e_fn)),
                env_state,
                is_complete: false,
            })),
        ))
    }
}

impl TCState for TLetrgnState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        let mut e_return = (self.e_fn.take().unwrap())()?;
        if let (_, TCReturn::TCResult(_)) = e_return {
            self.is_complete = true;
        }
        e_return.0 = Some(self);
        Ok(e_return)
    }

    fn step_over(self: Box<Self>) -> TCReturnResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        let mut state_machine = TCStateMachine::new(self);
        state_machine.evaluate()
    }

    fn handle_result(
        mut self: Box<Self>,
        result: TCResult,
    ) -> Result<(Box<dyn TCState>, Option<TCReturn>), OxidizedError> {
        if let Some(result) = self.e_result.clone() {
            let pop_var_val = (self.pop_vars_fn.take().unwrap())(result)?;
            return Ok((self, Some(pop_var_val.1)));
        } else {
            self.e_result = Some(result.clone())
        }
        Ok((self, Some(TCReturn::TCResult(result))))
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn should_remain_on_stack(&self) -> bool {
        true
    }
}

impl TLetrgnState {
    pub(crate) fn new_obj<
        TCFn: FnOnce() -> TCReturnResult + 'static,
        TCPopVarsFn: FnOnce(TCResult) -> TCReturnResult + 'static,
    >(
        e_fn: TCFn,
        pop_vars_fn: TCPopVarsFn,
        env_state: (ConcreteTyping, ConcreteTyping),
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e_fn: Some(Box::new(e_fn)),
                pop_vars_fn: Some(Box::new(pop_vars_fn)),
                env_state,
                e_result: None,
                is_complete: false,
            })),
        ))
    }
}

impl TCState for TSeqState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        let mut e1_result = (self.e1_fn.take().unwrap())()?;
        e1_result.0 = Some(self);
        Ok(e1_result)
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        let mut state_machine = TCStateMachine::new(self);
        state_machine.evaluate()
    }

    fn handle_result(
        mut self: Box<Self>,
        result: TCResult,
    ) -> Result<(Box<dyn TCState>, Option<TCReturn>), OxidizedError> {
        self.e1_result = Some(result);
        Ok((self, None))
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn step_over(mut self: Box<Self>) -> TCReturnResult {
        if let Some(result) = self.e1_result.clone() {
            let mut e2_return = (self.e2_fn.take().unwrap())(result?.0)?;
            if let (_, TCReturn::TCResult(_)) = e2_return {
                self.is_complete = true;
            }
            e2_return.0 = Some(self);
            Ok(e2_return)
        } else {
            let e1_return = (self.e1_fn.take().unwrap())()?;
            match e1_return {
                (_, TCReturn::TCResult(result)) => {
                    let e1_value = result?;
                    let mut e2_return = (self.e2_fn.take().unwrap())(e1_value.0)?;
                    if let (_, TCReturn::TCResult(_)) = e2_return {
                        self.is_complete = true;
                    }
                    e2_return.0 = Some(self);
                    Ok(e2_return)
                }
                (_, TCReturn::TCState(state)) => {
                    let e1_value = state.evaluate()?;
                    let mut e2_return = (self.e2_fn.take().unwrap())(e1_value.0)?;
                    if let (_, TCReturn::TCResult(_)) = e2_return {
                        self.is_complete = true;
                    }
                    e2_return.0 = Some(self);
                    Ok(e2_return)
                }
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }
}

impl TSeqState {
    pub(crate) fn new_obj<
        TCFn: FnOnce() -> TCReturnResult + 'static,
        TCConsumerFn: FnOnce(ConcreteTyping) -> TCReturnResult + 'static,
    >(
        e1_fn: TCFn,
        e2_fn: TCConsumerFn,
        env_state: (ConcreteTyping, ConcreteTyping),
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e1_fn: Some(Box::new(e1_fn)),
                e2_fn: Some(Box::new(e2_fn)),
                env_state,
                is_complete: false,
                e1_result: None,
            })),
        ))
    }
}

impl TCState for TLetState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        let mut e1_result = (self.e1_fn.take().unwrap())()?;
        if let (_, TCReturn::TCResult(_)) = e1_result {
            self.is_complete = true;
        }
        e1_result.0 = Some(self);
        Ok(e1_result)
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        let mut state_machine = TCStateMachine::new(self);
        state_machine.evaluate()
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn step_over(mut self: Box<Self>) -> TCReturnResult {
        if let Some(result) = self.binding_result.clone() {
            let mut e2_return = (self.e2_fn.take().unwrap())(result?.0)?;
            if let (_, TCReturn::TCResult(ref result)) = e2_return {
                self.e2_result = Some(result.clone());
            }
            e2_return.0 = Some(self);
            Ok(e2_return)
        } else {
            let e1_return = (self.e1_fn.take().unwrap())()?;
            match e1_return {
                (_, TCReturn::TCResult(result)) => {
                    self.e1_result = Some(result.clone());
                    let e1_value = err_if_dead(result)?;
                    let binding_return = (self.binding_fn.take().unwrap())(e1_value.0, e1_value.1)?;
                    match binding_return {
                        (_, TCReturn::TCResult(result)) => {
                            let binding_value = result?;
                            let mut e2_return = (self.e2_fn.take().unwrap())(binding_value.0)?;
                            if let (_, TCReturn::TCResult(ref result)) = e2_return {
                                self.e2_result = Some(result.clone());
                            }
                            e2_return.0 = Some(self);
                            Ok(e2_return)
                        }
                        (_, TCReturn::TCState(state)) => {
                            panic!("Unexpected match");
                            // let binding_value = state.evaluate()?;
                            // let mut e2_return = (self.e2_fn.take().unwrap())(binding_value.0)?;
                            // if let (_, TCReturn::TCResult(_)) = e2_return {
                            //     self.is_complete = true;
                            // }
                            // e2_return.0 = Some(self);
                            // Ok(e2_return)
                        }
                    }
                }
                (_, TCReturn::TCState(state)) => {
                    let e1_value = err_if_dead(state.evaluate())?;
                    self.e1_result = Some(Ok(e1_value.clone()));
                    let binding_return = (self.binding_fn.take().unwrap())(e1_value.0, e1_value.1)?;
                    match binding_return {
                        (_, TCReturn::TCResult(result)) => {
                            self.binding_result = Some(result.clone());
                            let binding_value = result?;
                            let mut e2_return = (self.e2_fn.take().unwrap())(binding_value.0)?;
                            if let (_, TCReturn::TCResult(ref result)) = e2_return {
                                self.e2_result = Some(result.clone());
                            }
                            e2_return.0 = Some(self);
                            Ok(e2_return)
                        }
                        (_, TCReturn::TCState(state)) => {
                            panic!("Unexpected match");
                            // let binding_value = state.evaluate()?;
                            // let mut e2_return = (self.e2_fn.take().unwrap())(binding_value.0)?;
                            // if let (_, TCReturn::TCResult(_)) = e2_return {
                            //     self.is_complete = true;
                            // }
                            // e2_return.0 = Some(self);
                            // Ok(e2_return)
                        }
                    }
                }
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn should_remain_on_stack(&self) -> bool {
        true
    }

    fn handle_result(
        mut self: Box<Self>,
        result: TCResult,
    ) -> Result<(Box<dyn TCState>, Option<TCReturn>), OxidizedError> {
        // if let Some(_) = self.e2_result { probably not needed?
        //     Ok((self, None))
        // } else
        if let Some(binding_result) = self.binding_result.clone() {
            self.e2_result = Some(result.clone());
            let result_val = err_if_dead(result)?;
            let binding_result_val = binding_result?;

            let gc_return_val =
                (self.gc_fn.take().unwrap())(result_val.0, result_val.1, binding_result_val.1)?;
            Ok((self, Some(gc_return_val.1)))
        } else if let Some(_) = self.e1_result {
            self.binding_result = Some(result.clone());
            Ok((self, None))
        } else {
            // FIXME: matters for step_into
            self.e1_result = Some(result.clone());
            let result_val = err_if_dead(result)?;

            let binding_return_val = (self.binding_fn.take().unwrap())(result_val.0, result_val.1)?;
            Ok((self, Some(binding_return_val.1)))
        }
    }
}

impl TLetState {
    pub(crate) fn new_obj<
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
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e1_fn: Some(Box::new(e1_fn)),
                binding_fn: Some(Box::new(binding_fn)),
                e2_fn: Some(Box::new(e2_fn)),
                gc_fn: Some(Box::new(gc_fn)),
                env_state,
                is_complete: false,
                e1_result: None,
                binding_result: None,
                bound_type: None,
                e2_result: None,
            })),
        ))
    }
}

impl TCStateMachine {
    pub(crate) fn new(state: Box<dyn TCState>) -> Self {
        Self {
            states: vec![state],
        }
    }

    pub(crate) fn evaluate(&mut self) -> TCResult {
        while self.states.len() > 0 {
            match self.step_over() {
                Ok(Some(result)) => return result,
                Ok(None) => continue,
                Err(err) => return Err(err),
            }
        }
        panic!("failed to evaluate");
    }

    pub(crate) fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        if let Some(state) = self.states.last() {
            state.get_env_state()
        } else {
            (ConcreteTyping::new(), ConcreteTyping::new())
        }
    }

    pub(crate) fn step_into(&mut self) -> Result<(), OxidizedError> {
        let top = self.states.pop().unwrap();
        let return_result = top.step_into()?;
        match return_result {
            (_, TCReturn::TCResult(result)) => {
                let next_top = self.states.pop().unwrap();
                let is_complete = next_top.is_complete();
                let handle_result_state = next_top.handle_result(result)?;
                if !is_complete {
                    self.states.push(handle_result_state.0);
                }
                match handle_result_state.1 {
                    Some(TCReturn::TCResult(result)) => {
                        // I think this only needs to be done once and not recursively since you can't have
                        // multiple gc fn or pop fn returns cascade into each other.  IDK tho....
                        let next_top = self.states.pop().unwrap();
                        let is_complete = next_top.is_complete();
                        let handle_result_state = next_top.handle_result(result)?;
                        if !is_complete {
                            self.states.push(handle_result_state.0);
                        }
                        if let (_, Some(TCReturn::TCState(state))) = handle_result_state {
                            self.states.push(state);
                        }
                    }
                    Some(TCReturn::TCState(state)) => {
                        self.states.push(state);
                    }
                    _ => (),
                }
                Ok(())
            }
            (top, TCReturn::TCState(state)) => {
                self.states.push(top.unwrap());
                self.states.push(state);
                Ok(())
            }
        }
    }

    pub(crate) fn step_over(&mut self) -> Result<Option<TCResult>, OxidizedError> {
        let top = self.states.pop().unwrap();
        let return_result = top.step_over()?;
        match return_result {
            (_, TCReturn::TCResult(result)) => {
                if let Some(next_top) = self.states.pop() {
                    let is_complete = next_top.is_complete();
                    let handle_result_state = next_top.handle_result(result)?;
                    // if !is_complete {
                    //     self.states.push(handle_result_state.0);
                    // }
                    match handle_result_state.1 {
                        Some(TCReturn::TCResult(result)) => {
                            // I think this only needs to be done once and not recursively since you can't have
                            // multiple gc fn or pop fn returns cascade into each other.  IDK tho....
                            let next_top = self.states.pop().unwrap();
                            let is_complete = next_top.is_complete();
                            let handle_result_state = next_top.handle_result(result.clone())?;
                            // if !is_complete {
                            //     self.states.push(handle_result_state.0);
                            // }
                            if let (_, Some(TCReturn::TCState(state))) = handle_result_state {
                                // println!("1st: {}", state);
                                self.states.push(state);
                            }
                            Ok(Some(result))
                        }
                        Some(TCReturn::TCState(state)) => {
                            // println!("2nd: {}", state);
                            self.states.push(state);
                            Ok(None)
                        }
                        _ => Ok(None),
                    }
                } else {
                    Ok(Some(result))
                }
            }
            (last_top, TCReturn::TCState(state)) => {
                if let Some(state) = last_top {
                    if state.should_remain_on_stack() {
                        // println!("3rd: {}", state);
                        self.states.push(state);
                    }
                }
                // println!("4th: {}", state);
                self.states.push(state);
                Ok(None)
            }
        }
    }
}
