use core::panic;
use std::todo;
use std::vec;

/*

tuples are the only thing that need to add any types to the cont typing (for now).  return types of individual expressions are not
added to the cont typing unless they are returning into an element of a tuple.  so in a seq statement e1; e2, the return type of
e1 only matters to e2 insofar as it affects the output typing gamma1, but the e1 return type is not added to the cont typing.

helper fns

kill type (in env)

FIXME:
there's a little more to the regions() metafunction than just accessing the regions.  this will be important for getting the regions
in the cont typing.  probably can forget about it for now.

TODO: see if theres a way to get rid of the unexpected match conditions so that it is logically impossible according to the type system.
TODO: better new, get, and set methods for various types. prob could use some helper methods in general

*/
use super::data_structures::*;
use super::tc_state_machine::*;
use crate::ast::Constant;
use crate::ast::Expression;
use crate::ast::OwnershipQualifier;
use crate::static_analysis::auxiliary::err_if_dead;
use crate::static_analysis::auxiliary::get_unit;

impl Expression {
    pub fn typecheck(
        self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        match self {
            Expression::Letrgn(..) => self.t_letrgn(stack_typing, continuation_typing),
            Expression::Sequence(..) => self.t_seq(stack_typing, continuation_typing),
            Expression::Let { .. } => self.t_let(
                stack_typing,
                continuation_typing,
                Expression::Constant(Constant::Unit),
            ), // If a let is by itself not sequenced then its basically just dead.
            Expression::Ref(..) => self.t_borrow(stack_typing, continuation_typing),
            Expression::Variable(_) | Expression::PathAccess(..) | Expression::Deref(_) => {
                Self::place_expression_usage(
                    TryInto::<PlaceExpression>::try_into(self.clone()).expect("Unexpected error"),
                    stack_typing,
                    continuation_typing,
                )
            }
            Expression::Assign(ref var, _) => {
                if let Ok(place) = TryInto::<Place>::try_into((**var).clone()) {
                    self.t_assign(place, stack_typing, continuation_typing)
                } else {
                    panic!("Unexpected error");
                }
            }
            Expression::Constant(c) => {
                let st_clone = stack_typing.clone();
                TExprState::new_obj(
                    move || {
                        let stack_typing = st_clone;

                        Ok((
                            None,
                            TCReturn::TCResult(Ok((stack_typing, Self::t_constant(&c)?))),
                        ))
                    },
                    (stack_typing, continuation_typing),
                    "t_constant".to_string(),
                )
            }
            _ => panic!("Unexpected error"),
        }
    }

    // Typing
    fn t_constant(constant: &Constant) -> Result<Type, OxidizedError> {
        match constant {
            crate::ast::Constant::Unit => Ok(Type {
                binding: None,
                variant: TypeVariant::Base(BaseType::Unit),
                is_sized: Ternary::Yes,
                is_dead: Ternary::No,
            }),
            crate::ast::Constant::True => todo!(),
            crate::ast::Constant::False => todo!(),
            crate::ast::Constant::Number(_) => Ok(Type {
                binding: None,
                variant: TypeVariant::Base(BaseType::U32),
                is_sized: Ternary::Yes,
                is_dead: Ternary::No,
            }),
        }
    }

    fn place_expression_usage(
        place_expression: PlaceExpression,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        if let Ok(typ) =
            stack_typing.flat_compute_type(place_expression.clone(), OwnershipQualifier::Shrd)
        {
            if typ.copyable() {
                let _ = stack_typing.ownership_check(
                    &continuation_typing,
                    place_expression,
                    OwnershipQualifier::Shrd,
                )?;

                Expression::t_copy(typ, stack_typing, continuation_typing)
            } else {
                match stack_typing
                    .flat_compute_type(place_expression.clone(), OwnershipQualifier::Uniq)
                {
                    Ok(typ) => {
                        let _ = stack_typing.ownership_check(
                            &continuation_typing,
                            place_expression,
                            OwnershipQualifier::Uniq,
                        )?;

                        Expression::t_move(typ, stack_typing, continuation_typing)
                    }
                    Err(err) => Err(err),
                }
            }
        } else {
            match stack_typing.flat_compute_type(place_expression.clone(), OwnershipQualifier::Uniq)
            {
                Ok(typ) => {
                    let _ = stack_typing.ownership_check(
                        &continuation_typing,
                        place_expression,
                        OwnershipQualifier::Uniq,
                    )?;

                    Expression::t_move(typ, stack_typing, continuation_typing)
                }
                Err(err) => Err(err),
            }
        }
    }

