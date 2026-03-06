#[derive(Debug)]
pub enum Type {
    Base(BaseType),
    Aggregate(AggregateType),
}

#[derive(Debug)]
pub struct AggregateType {}

#[derive(Debug)]
pub enum BaseType {
    Bool,
    U32,
    Unit,
}

#[derive(Debug)]
pub enum Expression {
    Constant(Constant),
    Sequence(Box<Expression>, Box<Expression>),
    Let(String, Box<Expression>),
}

#[derive(Debug)]
pub enum Constant {
    Unit,
    True,
    False,
    Number(String),
}

#[derive(Debug)]
pub enum Keyword {
    Struct,
    Letrgn,
    OwnershipQualifier(OwnershipQualifier),
}

#[derive(Debug)]
pub enum OwnershipQualifier {
    Shrd,
    Uniq,
}
