use oxidized::parser::ExprParser;
use oxidized::set_snapshot_suffix;
use oxidized::testlib::oxide_programs::TestProgram;
use oxidized::testlib::oxide_programs::check_rust_code;
use rstest::*;

use oxidized::debugging::debugger::*;
use oxidized::debugging::tc_state::*;
use oxidized::static_analysis::data_structures::ConcreteTyping;
use oxidized::testlib::oxide_programs;

fn unwrap_state(result: StepResult) -> Box<dyn DebuggerState> {
    match result {
        Ok(result) => match result.result {
            FnReturn::FnResult(_) => todo!(),
            FnReturn::DebuggerState(state) => state,
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
#[case(oxide_programs::uniq_lifetime(), 26)]
#[case(oxide_programs::move_inner_ref(), 27)]
#[case(oxide_programs::complex_nesting(), 28)]
#[case(oxide_programs::primitive_assignment(), 29)]
#[case(oxide_programs::assign_under_ref_err(), 30)]
#[case(oxide_programs::reassign_ref_err(), 31)]
#[case(oxide_programs::reassign_ref(), 32)]
#[case(oxide_programs::primitive_assignment(), 33)]
#[case(oxide_programs::dangling_ref_err(), 34)]
#[case(oxide_programs::dangling_ref_err2(), 35)]
#[case(oxide_programs::basic_deref(), 36)]
#[case(oxide_programs::reborrow(), 37)]
#[case(oxide_programs::reborrow_err(), 38)]
#[case(oxide_programs::reborrow_move_ref_err(), 39)]
// #[case(oxide_programs::deref2x(), 40)]
fn full_program(#[case] program: TestProgram, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    let ast = ExprParser::new().parse(program.oxide).unwrap();
    dbg!(&ast);

    insta::with_settings!({
        description => format!("({})\n\nOxide:\n{}\nRust:\n{}", program.program_name, program.oxide, program.rust),
        omit_expression => true
    }, {
        insta::assert_snapshot!({
            let state = unwrap_state(ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()));
            let mut state_machine = Debugger::new(state);
            let results = state_machine.deep_evaluate();
            let results = results
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
                .join("");

            format!("{results}\n{}",  match check_rust_code(program.rust) {
                Ok(_) => "Rust compiles".to_string(),
                Err(err) => format!("Rust does not compile:\n{err}"),
            })
        })
    });
}
