/*
NOTES:

local functions do not exist.  otherwise it would not be true that abstract regions always outlive concrete ones.

i think oxide is structurally typed so basically something like struct Object(u32,u32) would become (u32, u32) in every instance of Object.

typechecking at the global level exists strictly for function definitions, so these are the only things that actually "exist" at the global
level unlike type definitions which are merely aliases.

OMISSIONS:

unsized types (i.e. slices)

closures (so i think i don't need frame typings.
    idk thi might be relevant just for functions.
    though initially i think i can just work with global code)

binary sums (i.e Either<...>)

branching

abstract regions.  letrgns are concrete

PARSER CHANGES:
treat everything as an expression


Plan and pseudocode:

figure out what the structure of a datatype should be.

figure out what typing rules need to be implemented first and then how will that be translated into code.
how directly can these rules be translated to code?  Should I write a higher order function? (probably not)


enum SizedType {
    base type,
    type var,
    reference, // this, for example would be a struct type to contain this type information.  many of these variants can be struct types.
    etc..
}

enum TypeVariant {
    SizedType,
    etc..
}

struct ConcreteRegion {
    str name,
    enum {
        Vec<Binding> loans,
        owned value
    }
}

struct Binding {
    PlaceExpr,
    ConcreteRegion&
}

struct Type {
    TypeVariant,
    Region& region
}

// different envs (abstract, stack, etc.) could be different instances of this type
// naw uh cna have differnt types bc uh the different envs hold diff data

// I think the stack and continuation typings will be structurally identical but semantically distinct.  I.e. they will be stored
// in separate variables.
struct AbstractTyping {
    // I think these would be "kinds"
    Vec<AbstractTypes>,
    Vec<AbstractRegions>,
    Vec<Frames>
}

struct StackTyping {
    Vec<ConcreteTypes>,
    Vec<ConcreteRegions>,
    Vec<Frames> // ?
}


// for now i think i can start with the rule for chekcing the innner expression of a function (that is, the global code) , then gofrom there.
// assuming expr is file scope
fn WFGlobalEnv(Expr e) {
    match e.firstChild() {
        Start here at the global level, go through each child in the ast and  run the type checking rules, the type rules should basically be
        mutually recursive functions.
    }
}

// Ok i think the continuation typing can be local to each function and then the fn will return the updated stack typing based on the tmp one.
// So then the stack typing is not global state but is a snapshot passed into each function

pseudocode for:
    struct Obj(u32, u32);
    letrgn<'y, 'z> {
        let mut x = Obj(5, 4);
        let y = &'y uniq x;
        let z = &'z uniq *y;
    }


fn t_letrgn(exp, stack_typing) {
    add named empty regions to the stack typing
    let continuation_typing = [];
    match inner_exp {
        case seq:
            return t_seq(inner_exp, continuation_typing);
        ...
    }
}

fn t_seq(exp, stack_typing) {
    let continuation_typing = [];

    match e1 {
        case let:
            st1 = t_let(e1, stack_typing);
        ...
    }
    match e2 {
        case seq:
            return t_seq(e2, gc_loans(t1));
        ...
    }
}


*/
use crate::ast::{Expression, OwnershipQualifier, PlaceExpression};

struct Type<'a> {
    variant: TypeVariant<'a>,
    region: &'a Region,
    is_sized: Ternary,
    is_dead: Ternary,
    copyable: bool,
}

enum BaseType {
    Bool,
    U32,
    Unit,
}

struct ReferenceType<'a> {
    // region_name: String,
    ownership_qualifier: OwnershipQualifier,
    target_type: Box<Type<'a>>,
}

enum TypeVariant<'a> {
    Base(BaseType),
    Reference(ReferenceType<'a>),
    Tuple(Vec<Box<Type<'a>>>),
}

enum RegionVariant {
    Loans(Vec<PlaceExpression>),
    Owned(PlaceExpression),
}

struct ConcreteRegion {
    name: String,
    variant: RegionVariant,
}

enum RegionLevel {
    ConcreteRegion(ConcreteRegion),
    // AbstractRegion
}

struct Region {
    name: Option<String>,
    region: RegionLevel,
}

enum Ternary {
    Yes,
    No,
    Maybe,
}

struct ConcreteTyping<'a> {
    types: Vec<Type<'a>>,
    regions: Vec<Region>,
}

impl Expression {
    // type rules here
}