    fn t_move(
        typ: Type,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        let st_clone = stack_typing.clone();
        let ct_clone = continuation_typing.clone();
        TExprState::new_obj(
            move || {
                let mut stack_typing = st_clone;
                let continuation_typing = ct_clone;
                let typ = err_if_dead(typ)?;
                let mut return_type = typ.clone();
                return_type.binding = None;
                let mut dead_type = typ.clone();
                dead_type.is_dead = Ternary::Yes;
                stack_typing = stack_typing.set_type(typ, dead_type);

                Ok((None, TCReturn::TCResult(Ok((stack_typing, return_type)))))
            },
            (stack_typing, continuation_typing),
            "t_move".to_string(),
        )
    }

    fn t_copy(
        typ: Type,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        let st_clone = stack_typing.clone();
        let ct_clone = continuation_typing.clone();
        TExprState::new_obj(
            move || {
                let stack_typing = st_clone;
                let continuation_typing = ct_clone;
                let typ = err_if_dead(typ)?;
                let mut return_type = typ.clone();
                return_type.binding = None;

                Ok((None, TCReturn::TCResult(Ok((stack_typing, return_type)))))
            },
            (stack_typing, continuation_typing),
            "t_copy".to_string(),
        )
    }

    fn t_borrow(
        self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        let st_clone = stack_typing.clone();
        let ct_clone = continuation_typing.clone();
        TExprState::new_obj(
            move || {
                let stack_typing = st_clone;
                let continuation_typing = ct_clone;
                if let Expression::Ref(ref_annotation, e) = self {
                    let mut region = stack_typing.get_region(ref_annotation.0.to_string())?;
                    if !region.loans.0.is_empty() {
                        return Err(OxidizedError::NonemptyRegion(
                            ref_annotation.0.clone(),
                            region.loans.clone(),
                        ));
                    }
                    // TODO: rnic

                    let place_expression: PlaceExpression = (*e).clone().try_into()?;

                    let loans = stack_typing.ownership_check(
                        &continuation_typing,
                        place_expression.clone(),
                        ref_annotation.1,
                    )?;

                    let place_expression_type = err_if_dead(
                        stack_typing
                            .flat_compute_type(place_expression, ref_annotation.1)?
                            .clone(),
                    )?;

                    let ref_type = Type {
                        binding: None,
                        variant: TypeVariant::Reference(ReferenceType {
                            region_name: ref_annotation.0.to_string(),
                            ownership_qualifier: ref_annotation.1,
                            target_type: Box::new(place_expression_type),
                        }),
                        is_sized: Ternary::Yes,
                        is_dead: Ternary::No,
                    };
                    
                    region = region.set_loans(loans);

                    Ok((
                        None,
                        TCReturn::TCResult(Ok((
                            stack_typing.set_region(ref_annotation.0.to_string(), region),
                            ref_type,
                        ))),
                    ))
                } else {
                    panic!("Unexpected error");
                }
            },
            (stack_typing.clone(), continuation_typing),
            "t_borrow".to_string(),
        )
    }

    pub fn t_letrgn(
        self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        let st_clone = stack_typing.clone();
        let cont_clone = continuation_typing.clone();
        let st_clone2 = stack_typing.clone();
        TLetrgnState::new_obj(
            move || {
                let stack_typing = st_clone;
                if let Expression::Letrgn(region_names, inner_exp) = self {
                    stack_typing.check_conflicting_region_names(&region_names)?;
                    let scope = if let Some(region) = stack_typing.regions.0.last() {
                        region.scope + 1
                    } else {
                        0
                    };

                    let local_types = ConcreteTyping::new_with_regions(&region_names, scope);
                    inner_exp.typecheck(stack_typing.union(local_types), cont_clone)
                } else {
                    panic!("Unexpected error");
                }
            },
            move |result: TCResult| {
                let stack_typing = st_clone2;
                let result = result?;
                let env = result.0;
                let return_type = result.1;

                let new_stack_typing = ConcreteTyping {
                    regions: VecSet((env.regions.0[..stack_typing.regions.0.len()]).to_vec()),
                    types: VecSet((env.types.0[..stack_typing.types.0.len()]).to_vec()),
                };

                // Similar to one part of the WF-StackTyping rule. The accuracy of this compared to the paper is questionable.
                for region in &new_stack_typing.regions.0 {
                    for loan in &region.loans.0 {
                        match env.flat_compute_type_no_ord(loan.clone().into()) {
                            Ok(typ) => {
                                if typ.clone().is_dead != Ternary::No {
                                    return Err(OxidizedError::LifetimeError(
                                        typ,
                                        region.name.clone(),
                                    ));
                                }
                            }
                            Err(err) => return Err(err),
                        }
                    }
                }

                Ok((
                    None,
                    TCReturn::TCResult(Ok((new_stack_typing, return_type))),
                ))
            },
            (stack_typing.clone(), continuation_typing.clone()),
            "t_letrgn".to_string(),
        )
    }

