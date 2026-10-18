use crate::{
    ast::{BaseTypeAnnotation, Expression, OwnershipQualifier, TypeAnnotation},
    static_analysis::{auxiliary::err_if_dead, data_structures::TypeComputation::DerefChain},
};
use serde::Serialize;
use std::{fmt::Debug, ops::Index, panic, slice::SliceIndex, todo, vec};
// use color_print;
use thiserror::Error;

// Types
#[derive(Serialize, Debug, Clone)]
pub struct ConcreteTyping {
    pub regions: VecSet<ConcreteRegion>,
    pub types: VecSet<Type>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Type {
    pub binding: Option<Binding>,
    pub variant: TypeVariant,
    pub is_sized: Ternary,
    pub is_dead: Ternary,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub enum PlaceExpression {
    Deref(Box<PlaceExpression>),
    PathAccess(Box<PlaceExpression>, Vec<u32>),
    Place(Place),
}

#[derive(Serialize, Debug, Clone)]
pub struct Binding {
    pub variable: String,
    pub is_mut: bool,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Place {
    pub variable: String,
    pub path: Vec<u32>,
}

#[derive(Serialize, Debug, Clone)]
pub enum TypeVariant {
    Base(BaseType),
    Reference(ReferenceType),
    Tuple(Vec<Box<Type>>),
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub enum BaseType {
    Bool,
    U32,
    Unit,
}

#[derive(Serialize, Debug, Clone)]
pub struct ReferenceType {
    pub region_name: String,
    pub ownership_qualifier: OwnershipQualifier,
    pub target_type: Box<Type>,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub enum Ternary {
    Yes,
    No,
    // Maybe,
}

#[derive(Serialize, Debug, Clone)]
pub struct Region(pub RegionLevel);

#[derive(Serialize, Debug, Clone)]
pub enum RegionLevel {
    ConcreteRegion(ConcreteRegion),
    // AbstractRegion
}

// TODO: Maybe regions should also be sets. same with type envs
#[derive(Serialize, Debug, Clone)]
pub struct ConcreteRegion {
    pub name: String,
    pub loans: VecSet<Loan>,
    pub scope: u32,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Loan {
    pub ownership_qualifier: OwnershipQualifier,
    pub place: Place,
    pub is_reborrow: bool,
}

#[derive(Serialize, Debug, Clone)]
pub enum RegionRewriteMode {
    Plus,
    Equals,
}

#[derive(Serialize, Debug, Clone)]
pub struct VecSet<T>(Vec<T>);

#[derive(Debug, Clone, Error)]
pub enum OxidizedError {
    #[error("'{0} must be unique to {1} in {2}")]
    RegionMustBeUnique(String, Place, ConcreteTyping),
    #[error("Cannot create a let binding to a type containing reborrows in its free regions: {0}")]
    CannotLetBindReborrowedRegions(Type),
    #[error("A region with name {0} already exists")]
    RegionNameConflict(String),
    #[error("Use of moved value with type: {0}")]
    UseOfMovedValue(Type),
    // #[error("Cannot move out of {0} while it is behind a shared reference")]
    // CannotMove(Type),
    #[error("Cannot use {0} in a uniq context")]
    ViolatedOwnershipOrdering(ReferenceType),
    #[error("Region {0} must outlive region {1}")]
    ConcreteOutlivesError(ConcreteRegion, ConcreteRegion),
    #[error("Cannot borrow with nonempty region '{0} containing loans: {1}")]
    NonemptyRegion(String, VecSet<Loan>),
    #[error("Cannot borrow with nonexistent region '{0}")]
    NonexistentRegion(String),
    #[error("{0} must outlive region '{1}")]
    LifetimeError(Type, String),
    #[error(
        "Cannot use {place} {ownership_qualifier}ly while a {ownership_qualifier2} reference to {place_expression2} exists"
    )]
    UnsafePlace {
        place: Place,
        ownership_qualifier: String,
        place_expression2: PlaceExpression,
        ownership_qualifier2: String,
    },
    #[error("Cannot mutate immutable variable {0}")]
    NoMutAnnotation(String),
    #[error("{0} is not a valid place expression")]
    InvalidPlaceExpression(Expression),
    #[error("{0} is not a valid place")]
    InvalidPlaceFromExp(Expression),
    #[error("{0} is not a valid place")]
    InvalidPlace(PlaceExpression),
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
pub trait Set<T>
where
    T: Eq + Clone,
{
    fn union(&self, other: Self) -> Self;
}

// FIXME: This should be the set trait stoopid
pub trait TypeEnv {
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

#[derive(Clone)]
pub struct TypeComputationEntry {
    pub typ: Type,
    pub place: Place,
    pub is_deref: bool,
}

pub enum TypeComputation {
    DerefChain(Vec<TypeComputationEntry>),
    Place(TypeComputationEntry),
}

impl TypeComputation {
    fn push(&mut self, entry: TypeComputationEntry) {
        if let DerefChain(vec) = self {
            vec.push(entry);
        } else {
            panic!("Not a deref chain");
        }
    }
}

// impl IntoIterator for TypeComputation {
//     type Item = TypeComputationEntry;

//     type IntoIter = std::vec::IntoIter<Self::Item>;

//     fn into_iter(self) -> Self::IntoIter {
//         if let DerefChain(vec) = self {
//             vec.into_iter()
//         } else {
//             panic!("Not a deref chain");
//         }
//     }
// }

impl ConcreteTyping {
    pub fn lookup_type(&self, place: Place) -> Result<Type, OxidizedError> {
        if let Some(typ) = self
            .types
            .0
            .iter()
            .find(|t| t.binding.clone().unwrap().variable == place.variable)
        {
            // I thikn these err calls might be redundant anyway
            // let typ = err_if_dead(typ.clone())?;
            Ok(typ.clone())
        } else {
            Err(OxidizedError::UseOfUndeclared(place.variable))
        }
    }

    /*

    let a = 5;
    let b = &a;
    let c = &b;

    comp(**c) {
        comp(*c) {
            comp(c) {
                TC { [c], [&&int] }
            }

            target == (b, &int)


            TC { [c, b], [&&int, &int]}
        }

        target == (a, int)

        TC { [c, b, a], [&&int, &int, int]}
    }



    *b;
    a;


     */

    pub fn compute_type(
        &self,
        place_expression: PlaceExpression,
        ownership_qualifier: OwnershipQualifier,
    ) -> Result<TypeComputation, OxidizedError> {
        self._compute_type(place_expression, ownership_qualifier, true)
    }

    fn _compute_type(
        &self,
        place_expression: PlaceExpression,
        ownership_qualifier: OwnershipQualifier,
        do_ord_check: bool,
    ) -> Result<TypeComputation, OxidizedError> {
        match place_expression {
            PlaceExpression::Deref(inner_pe) => {
                let tc = TypeComputation::DerefChain(vec![]);
                let (mut tc, (type_val, place)) =
                    self.__compute_type(tc, *inner_pe, ownership_qualifier, do_ord_check)?;

                tc.push(TypeComputationEntry {
                    typ: type_val,
                    place,
                    is_deref: false,
                });

                Ok(tc)
            }
            PlaceExpression::Place(place) => Ok(TypeComputation::Place(TypeComputationEntry {
                typ: self.lookup_type(place.clone())?,
                place,
                is_deref: false,
            })),
            PlaceExpression::PathAccess(..) => todo!(),
        }
    }

    fn __compute_type(
        &self,
        mut tc: TypeComputation,
        place_expression: PlaceExpression,
        ownership_qualifier: OwnershipQualifier,
        do_ord_check: bool,
    ) -> Result<(TypeComputation, (Type, Place)), OxidizedError> {
        match place_expression {
            PlaceExpression::Place(place) => {
                let typ = self.lookup_type(place.clone())?;
                tc.push(TypeComputationEntry {
                    typ: typ.clone(),
                    place,
                    is_deref: true,
                });

                if let TypeVariant::Reference(ref_type) = typ.variant {
                    let target = err_if_dead(*ref_type.target_type.clone())?;

                    if do_ord_check && ownership_qualifier > ref_type.ownership_qualifier {
                        return Err(OxidizedError::ViolatedOwnershipOrdering(ref_type));
                    }

                    let referenced_place = self
                        .get_region(ref_type.region_name)?
                        .get_referenced_place();

                    Ok((tc, (target, referenced_place)))
                } else {
                    panic!("Unexpected match")
                }
            }
            PlaceExpression::Deref(place_expression) => {
                let (mut tc, (prev_target, prev_referenced_place)) =
                    self.__compute_type(tc, *place_expression, ownership_qualifier, do_ord_check)?;

                tc.push(TypeComputationEntry {
                    typ: prev_target.clone(),
                    place: prev_referenced_place,
                    is_deref: true,
                });

                if let TypeVariant::Reference(ref_type) = prev_target.variant {
                    let target = err_if_dead(*ref_type.target_type.clone())?;

                    if do_ord_check && ownership_qualifier > ref_type.ownership_qualifier {
                        return Err(OxidizedError::ViolatedOwnershipOrdering(ref_type));
                    }

                    let referenced_place = self
                        .get_region(ref_type.region_name)?
                        .get_referenced_place();

                    Ok((tc, (target, referenced_place)))
                } else {
                    panic!("Unexpected match")
                }
            }
            PlaceExpression::PathAccess(place_expression, items) => todo!(),
        }
    }

    pub fn flat_compute_type_no_ord(
        &self,
        place_expression: PlaceExpression,
    ) -> Result<Type, OxidizedError> {
        match self._compute_type(
            place_expression,
            OwnershipQualifier::Shrd, /* Dummy value */
            false,
        )? {
            DerefChain(entries) => Ok(entries.last().unwrap().typ.clone()),
            TypeComputation::Place(type_computation_entry) => Ok(type_computation_entry.typ),
        }
    }

    pub fn flat_compute_type(
        &self,
        place_expression: PlaceExpression,
        ownership_qualifier: OwnershipQualifier,
    ) -> Result<Type, OxidizedError> {
        match self.compute_type(place_expression, ownership_qualifier)? {
            DerefChain(entries) => Ok(entries.last().unwrap().typ.clone()),
            TypeComputation::Place(type_computation_entry) => Ok(type_computation_entry.typ),
        }
    }

    pub fn add_region(&self, new_region: ConcreteRegion) -> Self {
        ConcreteTyping {
            regions: VecSet([&self.regions.0[..], &vec![new_region][..]].concat()),
            types: self.types.clone(),
        }
    }

    pub fn add_type(&self, new_type: Type) -> Self {
        ConcreteTyping {
            regions: self.regions.clone(),
            types: VecSet([&self.types.0[..], &vec![new_type][..]].concat()),
        }
    }

    pub fn set_type(&self, t1: Type, t2: Type) -> Self {
        let mut new_env = self.clone();
        if let Some(t) = new_env
            .types
            .0
            .iter_mut()
            .find(|t| t == &&t1 && t.binding == t1.binding)
        {
            *t = t2;
            new_env
        } else {
            new_env
        }
    }

    pub fn get_region(&self, name: String) -> Result<ConcreteRegion, OxidizedError> {
        if let Some(region) = self.regions.0.clone().into_iter().find(|r| r.name == name) {
            Ok(region)
        } else {
            Err(OxidizedError::NonexistentRegion(name))
        }
    }

    pub fn set_region(&self, name: String, new_region: ConcreteRegion) -> Self {
        let mut new_env = self.clone();
        if let Some(r) = new_env.regions.0.iter_mut().find(|r| r.name == name) {
            *r = new_region;
            new_env
        } else {
            new_env
        }
    }

    pub fn new_with_regions(region_names: &Vec<String>, scope: u32) -> Self {
        ConcreteTyping {
            types: VecSet::new_empty(),
            regions: VecSet::new(
                region_names
                    .iter()
                    .map(|r| ConcreteRegion::new(r.to_string(), scope))
                    .collect(),
            ),
        }
    }

    pub fn new() -> Self {
        ConcreteTyping {
            types: VecSet::new_empty(),
            regions: VecSet::new_empty(),
        }
    }

    pub fn explode(&self) -> VecSet<(Place, Type)> {
        let binding_pairs = VecSet::new(
            self.types
                .0
                .iter()
                .map(|t| {
                    (
                        Place {
                            variable: t.binding.clone().unwrap().variable,
                            path: vec![],
                        },
                        t.clone(),
                    )
                })
                .collect(),
        );

        let all_concrete_regions: Vec<ConcreteRegion> =
            self.regions.0.iter().map(|r| r.clone()).collect();
        let all_places: Vec<Place> = all_concrete_regions
            .iter()
            .map(|r| r.loans.clone())
            .reduce(|acc_loans, current_loans| acc_loans.union(current_loans))
            .unwrap()
            .0
            .iter()
            .map(|l| l.place.clone())
            .collect();
        let place_pairs = VecSet(
            all_places
                .iter()
                .map(|p| {
                    (
                        p.clone(),
                        self.lookup_type(p.clone())
                            .expect("Unexpected error")
                            .clone(),
                    )
                })
                .collect(),
        );

        let all_pairs = binding_pairs.union(place_pairs);

        let mut result = VecSet::new_empty();

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
                let mut exploded = VecSet::new_empty();
                for (i, type_info) in types.iter().enumerate() {
                    let new_place = Place {
                        variable: (&place).clone().variable,
                        path: [&(&place).clone().path[..], &vec![i as u32][..]].concat(),
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
    pub fn copyable(&self) -> bool {
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
    pub fn eq_unchecked_regions(&self, other: &Self) -> bool {
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
}

impl From<Binding> for Place {
    fn from(value: Binding) -> Self {
        Place {
            variable: value.variable,
            path: vec![],
        }
    }
}

impl From<Loan> for PlaceExpression {
    fn from(loan: Loan) -> Self {
        if loan.is_reborrow {
            PlaceExpression::Deref(Box::new(loan.place.into()))
        } else {
            loan.place.into()
        }
    }
}

impl From<Place> for PlaceExpression {
    fn from(place: Place) -> Self {
        PlaceExpression::Place(Place {
            variable: place.variable,
            path: place.path,
        })
    }
}

impl TryFrom<Expression> for Place {
    type Error = OxidizedError;

    fn try_from(value: Expression) -> Result<Self, Self::Error> {
        let place_expression = value.clone().try_into()?;
        if let PlaceExpression::Place(place) = place_expression {
            Ok(place)
        } else {
            Err(OxidizedError::InvalidPlaceExpression(value))
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
                if let Expression::Variable(var) = *e {
                    Ok(Self::Place(Place {
                        variable: var,
                        path: p,
                    }))
                } else {
                    let place_expression = (*e).try_into()?;
                    Ok(Self::PathAccess(Box::new(place_expression), p))
                }
            }
            Expression::Variable(var) => Ok(Self::Place(Place {
                variable: var,
                path: vec![],
            })),
            _ => Err(OxidizedError::InvalidPlaceExpression(value)),
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
            PlaceExpression::Place(place) => place.variable.to_string(),
        }
    }
}

impl PartialOrd for OwnershipQualifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (OwnershipQualifier::Shrd, OwnershipQualifier::Shrd) => Some(std::cmp::Ordering::Equal),
            (OwnershipQualifier::Shrd, OwnershipQualifier::Uniq) => Some(std::cmp::Ordering::Less),
            (OwnershipQualifier::Uniq, OwnershipQualifier::Shrd) => {
                Some(std::cmp::Ordering::Greater)
            }
            (OwnershipQualifier::Uniq, OwnershipQualifier::Uniq) => Some(std::cmp::Ordering::Equal),
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
    pub fn get_concrete(&self) -> &ConcreteRegion {
        let RegionLevel::ConcreteRegion(cr) = &self.0;
        cr
    }

    pub fn new_concrete(name: String, scope: u32) -> Self {
        Region(RegionLevel::ConcreteRegion(ConcreteRegion::new(
            name, scope,
        )))
    }
}

impl ConcreteRegion {
    pub fn new(name: String, scope: u32) -> Self {
        ConcreteRegion {
            name: name,
            loans: VecSet::new_empty(),
            scope,
        }
    }

    pub fn set_loans(&self, loans: VecSet<Loan>) -> ConcreteRegion {
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

impl<T> VecSet<T> {
    pub fn new(vec: Vec<T>) -> Self {
        VecSet(vec)
    }

    pub fn new_empty() -> Self {
        VecSet(vec![])
    }

    pub fn push(&mut self, item: T) {
        self.0.push(item);
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        self.0.get(i)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.0.iter_mut()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn last(&self) -> Option<&T> {
        self.0.last()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.0.contains(item)
    }
}

impl<T, Idx> Index<Idx> for VecSet<T>
where
    Idx: SliceIndex<[T]>,
{
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IntoIterator for VecSet<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ConcreteRegion {
    pub fn get_referenced_place(&self) -> Place {
        let loan = self.loans.0.last().unwrap();
        if loan.is_reborrow {
            self.loans.0.iter().nth_back(1).unwrap().place.clone()
        } else {
            loan.place.clone()
        }
    }
}
