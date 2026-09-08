use std::println;

use super::data_structures::*;

impl ConcreteTyping {
    // FIXME: make this look less stupid
    pub(crate) fn rnrb(&self, region: ConcreteRegion) -> bool {
        let exploded = self.explode();
        exploded.0.iter().all(|pair| {
            if let TypeVariant::Reference(reference) = pair.1.variant.clone() {
                if reference.region_name == region.name {
                    !self.regions.0.iter().any(|r| {
                        if r.name != region.name {
                            r.loans.0.iter().any(|loan| {
                                if loan.is_reborrow {
                                    loan.place == pair.0
                                } else {
                                    false
                                }
                            })
                        } else {
                            false
                        }
                    })
                } else {
                    true
                }
            } else {
                true
            }
        })
    }

    // TODO: Actually implement NLL
    pub(crate) fn gc_loans(&self, continuation_typing: Self) -> Self {
        let mut stack_clone = self.clone();
        let env_union = stack_clone.types.union(continuation_typing.types).0;
        for r in stack_clone.regions.0.iter_mut() {
            if !env_union.iter().any(|t| {
                t.is_dead != Ternary::Yes
                    && if let TypeVariant::Reference(ref_type) = &t.variant {
                        r.name == ref_type.region_name
                    } else {
                        false
                    }
            }) {
                r.loans.0.clear();
            }
        }

        stack_clone
    }

    pub(crate) fn check_conflicting_region_names(
        &self,
        region_names: &Vec<String>,
    ) -> Result<(), OxidizedError> {
        if let Some(conflicting_region) = self
            .regions
            .0
            .iter()
            .find(|r| region_names.contains(&r.name))
        {
            Err(OxidizedError::RegionNameConflict(
                conflicting_region.name.to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

impl PlaceExpression {
    pub(crate) fn overlaps(&self, other: &PlaceExpression) -> bool {
        self.is_prefix_of(other.clone()) || other.is_prefix_of(self.clone()) || self == other
    }

    fn is_prefix_of(&self, other: PlaceExpression) -> bool {
        let mut depth = 0;
        let mut current_node = self;
        loop {
            match current_node {
                PlaceExpression::Deref(place_expression) => {
                    depth += 1;
                    current_node = place_expression;
                }
                PlaceExpression::PathAccess(place_expression, _) => {
                    depth += 1;
                    current_node = place_expression;
                }
                PlaceExpression::Place(_) => {
                    depth += 1;
                    break;
                }
            }
        }
        let mut other_levels = Vec::new();
        let mut current_node = other;
        loop {
            match current_node.clone() {
                PlaceExpression::Deref(place_expression) => {
                    other_levels.push(current_node.clone());
                    current_node = *place_expression;
                }
                PlaceExpression::PathAccess(place_expression, _) => {
                    other_levels.push(current_node.clone());
                    current_node = *place_expression;
                }
                PlaceExpression::Place(_) => {
                    other_levels.push(current_node.clone());
                    break;
                }
            }
        }
        if other_levels.len() < depth {
            return false;
        }
        other_levels.reverse();
        if other_levels[depth - 1] == *self {
            return true;
        }
        // If the expression is equal except for depth 0 and the first depth 0 path is a prefix of the second
        if let PlaceExpression::PathAccess(place_expression1, path1) = self {
            if let PlaceExpression::PathAccess(place_expression2, path2) =
                other_levels[depth - 1].clone()
            {
                return *place_expression1 == place_expression2
                    && PlaceExpression::is_path_prefix(path1.clone(), path2.clone());
            } else {
                return false;
            }
        }

        // Kinda forgot how this works no i'm not sure if the previous if lets are necessary but adding this did fix the failing tests.
        if let PlaceExpression::Place(place1) = self {
            if let PlaceExpression::Place(place2) = other_levels[depth - 1].clone() {
                place1.variable == place2.variable
                    && PlaceExpression::is_path_prefix(place1.clone().path, place2.path)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_path_prefix(path1: Vec<u32>, path2: Vec<u32>) -> bool {
        if path1.len() < path2.len() {
            for (i, projection) in path1.iter().enumerate() {
                if *projection != path2[i] {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

pub(crate) fn err_if_dead(
    res: Result<(ConcreteTyping, Type), OxidizedError>,
) -> Result<(ConcreteTyping, Type), OxidizedError> {
    let ret_type = res.clone()?.1;
    if ret_type.is_dead == Ternary::Yes {
        Err(OxidizedError::UseOfMovedValue(ret_type))
    } else {
        res
    }
}

pub(crate) fn get_unit() -> Type {
    Type {
        binding: None,
        variant: TypeVariant::Base(BaseType::Unit),
        is_sized: Ternary::Yes,
        is_dead: Ternary::No,
    }
}