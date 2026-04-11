use crate::{
    oxidized,
    static_analysis::data_structures::{ConcreteTyping, PlaceExpression, Type},
    testing::{
        set_snapshot_suffix,
        unit_tests::{concrete_env1, tup1, tup2},
    },
};
use rstest::*;

#[fixture]
fn pe1() -> PlaceExpression {
    (*oxidized::ExprParser::new().parse("(*var.1).0").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn pe2() -> PlaceExpression {
    (*oxidized::ExprParser::new().parse("*var.1.0").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn pe3() -> PlaceExpression {
    (*oxidized::ExprParser::new()
        .parse("(*(*(*var).1).0)")
        .unwrap())
    .try_into()
    .unwrap()
}

fn unbound_tup() -> Type {
    let mut tup = tup2();
    tup.binding = None;
    tup
}

#[rstest]
#[case(pe1(), 0)]
#[case(pe2(), 1)]
#[case(pe3(), 2)]
fn pe_display(#[case] pe: PlaceExpression, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);
    insta::assert_snapshot!(format!("{}", pe));
}

#[rstest]
#[case(tup1(), 0)]
#[case(tup2(), 1)]
#[case(unbound_tup(), 2)]
fn type_display(#[case] typ: Type, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    // // Seems I can't display color in snapshot view unfortunately.
    // let mut settings = insta::Settings::clone_current();
    // settings.add_filter(r"\x1b\[[0-9;]*m", "");
    // settings.add_filter(r"\x1b\(B", "");

    // settings.bind(|| {
    //     insta::assert_snapshot!(format!("{}", typ));
    // });
    insta::assert_snapshot!(format!("{}", typ));
}

#[rstest]
#[case(concrete_env1(), 0)]
fn concrete_env_display(#[case] env: ConcreteTyping, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    // // Seems I can't display color in snapshot view unfortunately.
    // let mut settings = insta::Settings::clone_current();
    // settings.add_filter(r"\x1b\[[0-9;]*m", "");
    // settings.add_filter(r"\x1b\(B", "");

    // settings.bind(|| {
    //     insta::assert_snapshot!(format!("{}", env));
    // });
    insta::assert_snapshot!(format!("{}", env));
}
