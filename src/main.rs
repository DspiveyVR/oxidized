mod ast;
mod debugging;
mod static_analysis;
mod testing;

use crate::static_analysis::tc_state_machine::*;
use lalrpop_util::lalrpop_mod;

use crate::static_analysis::data_structures::*;

lalrpop_mod!(pub oxidized);

fn main() {
    let program = "
    letrgn<'x> {
        let mut x: u32 = 5;
        letrgn<'z> {
            let y: &'x shrd u32 = &'x shrd x; ()
        };
        let z: &'x uniq u32 = &'x uniq x; ()
    }
    ";
    let ast = oxidized::ExprParser::new().parse(program).unwrap();
    println!("{}", ast);

    let state = unwrap_state(ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()));
    let mut state_machine = TCStateMachine::new(state);
    // state_machine.step_into().unwrap();
    // state_machine.step_over().unwrap();
    // state_machine.step_over().unwrap();
    // state_machine.step_over().unwrap();
    // state_machine.step_over().unwrap();
    // println!("{}", state_machine.states.pop().unwrap().get_env_state().0);
    // println!(
    //     "{}",
    //     match ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()) {
    //         Ok(result) => {
    //             match result.1 {
    //                 TCReturn::TCResult(result) => match result {
    //                     Ok(val) => format!("{}", val.0),
    //                     Err(err) => format!("{}", err),
    //                 },
    //                 TCReturn::TCState(state) => {
    //                     let result = TCStateMachine::evaluate(state);
    //                     match result {
    //                         Ok(val) => format!("{}", val.0),
    //                         Err(err) => format!("{}", err),
    //                     }
    //                 }
    //             }
    //         }
    //         Err(err) => format!("{}", err),
    //     }
    // );

    let result = state_machine.evaluate();
    match result {
        Ok(val) => println!("{}", val.0),
        Err(err) => println!("{}", err),
    }
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
