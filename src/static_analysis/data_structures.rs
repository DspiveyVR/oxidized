use crate::ast::{BaseTypeAnnotation, Expression, OwnershipQualifier, TypeAnnotation};
use std::{fmt::Debug, vec};
// use color_print;
use thiserror::Error;

// Types
#[derive(Debug, Clone)]
pub(crate) struct ConcreteTyping {
    pub(crate) regions: VecSet<Region>, // FIXME: this will likely have to be double vecs to represent scopes since strictly speaking regions in the same letrgn block will have different scopes in my code.
    pub(crate) types: VecSet<Type>,
}

#[derive(Debug, Clone)]
pub(crate) struct Type {
    // TODO: needs a 'mutable' bool
    pub(crate) binding: Option<Binding>,
    pub(crate) variant: TypeVariant,
    pub(crate) is_sized: Ternary,
    pub(crate) is_dead: Ternary,
    pub(crate) copyable: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum PlaceExpression {
    Deref(Box<PlaceExpression>),
    PathAccess(Box<PlaceExpression>, Vec<u32>),
    Variable(String),
}

#[derive(Debug, Clone)]
pub(crate) struct Binding {
    pub(crate) place_expression: PlaceExpression,
    pub(crate) is_mut: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Place {
    pub(crate) variable: String,
    pub(crate) path: Option<Vec<u32>>,
}

#[derive(Debug, Clone)]
pub(crate) enum TypeVariant {
    Base(BaseType),
    Reference(ReferenceType),
    Tuple(Vec<Box<Type>>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum BaseType {
    Bool,
    U32,
    Unit,
}

#[derive(Debug, Clone)]
pub(crate) struct ReferenceType {
    pub(crate) region_name: String,
    pub(crate) ownership_qualifier: OwnershipQualifier,
    pub(crate) target_type: Box<Type>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Ternary {
    Yes,
    No,
    Maybe,
}

#[derive(Debug, Clone)]
pub(crate) struct Region(pub(crate) RegionLevel);

#[derive(Debug, Clone)]
pub(crate) enum RegionLevel {
    ConcreteRegion(ConcreteRegion),
    // AbstractRegion
}

#[derive(Debug, Clone)]
pub(crate) struct ConcreteRegion {
    pub(crate) name: String,
    pub(crate) loans: VecSet<Loan>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Loan {
    pub(crate) ownership_qualifier: OwnershipQualifier,
    pub(crate) place_expression: PlaceExpression,
}

#[derive(Debug, Clone)]
pub(crate) enum RegionRewriteMode {
    Plus,
    Equals,
}

#[derive(Debug, Clone)]
pub(crate) struct VecSet<T>(pub(crate) Vec<T>);

#[derive(Debug, Clone, Error)]
pub(crate) enum OxidizedError {
    #[error("A region with name {0} already exists")]
    RegionNameConflict(String),
    #[error("Type {0:?} is dead")]
    UseOfMovedValue(Type),
    #[error("Region {0:?} must outlive region {1:?}")]
    ConcreteOutlivesError(ConcreteRegion, ConcreteRegion),
    #[error("Cannot borrow with nonempty region '{0} containing loans: {1:?}")]
    NonemptyRegion(String, VecSet<Loan>),
    #[error("Cannot borrow with nonexistent region '{0}")]
    NonexistentRegion(String),
    #[error(
        "Cannot borrow {place:?} {ownership_qualifier}ly while a {ownership_qualifier2} reference to {place_expression2:?} exists"
    )]
    UnsafePlace {
        place: Place,
        ownership_qualifier: String,
        place_expression2: PlaceExpression,
        ownership_qualifier2: String,
    },
    #[error("{0} is not a valid place expression")]
    InvalidPlaceExpression(Expression),
    #[error("{0:?} was not found in the stack typing")]
    UseOfUndeclaredBinding(Binding),
    #[error("Expected {expected:?}, found {found:?} ")]
    MismatchedTypes { expected: Type, found: Type },
}

// Traits
pub(crate) trait Set<T>
where
    T: Eq + Clone,
{
    fn new() -> Self;
    fn union(&self, other: Self) -> Self;
}

pub(crate) trait TypeEnv {
    fn union(&self, other: Self) -> Self;
}

// Impls
impl TypeEnv for ConcreteTyping {
    fn union(&self, other: Self) -> Self {
        Self {
            types: self.types.union(other.types),
            regions: self.regions.union(other.regions),
        }
    }
}

impl ConcreteTyping {
    pub(crate) fn get_binding_type(&self, binding: Binding) -> Result<Type, OxidizedError> {
        if let Some(found_type) = self.types.0.iter().find(|t| {
            if let Some(matched_binding) = t.binding.clone() {
                matched_binding == binding
            } else {
                false
            }
        }) {
            Ok(found_type.clone())
        } else {
            Err(OxidizedError::UseOfUndeclaredBinding(binding))
        }
    }

    pub(crate) fn add_type(&self, new_type: Type) -> Self {
        ConcreteTyping {
            regions: self.regions.clone(),
            types: VecSet([&self.types.0[..], &vec![new_type][..]].concat()),
        }
    }

    pub(crate) fn set_type(&self, t1: Type, t2: Type) -> Self {
        let mut new_env = self.clone();
        if let Some(t) = new_env.types.0.iter_mut().find(|t| t == &&t1) {
            *t = t2;
            new_env
        } else {
            new_env
        }
    }

    pub(crate) fn get_region(&self, name: String) -> Result<Region, OxidizedError> {
        if let Some(region) = self
            .regions
            .0
            .clone()
            .into_iter()
            .find(|r| r.get_concrete().name == name)
        {
            Ok(region)
        } else {
            Err(OxidizedError::NonexistentRegion(name))
        }
    }

    pub(crate) fn set_region(&self, name: String, new_region: Region) -> Self {
        let mut new_env = self.clone();
        if let Some(r) = new_env
            .regions
            .0
            .iter_mut()
            .find(|r| r.get_concrete().name == name)
        {
            *r = new_region;
            new_env
        } else {
            new_env
        }
    }

    pub(crate) fn new_with_regions(region_names: &Vec<String>) -> Self {
        ConcreteTyping {
            types: VecSet::new(),
            regions: VecSet(
                region_names
                    .iter()
                    .map(|r| Region::new_concrete(r.to_string()))
                    .collect(),
            ),
        }
    }

    pub(crate) fn new() -> Self {
        ConcreteTyping {
            types: VecSet::new(),
            regions: VecSet::new(),
        }
    }

    // is this relevant if i'm just looking for the regions?  maybe i don't need to go this deeply nested since they're all present in the stack typing.
    pub(crate) fn explode(&self) -> VecSet<Type> {
        let mut types = VecSet::new();
        let self_types = self.types.0.clone();
        for t in self_types {
            types = types.union(t.explode());
        }
        types
    }
}

impl Type {
    // Same as partial eq impl but doesn't check region names.  This is useful when I need to check type equality before region rewriting.
    pub(crate) fn eq_unchecked_regions(&self, other: &Self) -> bool {
        let variant_equal = match &self.variant {
            TypeVariant::Base(base_type) => {
                if let TypeVariant::Base(other_type) = &other.variant {
                    base_type == other_type
                } else {
                    false
                }
            }
            TypeVariant::Reference(reference_type) => {
                if let TypeVariant::Reference(other_type) = &other.variant {
                    reference_type.ownership_qualifier == other_type.ownership_qualifier
                        && reference_type
                            .target_type
                            .eq_unchecked_regions(&*other_type.target_type)
                } else {
                    false
                }
            }
            TypeVariant::Tuple(items) => {
                if let TypeVariant::Tuple(other_type) = &other.variant {
                    items
                        .iter()
                        .zip(other_type.iter())
                        .all(|tup| tup.0 == tup.1)
                } else {
                    false
                }
            }
        };

        variant_equal
            && (self.is_sized == other.is_sized)
            && (self.is_dead == other.is_dead)
            && (self.copyable == other.copyable)
    }

    pub(crate) fn explode(&self) -> VecSet<Type> {
        match self.variant.clone() {
            TypeVariant::Tuple(inner_types) => {
                let mut new_types = VecSet::new();
                for t in inner_types {
                    new_types = new_types.union(t.explode());
                }
                new_types
            }
            _ => VecSet(vec![self.clone()]),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        let variant_equal = match &self.variant {
            TypeVariant::Base(base_type) => {
                if let TypeVariant::Base(other_type) = &other.variant {
                    base_type == other_type
                } else {
                    false
                }
            }
            TypeVariant::Reference(reference_type) => {
                if let TypeVariant::Reference(other_type) = &other.variant {
                    reference_type.ownership_qualifier == other_type.ownership_qualifier
                        && reference_type.region_name == other_type.region_name
                        && reference_type.target_type == other_type.target_type
                } else {
                    false
                }
            }
            TypeVariant::Tuple(items) => {
                if let TypeVariant::Tuple(other_type) = &other.variant {
                    items
                        .iter()
                        .zip(other_type.iter())
                        .all(|tup| tup.0 == tup.1)
                } else {
                    false
                }
            }
        };

        variant_equal
            && (self.is_sized == other.is_sized)
            && (self.is_dead == other.is_dead)
            && (self.copyable == other.copyable)
    }
}
impl Eq for Type {}

impl From<TypeAnnotation> for Type {
    fn from(value: TypeAnnotation) -> Self {
        Self {
            binding: None,
            variant: match value {
                TypeAnnotation::Base(base_type_annotation) => {
                    TypeVariant::Base(match base_type_annotation {
                        BaseTypeAnnotation::Bool => BaseType::Bool,
                        BaseTypeAnnotation::U32 => BaseType::U32,
                        BaseTypeAnnotation::Unit => BaseType::Unit,
                    })
                }
                TypeAnnotation::Reference(reference_annotation, type_annotation) => {
                    TypeVariant::Reference(ReferenceType {
                        region_name: reference_annotation.0,
                        ownership_qualifier: reference_annotation.1,
                        target_type: Box::new((*type_annotation).into()),
                    })
                }
                TypeAnnotation::Tuple(type_annotations) => TypeVariant::Tuple(
                    type_annotations
                        .into_iter()
                        .map(|ta| Box::new((*ta).into()))
                        .collect(),
                ),
            },
            is_sized: Ternary::Yes,
            is_dead: Ternary::No,
            copyable: false, // TODO: actually implement copyable logic
        }
    }
}

impl TryFrom<Expression> for PlaceExpression {
    type Error = OxidizedError;

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
            _ => Err(OxidizedError::InvalidPlaceExpression(value)),
        }
    }
}

impl TryFrom<PlaceExpression> for Place {
    type Error = String;

