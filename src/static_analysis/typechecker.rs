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

// FIXME: figure out where to put err_if_dead calls

pub(crate) enum TCReturn {
    TCResult(TCResult),
    TCState(Box<dyn TCState>),
}
pub(crate) type TCReturnResult = Result<(Option<Box<dyn TCState>>, TCReturn), OxidizedError>;
pub(crate) type TCResult = Result<(ConcreteTyping, Type), OxidizedError>;
// FIXME: For now i'll ignore the continuation typing
type TCFn = dyn FnOnce() -> TCReturnResult;
type TCConsumerFn = dyn FnOnce(ConcreteTyping) -> TCReturnResult;
type TCPopVarsFn = dyn FnOnce(TCResult) -> TCReturnResult;
type TC2ConsumerFn = dyn FnOnce(ConcreteTyping, Type) -> TCReturnResult;
type TC3ConsumerFn = dyn FnOnce(ConcreteTyping, Type, Type) -> TCReturnResult;

pub(crate) struct TCStateMachine {
    pub(crate) states: Vec<Box<dyn TCState>>,
}

pub(crate) trait TCState {
    fn step_into(self: Box<Self>) -> TCReturnResult;

    fn step_over(self: Box<Self>) -> TCReturnResult;

    fn evaluate(self: Box<Self>) -> TCResult;

    fn handle_result(
        self: Box<Self>,
        result: TCResult,
    ) -> Result<(Box<dyn TCState>, Option<TCReturn>), OxidizedError>;

    fn is_complete(&self) -> bool;

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping);
}

struct TExprState {
    e_fn: Option<Box<TCFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    is_complete: bool,
}

struct TLetrgnState {
    e_fn: Option<Box<TCFn>>,
    pop_vars_fn: Option<Box<TCPopVarsFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    e_result: Option<TCResult>,
    is_complete: bool,
}

struct TSeqState {
    e1_fn: Option<Box<TCFn>>,
    e2_fn: Option<Box<TCConsumerFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    e1_result: Option<TCResult>,
    is_complete: bool,
}

struct TLetState {
    e1_fn: Option<Box<TCFn>>,
    binding_fn: Option<Box<TC2ConsumerFn>>,
    e2_fn: Option<Box<TCConsumerFn>>,
    gc_fn: Option<Box<TC3ConsumerFn>>,
    env_state: (ConcreteTyping, ConcreteTyping),
    bound_type: Option<Type>,
    e1_result: Option<TCResult>,
    e2_result: Option<TCResult>,
    binding_result: Option<TCResult>,
    is_complete: bool,
}

impl TCState for TExprState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        let mut e_return = (self.e_fn.take().unwrap())()?;
        if let (_, TCReturn::TCResult(_)) = e_return {
            self.is_complete = true;
        }
        e_return.0 = Some(self);
        Ok(e_return)
    }

    fn step_over(self: Box<Self>) -> TCReturnResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        let mut state_machine = TCStateMachine::new(self);
        state_machine.evaluate()
    }

    fn handle_result(
        self: Box<Self>,
        _: TCResult,
    ) -> Result<(Box<dyn TCState>, Option<TCReturn>), OxidizedError> {
        Ok((self, None))
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }
}

impl TExprState {
    fn new_obj<TCFn: FnOnce() -> TCReturnResult + 'static>(
        e_fn: TCFn,
        env_state: (ConcreteTyping, ConcreteTyping),
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e_fn: Some(Box::new(e_fn)),
                env_state,
                is_complete: false,
            })),
        ))
    }
}

impl TCState for TLetrgnState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        let mut e_return = (self.e_fn.take().unwrap())()?;
        if let (_, TCReturn::TCResult(_)) = e_return {
            self.is_complete = true;
        }
        e_return.0 = Some(self);
        Ok(e_return)
    }

    fn step_over(self: Box<Self>) -> TCReturnResult {
        self.step_into()
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        let mut state_machine = TCStateMachine::new(self);
        state_machine.evaluate()
    }

    fn handle_result(
        mut self: Box<Self>,
        result: TCResult,
    ) -> Result<(Box<dyn TCState>, Option<TCReturn>), OxidizedError> {
        if let Some(result) = self.e_result.clone() {
            let pop_var_val = (self.pop_vars_fn.take().unwrap())(result)?;
            return Ok((pop_var_val.0.unwrap(), Some(pop_var_val.1)));
        } else {
            self.e_result = Some(result.clone())
        }
        Ok((self, None))
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }
}

