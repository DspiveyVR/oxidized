use rstest::*;

use crate::oxidized;
use crate::static_analysis::data_structures::ConcreteTyping;
use crate::testing::oxide_programs;
use crate::testing::set_snapshot_suffix;

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
fn full_program(#[case] program: &'static str, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    let ast = oxidized::ExprParser::new().parse(program).unwrap();

    insta::with_settings!({
        description => program,
        omit_expression => true
    }, {
        insta::assert_snapshot!(match ast.t_letrgn(ConcreteTyping::new(), ConcreteTyping::new()) {
            Ok(result) => format!("{}", result.0),
            Err(err) => format!("{}", err)
        })
    });
}

#[rstest]
#[case(oxide_programs::double_ref(), 0)]
#[case(oxide_programs::triple_ref(), 1)]
fn empty_st(#[case] program: &'static str, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    let ast = oxidized::ExprParser::new().parse(program).unwrap();

    insta::with_settings!({
        description => program,
        omit_expression => true
    }, {
        insta::assert_snapshot!(match ast.typecheck(ConcreteTyping::new(), ConcreteTyping::new()) {
            Ok(result) => format!("{}", result.0),
            Err(err) => format!("{}", err)
        })
    });
}