    fn t_assign(
        self,
        place: Place,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        if let Expression::Assign(_, e) = self {
            let st_clone = stack_typing.clone();
            let st_clone2 = stack_typing.clone();
            let ct_clone = continuation_typing.clone();
            let ct_clone2 = continuation_typing.clone();
            TAssignState::new_obj(
                move || e.typecheck(st_clone, ct_clone),
                move |gamma1: ConcreteTyping, return_type: Type| {
                    let place_type = gamma1.lookup_type(place.clone().into())?;
                    if !return_type.eq_unchecked_regions(&place_type) {
                        return Err(OxidizedError::MismatchedTypes {
                            expected: place_type,
                            found: return_type,
                        });
                    }

                    let gamma1_prime = gamma1.kill_rules(place.clone());

                    if let TypeVariant::Reference(assignment_ref_type) = &place_type.variant {
                        let r = &assignment_ref_type.region_name;
                        for (place_prime, typ) in gamma1_prime.explode().0.iter() {
                            if let TypeVariant::Reference(ref_type) = &typ.variant {
                                let r_prime = &ref_type.region_name;
                                if &place != place_prime && r == r_prime {
                                    return Err(OxidizedError::RegionMustBeUnique(
                                        r.clone(),
                                        place.clone(),
                                        st_clone2,
                                    ));
                                }
                            }
                        }
                    }

                    gamma1_prime.rewrite_regions(
                        return_type.clone(),
                        place_type.clone(),
                        RegionRewriteMode::Equals,
                    )?;

                    if !place_type.binding.as_ref().unwrap().is_mut {
                        return Err(OxidizedError::NoMutAnnotation(
                            place_type.binding.unwrap().variable,
                        ));
                    }

                    if let Err(err) = gamma1_prime.o_safe_place(
                        &ct_clone2,
                        place,
                        OwnershipQualifier::Uniq,
                        &VecSet::new(),
                    ) {
                        if place_type.is_dead != Ternary::Yes {
                            return Err(err);
                        }
                    }

                    let mut bound_type = return_type.clone();
                    bound_type.binding = place_type.clone().binding;

                    Ok((
                        None,
                        TCReturn::TCResult(Ok((
                            gamma1_prime.set_type(place_type.clone(), bound_type),
                            get_unit(),
                        ))),
                    ))
                },
                (stack_typing.clone(), continuation_typing.clone()),
                "t_assign".to_string(),
            )
        } else {
            panic!("Unexpected error");
        }
    }

    fn t_seq(
        self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        if let Expression::Sequence(e1, e2) = self {
            if let Expression::Let { .. } = *e1 {
                e1.t_let(stack_typing, continuation_typing, *e2)
            } else {
                let st_clone = stack_typing.clone();
                let cont_clone = continuation_typing.clone();
                let cont_clone2 = continuation_typing.clone();
                TSeqState::new_obj(
                    move || e1.typecheck(st_clone, cont_clone),
                    move |gamma1: ConcreteTyping| {
                        e2.typecheck(
                            gamma1.gc_loans(continuation_typing.clone()),
                            continuation_typing.clone(),
                        )
                    },
                    (stack_typing.clone(), cont_clone2),
                    "t_seq".to_string(),
                )
            }
        } else {
            panic!("Unexpected error");
        }
    }

