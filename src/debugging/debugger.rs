use crate::debugging::tc_state::*;
use color_print::cprintln;
use std::println;
use std::vec;

pub const DEBUG_OUTPUT: bool = true;
pub const PRINT_STACK: bool = false;

pub struct Debugger {
    states: Vec<DebuggerState>,
    si_count: u32,
    so_count: u32,
}

// FIXME: NO ENUM
pub enum DebuggerState {
    TCState(Box<dyn TCState>),
    // OCState(Box<dyn OCState>)
}

impl Debugger {
    fn print_stack(&self) {
        println!("======================");
        for (i, state) in self.states.iter().enumerate() {
            cprintln!("<blue>[{}]</blue><green>\n{}</green>", i, state);
        }
        println!("----------------------");
    }

    fn push_state(&mut self, state: DebuggerState) {
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

    fn pop_state(&mut self) -> Option<DebuggerState> {
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

    pub fn new(state: DebuggerState) -> Self {
        Self {
            states: vec![state],
            si_count: 0,
            so_count: 0,
        }
    }

    pub fn evaluate(&mut self) -> TCResult {
        if DEBUG_OUTPUT {
            cprintln!("<cyan>Eval Begin</cyan>");
        }
        while !self.states.is_empty() {
            match self.step_over() {
                Ok((Some(final_result), Some(prev_result))) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    println!("[[final_result]]:\n{}\n[[prev_result]]:", final_result?.0);
                    return prev_result;
                }
                Ok((Some(final_result), None)) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    return final_result;
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

    pub fn deep_evaluate(&mut self) -> Vec<(TCResult, String)> {
        if DEBUG_OUTPUT {
            cprintln!("<cyan>Eval Begin</cyan>");
        }
        let mut results = Vec::new();

        while !self.states.is_empty() {
            match self.step_into() {
                Ok((Some(final_result), Some(prev_result))) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }

                    results.push((prev_result, "PREV".to_string()));
                    results.push((final_result, "END".to_string()));
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
                        let env_state = state.get_env_state();
                        results.push((
                            Ok((env_state.0, get_unit())),
                            state.get_method_name(),
                        ));
                    }
                }
            }
        }

        results
    }

    // I wanted to be able to return the final and previous values in the eval method but that only works at the last step.
    // Since this method is called recursively by the other tc_state methods, returning the previous result multiple times
    // in this recursive loop leads to incorrect outputs.
    pub fn _evaluate(&mut self) -> TCResult {
        if DEBUG_OUTPUT {
            cprintln!("<cyan>Eval Begin</cyan>");
        }
        while self.states.len() > 0 {
            match self.step_over() {
                Ok((Some(final_result), _)) => {
                    if DEBUG_OUTPUT {
                        cprintln!("<cyan>Eval End</cyan>");
                    }
                    return final_result;
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
            state.get_env_state()
        } else {
            (ConcreteTyping::new(), ConcreteTyping::new())
        }
    }

    pub fn step_into(&mut self) -> Result<(Option<TCResult>, Option<TCResult>), OxidizedError> {
        self.si_count += 1;
        if DEBUG_OUTPUT {
            cprintln!("<cyan>STATE_MACHINE step_into #{}</cyan>", self.si_count);
        }

        let top = self.pop_state().unwrap();
        let return_result = top.step_into()?;

        self.step(return_result)
    }

    pub fn step_over(&mut self) -> Result<(Option<TCResult>, Option<TCResult>), OxidizedError> {
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
        return_result: (Option<DebuggerState>, TCReturn),
    ) -> Result<(Option<TCResult>, Option<TCResult>), OxidizedError> {
        match return_result {
            (_, TCReturn::TCResult(result)) => {
                if let Some(next_top) = self.pop_state() {
                    let mut prev_result = result.clone();
                    let mut handle_result_state = next_top.handle_result(result.clone())?;

                    // Passes the results of evaluation to their parent expressions.
                    loop {
                        match handle_result_state {
                            TCReturn::TCState(state) => {
                                self.push_state(state);
                                break Ok((None, None));
                            }
                            TCReturn::TCResult(next_result) => {
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
            (last_top, TCReturn::TCState(state)) => {
                if let Some(state) = last_top {
                    self.push_state(state);
                }
                self.push_state(state);
                Ok((None, None))
            }
        }
    }
}