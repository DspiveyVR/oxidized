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
    Let(String, Box<Expression>),
    PlaceExpression(PlaceExpression),
}

#[derive(Debug)]
pub struct PlaceLiteral(pub Vec<String>);

#[derive(Debug)]
pub enum PlaceExpression {
    Variable(String),
    Deref(Vec<Box<PlaceExpression>>),
    Place(PlaceLiteral),
    PlaceExpressions(Vec<Box<PlaceExpression>>),
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