    fn t_let(
        self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
        e2: Expression,
    ) -> TCReturnResult {
        if let Expression::Let {
            name,
            is_mut,
            type_annotation,
            e: e1,
        } = self
        {
            let st_clone = stack_typing.clone();
            let st_clone2 = stack_typing.clone();
            let ct_clone = continuation_typing.clone();
            let ct_clone2 = continuation_typing.clone();
            TLetState::new_obj(
                move || e1.typecheck(st_clone, ct_clone),
                move |gamma1: ConcreteTyping, return_type: Type| {
                    let annotated_type: Type = type_annotation.clone().into();
                    if !return_type.eq_unchecked_regions(&annotated_type) {
                        return Err(OxidizedError::MismatchedTypes {
                            expected: annotated_type,
                            found: return_type,
                        });
                    }

                    for region in annotated_type.free_regions(&st_clone2) {
                        if !st_clone2.rnrb(region) {
                            return Err(OxidizedError::CannotLetBindReborrowedRegions(
                                annotated_type,
                            ));
                        }
                    }

                    let mut bound_type = annotated_type;
                    bound_type.binding = Some(Binding {
                        variable: name.to_string(),
                        is_mut,
                    });
                    let mut gamma1_modified = gamma1.add_type(bound_type.clone());
                    gamma1_modified = gamma1_modified.rewrite_regions(
                        return_type,
                        bound_type.clone(),
                        RegionRewriteMode::Plus,
                    )?;

                    Ok((None, TCReturn::TCResult(Ok((gamma1_modified, bound_type)))))
                },
                |gamma1_modified: ConcreteTyping| {
                    e2.typecheck(gamma1_modified.gc_loans(ct_clone2.clone()), ct_clone2)
                },
                |gamma2: ConcreteTyping, return_type2: Type, bound_type: Type| {
                    let mut dead_bound_type = bound_type.clone();
                    dead_bound_type.is_dead = Ternary::Yes;

                    Ok((
                        None,
                        TCReturn::TCResult(Ok((
                            gamma2.set_type(bound_type, dead_bound_type),
                            return_type2,
                        ))),
                    ))
                },
                (stack_typing.clone(), continuation_typing.clone()),
                "t_let".to_string(),
            )
        } else {
            panic!("Unexpected error");
        }
    }
}

// TODO: well-formedness rules
impl ConcreteTyping {
    // Ownership Safety
    fn ownership_check(
        &self,
        continuation_typing: &Self,
        place_expression: PlaceExpression,
        ownership_qualifier: OwnershipQualifier,
    ) -> Result<VecSet<Loan>, OxidizedError> {
        self._ownership_check(
            continuation_typing,
            place_expression,
            ownership_qualifier,
            &VecSet::new(),
        )
    }

    fn _ownership_check(
        &self,
        continuation_typing: &Self,
        place_expression: PlaceExpression,
        ownership_qualifier: OwnershipQualifier,
        reborrow_exclusion_list: &VecSet<Place>,
    ) -> Result<VecSet<Loan>, OxidizedError> {
        let tc = self.compute_type(place_expression, ownership_qualifier)?;
        let mut final_loans = VecSet::new();
        for (typ, pe_ptr) in tc {
            match pe_ptr.place_expression.clone() {
                PlaceExpression::Deref(_) => {
                    final_loans = self.o_deref(
                        continuation_typing,
                        ownership_qualifier,
                        reborrow_exclusion_list,
                        typ,
                        pe_ptr.points_to,
                    )?
                }
                PlaceExpression::Place(place) => {
                    final_loans = self.o_safe_place(
                        continuation_typing,
                        place,
                        ownership_qualifier,
                        reborrow_exclusion_list,
                    )?
                }
                PlaceExpression::PathAccess(place_expression, items) => todo!(),
            }
        }

        Ok(final_loans)
    }

