use std::fmt::Display;
// use color_print;

// #[derive(Debug, Clone)]
// pub enum Type {
//     Base(BaseType),
//     Aggregate(AggregateType),
// }

// #[derive(Debug, Clone)]
// pub struct AggregateType {}

// #[derive(Debug, Clone)]
// pub enum BaseType {
//     Bool,
//     U32,
//     Unit,
// }

#[derive(Debug, Clone)]
pub enum Expression {
    Constant(Constant),
    Variable(String),
    Let {
        name: String,
        is_mut: bool,
        ref_decl: Option<ReferenceDeclaration>,
        e: Box<Expression>,
    },
    Deref(Box<Expression>),
    PathAccess(Box<Expression>, Vec<u32>),
    Letrgn(Vec<String>, Box<Expression>),
    Sequence(Box<Expression>, Box<Expression>),
    CSV(Vec<Box<Expression>>),
    // ConstructorCall(String, Box<Expression>), // doesn't matter rn cause its just named tuples
}

pub enum PlaceExpression {
    Deref(Box<PlaceExpression>),
    PathAccess(Box<PlaceExpression>, Vec<u32>),
    Variable(String),
}

impl TryFrom<Expression> for PlaceExpression {
    type Error = String;

    fn try_from(value: Expression) -> Result<Self, Self::Error> {
        match value {
            Expression::Deref(e) => {
                let place_expression = (*e).try_into()?;
                Ok(Self::Deref(Box::new(place_expression)))
            }
            Expression::PathAccess(e, p) => {
                let place_expression = (*e).try_into()?;
                Ok(Self::PathAccess(Box::new(place_expression), p))
            }
            Expression::Variable(v) => Ok(Self::Variable(v)),
            _ => Err("Not a valid place expression".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Constant {
    Unit,
    True,
    False,
    Number(u32),
}

#[derive(Debug, Clone)]
pub struct ReferenceDeclaration(pub String, pub OwnershipQualifier);

#[derive(Debug, Clone)]
pub enum OwnershipQualifier {
    Shrd,
    Uniq,
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Sequence(e1, e2) => return write!(f, "\n    {};\n    {}", e1, e2),
            Expression::Let {
                name,
                is_mut,
                ref_decl,
                e,
            } => {
                return write!(
                    f,
                    "let {} {} = {:?} {}",
                    if *is_mut { "mut" } else { "" },
                    name,
                    ref_decl,
                    e
                );
            }
            Expression::Letrgn(regions, e) => {
                return write!(f, "letrgn <{:?}> {{{}\n}}", regions, e);
            }
            _ => return write!(f, "{:?}", self),
        }
    }
}
