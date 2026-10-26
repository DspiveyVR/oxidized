/*
    For TC and OC states:
        fn fields have optional types so that they can be taken in order to prevent "partailly moved" errors when trying to return self
        after calling one of these functions. self is returned by the step functions in order to transfer ownership back to the caller.
*/

use crate::debugging::oc_state::OCResult;
use crate::debugging::oc_state::OSafePlaceStateCapture;
use crate::debugging::tc_state::*;
use crate::static_analysis::auxiliary::get_unit;
use crate::static_analysis::data_structures::*;
use color_print::cprintln;
use std::fmt::Display;
use std::println;
use std::vec;

pub const DEBUG_OUTPUT: bool = true;
pub const PRINT_STACK: bool = false;

#[derive(Clone)]
pub enum FnEnvelope {
    TCResult(TCFnResult),
    OCResult(OCResult),
}

pub type FnResult = Result<FnEnvelope, OxidizedError>;

pub enum FnReturn {
    FnResult(FnResult),
    DebuggerState(Box<dyn DebuggerState>),
}

pub struct StepEnvelope {
    pub yielded_state: Option<Box<dyn DebuggerState>>, // Returns ownership to the caller. Functions stored in the state are consumed with .take() so that ownership of the state can be transferred again.
    pub result: FnReturn,                              // Result of TC/OC function.
}

pub type StepResult = Result<StepEnvelope, OxidizedError>;

#[derive(Clone)]
pub enum StateCapture {
    OSafePlaceStateCapture(OSafePlaceStateCapture),
    TCEnvStateCapture(TCEnvStateCapture),
}

pub trait DebuggerState: Display {
    fn step_into(self: Box<Self>) -> StepResult;

    fn step_over(self: Box<Self>) -> StepResult;

    fn evaluate(self: Box<Self>) -> FnEnvelope;

    // If a step method on a state returns a result, the state machine will call this method on the next
    // state on the stack so it can assign its result properties and/or call other functions like gc_fn.
    // NOTE: Results passed in here seem to consistently have unit return types so i'm not sure if the return types could
    // be incorrect coming from some typing rules or if they aren't passed through states properly. May be a non issue.
    fn handle_result(self: Box<Self>, result: FnEnvelope) -> Result<FnReturn, OxidizedError>;

    fn get_state_capture(&self) -> StateCapture;

    fn get_state_name(&self) -> String;

    fn get_method_name(&self) -> String;
}

pub struct Debugger {
    states: Vec<Box<dyn DebuggerState>>,
    si_count: u32,
    so_count: u32,
}

impl Debugger {
    fn print_stack(&self) {
        println!("======================");
        for (i, state) in self.states.iter().enumerate() {
            cprintln!("<blue>[{}]</blue><green>\n{}</green>", i, state);
        }
        println!("----------------------");
    }

    fn push_state(&mut self, state: Box<dyn DebuggerState>) {
        self.states.push(state);
        if DEBUG_OUTPUT {
            cprintln!(
                "<yellow>(push) {}</yellow>",
                if !PRINT_STACK {
                    format!("{}", self.states.last().unwrap())
                } else {
                    "".to_string()
                }
            );
            if PRINT_STACK {
                self.print_stack();
            }
        }
    }

