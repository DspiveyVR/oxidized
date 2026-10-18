use oxidized::{
    ast::OwnershipQualifier, parser::ExprParser, set_snapshot_suffix,
    static_analysis::data_structures::*,
};
use rstest::*;
use similar_asserts::assert_eq;

#[fixture]
pub fn new_concrete() -> ConcreteTyping {
    ConcreteTyping::new_with_regions(&vec!["a".to_string()], 0)
}

pub fn get_u32() -> Box<Type> {
    Box::new(Type {
        variable: None,
        variant: TypeVariant::Base(BaseType::U32),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
    })
}

pub fn get_ref_to(
    typ: Box<Type>,
    ownership_qualifier: OwnershipQualifier,
    region: String,
) -> Box<Type> {
    Box::new(Type {
        variable: None,
        variant: TypeVariant::Reference(ReferenceType {
            region_name: region,
            ownership_qualifier: ownership_qualifier,
            target_type: typ,
        }),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
    })
}

#[fixture]
pub fn tup1() -> Type {
    Type {
        variable: Some("var".to_string()),
        variant: TypeVariant::Tuple(vec![
            get_u32(),
            Box::new(Type {
                variable: None,
                variant: TypeVariant::Tuple(vec![get_u32(), get_u32()]),
                is_sized: Ternary::Yes,
                is_dead: Ternary::No,
            }),
        ]),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
    }
}

#[fixture]
pub fn tup2() -> Type {
    Type {
        variable: Some("var2".to_string()),
        variant: TypeVariant::Tuple(vec![
            get_u32(),
            get_ref_to(
                Box::new(Type {
                    variable: None,
                    variant: TypeVariant::Tuple(vec![get_u32(), get_u32()]),
                    is_sized: Ternary::Yes,
                    is_dead: Ternary::No,
                }),
                OwnershipQualifier::Uniq,
                "a".to_string(),
            ),
        ]),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
    }
}

#[fixture]
pub fn tup3() -> Type {
    Type {
        variable: Some("var2".to_string()),
        variant: TypeVariant::Tuple(vec![
            get_u32(),
            get_ref_to(
                Box::new(Type {
                    variable: None,
                    variant: TypeVariant::Tuple(vec![get_u32(), get_u32()]),
                    is_sized: Ternary::Yes,
                    is_dead: Ternary::No,
                }),
                OwnershipQualifier::Shrd,
                "a".to_string(),
            ),
        ]),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
    }
}

#[fixture]
pub fn tup2_inner_binding() -> Type {
    Type {
        variable: Some("var2".to_string()),
        variant: TypeVariant::Tuple(vec![
            get_u32(),
            get_ref_to(
                Box::new(Type {
                    variable: Some("var2".to_string()),
                    variant: TypeVariant::Tuple(vec![get_u32(), get_u32()]),
                    is_sized: Ternary::Yes,
                    is_dead: Ternary::No,
                }),
                OwnershipQualifier::Uniq,
                "a".to_string(),
            ),
        ]),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
    }
}

