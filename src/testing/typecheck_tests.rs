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
fn full_program(#[case] program: &'static str, #[case] n: u32) {
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
