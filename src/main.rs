use std::println;

use oxidized::{
    parser::ExprParser,
    static_analysis::{
        data_structures::{ConcreteTyping, OxidizedError},
        tc_state_machine::{TCReturn, TCReturnResult, TCState, TCStateMachine},
    },
};

fn main() {
    let program = "

    letrgn<'x, 'y, 'z> {
        let x: u32 = 5;
        let y: &'x uniq u32 = &'x uniq x;
        let z: &'y uniq &'x uniq u32 = &'y uniq y;
        let w: &'z uniq &'x uniq u32 = &'z uniq *z;
        *w
    }


    ";
    let ast = oxidized::parser::ExprParser::new().parse(program).unwrap();
    println!("{}", ast);

    let state = unwrap_state(ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()));
    let mut state_machine = TCStateMachine::new(state);

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
