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
pub enum Expression {
    Constant(Constant),
    Variable(String),
    Let(String, Box<Expression>),
    Deref(Box<Expression>),
    PathAccess(Box<Expression>, Vec<String>)
}

#[derive(Debug, Clone)]
pub enum Constant {
    Unit,
    True,
    False,
    Number(String),
}

#[derive(Debug, Clone)]
pub enum Keyword {
    Struct,
    Letrgn,
    OwnershipQualifier(OwnershipQualifier),
}

#[derive(Debug, Clone)]
pub enum OwnershipQualifier {
    Shrd,
    Uniq,
}
