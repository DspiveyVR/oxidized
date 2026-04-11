use crate::{
    ast::OwnershipQualifier, debugging::display_impls::vec_display_elements, oxidized,
    static_analysis::data_structures::*, testing::set_snapshot_suffix,
};
use rstest::*;
use similar_asserts::assert_eq;

#[fixture]
pub(crate) fn new_concrete() -> ConcreteTyping {
    ConcreteTyping::new_with_regions(&vec!["a".to_string()])
}

pub(crate) fn get_u32() -> Box<Type> {
    Box::new(Type {
        binding: None,
        variant: TypeVariant::Base(BaseType::U32),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
        copyable: false,
    })
}

pub(crate) fn get_ref_to(
    typ: Box<Type>,
    ownership_qualifier: OwnershipQualifier,
    region: String,
) -> Box<Type> {
    Box::new(Type {
        binding: None,
        variant: TypeVariant::Reference(ReferenceType {
            region_name: region,
            ownership_qualifier: ownership_qualifier,
            target_type: typ,
        }),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
        copyable: false,
    })
}

#[fixture]
pub(crate) fn tup1() -> Type {
    Type {
        binding: Some(Binding {
            variable: "var".to_string(),
            is_mut: false,
        }),
        variant: TypeVariant::Tuple(vec![
            get_u32(),
            Box::new(Type {
                binding: None,
                variant: TypeVariant::Tuple(vec![get_u32(), get_u32()]),
                is_sized: Ternary::Yes,
                is_dead: Ternary::No,
                copyable: false,
            }),
        ]),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
        copyable: false,
    }
}

#[fixture]
fn tup1_pe() -> PlaceExpression {
    (*oxidized::ExprParser::new().parse("var.1.0").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
pub(crate) fn tup2() -> Type {
    Type {
        binding: Some(Binding {
            variable: "var".to_string(),
            is_mut: false,
        }),
        variant: TypeVariant::Tuple(vec![
            get_u32(),
            get_ref_to(
                Box::new(Type {
                    binding: None,
                    variant: TypeVariant::Tuple(vec![get_u32(), get_u32()]),
                    is_sized: Ternary::Yes,
                    is_dead: Ternary::No,
                    copyable: false,
                }),
                OwnershipQualifier::Uniq,
                "a".to_string(),
            ),
        ]),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
        copyable: false,
    }
}

#[fixture]
fn tup2_pe() -> PlaceExpression {
    (*oxidized::ExprParser::new().parse("(*var.1).0").unwrap())
        .try_into()
        .unwrap()
}

#[fixture]
fn var_pe() -> PlaceExpression {
    (*oxidized::ExprParser::new().parse("var").unwrap())
        .try_into()
        .unwrap()
}

#[rstest]
#[case(tup1(), tup1_pe())]
#[case(tup2(), tup2_pe())]
fn u32_lookup(#[case] typ: Type, #[case] pe: PlaceExpression, new_concrete: ConcreteTyping) {
    let mut env = new_concrete.clone();
    env = env.add_type(typ);
    assert_eq!(
        format!("{}", env.type_lookup(pe).unwrap()),
        format!("{}", *get_u32())
    );
}

#[rstest]
#[case(tup1(), var_pe())]
#[case(tup2(), var_pe())]
fn self_lookup(#[case] typ: Type, #[case] var: PlaceExpression, new_concrete: ConcreteTyping) {
    let mut env = new_concrete.clone();
    env = env.add_type(typ.clone());
    assert_eq!(
        format!("{}", env.type_lookup(var).unwrap()),
        format!("{}", typ)
    );
}

#[fixture]
pub(crate) fn concrete_env1() -> ConcreteTyping {
    let mut env = ConcreteTyping::new();
    let a = Region(RegionLevel::ConcreteRegion(ConcreteRegion {
        name: "a".to_string(),
        loans: VecSet(vec![
            Loan {
                ownership_qualifier: OwnershipQualifier::Shrd,
                place_expression: tup1_pe(),
            },
            Loan {
                ownership_qualifier: OwnershipQualifier::Uniq,
                place_expression: tup2_pe(),
            },
        ]),
    }));
    env = env.add_region(a);
    env = env.add_type(tup1());
    env = env.add_type(tup2());

    env
}

fn format_explode_pair(pair: (Place, Type)) -> String {
    format!("{{{}, {}}}", pair.0, pair.1)
}

#[rstest]
#[case(concrete_env1(), 0)]
fn explode_env(#[case] env: ConcreteTyping, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    insta::assert_snapshot!(vec_display_elements(
        &env.explode()
            .0
            .iter()
            .map(|pair| format_explode_pair(pair.clone()))
            .collect()
    ));
}

#[rstest]
#[case(*get_u32(), 0)]
#[case(tup1(), 1)]
#[case(tup2(), 2)]
fn explode_type(#[case] typ: Type, #[case] n: u32) {
    set_snapshot_suffix!("{}", n);

    insta::assert_snapshot!(vec_display_elements(&typ.explode().0));
}
