use libloading::Symbol;
use std::{collections::BTreeMap, sync::Arc};
use crate::{pvxs_library::PvxsLibrary, std_types::{Member, StdBasicString, StdSSOString, StdSharedPtr, StoreType, Value}};

/// Equivalent of `CommonBase::Req` in C++
#[repr(C)]
#[derive(Debug)]
pub struct Req {
    pv_request: Value,
    fields: Member,
    options: BTreeMap<StdBasicString, Value>,
}

#[repr(C)]
#[derive(Debug)]
pub struct CommonBase {
    pub ctx: StdSharedPtr<*mut std::ffi::c_void>,
    pub _name: StdSSOString,
    pub _server: StdSSOString,
    pub req: StdSharedPtr<*mut Req>,
    pub _prio: std::ffi::c_uint,
    pub _autoexec: bool,
    pub _sync_cancel: bool,
}

#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_detail_CommonBase"]
        [::std::mem::size_of::<CommonBase>() - 104usize];
    ["Alignment of pvxs_client_detail_CommonBase"]
        [::std::mem::align_of::<CommonBase>() - 8usize];
    ["Offset of field: pvxs_client_detail_CommonBase::ctx"]
        [::std::mem::offset_of!(CommonBase, ctx) - 0usize];
    ["Offset of field: pvxs_client_detail_CommonBase::_name"]
        [::std::mem::offset_of!(CommonBase, _name) - 16usize];
    ["Offset of field: pvxs_client_detail_CommonBase::_server"]
        [::std::mem::offset_of!(CommonBase, _server) - 48usize];
    ["Offset of field: pvxs_client_detail_CommonBase::req"]
        [::std::mem::offset_of!(CommonBase, req) - 80usize];
    ["Offset of field: pvxs_client_detail_CommonBase::_prio"]
        [::std::mem::offset_of!(CommonBase, _prio) - 96usize];
    ["Offset of field: pvxs_client_detail_CommonBase::_autoexec"]
        [::std::mem::offset_of!(CommonBase, _autoexec) - 100usize];
    ["Offset of field: pvxs_client_detail_CommonBase::_syncCancel"]
        [::std::mem::offset_of!(CommonBase, _sync_cancel) - 101usize];
};

impl CommonBase {
    pub unsafe fn _raw_request(&mut self, arg1: *const Value, pvxs_library: Arc<PvxsLibrary>) {
        pvxs_client_detail_common_base_raw_request(self, arg1, pvxs_library)
    }
    #[inline]
    pub unsafe fn _field(&mut self, s: *const StdSSOString, pvxs_library: Arc<PvxsLibrary>) {
        pvxs_client_detail_common_base_field(self, s, pvxs_library)
    }
    #[inline]
    pub unsafe fn _record(
        &mut self,
        key: *const StdSSOString,
        value: *const ::std::os::raw::c_void,
        vtype: StoreType,
        pvxs_library: Arc<PvxsLibrary>,
    ) {
        pvxs_client_detail_common_base_record(self, key, value, vtype, pvxs_library)
    }
    #[inline]
    pub unsafe fn _parse(&mut self, req: *const StdSSOString, pvxs_library: Arc<PvxsLibrary>) {
        pvxs_client_detail_common_base_parse(self, req, pvxs_library)
    }
    #[inline]
    pub unsafe fn _build_req(&self, pvxs_library: Arc<PvxsLibrary>) -> Value {
        pvxs_client_detail_common_base_build_req(self, pvxs_library)
    }
    #[inline]
    pub unsafe fn destruct(&mut self, pvxs_library: Arc<PvxsLibrary>) {
        pvxs_client_detail_common_base_common_base_destructor(self, pvxs_library)
    }
}

pub unsafe fn pvxs_client_detail_common_base_raw_request(this: *mut CommonBase, arg1: *const Value, pvxs_library: Arc<PvxsLibrary>)
{
    let func: Symbol<unsafe extern "C" fn(*mut CommonBase, *const Value) -> *const std::os::raw::c_char> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"?_rawRequest@CommonBase@detail@client@pvxs@@IEAAXAEBVValue@4@@Z"
    } else if cfg!(target_os = "linux") {
        b""
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for Context::info");
    func(this, arg1);
}

pub unsafe  fn pvxs_client_detail_common_base_field(this: *mut CommonBase,s: *const StdSSOString, pvxs_library: Arc<PvxsLibrary>)
{
    let func: Symbol<unsafe extern "C" fn(*mut CommonBase, *const StdSSOString) -> *const std::os::raw::c_char> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"?_field@CommonBase@detail@client@pvxs@@IEAAXAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z"
    } else if cfg!(target_os = "linux") {
        b""
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for Context::info");
    func(this, s);
}

pub unsafe fn pvxs_client_detail_common_base_record(this: *mut CommonBase, key: *const StdSSOString, 
    value: *const ::std::os::raw::c_void, vtype: StoreType, pvxs_library: Arc<PvxsLibrary>)
{
    let func: Symbol<unsafe extern "C" fn(*mut CommonBase, *const StdSSOString, *const ::std::os::raw::c_void, StoreType) -> *const std::os::raw::c_char> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"?_record@CommonBase@detail@client@pvxs@@IEAAXAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@PEBXW4StoreType@4@@Z"
    } else if cfg!(target_os = "linux") {
        b""
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for Context::info");
    func(this, key, value, vtype);
}

pub unsafe fn pvxs_client_detail_common_base_parse(this: *mut CommonBase, req: *const StdSSOString, pvxs_library: Arc<PvxsLibrary>)
{
    let func: Symbol<unsafe extern "C" fn(*mut CommonBase, *const StdSSOString) -> *const std::os::raw::c_char> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"?_parse@CommonBase@detail@client@pvxs@@IEAAXAEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z"
    } else if cfg!(target_os = "linux") {
        b""
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for Context::info");
    func(this, req);
}

pub unsafe fn pvxs_client_detail_common_base_build_req(this: *const CommonBase, pvxs_library: Arc<PvxsLibrary>) -> Value
{
    let func: Symbol<unsafe extern "C" fn(*const CommonBase) -> Value> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"?_buildReq@CommonBase@detail@client@pvxs@@IEBA?AVValue@4@XZ"
    } else if cfg!(target_os = "linux") {
        b""
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for Context::info");
    func(this)
}

pub unsafe fn pvxs_client_detail_common_base_common_base_destructor(this: *mut CommonBase, pvxs_library: Arc<PvxsLibrary>)
{
    let func: Symbol<unsafe extern "C" fn(*mut CommonBase) -> *const std::os::raw::c_char> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"??1CommonBase@detail@client@pvxs@@IEAA@XZ"
    } else if cfg!(target_os = "linux") {
        b""
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for Context::info");
    func(this);
}
