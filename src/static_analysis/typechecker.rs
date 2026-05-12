/*

tuples are the only thing that need to add any types to the cont typing (for now).  return types of individual expressions are not
added to the cont typing unless they are returning into an element of a tuple.  so in a seq statement e1; e2, the return type of
e1 only matters to e2 insofar as it affects the output typing gamma1, but the e1 return type is not added to the cont typing.

helper fns

kill type (in env)

FIXME:
there's a little more to the regions() metafunction than just accessing the regions.  this will be important for getting the regions
in the cont typing.  probably can forget about it for now.


step debugging:
    could each tc function return an object containing closures with the tc body inside in multiple parts as well as the type environment?

*/

use super::auxiliary::*;
use super::data_structures::*;
use crate::ast::Constant;
use crate::ast::Expression;
use crate::ast::OwnershipQualifier;

/*

okay i think this actually does need to be polymorphic

t_let will need its own type to handle the 3rd part where it kills the type.  I think the bound type will need to be stored in
the struct and then there needs to be an e3_fn as well that is called by step_over. 
maybe these structs need to also contain an enum representing which type they are. idk if this helps

*/


enum TCReturn {
    TCResult(TCResult),
    TCState(TCState),
}
type TCReturnResult = Result<TCReturn, OxidizedError>;
type TCResult = Result<(ConcreteTyping, Type), OxidizedError>;
type TCFn =
    Box<dyn Fn(Option<Type>, Option<ConcreteTyping>, Option<ConcreteTyping>) -> TCReturnResult>;

struct TCStateMachine {
    states: Vec<TCState>,
    results: Vec<TCResult>,
}

struct TCState {
    e1_fn: Option<TCFn>,
    e2_fn: Option<TCFn>,
    current_state: (ConcreteTyping, ConcreteTyping),
}

impl TCStateMachine {
    fn step_over(&mut self) -> Result<(), OxidizedError> {
        let top = self.states.pop().unwrap();
        let e1_result: TCResult;

        if let Some(e1_fn) = top.e1_fn {
            let fn_return = e1_fn(None, None, None)?;
            match fn_return {
                TCReturn::TCResult(result) => {
                    e1_result = result;
                }
                TCReturn::TCState(state) => {
                    e1_result = state.evaluate();
                }
            };
        } else {
            e1_result = self.results.pop().unwrap();
        }

        let unwrapped_result = e1_result?;
        // TODO: probably should modify current state to represent the state at e2.
        let e2_result =
            top.e2_fn.unwrap()(Some(unwrapped_result.1), Some(unwrapped_result.0), None)?;

        match e2_result {
            TCReturn::TCResult(result) => Ok(self.results.push(result)),
            TCReturn::TCState(state) => Ok(self.states.push(state)),
        }
    }

    fn step_into(&mut self) -> Result<(), OxidizedError> {
        let top = self.states.pop().unwrap();
        let return_result = top.step_into()?;
        match return_result {
            TCReturn::TCResult(result) => Ok(self.results.push(result)),
            TCReturn::TCState(state) => Ok(self.states.push(state)),
        }
    }

    fn evaluate(&mut self) -> TCResult {
        let top = self.states.pop().unwrap();
        if let Some(_) = top.e2_fn {
            let states_len = self.states.len();
            self.step_over()?;
            if (self.states.len() != states_len) {
                self.states.pop().unwrap().evaluate()
            } else {
                self.results.pop().unwrap()
            }
        } else {
            match top.step_into()? {
                TCReturn::TCResult(result) => result,
                TCReturn::TCState(state) => state.evaluate(),
            }
        }
    }

    fn new(state: TCState) -> Self {
        Self {
            states: vec![state],
            results: Vec::new(),
        }
    }

    fn get_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.states.iter().last().unwrap().current_state.clone()
    }

    fn get_result(&self) -> TCResult {
        self.results.iter().last().unwrap().clone()
    }
}

impl TCState {
    fn step_into(mut self) -> TCReturnResult {
        let e1_fn: TCFn = self.e1_fn.unwrap(); // FIXME: unwrap
        self.e1_fn = None;
        e1_fn(None, None, None)
    }

