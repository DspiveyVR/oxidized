use super::auxiliary::*;
use super::data_structures::*;
use color_print::cprintln;
use std::fmt::Display;

const DEBUG_TCSTATE: bool = true;

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
    states: Vec<Box<dyn TCState>>,
}

pub(crate) trait TCState: Display {
    fn step_into(self: Box<Self>) -> TCReturnResult;

    fn step_over(self: Box<Self>) -> TCReturnResult;

    fn evaluate(self: Box<Self>) -> TCResult;

    // If a step method on a state returns a result, the state machine will call this method on the next
    // state on the stack so it can assign its result properties and/or call other functions like gc_fn.
    fn handle_result(self: Box<Self>, result: TCResult) -> Result<Option<TCReturn>, OxidizedError>;

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
}

impl Display for TLetrgnState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TLetrgnState {{ state: ({}, {}), e_result: {} }}",
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

pub(crate) struct TSeqState {
    e1_fn: Option<Box<TCFn>>,
    e2_fn: Option<Box<TCConsumerFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    e1_result: Option<TCResult>,
}

impl Display for TSeqState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TSeqState {{ state: ({}, {}), e1_result: {} }}",
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

pub(crate) struct TLetState {
    e1_fn: Option<Box<TCFn>>,
    binding_fn: Option<Box<TC2ConsumerFn>>,
    e2_fn: Option<Box<TCConsumerFn>>,
    gc_fn: Option<Box<TC3ConsumerFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    e1_result: Option<TCResult>,
    binding_result: Option<TCResult>,
}

impl Display for TLetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TLetState {{ state: ({}, {}), e1_result: {}, binding_result: {} }}",
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

impl TCState for TExprState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        if DEBUG_TCSTATE {
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
        if DEBUG_TCSTATE {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut state_machine = TCStateMachine::new(self);
        state_machine._evaluate()
    }

    fn handle_result(self: Box<Self>, result: TCResult) -> Result<Option<TCReturn>, OxidizedError> {
        result?;
        if DEBUG_TCSTATE {
            cprintln!("<blue>[[handle_result]]\n{}</blue>", self);
        }
        Ok(None)
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
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
            })),
        ))
    }
}

impl TCState for TLetrgnState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        if DEBUG_TCSTATE {
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
        if DEBUG_TCSTATE {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut state_machine = TCStateMachine::new(self);
        state_machine._evaluate()
    }

    fn handle_result(
        mut self: Box<Self>,
        result: TCResult,
    ) -> Result<Option<TCReturn>, OxidizedError> {
        if DEBUG_TCSTATE {
            cprintln!(
                "<red>[[handle_result]]: \n<white>{}</white>\n{}</red>",
                result.clone()?.0,
                self
            );
            cprintln!(
                "<yellow>[called]: pop_vars_fn(({}, {}))</yellow>",
                result.clone()?.0,
                result.clone()?.1
            );
        }
        let pop_var_val = (self.pop_vars_fn.take().unwrap())(result)?;
        Ok(Some(pop_var_val.1))
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
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
            })),
        ))
    }
}

impl TCState for TSeqState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        if let Some(_) = self.e1_result {
            return self.step_over();
        }
        if DEBUG_TCSTATE {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e1_fn()</yellow>");
        }
        let mut e1_result = (self.e1_fn.take().unwrap())()?;
        e1_result.0 = Some(self);
        Ok(e1_result)
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        if DEBUG_TCSTATE {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut state_machine = TCStateMachine::new(self);
        state_machine._evaluate()
    }

    fn handle_result(
        mut self: Box<Self>,
        result: TCResult,
    ) -> Result<Option<TCReturn>, OxidizedError> {
        if DEBUG_TCSTATE {
            cprintln!(
                "<red>[[handle_result]]: \n<white>{}</white>\n{}</red>",
                result.clone()?.0,
                self
            );
        }
        self.e1_result = Some(result);
        Ok(None)
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn step_over(mut self: Box<Self>) -> TCReturnResult {
        if DEBUG_TCSTATE {
            cprintln!("<red>[[step_over]]\n{}</red>", self);
        }
        if let Some(result) = self.e1_result.clone() {
            if DEBUG_TCSTATE {
                cprintln!("<yellow>[called]: e2_fn({})</yellow>", result.clone()?.0,);
            }
            let mut e2_return = (self.e2_fn.take().unwrap())(result?.0)?;
            e2_return.0 = Some(self);
            Ok(e2_return)
        } else {
            if DEBUG_TCSTATE {
                cprintln!("<yellow>[called]: e1_fn()</yellow>");
            }
            let e1_return = (self.e1_fn.take().unwrap())()?;
            match e1_return {
                (_, TCReturn::TCResult(result)) => {
                    let e1_value = err_if_dead(result)?;
                    if DEBUG_TCSTATE {
                        cprintln!("<yellow>[called]: e2_fn({})</yellow>", e1_value.clone().0,);
                    }
                    let mut e2_return = (self.e2_fn.take().unwrap())(e1_value.0)?;
                    e2_return.0 = Some(self);
                    Ok(e2_return)
                }
                (_, TCReturn::TCState(state)) => {
                    let e1_value = err_if_dead(state.evaluate())?;
                    if DEBUG_TCSTATE {
                        cprintln!("<yellow>[called]: e2_fn({})</yellow>", e1_value.clone().0,);
                    }
                    let mut e2_return = (self.e2_fn.take().unwrap())(e1_value.0)?;
                    e2_return.0 = Some(self);
                    Ok(e2_return)
                }
            }
        }
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
                e1_result: None,
            })),
        ))
    }
}