impl TLetrgnState {
    fn new_obj<
        TCFn: FnOnce() -> TCReturnResult + 'static,
        TCPopVarsFn: FnOnce(TCResult) -> TCReturnResult + 'static,
    >(
        e_fn: TCFn,
        pop_vars_fn: TCPopVarsFn,
        env_state: (ConcreteTyping, ConcreteTyping),
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e_fn: Some(Box::new(e_fn)),
                pop_vars_fn: Some(Box::new(pop_vars_fn)),
                env_state,
                e_result: None,
                is_complete: false,
            })),
        ))
    }
}

impl TCState for TSeqState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        let mut e1_result = (self.e1_fn.take().unwrap())()?;
        e1_result.0 = Some(self);
        Ok(e1_result)
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        let mut state_machine = TCStateMachine::new(self);
        state_machine.evaluate()
    }

    fn handle_result(
        mut self: Box<Self>,
        result: TCResult,
    ) -> Result<(Box<dyn TCState>, Option<TCReturn>), OxidizedError> {
        self.e1_result = Some(result);
        Ok((self, None))
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn step_over(mut self: Box<Self>) -> TCReturnResult {
        if let Some(result) = self.e1_result.clone() {
            let mut e2_return = (self.e2_fn.take().unwrap())(result?.0)?;
            if let (_, TCReturn::TCResult(_)) = e2_return {
                self.is_complete = true;
            }
            e2_return.0 = Some(self);
            Ok(e2_return)
        } else {
            let e1_return = (self.e1_fn.take().unwrap())()?;
            match e1_return {
                (_, TCReturn::TCResult(result)) => {
                    let e1_value = result?;
                    let mut e2_return = (self.e2_fn.take().unwrap())(e1_value.0)?;
                    if let (_, TCReturn::TCResult(_)) = e2_return {
                        self.is_complete = true;
                    }
                    e2_return.0 = Some(self);
                    Ok(e2_return)
                }
                (_, TCReturn::TCState(state)) => {
                    let e1_value = state.evaluate()?;
                    let mut e2_return = (self.e2_fn.take().unwrap())(e1_value.0)?;
                    if let (_, TCReturn::TCResult(_)) = e2_return {
                        self.is_complete = true;
                    }
                    e2_return.0 = Some(self);
                    Ok(e2_return)
                }
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }
}

impl TSeqState {
    fn new_obj<
        TCFn: FnOnce() -> TCReturnResult + 'static,
        TCConsumerFn: FnOnce(ConcreteTyping) -> TCReturnResult + 'static,
    >(
        e1_fn: TCFn,
        e2_fn: TCConsumerFn,
        env_state: (ConcreteTyping, ConcreteTyping),
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e1_fn: Some(Box::new(e1_fn)),
                e2_fn: Some(Box::new(e2_fn)),
                env_state,
                is_complete: false,
                e1_result: None,
            })),
        ))
    }
}

impl TCState for TLetState {
    fn step_into(mut self: Box<Self>) -> TCReturnResult {
        let mut e1_result = (self.e1_fn.take().unwrap())()?;
        if let (_, TCReturn::TCResult(_)) = e1_result {
            self.is_complete = true;
        }
        e1_result.0 = Some(self);
        Ok(e1_result)
    }

    fn evaluate(self: Box<Self>) -> TCResult {
        let mut state_machine = TCStateMachine::new(self);
        state_machine.evaluate()
    }

    fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        self.env_state.clone()
    }

    fn step_over(mut self: Box<Self>) -> TCReturnResult {
        if let Some(result) = self.binding_result.clone() {
            let mut e2_return = (self.e2_fn.take().unwrap())(result?.0)?;
            if let (_, TCReturn::TCResult(_)) = e2_return {
                self.is_complete = true;
            }
            e2_return.0 = Some(self);
            Ok(e2_return)
        } else {
            let e1_return = (self.e1_fn.take().unwrap())()?;
            match e1_return {
                (_, TCReturn::TCResult(result)) => {
                    let e1_value = err_if_dead(result)?;
                    let binding_return = (self.binding_fn.take().unwrap())(e1_value.0, e1_value.1)?;
                    match binding_return {
                        (_, TCReturn::TCResult(result)) => {
                            let binding_value = result?;
                            let mut e2_return = (self.e2_fn.take().unwrap())(binding_value.0)?;
                            if let (_, TCReturn::TCResult(_)) = e2_return {
                                self.is_complete = true;
                            }
                            e2_return.0 = Some(self);
                            Ok(e2_return)
                        }
                        (_, TCReturn::TCState(state)) => {
                            let binding_value = state.evaluate()?;
                            let mut e2_return = (self.e2_fn.take().unwrap())(binding_value.0)?;
                            if let (_, TCReturn::TCResult(_)) = e2_return {
                                self.is_complete = true;
                            }
                            e2_return.0 = Some(self);
                            Ok(e2_return)
                        }
                    }
                }
                (_, TCReturn::TCState(state)) => {
                    let e1_value = err_if_dead(state.evaluate())?;
                    let binding_return = (self.binding_fn.take().unwrap())(e1_value.0, e1_value.1)?;
                    match binding_return {
                        (_, TCReturn::TCResult(result)) => {
                            let binding_value = result?;
                            let mut e2_return = (self.e2_fn.take().unwrap())(binding_value.0)?;
                            if let (_, TCReturn::TCResult(_)) = e2_return {
                                self.is_complete = true;
                            }
                            e2_return.0 = Some(self);
                            Ok(e2_return)
                        }
                        (_, TCReturn::TCState(state)) => {
                            let binding_value = state.evaluate()?;
                            let mut e2_return = (self.e2_fn.take().unwrap())(binding_value.0)?;
                            if let (_, TCReturn::TCResult(_)) = e2_return {
                                self.is_complete = true;
                            }
                            e2_return.0 = Some(self);
                            Ok(e2_return)
                        }
                    }
                }
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn handle_result(
        mut self: Box<Self>,
        result: TCResult,
    ) -> Result<(Box<dyn TCState>, Option<TCReturn>), OxidizedError> {
        if let Some(_) = self.e2_result {
            Ok((self, None))
        } else if let Some(binding_result) = self.binding_result.clone() {
            self.e2_result = Some(result.clone());
            let result_val = err_if_dead(result)?;
            let binding_result_val = binding_result?;

            let gc_return_val =
                (self.gc_fn.take().unwrap())(result_val.0, binding_result_val.1, result_val.1)?;
            Ok((gc_return_val.0.unwrap(), Some(gc_return_val.1)))
        } else if let Some(_) = self.e1_result {
            self.binding_result = Some(result.clone());
            Ok((self, None))
        } else {
            self.e1_result = Some(result.clone());
            let result_val = err_if_dead(result)?;

            let binding_return_val = (self.binding_fn.take().unwrap())(result_val.0, result_val.1)?;
            Ok((binding_return_val.0.unwrap(), Some(binding_return_val.1)))
        }
    }
}

impl TLetState {
    fn new_obj<
        TCFn: FnOnce() -> TCReturnResult + 'static,
        TCConsumerFn: FnOnce(ConcreteTyping) -> TCReturnResult + 'static,
        TC2ConsumerFn: FnOnce(ConcreteTyping, Type) -> TCReturnResult + 'static,
        TC3ConsumerFn: FnOnce(ConcreteTyping, Type, Type) -> TCReturnResult + 'static,
    >(
        e1_fn: TCFn,
        binding_fn: TC2ConsumerFn,
        e2_fn: TCConsumerFn,
        gc_fn: TC3ConsumerFn,
        env_state: (ConcreteTyping, ConcreteTyping),
    ) -> TCReturnResult {
        Ok((
            None,
            TCReturn::TCState(Box::new(Self {
                e1_fn: Some(Box::new(e1_fn)),
                binding_fn: Some(Box::new(binding_fn)),
                e2_fn: Some(Box::new(e2_fn)),
                gc_fn: Some(Box::new(gc_fn)),
                env_state,
                is_complete: false,
                e1_result: None,
                binding_result: None,
                bound_type: None,
                e2_result: None,
            })),
        ))
    }
}

impl TCStateMachine {
    pub(crate) fn new(state: Box<dyn TCState>) -> Self {
        Self {
            states: vec![state],
        }
    }

    pub(crate) fn evaluate(&mut self) -> TCResult {
        while self.states.len() > 0 {
            match self.step_over() {
                Ok(Some(result)) => return result,
                Ok(None) => continue,
                Err(err) => return Err(err),
            }
        }
        panic!("failed to evaluate");
    }

    pub(crate) fn get_env_state(&self) -> (ConcreteTyping, ConcreteTyping) {
        if let Some(state) = self.states.last() {
            state.get_env_state()
        } else {
            (ConcreteTyping::new(), ConcreteTyping::new())
        }
    }

    pub(crate) fn step_into(&mut self) -> Result<(), OxidizedError> {
        let top = self.states.pop().unwrap();
        let return_result = top.step_into()?;
        match return_result {
            (_, TCReturn::TCResult(result)) => {
                let next_top = self.states.pop().unwrap();
                let is_complete = next_top.is_complete();
                let handle_result_state = next_top.handle_result(result)?;
                if !is_complete {
                    self.states.push(handle_result_state.0);
                }
                match handle_result_state.1 {
                    Some(TCReturn::TCResult(result)) => {
                        // I think this only needs to be done once and not recursively since you can't have
                        // multiple gc fn or pop fn returns cascade into each other.  IDK tho....
                        let next_top = self.states.pop().unwrap();
                        let is_complete = next_top.is_complete();
                        let handle_result_state = next_top.handle_result(result)?;
                        if !is_complete {
                            self.states.push(handle_result_state.0);
                        }
                        if let (_, Some(TCReturn::TCState(state))) = handle_result_state {
                            self.states.push(state);
                        }
                    }
                    Some(TCReturn::TCState(state)) => {
                        self.states.push(state);
                    }
                    _ => (),
                }
                Ok(())
            }
            (top, TCReturn::TCState(state)) => {
                self.states.push(top.unwrap());
                self.states.push(state);
                Ok(())
            }
        }
    }

    pub(crate) fn step_over(&mut self) -> Result<Option<TCResult>, OxidizedError> {
        let top = self.states.pop().unwrap();
        let return_result = top.step_over()?;
        match return_result {
            (_, TCReturn::TCResult(result)) => {
                if let Some(next_top) = self.states.pop() {
                    let is_complete = next_top.is_complete();
                    let handle_result_state = next_top.handle_result(result)?;
                    if !is_complete {
                        self.states.push(handle_result_state.0);
                    }
                    match handle_result_state.1 {
                        Some(TCReturn::TCResult(result)) => {
                            // I think this only needs to be done once and not recursively since you can't have
                            // multiple gc fn or pop fn returns cascade into each other.  IDK tho....
                            let next_top = self.states.pop().unwrap();
                            let is_complete = next_top.is_complete();
                            let handle_result_state = next_top.handle_result(result.clone())?;
                            if !is_complete {
                                self.states.push(handle_result_state.0);
                            }
                            if let (_, Some(TCReturn::TCState(state))) = handle_result_state {
                                self.states.push(state);
                            }
                            Ok(Some(result))
                        }
                        Some(TCReturn::TCState(state)) => {
                            self.states.push(state);
                            Ok(None)
                        }
                        _ => Ok(None),
                    }
                } else {
                    Ok(Some(result))
                }
            }
            (_, TCReturn::TCState(state)) => {
                self.states.push(state);
                Ok(None)
            }
        }
    }
}

impl Expression {
    pub(crate) fn typecheck(
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
                Self::t_place_expression(
                    TryInto::<PlaceExpression>::try_into(self.clone()).expect("Unexpected error"),
                    stack_typing,
                    continuation_typing,
                )
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
    ) -> TCReturnResult {
        let found_type = stack_typing.type_lookup(place_expression.clone())?;
        if found_type.is_dead != Ternary::No {
            return Err(OxidizedError::UseOfMovedValue(found_type));
        }
        if found_type.copyable() {
            Self::t_copy(
                place_expression.try_into().unwrap(),
                stack_typing,
                continuation_typing,
            )
        } else {
            if let Ok(place) = place_expression.try_into() {
                Self::t_move(place, found_type, stack_typing, continuation_typing)
            } else {
                Err(OxidizedError::CannotMove(found_type))
            }
        }
    }

    fn t_move(
        place: Place,
        found_type: Type,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        let st_clone = stack_typing.clone();
        let ct_clone = continuation_typing.clone();
        TExprState::new_obj(
            move || {
                let mut stack_typing = st_clone;
                let continuation_typing = ct_clone;
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

                Ok((None, TCReturn::TCResult(Ok((stack_typing, return_type)))))
            },
            (stack_typing, continuation_typing),
        )
    }

    fn t_copy(
        place_expression: PlaceExpression,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        let st_clone = stack_typing.clone();
        let ct_clone = continuation_typing.clone();
        TExprState::new_obj(
            move || {
                let stack_typing = st_clone;
                let continuation_typing = ct_clone;
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

                Ok((None, TCReturn::TCResult(Ok((stack_typing, return_type)))))
            },
            (stack_typing, continuation_typing),
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
        )
    }

    pub(crate) fn t_letrgn(
        self,
        stack_typing: ConcreteTyping,
        continuation_typing: ConcreteTyping,
    ) -> TCReturnResult {
        let st_clone = stack_typing.clone();
        let cont_clone = continuation_typing.clone();
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
                let result = result?;
                let env = result.0;
                let return_type = result.1;

                let new_stack_typing = ConcreteTyping {
                    regions: VecSet((env.regions.0[..env.regions.0.len()]).to_vec()),
                    types: VecSet((env.types.0[..env.types.0.len()]).to_vec()),
                };

                Ok((
                    None,
                    TCReturn::TCResult(Ok((new_stack_typing, return_type))),
                ))
            },
            (stack_typing.clone(), continuation_typing.clone()),
        )
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
                    // TODO: check for all free regions in ta. r rnrb
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