    fn evaluate(self) -> TCResult {
        let mut state_machine = TCStateMachine::new(self);
        state_machine.evaluate()
    }

    fn new() -> Self {
        Self {
            e1_fn: None,
            e2_fn: None,
            current_state: (ConcreteTyping::new(), ConcreteTyping::new()),
        }
    }
}

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
            Expression::Variable(_) | Expression::PathAccess(..) | Expression::Deref(_) => {
                Self::t_place_expression(
                    TryInto::<PlaceExpression>::try_into(self.clone()).expect("Unexpected error"),
                    stack_typing,
                    continuation_typing,
                )
            }
            Expression::Constant(c) => Ok((stack_typing.clone(), Self::t_constant(c)?)),
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

    // Typing
    fn t_constant(constant: &Constant) -> Result<Type, OxidizedError> {
        match constant {
            crate::ast::Constant::Unit => Ok(Type {
                binding: None,
                variant: TypeVariant::Base(BaseType::Unit),
                is_sized: Ternary::No,
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

    fn t_place_expression(
        place_expression: PlaceExpression,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> Result<(ConcreteTyping, Type), OxidizedError> {
        let found_type = stack_typing.type_lookup(place_expression.clone())?;
        if found_type.is_dead != Ternary::No {
            return Err(OxidizedError::UseOfMovedValue(found_type));
        }
        if found_type.copyable() {
            return Self::t_copy(
                place_expression.try_into().unwrap(),
                stack_typing,
                continuation_typing,
            ); // FIXME: unwrap is temporary
        } else {
            if let Ok(place) = place_expression.try_into() {
                return Self::t_move(place, found_type, stack_typing, continuation_typing);
            } else {
                return Err(OxidizedError::CannotMove(found_type));
            }
        }
    }

    fn t_move(
        place: Place,
        found_type: Type,
        mut stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> Result<(ConcreteTyping, Type), OxidizedError> {
        let _ = stack_typing.o_safe_place(
            continuation_typing,
            place.clone(),
            OwnershipQualifier::Uniq,
        )?;
        let mut return_type = found_type.clone();
        return_type.binding = None;
        let mut dead_type = found_type.clone();
        dead_type.is_dead = Ternary::Yes;
        stack_typing = stack_typing.set_type(found_type, dead_type);

        Ok((stack_typing, return_type))
    }

    fn t_copy(
        place_expression: PlaceExpression,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> Result<(ConcreteTyping, Type), OxidizedError> {
        let _ = stack_typing.ownership_check(
            continuation_typing,
            place_expression.clone(),
            OwnershipQualifier::Shrd,
        )?;
        let found_type = stack_typing.type_lookup(place_expression)?;
        if found_type.is_dead != Ternary::No {
            return Err(OxidizedError::UseOfMovedValue(found_type));
        }
        let mut return_type = found_type.clone();
        return_type.binding = None;

        Ok((stack_typing, return_type))
    }

    fn t_borrow(
        &self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> Result<(ConcreteTyping, Type), OxidizedError> {
        if let Expression::Ref(ref_annotation, e) = self {
            let mut region = stack_typing.get_region(ref_annotation.0.to_string())?;
            if !region.loans.0.is_empty() {
                return Err(OxidizedError::NonemptyRegion(
                    ref_annotation.0.clone(),
                    region.loans.clone(),
                ));
            }
            // TODO: rnic

            let place_expression: PlaceExpression = (**e).clone().try_into()?;

            let loans = stack_typing.ownership_check(
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
            };

            region = region.set_loans(loans);

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
                region.scope + 1
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
    fn ownership_check(
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
            for loan in region.clone().loans.0 {
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
        region1: ConcreteRegion,
        region2: ConcreteRegion,
        mode: RegionRewriteMode,
    ) -> Result<Self, OxidizedError> {
        match mode {
            RegionRewriteMode::Plus => self.ol_combine_concrete(region1, region2),
        }
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
