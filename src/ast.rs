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
    Statement(Statement)
}

#[derive(Debug, Clone)]
pub enum Block {
    Letrgn(Vec<String>, Vec<Term>),
    EmptyBlock(Vec<Term>)
}

#[derive(Debug, Clone)]
pub enum Statement {
    StructDeclaration(String, Vec<Type>),
    Expression(Expression)
}

#[derive(Debug, Clone)]
pub enum Expression {
    Constant(Constant),
    Variable(String),
    Let(String, bool, Option<ReferenceDeclaration>, Box<Expression>),
    Deref(Box<Expression>),
    PathAccess(Box<Expression>, Vec<u32>),
    CSV(Vec<Box<Expression>>),
    ConstructorCall(String, Box<Expression>)
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