#[fixture]
fn var_pe() -> PlaceExpression {
    (*ExprParser::new().parse("var").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn var2_pe() -> PlaceExpression {
    (*ExprParser::new().parse("var2").unwrap())
        .try_into()
        .unwrap()
}

// #[rstest]
// #[case(tup1(), tup1_pe())]
// #[case(tup2(), tup2_pe())]
// fn u32_lookup(#[case] typ: Type, #[case] pe: PlaceExpression, new_concrete: ConcreteTyping) {
//     let mut env = new_concrete.clone();
//     env = env.add_type(typ);
//     assert_eq!(
//         format!("{}", env.type_lookup(pe).unwrap()),
//         format!("{}", *get_u32())
//     );
// }

// #[rstest]
// #[case(tup1(), var_pe())]
// #[case(tup2(), var2_pe())]
// fn self_lookup(#[case] typ: Type, #[case] var: PlaceExpression, new_concrete: ConcreteTyping) {
//     let mut env = new_concrete.clone();
//     env = env.add_type(typ.clone());
//     assert_eq!(
//         format!("{}", env.lookup_type(var).unwrap()),
//         format!("{}", typ)
//     );
// }

// #[rstest]
// #[case(concrete_env1(), 0)]
// fn explode_env(#[case] env: ConcreteTyping, #[case] n: u32) {
//     set_snapshot_suffix!("{}", n);

//     insta::with_settings!({
//         description => format!("{}", &env),
//         omit_expression => true
//     }, {
//         insta::assert_snapshot!(vec_display_elements(
//             &env.explode()
//                 .0
//                 .iter()
//                 .map(|pair| format_explode_pair(pair.clone()))
//                 .collect()
//         ))
//     });
// }

// #[rstest]
// #[case(*get_u32(), 0)]
// #[case(tup1(), 1)]
// #[case(tup2(), 2)]
// fn explode_type(#[case] typ: Type, #[case] n: u32) {
//     set_snapshot_suffix!("{}", n);

//     insta::assert_snapshot!(vec_display_elements(&typ.explode().0));
// }

#[fixture]
fn test_pe1() -> PlaceExpression {
    (*ExprParser::new().parse("var.1.0").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn test_pe2() -> PlaceExpression {
    (*ExprParser::new().parse("var.0.1").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn test_pe3() -> PlaceExpression {
    (*ExprParser::new().parse("var.0.0").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn test_pe4() -> PlaceExpression {
    (*ExprParser::new().parse("var.1").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn test_pe5() -> PlaceExpression {
    (*ExprParser::new().parse("var.1.0.1").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn test_pe6() -> PlaceExpression {
    (*ExprParser::new().parse("(*var.1).0.1").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn test_pe7() -> PlaceExpression {
    (*ExprParser::new().parse("*var.0.0").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn test_pe8() -> PlaceExpression {
    (*ExprParser::new().parse("*(*var.1).0.1").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn test_pe9() -> PlaceExpression {
    (*ExprParser::new().parse("*var.1.0").unwrap())
        .try_into()
        .unwrap()
}

#[rstest]
#[case(var_pe(), test_pe1(), 0)]
#[case(var_pe(), test_pe2(), 1)]
#[case(test_pe1(), test_pe2(), 2)]
#[case(test_pe1(), test_pe3(), 3)]
#[case(test_pe1(), test_pe4(), 4)]
#[case(test_pe4(), test_pe5(), 6)]
#[case(test_pe5(), test_pe6(), 7)]
#[case(test_pe3(), test_pe7(), 8)]
#[case(test_pe6(), test_pe8(), 9)]
#[case(test_pe9(), test_pe1(), 10)]
fn overlaps(#[case] pe1: PlaceExpression, #[case] pe2: PlaceExpression, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    insta::with_settings!({
        description => format!("{} overlaps {}", &pe1, &pe2),
        omit_expression => true
    }, {
        insta::assert_snapshot!(format!("{}", pe1.overlaps(&pe2)))
    });
}

#[rstest]
#[case(tup2(), tup2_inner_binding())]
fn not_equal(#[case] ty1: Type, #[case] ty2: Type) {
    assert_eq!(ty1, ty2)
}

#[rstest]
#[case(*get_u32(), 0)]
#[case(tup1(), 1)]
#[case(tup2(), 2)]
#[case(*get_ref_to(get_u32(), OwnershipQualifier::Shrd, "a".to_string()), 3)]
#[case(*get_ref_to(get_u32(), OwnershipQualifier::Uniq, "a".to_string()), 4)]
#[case(*get_ref_to(Box::new(tup1()), OwnershipQualifier::Shrd, "a".to_string()), 5)]
#[case(*get_ref_to(Box::new(tup1()), OwnershipQualifier::Uniq, "a".to_string()), 6)]
#[case(*get_ref_to(Box::new(tup2()), OwnershipQualifier::Shrd, "a".to_string()), 7)]
#[case(*get_ref_to(Box::new(tup2()), OwnershipQualifier::Uniq, "a".to_string()), 8)]
#[case(tup3(), 9)]
#[case(*get_ref_to(Box::new(tup3()), OwnershipQualifier::Shrd, "a".to_string()), 10)]
#[case(*get_ref_to(Box::new(tup3()), OwnershipQualifier::Uniq, "a".to_string()), 11)]
fn copyable(#[case] typ: Type, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    insta::with_settings!({
        description => format!("{} is copyable", &typ),
        omit_expression => true
    }, {
        insta::assert_snapshot!(format!("{}", typ.copyable()))
    });
}
