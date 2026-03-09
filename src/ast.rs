use std::fmt::{Display, format};

use color_print;

#[derive(Debug, Clone)]
pub enum Type {
    Base(BaseType),
    Aggregate(AggregateType),
}

#[derive(Debug, Clone)]
pub struct AggregateType {}

#[derive(Debug, Clone)]
pub enum BaseType {
    Bool,
    U32,
    Unit,
}

#[derive(Debug, Clone)]
pub enum Term {
    Block(Block),
    Statement(Statement),
}

#[derive(Debug, Clone)]
pub enum Block {
    Letrgn(Vec<String>, Vec<Term>),
    EmptyBlock(Vec<Term>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    StructDeclaration(String, Vec<Type>),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Constant(Constant),
    Variable(String),
    Let(String, bool, Option<ReferenceDeclaration>, Box<Expression>),
    Deref(Box<Expression>),
    PathAccess(Box<Expression>, Vec<u32>),
    CSV(Vec<Box<Expression>>),
    ConstructorCall(String, Box<Expression>),
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

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Block(block) => return write!(f, "{}", block),
            Term::Statement(statement) => return write!(f, "{:?}\n\n", statement),
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::Letrgn(regions, terms) => {
                let region_str = regions
                    .iter()
                    .map(|r| format!("'{}", r))
                    .collect::<Vec<String>>()
                    .join(", ");
                let printed_terms = terms
                    .iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<String>>();
                let tabbed_terms = printed_terms
                    .iter()
                    .map(|term| {
                        term.lines()
                            .map(|l| format!("\t{}", l))
                            .collect::<Vec<String>>()
                            .join("\n")
                    })
                    .collect::<Vec<String>>();
                let body = tabbed_terms.join("");
                return write!(f, "letrgn<{}> {{\n{}\n}}", region_str, body);
            }
            Block::EmptyBlock(terms) => {
                let printed_terms = terms
                    .iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<String>>();
                let tabbed_terms = printed_terms
                    .iter()
                    .map(|term| {
                        term.lines()
                            .map(|l| format!("\t{}", l))
                            .collect::<Vec<String>>()
                            .join("\n")
                    })
                    .collect::<Vec<String>>();
                let body = tabbed_terms.join("");
                return write!(f, "{{\n{}\n}}", body);
            }
        }
    }
}
