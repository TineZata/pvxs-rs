use crate::std_types::StdFunction64;
use super::detail::common_base::CommonBase;

#[doc = "! Prepare a remote GET or GET_FIELD (info) operation.\n! See Context::get()"]
#[repr(C)]
#[derive(Debug)]
pub struct GetBuilder {
    pub _base: CommonBuilder<CommonBase>,
    pub _on_init: StdFunction64,
    pub _result: StdFunction64,
    pub _get: bool,
}
/*#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_GetBuilder"][::std::mem::size_of::<pvxs_client_GetBuilder>() - 240usize];
    ["Alignment of pvxs_client_GetBuilder"]
        [::std::mem::align_of::<pvxs_client_GetBuilder>() - 8usize];
    ["Offset of field: pvxs_client_GetBuilder::_onInit"]
        [::std::mem::offset_of!(pvxs_client_GetBuilder, _onInit) - 104usize];
    ["Offset of field: pvxs_client_GetBuilder::_result"]
        [::std::mem::offset_of!(pvxs_client_GetBuilder, _result) - 168usize];
    ["Offset of field: pvxs_client_GetBuilder::_get"]
        [::std::mem::offset_of!(pvxs_client_GetBuilder, _get) - 232usize];
};*/

#[doc = "! Options common to all operations"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CommonBuilder<Base> {
    pub _phantom_0: ::std::marker::PhantomData<::std::cell::UnsafeCell<Base>>,
    pub _base: Base,
}