impl TCState for TLetState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        if let Some(_) = self.e1_result {
            return self.step_over();
        }
        if DEBUG_TCSTATE {
            cprintln!("<red>[[step_into]]\n{}</red>", self);
            cprintln!("<yellow>[called]: e1_fn()</yellow>");
        }
        let mut e1_result = (self.e1_fn.take().unwrap())()?;
        e1_result.0 = Some(self);
        Ok(e1_result)
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        if DEBUG_TCSTATE {
            cprintln!("<red>[[evaluate]]\n{}</red>", self);
        }
        let mut state_machine = TCStateMachine::new(self);
        state_machine._evaluate()
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn step_over(mut self: Box<Self>) -> TCReturnResult {
        if DEBUG_TCSTATE {
            cprintln!("<red>[[step_over]]\n{}</red>", self);
        }
        if let Some(result) = self.binding_result.clone() {
            if DEBUG_TCSTATE {
                cprintln!("<yellow>[called]: e2_fn({})</yellow>", result.clone()?.0,);
            }
            let mut e2_return = (self.e2_fn.take().unwrap())(result?.0)?;
            e2_return.0 = Some(self);
            Ok(e2_return)
        } else {
            if DEBUG_TCSTATE {
                cprintln!("<yellow>[called]: e1_fn()</yellow>");
            }
            let e1_return = (self.e1_fn.take().unwrap())()?;
            match e1_return {
                (_, TCReturn::TCResult(result)) => {
                    self.e1_result = Some(result.clone());
                    let e1_value = err_if_dead(result)?;
                    if DEBUG_TCSTATE {
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
                            if DEBUG_TCSTATE {
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
                    let e1_value = err_if_dead(state.evaluate())?;
                    self.e1_result = Some(Ok(e1_value.clone()));
                    if DEBUG_TCSTATE {
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
                            if DEBUG_TCSTATE {
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

    fn should_remain_on_stack(&self) -> bool {
        true
    }

    fn handle_result(
        mut self: Box<Self>,
        result: TCResult,
    ) -> Result<Option<TCReturn>, OxidizedError> {
        if DEBUG_TCSTATE {
            cprintln!(
                "<red>[[handle_result]]: \n<white>{}</white>\n{}</red>",
                result.clone()?.0,
                self
            );
        }
        if let Some(binding_result) = self.binding_result.clone() {
            let result_val = err_if_dead(result)?;
            let binding_result_val = binding_result?;

            if DEBUG_TCSTATE {
                cprintln!(
                    "<yellow>[called]: gc_fn({}, {}, {})</yellow>",
                    result_val.clone().0,
                    result_val.clone().1,
                    binding_result_val.clone().1
                );
            }
            let gc_return_val =
                (self.gc_fn.take().unwrap())(result_val.0, result_val.1, binding_result_val.1)?;
            Ok(Some(gc_return_val.1))
        } else {
            self.e1_result = Some(result.clone());
            let result_val = err_if_dead(result)?;

            if DEBUG_TCSTATE {
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
            Ok(Some(TCReturn::TCState(self)))
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
                e1_result: None,
                binding_result: None,
            })),
        ))
    }
}

impl TCStateMachine {
    fn print_stack(&self) {
        println!("======================");
        for (i, state) in self.states.iter().enumerate() {
            cprintln!("<blue>[{}]</blue><green>\n{}</green>", i, state);
        }
        println!("----------------------");
    }

    fn push_state(&mut self, state: Box<dyn TCState>) {
        self.states.push(state);
        if DEBUG_TCSTATE {
            cprintln!("<yellow>(push)</yellow>");
            self.print_stack();
        }
    }

    fn pop_state(&mut self) -> Option<Box<dyn TCState>> {
        let state = self.states.pop();
        if DEBUG_TCSTATE {
            cprintln!("<yellow>(pop)</yellow>");
            self.print_stack();
        }
        state
    }

    pub(crate) fn new(state: Box<dyn TCState>) -> Self {
        Self {
            states: vec![state],
        }
    }

    pub(crate) fn evaluate(&mut self) -> TCResult {
        if DEBUG_TCSTATE {
            cprintln!("<cyan>Eval Begin</cyan>");
        }
        while self.states.len() > 0 {
            match self.step_over() {
                Ok((Some(final_result), Some(prev_result))) => {
                    if DEBUG_TCSTATE {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    println!("[[final_result]]:\n{}\n[[prev_result]]:", final_result?.0);
                    return prev_result;
                }
                Ok((Some(final_result), None)) => {
                    if DEBUG_TCSTATE {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    return final_result;
                }
                Err(err) => {
                    if DEBUG_TCSTATE {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    return Err(err);
                }
                _ => continue,
            }
        }
        panic!("failed to evaluate");
    }

    // I wanted to be able to return the final and previous values in the eval method but that only works at the last step.
    // Since this method is called recursively by the other tc_state methods, returning the previous result multiple times
    // in this recursive loop leads to incorrect outputs.
    fn _evaluate(&mut self) -> TCResult {
        if DEBUG_TCSTATE {
            cprintln!("<cyan>Eval Begin</cyan>");
        }
        while self.states.len() > 0 {
            match self.step_over() {
                Ok((Some(final_result), _)) => {
                    if DEBUG_TCSTATE {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    return final_result;
                }
                Err(err) => {
                    if DEBUG_TCSTATE {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    return Err(err);
                }
                _ => continue,
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

    pub(crate) fn step_into(
        &mut self,
    ) -> Result<(Option<TCResult>, Option<TCResult>), OxidizedError> {
        if DEBUG_TCSTATE {
            cprintln!("<cyan>STATE_MACHINE step_into</cyan>");
        }
        let top = self.pop_state().unwrap();
        let return_result = top.step_into()?;
        match return_result {
            (_, TCReturn::TCResult(result)) => {
                if let Some(next_top) = self.pop_state() {
                    let handle_result_state = next_top.handle_result(result)?;
                    match handle_result_state {
                        Some(TCReturn::TCResult(result)) => {
                            let next_top = self.pop_state().unwrap();
                            let mut prev_result = result.clone();
                            let mut handle_result_state = next_top.handle_result(result.clone())?;

                            // Handles cascading calls to exit functions such as gc_fn
                            loop {
                                match handle_result_state {
                                    Some(TCReturn::TCState(state)) => {
                                        self.push_state(state);
                                        break Ok((None, None));
                                    }
                                    Some(TCReturn::TCResult(next_result)) => {
                                        if self.states.len() > 0 {
                                            let next_top = self.pop_state().unwrap();
                                            prev_result = next_result.clone();
                                            handle_result_state =
                                                next_top.handle_result(next_result.clone())?;
                                        } else {
                                            break Ok((Some(next_result), Some(prev_result)));
                                        }
                                    }
                                    _ => break Ok((None, None)),
                                }
                            }
                        }
                        Some(TCReturn::TCState(state)) => {
                            self.push_state(state);
                            Ok((None, None))
                        }
                        _ => Ok((None, None)),
                    }
                } else {
                    Ok((Some(result), None))
                }
            }
            (last_top, TCReturn::TCState(state)) => {
                if let Some(state) = last_top {
                    if state.should_remain_on_stack() {
                        self.push_state(state);
                    }
                }
                self.push_state(state);
                Ok((None, None))
            }
        }
    }

    pub(crate) fn step_over(
        &mut self,
    ) -> Result<(Option<TCResult>, Option<TCResult>), OxidizedError> {
        if DEBUG_TCSTATE {
            cprintln!("<cyan>STATE_MACHINE step_over</cyan>");
        }
        let top = self.pop_state().unwrap();
        let return_result = top.step_over()?;
        match return_result {
            (_, TCReturn::TCResult(result)) => {
                if let Some(next_top) = self.pop_state() {
                    let handle_result_state = next_top.handle_result(result)?;
                    match handle_result_state {
                        Some(TCReturn::TCResult(result)) => {
                            let next_top = self.pop_state().unwrap();
                            let mut prev_result = result.clone();
                            let mut handle_result_state = next_top.handle_result(result.clone())?;

                            // Handles cascading calls to exit functions such as gc_fn
                            loop {
                                match handle_result_state {
                                    Some(TCReturn::TCState(state)) => {
                                        self.push_state(state);
                                        break Ok((None, None));
                                    }
                                    Some(TCReturn::TCResult(next_result)) => {
                                        if self.states.len() > 0 {
                                            let next_top = self.pop_state().unwrap();
                                            prev_result = next_result.clone();
                                            handle_result_state =
                                                next_top.handle_result(next_result.clone())?;
                                        } else {
                                            break Ok((Some(next_result), Some(prev_result)));
                                        }
                                    }
                                    _ => break Ok((None, None)),
                                }
                            }
                        }
                        Some(TCReturn::TCState(state)) => {
                            self.push_state(state);
                            Ok((None, None))
                        }
                        _ => Ok((None, None)),
                    }
                } else {
                    Ok((Some(result), None))
                }
            }
            (last_top, TCReturn::TCState(state)) => {
                if let Some(state) = last_top {
                    if state.should_remain_on_stack() {
                        self.push_state(state);
                    }
                }
                self.push_state(state);
                Ok((None, None))
            }
        }
    }
}