    fn o_deref(
        &self,
        continuation_typing: &Self,
        ownership_qualifier: OwnershipQualifier,
        reborrow_exclusion_list: &VecSet<Place>,
        typ: Type,
        place: Place,
    ) -> Result<VecSet<Loan>, OxidizedError> {
        if let TypeVariant::Reference(ref_type) = typ.variant {
            if ownership_qualifier > ref_type.ownership_qualifier {
                return Err(OxidizedError::ViolatedOwnershipOrdering(ref_type));
            }
            let region = self.get_region(ref_type.region_name)?;
            let r_loans = region.loans;
            let excl = VecSet(
                r_loans
                    .0
                    .iter()
                    .filter_map(|l| {
                        if l.is_reborrow {
                            Some(l.place.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Place>>(),
            );
            let new_excl_list = reborrow_exclusion_list
                .union(excl)
                .union(VecSet(vec![place.clone()]));

            let mut new_loans = VecSet::new();
            for loan in r_loans.0 {
                new_loans = new_loans.union(self._ownership_check(
                    continuation_typing,
                    loan.into(),
                    ownership_qualifier,
                    &new_excl_list,
                )?);
            }
            new_loans.0.push(Loan {
                ownership_qualifier,
                place: place.clone(),
                is_reborrow: true,
            });

            let converted_place = place.clone().into();
            // TODO: Figure out the condition about existence of a type in gamma but not theta. Not really sure what it means. Maybe not relevant until I have tuples.
            let check_res = (|| {
                for region in self.regions.union(continuation_typing.clone().regions).0 {
                    for loan in region.clone().loans.0 {
                        if (ownership_qualifier == OwnershipQualifier::Uniq
                            || loan.ownership_qualifier == OwnershipQualifier::Uniq)
                            && Into::<PlaceExpression>::into(loan.place.clone())
                                .overlaps(&converted_place)
                        {
                            return Err(OxidizedError::UnsafePlace {
                                place,
                                ownership_qualifier: ownership_qualifier.to_string(),
                                place_expression2: loan.place.into(),
                                ownership_qualifier2: loan.ownership_qualifier.to_string(),
                            });
                        }
                    }
                }
                Ok(())
            })();

            if let Err(err) = check_res {
                for (place, typ) in self.explode().0 {
                    if let TypeVariant::Reference(_) = typ.variant {
                        if !reborrow_exclusion_list.0.contains(&place) {
                            return Err(err);
                        }
                    }
                }
            }

            Ok(new_loans)
        } else {
            panic!("Unexpected match");
        }
    }

    fn o_safe_place(
        &self,
        continuation_typing: &Self,
        place: Place,
        ownership_qualifier: OwnershipQualifier,
        reborrow_exclusion_list: &VecSet<Place>,
    ) -> Result<VecSet<Loan>, OxidizedError> {
        let converted_place = place.clone().into();
        let place_clone = place.clone();
        // TODO: Figure out the condition about existence of a type in gamma but not theta. Not really sure what it means. Maybe not relevant until I have tuples.
        let check_res = (|| {
            for region in self.regions.union(continuation_typing.clone().regions).0 {
                for loan in region.clone().loans.0 {
                    if (ownership_qualifier == OwnershipQualifier::Uniq
                        || loan.ownership_qualifier == OwnershipQualifier::Uniq)
                        && Into::<PlaceExpression>::into(loan.place.clone())
                            .overlaps(&converted_place)
                    {
                        return Err(OxidizedError::UnsafePlace {
                            place: place_clone,
                            ownership_qualifier: ownership_qualifier.to_string(),
                            place_expression2: loan.place.into(),
                            ownership_qualifier2: loan.ownership_qualifier.to_string(),
                        });
                    }
                }
            }
            Ok(())
        })();

        if let Err(err) = check_res {
            for (place, typ) in self.explode().0 {
                if let TypeVariant::Reference(_) = typ.variant {
                    if !reborrow_exclusion_list.0.contains(&place) {
                        return Err(err);
                    }
                }
            }
        }

        Ok(VecSet(vec![Loan {
            ownership_qualifier,
            place,
            is_reborrow: false,
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
        region1: ConcreteRegion,
        region2: ConcreteRegion,
        mode: RegionRewriteMode,
    ) -> Result<Self, OxidizedError> {
        match mode {
            RegionRewriteMode::Plus => self.ol_combine_concrete(region1, region2),
            RegionRewriteMode::Equals => self.ol_check_concrete(region1, region2),
        }
    }

    fn ol_check_concrete(
        &self,
        region1: ConcreteRegion,
        region2: ConcreteRegion,
    ) -> Result<Self, OxidizedError> {
        if !(self.rnrb(region1.clone()) && self.rnrb(region2.clone())) {
            return Err(OxidizedError::ConcreteOutlivesError(
                region1.clone(),
                region2.clone(),
            ));
        }
        if !(self
            .regions
            .0
            .iter()
            .find(|r| r.name == region1.name)
            .unwrap()
            .scope
            <= self
                .regions
                .0
                .iter()
                .find(|r| r.name == region2.name)
                .unwrap()
                .scope)
        {
            return Err(OxidizedError::ConcreteOutlivesError(
                region1.clone(),
                region2.clone(),
            ));
        }

        Ok(self.clone())
    }

    // TODO: closure restriction rule
    fn ol_combine_concrete(
        &self,
        region1: ConcreteRegion,
        region2: ConcreteRegion,
    ) -> Result<Self, OxidizedError> {
        if !(self.rnrb(region1.clone()) && self.rnrb(region2.clone())) {
            return Err(OxidizedError::ConcreteOutlivesError(
                region1.clone(),
                region2.clone(),
            ));
        }
        if !(self
            .regions
            .0
            .iter()
            .find(|r| r.name == region1.name)
            .unwrap()
            .scope
            <= self
                .regions
                .0
                .iter()
                .find(|r| r.name == region2.name)
                .unwrap()
                .scope)
        {
            return Err(OxidizedError::ConcreteOutlivesError(
                region1.clone(),
                region2.clone(),
            ));
        }
        let new_loans = region1.loans.union(region2.clone().loans);
        let mut new_r2 = region2.clone();
        new_r2.loans = new_loans;

        Ok(self.set_region(region2.clone().name, new_r2))
    }
}