    fn try_from(place_expression: PlaceExpression) -> Result<Self, Self::Error> {
        match place_expression {
            PlaceExpression::PathAccess(inner_pe, path) => {
                if let PlaceExpression::Variable(variable) = *inner_pe {
                    Ok(Place {
                        variable,
                        path: Some(path),
                    })
                } else {
                    Err("Not a valid place".to_string())
                }
            }
            PlaceExpression::Variable(variable) => Ok(Place {
                variable,
                path: None,
            }),
            _ => Err("Not a valid place".to_string()),
        }
    }
}

impl From<Place> for PlaceExpression {
    fn from(place: Place) -> Self {
        if let Some(path) = place.path {
            PlaceExpression::PathAccess(Box::new(PlaceExpression::Variable(place.variable)), path)
        } else {
            PlaceExpression::Variable(place.variable)
        }
    }
}

impl From<PlaceExpression> for Binding {
    fn from(place_expression: PlaceExpression) -> Self {
        Binding {
            place_expression,
            is_mut: false, /* False is just assumed cause I think its irrelevant with how I'm using this method */
        }
    }
}

impl PartialEq for Region {
    fn eq(&self, other: &Self) -> bool {
        self.get_concrete().name == other.get_concrete().name
    }
}
impl Eq for Region {}

impl Region {
    pub(crate) fn get_concrete(&self) -> &ConcreteRegion {
        let RegionLevel::ConcreteRegion(cr) = &self.0;
        cr
    }

    pub(crate) fn new_concrete(name: String) -> Self {
        Region(RegionLevel::ConcreteRegion(ConcreteRegion {
            name: name,
            loans: VecSet::new(),
        }))
    }
}

impl ConcreteRegion {
    pub(crate) fn set_loans(&self, loans: VecSet<Loan>) -> Region {
        let mut new_region = self.clone();
        new_region.loans = loans;
        Region(RegionLevel::ConcreteRegion(new_region))
    }
}

impl PartialEq for Binding {
    fn eq(&self, other: &Self) -> bool {
        self.place_expression == other.place_expression
    }
}
impl Eq for Binding {}

impl<T> Set<T> for VecSet<T>
where
    T: Eq + Clone + Debug,
{
    fn new() -> Self {
        VecSet(Vec::new())
    }
    fn union(&self, other: Self) -> Self {
        let mut diff = Vec::new();

        for e in &self.0 {
            if !other.0.contains(&e) {
                diff.push(e.clone());
            }
        }

        VecSet([&other.0[..], &diff[..]].concat())
    }
}
