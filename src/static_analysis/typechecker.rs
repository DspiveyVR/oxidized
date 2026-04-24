/*

tuples are the only thing that need to add any types to the cont typing (for now).  return types of individual expressions are not
added to the cont typing unless they are returning into an element of a tuple.  so in a seq statement e1; e2, the return type of
e1 only matters to e2 insofar as it affects the output typing gamma1, but the e1 return type is not added to the cont typing.

helper fns

kill type (in env)

FIXME:
there's a little more to the regions() metafunction than just accessing the regions.  this will be important for getting the regions
in the cont typing.  probably can forget about it for now.


*/

use super::auxiliary::*;
use super::data_structures::*;
use crate::ast::Constant;
use crate::ast::Expression;
use crate::ast::OwnershipQualifier;

impl Expression {
    pub(crate) fn typecheck(
        &self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> Result<(ConcreteTyping, Type), OxidizedError> {
        match self {
            Expression::Letrgn(..) => Self::pop_vars(
                stack_typing.clone(),
                err_if_dead(self.t_letrgn(stack_typing, continuation_typing)),
            ),
            Expression::Sequence(..) => err_if_dead(self.t_seq(stack_typing, continuation_typing)),
            Expression::Let { .. } => err_if_dead(self.t_let(
                stack_typing,
                continuation_typing,
                &Expression::Constant(Constant::Unit),
            )), // If a let is by itself not sequenced then its basically just dead.
            Expression::Ref(..) => err_if_dead(self.t_borrow(stack_typing, continuation_typing)),
            Expression::Variable(var) => err_if_dead(Ok((
                stack_typing.clone(),
                stack_typing.type_lookup(PlaceExpression::Variable(var.to_string()))?,
            ))),
            // FIXME: For now constants will be treated as noncopyable. once copy logic is implemented this branch should not call err_if_dead.
            Expression::Constant(c) => {
                err_if_dead(Ok((stack_typing.clone(), Self::t_constant(c)?)))
            }
            _ => panic!("Unexpected error"),
        }
    }

    fn pop_vars(
        stack_typing: ConcreteTyping,
        tc_result: Result<(ConcreteTyping, Type), OxidizedError>,
    ) -> Result<(ConcreteTyping, Type), OxidizedError> {
        let new_stack_typing = ConcreteTyping {
            regions: VecSet(
                (tc_result.clone()?.0.regions.0[..stack_typing.regions.0.len()]).to_vec(),
            ),
            types: VecSet((tc_result.clone()?.0.types.0[..stack_typing.types.0.len()]).to_vec()),
        };

        Ok((new_stack_typing, tc_result?.1))
    }

    fn t_constant(constant: &Constant) -> Result<Type, OxidizedError> {
        match constant {
            crate::ast::Constant::Unit => Ok(Type {
                binding: None,
                variant: TypeVariant::Base(BaseType::Unit),
                is_sized: Ternary::No,
                is_dead: Ternary::No,
                copyable: false,
            }),
            crate::ast::Constant::True => todo!(),
            crate::ast::Constant::False => todo!(),
            crate::ast::Constant::Number(_) => Ok(Type {
                binding: None,
                variant: TypeVariant::Base(BaseType::U32),
                is_sized: Ternary::Yes,
                is_dead: Ternary::No,
                copyable: false,
            }),
        }
    }

    // Typing
    fn t_borrow(
        &self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> Result<(ConcreteTyping, Type), OxidizedError> {
        if let Expression::Ref(ref_annotation, e) = self {
            let mut region = stack_typing.get_region(ref_annotation.0.to_string())?;
            if !region.get_concrete().loans.0.is_empty() {
                return Err(OxidizedError::NonemptyRegion(
                    ref_annotation.0.clone(),
                    region.get_concrete().loans.clone(),
                ));
            }
            // TODO: rnic

            let place_expression: PlaceExpression = (**e).clone().try_into()?;

            let loans = stack_typing.borrow_check(
                continuation_typing,
                place_expression.clone(),
                ref_annotation.1,
            )?;

            let place_expression_type = stack_typing.type_lookup(place_expression)?;

            let ref_type = Type {
                binding: None,
                variant: TypeVariant::Reference(ReferenceType {
                    region_name: ref_annotation.0.to_string(),
                    ownership_qualifier: ref_annotation.1,
                    target_type: Box::new(place_expression_type),
                }),
                is_sized: Ternary::Yes,
                is_dead: Ternary::No,
                copyable: false,
            };

            region = region.get_concrete().set_loans(loans);

            Ok((
                stack_typing.set_region(ref_annotation.0.to_string(), region),
                ref_type,
            ))
        } else {
            panic!("Unexpected error");
        }
    }

    pub(crate) fn t_letrgn(
        &self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> Result<(ConcreteTyping, Type), OxidizedError> {
        if let Expression::Letrgn(region_names, inner_exp) = self {
            stack_typing.check_conflicting_region_names(region_names)?;
            let scope = if let Some(region) = stack_typing.regions.0.last() {
                region.get_concrete().scope + 1
            } else {
                0
            };

            let local_types = ConcreteTyping::new_with_regions(region_names, scope);
            inner_exp.typecheck(stack_typing.union(local_types), continuation_typing)
        } else {
            panic!("Unexpected error");
        }
    }

    fn t_seq(
        &self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> Result<(ConcreteTyping, Type), OxidizedError> {
        if let Expression::Sequence(e1, e2) = self {
            if let Expression::Let { .. } = &**e1 {
                e1.t_let(stack_typing, continuation_typing, e2)
            } else {
                let (gamma1, _) = e1.typecheck(stack_typing, continuation_typing.clone())?;
    
                e2.typecheck(
                    gamma1.gc_loans(continuation_typing.clone()),
                    continuation_typing.clone(),
                )
            }
        } else {
            panic!("Unexpected error");
        }
    }

    fn t_let(
        &self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
        e2: &Expression,
    ) -> Result<(ConcreteTyping, Type), OxidizedError> {
        if let Expression::Let {
            name,
            is_mut,
            type_annotation,
            e: e1,
        } = self
        {
            let (gamma1, return_type) = e1.typecheck(stack_typing, continuation_typing.clone())?;

            let annotated_type: Type = type_annotation.clone().into();
            if !return_type.eq_unchecked_regions(&annotated_type) {
                return Err(OxidizedError::MismatchedTypes {
                    expected: annotated_type,
                    found: return_type,
                });
            }
            // TODO: check for all free regions in ta. r rnrb
            let mut bound_type = annotated_type;
            bound_type.binding = Some(Binding {
                variable: name.to_string(),
                is_mut: *is_mut,
            });
            let mut gamma1_modified = gamma1.add_type(bound_type.clone());
            gamma1_modified = gamma1_modified.rewrite_regions(
                return_type,
                bound_type.clone(),
                RegionRewriteMode::Plus,
            )?;

            let (gamma2, return_type2) = e2.typecheck(
                gamma1_modified.gc_loans(continuation_typing.clone()),
                continuation_typing,
            )?;

            let mut dead_bound_type = bound_type.clone();
            dead_bound_type.is_dead = Ternary::Yes;

            Ok((gamma2.set_type(bound_type, dead_bound_type), return_type2))
        } else {
            panic!("Unexpected error");
        }
    }
}

// TODO: well-formedness rules
impl ConcreteTyping {
    // Ownership Safety
    fn borrow_check(
        &self,
        continuation_typing: Self,
        place_expression: PlaceExpression,
        ownership_qualifier: OwnershipQualifier,
    ) -> Result<VecSet<Loan>, OxidizedError> {
        let place_result = place_expression.try_into();
        if let Ok(place) = place_result {
            self.o_safe_place(continuation_typing, place, ownership_qualifier)
        } else {
            match place_expression {
                _ => panic!("Unexpected error"),
            }
        }
    }

    fn o_safe_place(
        &self,
        continuation_typing: Self,
        place: Place,
        ownership_qualifier: OwnershipQualifier,
    ) -> Result<VecSet<Loan>, OxidizedError> {
        let converted_place = place.clone().into();
        for region in self.regions.union(continuation_typing.regions).0 {
            for loan in region.get_concrete().clone().loans.0 {
                if (ownership_qualifier == OwnershipQualifier::Uniq
                    || loan.ownership_qualifier == OwnershipQualifier::Uniq)
                    && loan.place_expression.overlaps(&converted_place)
                {
                    return Err(OxidizedError::UnsafePlace {
                        place,
                        ownership_qualifier: ownership_qualifier.to_string(),
                        place_expression2: loan.place_expression,
                        ownership_qualifier2: loan.ownership_qualifier.to_string(),
                    });
                }
            }
        }

        // TODO: reborrow exclusion list (the or condition)

        Ok(VecSet(vec![Loan {
            ownership_qualifier,
            place_expression: converted_place,
        }]))
    }

    // I can't think of how the continuation typing would be relevant in the rr or outlives rules but we'll see
    // Should rewriting empty the original region?  The paper doesn't seem to indicate so but...... we'll see :\
    // Region Rewriting
    fn rewrite_regions(
        &self,
        type1: Type,
        type2: Type,
        mode: RegionRewriteMode,
    ) -> Result<Self, OxidizedError> {
        // Region rewriting is reflexive
        if type1 == type2 {
            Ok(self.clone())
        } else {
            match (type1.variant, type2.variant) {
                (TypeVariant::Reference(r1), TypeVariant::Reference(r2)) => self.region_outlives(
                    // FIXME: probably just make the rr_reference method for this
                    self.get_region(r1.region_name)?,
                    self.get_region(r2.region_name)?,
                    mode,
                ),
                (TypeVariant::Base(_), TypeVariant::Base(_)) => Ok(self.clone()),
                _ => panic!("Unexpected error"),
            }
        }
    }

    // Outlives
    fn region_outlives(
        &self,
        region1: Region,
        region2: Region,
        mode: RegionRewriteMode,
    ) -> Result<Self, OxidizedError> {
        match mode {
            RegionRewriteMode::Plus => self.ol_combine_concrete(region1, region2),
            _ => panic!("Unexpected error"),
        }
    }

    // TODO: closure restriction rule
    fn ol_combine_concrete(&self, region1: Region, region2: Region) -> Result<Self, OxidizedError> {
        if !(self.rnrb(region1.get_concrete().clone()) && self.rnrb(region2.get_concrete().clone()))
        {
            return Err(OxidizedError::ConcreteOutlivesError(
                region1.get_concrete().clone(),
                region2.get_concrete().clone(),
            ));
        }
        if !(self
            .regions
            .0
            .iter()
            .find(|r| r.get_concrete().name == region1.get_concrete().name)
            .unwrap()
            .get_concrete()
            .scope
            <= self
                .regions
                .0
                .iter()
                .find(|r| r.get_concrete().name == region2.get_concrete().name)
                .unwrap()
                .get_concrete()
                .scope)
        {
            return Err(OxidizedError::ConcreteOutlivesError(
                region1.get_concrete().clone(),
                region2.get_concrete().clone(),
            ));
        }
        let new_loans = region1
            .get_concrete()
            .loans
            .union(region2.get_concrete().clone().loans);
        let mut new_r2 = region2.clone();
        let RegionLevel::ConcreteRegion(ref mut cr) = new_r2.0;
        cr.loans = new_loans;

        Ok(self.set_region(region2.get_concrete().clone().name, new_r2))
    }
}
