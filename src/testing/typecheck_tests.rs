use rstest::*;

use crate::oxidized;
use crate::static_analysis::data_structures::ConcreteTyping;
use crate::static_analysis::data_structures::OxidizedError;
use crate::static_analysis::tc_state_machine::*;
use crate::testing::oxide_programs;
use crate::testing::set_snapshot_suffix;

fn unwrap_state(result: TCReturnResult) -> Box<dyn TCState> {
    match result {
        Ok(result) => match result.1 {
            TCReturn::TCResult(_) => todo!(),
            TCReturn::TCState(state) => state,
        },
        Err(_) => todo!(),
    }
}

#[rstest]
#[case(oxide_programs::double_ref(), 0)]
#[case(oxide_programs::double_ref_bad(), 1)]
#[case(oxide_programs::triple_ref(), 2)]
#[case(oxide_programs::triple_ref2(), 3)]
#[case(oxide_programs::double_ref2(), 4)]
#[case(oxide_programs::ref_rr(), 5)]
#[case(oxide_programs::ref_rr_scopes(), 6)]
#[case(oxide_programs::ref_rr_scopes_bad(), 7)]
#[case(oxide_programs::unsafe_place(), 8)]
#[case(oxide_programs::gc_test(), 9)]
#[case(oxide_programs::move_ref(), 10)]
#[case(oxide_programs::move_ref_err(), 11)]
#[case(oxide_programs::copy_ref(), 12)]
#[case(oxide_programs::copy_ref_usage(), 13)]
#[case(oxide_programs::move_to_place(), 14)]
#[case(oxide_programs::move_to_place_err(), 15)]
#[case(oxide_programs::copy_to_place(), 16)]
#[case(oxide_programs::copy_to_place_usage(), 17)]
#[case(oxide_programs::move_to_place_err2(), 18)]
#[case(oxide_programs::copy_to_place2(), 19)]
#[case(oxide_programs::copy_to_place3(), 20)]
#[case(oxide_programs::move_to_place2(), 21)]
#[case(oxide_programs::gc_test2(), 22)]
#[case(oxide_programs::move_to_place3(), 23)]
#[case(oxide_programs::move_to_place3_err(), 24)]
#[case(oxide_programs::complex_rewrite(), 25)]
fn full_program(#[case] program: &'static str, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    let ast = oxidized::ExprParser::new().parse(program).unwrap();
    dbg!(&ast);

    insta::with_settings!({
        description => program,
        omit_expression => true
    }, {
        insta::assert_snapshot!({
            let state = unwrap_state(ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()));
            let mut state_machine = TCStateMachine::new(state);
            let results = state_machine.deep_evaluate();
            results
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    format!(
                        "Step {}: {}\n{}\n\n",
                        i + 1,
                        r.1,
                        match &r.0 {
                            Ok(val) => format!("{}", val.0),
                            Err(err) => format!("{}", err),
                        }
                    )
                })
                .collect::<Vec<String>>()
                .join("")
        })
    });
}

#[rstest]
#[case(oxide_programs::ref_rr_scopes(), 0)]
fn into_over_x2(#[case] program: &'static str, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    let ast = oxidized::ExprParser::new().parse(program).unwrap();
    dbg!(&ast);

    insta::with_settings!({
        description => program,
        omit_expression => true
    }, {
        insta::assert_snapshot!({
            let state = unwrap_state(ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()));
            let mut state_machine = TCStateMachine::new(state);
            let maybe_error = (|| -> Result<(), OxidizedError> {
                state_machine.step_into()?;
                state_machine.step_over()?;
                state_machine.step_into()?;
                state_machine.step_over()?;
                Ok(())
            })();

            match maybe_error {
                Ok(_) => format!("{}", state_machine.get_env_state().0),
                Err(err) => format!("{}", err),
            }
        })
    });
}

// #[rstest]
// #[case(oxide_programs::move_to_place2(), 0)]
// fn into_over_x2_x2(#[case] program: &'static str, #[case] n: u32) {
//     set_snapshot_suffix!("{}", n);

//     let ast = oxidized::ExprParser::new().parse(program).unwrap();
//     dbg!(&ast);

//     insta::with_settings!({
//         description => program,
//         omit_expression => true
//     }, {
//         insta::assert_snapshot!({
//             let state = unwrap_state(ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()));
//             let mut state_machine = TCStateMachine::new(state);
//             let maybe_error = (|| -> Result<(), OxidizedError> {
//                 state_machine.step_into()?;
//                 state_machine.step_over()?;
//                 state_machine.step_over()?;
//                 state_machine.step_into()?;
//                 state_machine.step_over()?;
//                 Ok(())
//             })();

//             match maybe_error {
//                 Ok(_) => format!("{}", state_machine.get_env_state().0),
//                 Err(err) => format!("{}", err),
//             }
//         })
//     });
// }

#[rstest]
#[case(oxide_programs::move_ref(), 0)]
#[case(oxide_programs::move_to_place(), 1)]
#[case(oxide_programs::copy_ref(), 2)]
#[case(oxide_programs::copy_to_place(), 3)]
fn three_line_move_or_copy(#[case] program: &'static str, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    let ast = oxidized::ExprParser::new().parse(program).unwrap();
    dbg!(&ast);

    insta::with_settings!({
        description => program,
        omit_expression => true
    }, {
        insta::assert_snapshot!({
            let state = unwrap_state(ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()));
            let mut state_machine = TCStateMachine::new(state);
            let maybe_error = (|| -> Result<(), OxidizedError> {
                state_machine.step_into()?;
                state_machine.step_over()?;
                state_machine.step_over()?;
                state_machine.step_over()?;
                Ok(())
            })();

            match maybe_error {
                Ok(_) => format!("{}", state_machine.get_env_state().0),
                Err(err) => format!("{}", err),
            }
        })
    });
}

#[rstest]
#[case(oxide_programs::move_ref_err(), 0)]
#[case(oxide_programs::move_to_place_err(), 1)]
#[case(oxide_programs::copy_ref_usage(), 2)]
#[case(oxide_programs::copy_to_place_usage(), 3)]
fn four_line_move_or_copy(#[case] program: &'static str, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    let ast = oxidized::ExprParser::new().parse(program).unwrap();
    dbg!(&ast);

    insta::with_settings!({
        description => program,
        omit_expression => true
    }, {
        insta::assert_snapshot!({
            let state = unwrap_state(ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()));
            let mut state_machine = TCStateMachine::new(state);
            let maybe_error = (|| -> Result<(), OxidizedError> {
                state_machine.step_into()?;
                state_machine.step_over()?;
                state_machine.step_over()?;
                state_machine.step_over()?;
                state_machine.step_over()?;
                Ok(())
            })();

            match maybe_error {
                Ok(_) => format!("{}", state_machine.get_env_state().0),
                Err(err) => format!("{}", err),
            }
        })
    });
}
