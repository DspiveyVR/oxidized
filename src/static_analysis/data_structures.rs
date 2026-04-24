use crate::ast::{BaseTypeAnnotation, Expression, OwnershipQualifier, TypeAnnotation};
use serde::Serialize;
use std::{fmt::Debug, vec};
// use color_print;
use thiserror::Error;

// Types
#[derive(Serialize, Debug, Clone)]
pub(crate) struct ConcreteTyping {
    pub(crate) regions: VecSet<ConcreteRegion>,
    pub(crate) types: VecSet<Type>,
}

#[derive(Serialize, Debug, Clone)]
pub(crate) struct Type {
    pub(crate) binding: Option<Binding>,
    pub(crate) variant: TypeVariant,
    pub(crate) is_sized: Ternary,
    pub(crate) is_dead: Ternary,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub(crate) enum PlaceExpression {
    Deref(Box<PlaceExpression>),
    PathAccess(Box<PlaceExpression>, Vec<u32>),
    Variable(String),
}

#[derive(Serialize, Debug, Clone)]
pub(crate) struct Binding {
    pub(crate) variable: String,
    pub(crate) is_mut: bool,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub(crate) struct Place {
    pub(crate) variable: String,
    pub(crate) path: Option<Vec<u32>>,
}

#[derive(Serialize, Debug, Clone)]
pub(crate) enum TypeVariant {
    Base(BaseType),
    Reference(ReferenceType),
    Tuple(Vec<Box<Type>>),
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub(crate) enum BaseType {
    Bool,
    U32,
    Unit,
}

#[derive(Serialize, Debug, Clone)]
pub(crate) struct ReferenceType {
    pub(crate) region_name: String,
    pub(crate) ownership_qualifier: OwnershipQualifier,
    pub(crate) target_type: Box<Type>,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub(crate) enum Ternary {
    Yes,
    No,
    // Maybe,
}

#[derive(Serialize, Debug, Clone)]
pub(crate) struct Region(pub(crate) RegionLevel);

#[derive(Serialize, Debug, Clone)]
pub(crate) enum RegionLevel {
    ConcreteRegion(ConcreteRegion),
    // AbstractRegion
}

#[derive(Serialize, Debug, Clone)]
pub(crate) struct ConcreteRegion {
    pub(crate) name: String,
    pub(crate) loans: VecSet<Loan>,
    pub(crate) scope: u32,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub(crate) struct Loan {
    pub(crate) ownership_qualifier: OwnershipQualifier,
    pub(crate) place_expression: PlaceExpression,
}

#[derive(Serialize, Debug, Clone)]
pub(crate) enum RegionRewriteMode {
    Plus,
    // Equals,
}

#[derive(Serialize, Debug, Clone)]
pub(crate) struct VecSet<T>(pub(crate) Vec<T>);

#[derive(Debug, Clone, Error)]
pub(crate) enum OxidizedError {
    #[error("A region with name {0} already exists")]
    RegionNameConflict(String),
    #[error("Type {0} is dead")]
    UseOfMovedValue(Type),
    #[error("Region {0} must outlive region {1}")]
    ConcreteOutlivesError(ConcreteRegion, ConcreteRegion),
    #[error("Cannot borrow with nonempty region '{0} containing loans: {1}")]
    NonemptyRegion(String, VecSet<Loan>),
    #[error("Cannot borrow with nonexistent region '{0}")]
    NonexistentRegion(String),
    #[error(
        "Cannot borrow {place} {ownership_qualifier}ly while a {ownership_qualifier2} reference to {place_expression2} exists"
    )]
    UnsafePlace {
        place: Place,
        ownership_qualifier: String,
        place_expression2: PlaceExpression,
        ownership_qualifier2: String,
    },
    #[error("{0} is not a valid place expression")]
    InvalidPlaceExpression(Expression),
    #[error("{0} was not found in the stack typing")]
    UseOfUndeclared(String),
    #[error("Expected {expected} found {found} ")]
    MismatchedTypes { expected: Type, found: Type },
    #[error("{0:?} cannot be dereferenced")]
    CannotDereference(Type),
    #[error("{0:?} is not a valid path in {1}")]
    InvalidPathAccess(Vec<u32>, Type),
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
    pub(crate) fn type_lookup(
        &self,
        place_expression: PlaceExpression,
    ) -> Result<Type, OxidizedError> {
        let var = place_expression.get_inner_variable();
        let found_type = self
            .types
            .0
            .iter()
            .find(|t| t.binding.clone().unwrap().variable == var);
        if let Some(found_type) = found_type {
            found_type.walk(place_expression)
        } else {
            return Err(OxidizedError::UseOfUndeclared(var));
        }
    }

