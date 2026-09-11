use std::fmt::Display;

use serde::Serialize;
// use color_print;

//TODO: convert these tuple structs into named structs you idiot

#[derive(Debug, Clone)]
pub enum BaseTypeAnnotation {
    Bool,
    U32,
    Unit,
}

#[derive(Debug, Clone)]
pub enum TypeAnnotation {
    Base(BaseTypeAnnotation),
    Reference(ReferenceAnnotation, Box<TypeAnnotation>),
    Tuple(Vec<Box<TypeAnnotation>>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Constant(Constant),
    Variable(String),
    Let {
        name: String,
        is_mut: bool,
        type_annotation: TypeAnnotation,
        e: Box<Expression>,
    },
    Assign(Box<Expression>, Box<Expression>),
    Ref(ReferenceAnnotation, Box<Expression>),
    Deref(Box<Expression>),
    PathAccess(Box<Expression>, Vec<u32>),
    Letrgn(Vec<String>, Box<Expression>),
    Sequence(Box<Expression>, Box<Expression>),
    CSV(Vec<Box<Expression>>),
    // ConstructorCall(String, Box<Expression>), // doesn't matter rn cause its just named tuples
}

#[derive(Debug, Clone)]
pub enum Constant {
    Unit,
    True,
    False,
    Number(u32),
}

#[derive(Debug, Clone)]
pub struct ReferenceAnnotation(pub String, pub OwnershipQualifier);

impl Eq for OwnershipQualifier {}
#[derive(Serialize, Debug, Copy, Clone, Hash, PartialEq)]
pub enum OwnershipQualifier {
    Shrd,
    Uniq,
}

impl ToString for OwnershipQualifier {
    fn to_string(&self) -> String {
        if *self == OwnershipQualifier::Uniq {
            "unique".to_string()
        } else {
            "shared".to_string()
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Sequence(e1, e2) => return write!(f, "\n    {};\n    {}", e1, e2),
            Expression::Let {
                name,
                is_mut,
                type_annotation,
                e,
            } => {
                return write!(
                    f,
                    "\n    let {} {}: {:?} = {}",
                    if *is_mut { "mut" } else { "" },
                    name,
                    type_annotation,
                    e,
                );
            }
            Expression::Letrgn(regions, e) => {
                return write!(f, "letrgn <{:?}> {{{}\n}}", regions, e);
            }
            _ => return write!(f, "{:?}", self),
        }
    }
}
