
use std::println;

use oxidized::{
    parser::ExprParser,
    static_analysis::{
        data_structures::{ConcreteTyping, OxidizedError},
        tc_state_machine::{TCReturn, TCReturnResult, TCState, TCStateMachine},
    },
};

fn main() {
    let a = 5;
    let b = &mut a;
    let c = &mut b;
    **c;
}

fn unwrap_state(result: TCReturnResult) -> Box<dyn TCState> {
    match result {
        Ok(result) => match result.1 {
            TCReturn::TCResult(_) => todo!(),
            TCReturn::TCState(state) => state,
        },
        Err(_) => todo!(),
    }
}
