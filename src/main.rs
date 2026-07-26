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
    letrgn<'x, 'y> {
        let x: u32 = 5;
        let z: u32 = 5;
        let mut y: &'x uniq u32 = &'x uniq x;
        y = &'y uniq z;
        let w: &'x uniq u32 = &'x uniq x
    }
    ";
    let ast = oxidized::ExprParser::new().parse(program).unwrap();
    println!("{}", ast);

    let state = unwrap_state(ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()));
    let mut state_machine = TCStateMachine::new(state);
    // let maybe_error = (|| -> Result<(), OxidizedError> {
    //     // past 1st 2 lets
    //     state_machine.step_into()?;
    //     state_machine.step_over()?;
    //     state_machine.step_over()?;

    //     // into 2nd letrgn
    //     state_machine.step_into()?;
    //     state_machine.step_into()?;

    //     // out of 2nd letrgn
    //     state_machine.step_over()?;
    //     state_machine.step_over()?;

    //     // // into 3rd letrgn
    //     state_machine.step_into()?;
    //     state_machine.step_into()?;

    //     state_machine.step_over()?;
    //     state_machine.step_over()?;

    //     Ok(())
    // })();

    // match maybe_error {
    //     Ok(_) => println!("{}", state_machine.get_env_state().0),
    //     Err(err) => println!("{}", err),
    // }

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
