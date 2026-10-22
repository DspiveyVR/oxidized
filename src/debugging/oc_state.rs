// type TC3ConsumerFn = dyn FnOnce(ConcreteTyping, Type, Type) -> TCReturnResult;

// // pub struct ODerefState {
// //     check_fn: Option<Box<TCFn>>,
// //     env_state: (ConcreteTyping, ConcreteTyping),
// //     method_name: String,
// // }

// /*

//     fn o_safe_place(
//         &self,
//         continuation_typing: &Self,
//         place: Place,
//         ownership_qualifier: OwnershipQualifier,
//         reborrow_exclusion_list: &VecSet<Place>,
//     ) -> Result<VecSet<Loan>, OxidizedError> {


// */

// pub struct OSafePlaceState {
//     check_fn: Option<Box<TCFn>>,
//     env_state: (ConcreteTyping, ConcreteTyping),
//     method_name: String,
// }