    fn pop_state(&mut self) -> Option<Box<dyn DebuggerState>> {
        let state = self.states.pop();
        if DEBUG_OUTPUT {
            cprintln!(
                "<yellow>(pop) {}</yellow>",
                if !PRINT_STACK {
                    if let Some(state) = &state {
                        format!("{}", state)
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            );
            if PRINT_STACK {
                self.print_stack();
            }
        }
        state
    }

    pub fn new(state: Box<dyn DebuggerState>) -> Self {
        Self {
            states: vec![state],
            si_count: 0,
            so_count: 0,
        }
    }

    pub fn evaluate(&mut self) -> FnResult {
        if DEBUG_OUTPUT {
            cprintln!("<cyan>Eval Begin</cyan>");
        }
        while !self.states.is_empty() {
            match self.step_over() {
                Ok((Some(final_result), Some(prev_result))) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    match final_result {
                        FnEnvelope::TCResult(final_result) => {
                            println!("[[final_result]]:\n{}\n[[prev_result]]:", final_result?.0);
                        }
                        FnEnvelope::OCResult(ocfn_return) => todo!(),
                    }
                    return Ok(prev_result);
                }
                Ok((Some(final_result), None)) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    return Ok(final_result);
                }
                Err(err) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    return Err(err);
                }
                _ => continue,
            }
        }
        panic!("failed to evaluate");
    }

    pub fn deep_evaluate(&mut self) -> Vec<(FnResult, String)> {
        if DEBUG_OUTPUT {
            cprintln!("<cyan>Eval Begin</cyan>");
        }
        let mut results: Vec<(FnResult, String)> = Vec::new();

        while !self.states.is_empty() {
            match self.step_into() {
                Ok((Some(final_result), Some(prev_result))) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }

                    results.push((Ok(prev_result), "PREV".to_string()));
                    results.push((Ok(final_result), "END".to_string()));
                    break;
                }
                Err(err) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    results.push((Err(err), "END".to_string()));
                    break;
                }
                _ => {
                    if let Some(state) = self.states.last() {
                        let env_state = state.get_state_capture();
                        match env_state {
                            StateCapture::TCEnvStateCapture(env_state) => {
                                results.push((
                                    Ok(FnEnvelope::TCResult(Ok((
                                        env_state.stack_typing,
                                        get_unit(),
                                    )))),
                                    state.get_method_name(),
                                ));
                            }
                            StateCapture::OSafePlaceStateCapture(osafe_place_state_capture) => {
                                todo!()
                            }
                        }
                    }
                }
            }
        }

        results
    }

    // I wanted to be able to return the final and previous values in the eval method but that only works at the last step.
    // Since this method is called recursively by the other tc_state methods, returning the previous result multiple times
    // in this recursive loop leads to incorrect outputs.
    pub fn _evaluate(&mut self) -> FnResult {
        if DEBUG_OUTPUT {
            cprintln!("<cyan>Eval Begin</cyan>");
        }
        while self.states.len() > 0 {
            match self.step_over() {
                Ok((Some(final_result), _)) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    // NOTE: Could this lead to bugs? if there's an Err it will be wrapped in an Ok so it might get lost.
                    return Ok(final_result);
                }
                Err(err) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    return Err(err);
                }
                _ => continue,
            }
        }
        panic!("failed to evaluate");
    }

    pub fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        if let Some(state) = self.states.last() {
            match state.get_state_capture() {
                StateCapture::TCEnvStateCapture(state_capture) => (
                    state_capture.stack_typing,
                    state_capture.continuation_typing,
                ),
                StateCapture::OSafePlaceStateCapture(osafe_place_state_capture) => todo!(),
            }
        } else {
            (ConcreteTyping::new(), ConcreteTyping::new())
        }
    }

    pub fn step_into(&mut self) -> Result<(Option<FnEnvelope>, Option<FnEnvelope>), OxidizedError> {
        self.si_count += 1;
        if DEBUG_OUTPUT {
            cprintln!("<cyan>STATE_MACHINE step_into #{}</cyan>", self.si_count);
        }

        let top = self.pop_state().unwrap();
        let return_result = top.step_into()?;

        self.step(return_result)
    }

    pub fn step_over(&mut self) -> Result<(Option<FnEnvelope>, Option<FnEnvelope>), OxidizedError> {
        self.so_count += 1;
        if DEBUG_OUTPUT {
            cprintln!("<cyan>STATE_MACHINE step_over #{}</cyan>", self.so_count);
        }

        let top = self.pop_state().unwrap();
        let return_result = top.step_over()?;

        self.step(return_result)
    }

    fn step(
        &mut self,
        step_envelope: StepEnvelope,
    ) -> Result<(Option<FnEnvelope>, Option<FnEnvelope>), OxidizedError> {
        match step_envelope {
            StepEnvelope {
                result: FnReturn::FnResult(result),
                ..
            } => {
                if let Some(next_top) = self.pop_state() {
                    let mut prev_result = result.clone();
                    let mut handle_result_state = next_top.handle_result(result.clone())?;

                    // Passes the results of evaluation to their parent expressions.
                    loop {
                        match handle_result_state {
                            FnReturn::DebuggerState(state) => {
                                self.push_state(state);
                                break Ok((None, None));
                            }
                            FnReturn::FnResult(next_result) => {
                                if self.states.len() > 0 {
                                    let next_top = self.pop_state().unwrap();
                                    prev_result = next_result.clone();
                                    handle_result_state =
                                        next_top.handle_result(next_result.clone())?;
                                } else {
                                    break Ok((Some(next_result), Some(prev_result)));
                                }
                            }
                        }
                    }
                } else {
                    Ok((Some(result), None))
                }
            }
            StepEnvelope {
                yielded_state: last_top,
                result: FnReturn::DebuggerState(state),
            } => {
                if let Some(state) = last_top {
                    self.push_state(state);
                }
                self.push_state(state);
                Ok((None, None))
            }
        }
    }
}