    pub(crate) fn add_region(&self, new_region: ConcreteRegion) -> Self {
        ConcreteTyping {
            regions: VecSet([&self.regions.0[..], &vec![new_region][..]].concat()),
            types: self.types.clone(),
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

    pub(crate) fn get_region(&self, name: String) -> Result<ConcreteRegion, OxidizedError> {
        if let Some(region) = self.regions.0.clone().into_iter().find(|r| r.name == name) {
            Ok(region)
        } else {
            Err(OxidizedError::NonexistentRegion(name))
        }
    }

    pub(crate) fn set_region(&self, name: String, new_region: ConcreteRegion) -> Self {
        let mut new_env = self.clone();
        if let Some(r) = new_env.regions.0.iter_mut().find(|r| r.name == name) {
            *r = new_region;
            new_env
        } else {
            new_env
        }
    }

    pub(crate) fn new_with_regions(region_names: &Vec<String>, scope: u32) -> Self {
        ConcreteTyping {
            types: VecSet::new(),
            regions: VecSet(
                region_names
                    .iter()
                    .map(|r| ConcreteRegion::new(r.to_string(), scope))
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

    // these two funcitons are highly questionable .  so is the fact that i consistently misspell the word function
    pub(crate) fn explode(&self) -> VecSet<(Place, Type)> {
        let binding_pairs = VecSet(
            self.types
                .0
                .iter()
                .map(|t| {
                    (
                        Place {
                            variable: t.binding.clone().unwrap().variable,
                            path: None,
                        },
                        t.clone(),
                    )
                })
                .collect(),
        );

        let all_concrete_regions: Vec<ConcreteRegion> =
            self.regions.0.iter().map(|r| r.clone()).collect();
        let all_place_expressions: Vec<PlaceExpression> = all_concrete_regions
            .iter()
            .map(|r| r.loans.clone())
            .reduce(|acc_loans, current_loans| acc_loans.union(current_loans))
            .unwrap()
            .0
            .iter()
            .map(|l| l.place_expression.clone())
            .collect();
        let all_places: Vec<Place> = all_place_expressions
            .iter()
            .filter_map(|pe| {
                if let Ok(place) = TryInto::<Place>::try_into((*pe).clone()) {
                    Some(place)
                } else {
                    None
                }
            })
            .collect();
        let place_pairs = VecSet(
            all_places
                .iter()
                .map(|p| {
                    (
                        p.clone(),
                        self.type_lookup((*p).clone().into())
                            .expect("Unexpected error"),
                    )
                })
                .collect(),
        );

        let all_pairs = binding_pairs.union(place_pairs);

        let mut result = VecSet::new();

        for pair in all_pairs.0 {
            result = result.union(Self::explode_pair(pair.0, pair.1));
        }

        result
    }

    fn explode_pair(place: Place, associated_type: Type) -> VecSet<(Place, Type)> {
        match associated_type.variant {
            TypeVariant::Base(_) | TypeVariant::Reference(_) => {
                VecSet(vec![(place.clone(), associated_type)])
            }
            TypeVariant::Tuple(types) => {
                let mut exploded = VecSet::new();
                for (i, type_info) in types.iter().enumerate() {
                    let new_place = Place {
                        variable: (&place).clone().variable,
                        path: Some(
                            [
                                &(&place).clone().path.unwrap_or(Vec::new())[..],
                                &vec![i as u32][..],
                            ]
                            .concat(),
                        ),
                    };
                    exploded = exploded.union(Self::explode_pair(new_place, (**type_info).clone()))
                }

                exploded
            }
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
            && (self.copyable() == other.copyable())
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
        }
    }
}

impl Type {
    pub(crate) fn copyable(&self) -> bool {
        match &self.variant {
            TypeVariant::Base(_) => true,
            TypeVariant::Reference(reference_type) => {
                if reference_type.ownership_qualifier == OwnershipQualifier::Shrd {
                    true
                } else {
                    false
                }
            }
            TypeVariant::Tuple(types) => types.iter().all(|typ| typ.copyable()),
        }
    }

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
                        .all(|tup| tup.0.eq_unchecked_regions(tup.1))
                } else {
                    false
                }
            }
        };

        variant_equal
            && (self.is_sized == other.is_sized)
            && (self.is_dead == other.is_dead)
            && (self.copyable() == other.copyable())
    }

    // dun dunununun RE! dun dunununun SPECT! dun dunununun WALK! dun dun dun dununununununununun
    fn walk(&self, place_expression: PlaceExpression) -> Result<Self, OxidizedError> {
        match place_expression {
            PlaceExpression::Deref(inner_pe) => {
                let inner_type = self.walk((*inner_pe).clone())?;
                if let TypeVariant::Reference(ref_type) = inner_type.variant.clone() {
                    Ok(*ref_type.target_type)
                } else {
                    Err(OxidizedError::CannotDereference(inner_type.clone()))
                }
            }
            PlaceExpression::PathAccess(inner_pe, path) => {
                let accessed_type = self.walk(*inner_pe)?;
                accessed_type.walk_tuple(&path[..])
            }
            PlaceExpression::Variable(_) => Ok(self.clone()),
        }
    }

    fn walk_tuple(&self, path: &[u32]) -> Result<Self, OxidizedError> {
        if let TypeVariant::Tuple(types) = self.variant.clone() {
            let accessed_type = types.get(path[0] as usize);
            if let Some(accessed_type) = accessed_type {
                let new_path = &path[1..];
                if new_path.len() > 0 {
                    accessed_type.walk_tuple(new_path)
                } else {
                    Ok((**accessed_type).clone())
                }
            } else {
                Err(OxidizedError::InvalidPathAccess(
                    path.to_vec(),
                    self.clone(),
                ))
            }
        } else {
            Err(OxidizedError::InvalidPathAccess(
                path.to_vec(),
                self.clone(),
            ))
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

impl PlaceExpression {
    fn get_inner_variable(&self) -> String {
        match self {
            PlaceExpression::Deref(place_expression) => place_expression.get_inner_variable(),
            PlaceExpression::PathAccess(place_expression, _) => {
                place_expression.get_inner_variable()
            }
            PlaceExpression::Variable(var) => var.to_string(),
        }
    }
}

impl PartialEq for ConcreteRegion {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for ConcreteRegion {}

impl Region {
    pub(crate) fn get_concrete(&self) -> &ConcreteRegion {
        let RegionLevel::ConcreteRegion(cr) = &self.0;
        cr
    }

    pub(crate) fn new_concrete(name: String, scope: u32) -> Self {
        Region(RegionLevel::ConcreteRegion(ConcreteRegion::new(
            name, scope,
        )))
    }
}

impl ConcreteRegion {
    pub(crate) fn new(name: String, scope: u32) -> Self {
        ConcreteRegion {
            name: name,
            loans: VecSet::new(),
            scope,
        }
    }

    pub(crate) fn set_loans(&self, loans: VecSet<Loan>) -> ConcreteRegion {
        let mut new_region = self.clone();
        new_region.loans = loans;
        new_region
    }
}

impl PartialEq for Binding {
    fn eq(&self, other: &Self) -> bool {
        self.variable == other.variable
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

        VecSet([&diff[..], &other.0[..]].concat())
    }
}
