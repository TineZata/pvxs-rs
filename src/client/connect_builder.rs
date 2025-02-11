use core::sync;
use std::sync::Arc;
use libloading::Symbol;

use crate::bin::LoadLib;

use crate::std_types::{StdFunction, StdSharedPtr, StdString};
#[doc = "! cf. Context::connect()\n! @since 0.2.0"]
#[repr(C)]
#[derive(Debug)]
pub struct ConnectBuilder {
    pub ctx: StdSharedPtr,
    pub _pvname: StdString,
    pub _server: StdString,
    pub _on_conn: StdFunction,
    pub _on_dis: StdFunction,
    pub _sync_cancel: bool,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_ConnectBuilder"]
        [::std::mem::size_of::<ConnectBuilder>() - 216usize];
    ["Alignment of pvxs_client_ConnectBuilder"]
        [::std::mem::align_of::<ConnectBuilder>() - 8usize];
    ["Offset of field: pvxs_client_ConnectBuilder::ctx"]
        [::std::mem::offset_of!(ConnectBuilder, ctx) - 0usize];
    ["Offset of field: pvxs_client_ConnectBuilder::_pvname"]
        [::std::mem::offset_of!(ConnectBuilder, _pvname) - 16usize];
    ["Offset of field: pvxs_client_ConnectBuilder::_server"]
        [::std::mem::offset_of!(ConnectBuilder, _server) - 48usize];
    ["Offset of field: pvxs_client_ConnectBuilder::_onConn"]
        [::std::mem::offset_of!(ConnectBuilder, _on_conn) - 80usize];
    ["Offset of field: pvxs_client_ConnectBuilder::_onDis"]
        [::std::mem::offset_of!(ConnectBuilder, _on_dis) - 144usize];
    ["Offset of field: pvxs_client_ConnectBuilder::_syncCancel"]
        [::std::mem::offset_of!(ConnectBuilder, _sync_cancel) - 208usize];
};

pub unsafe fn pvxs_client_connect_builder_exec(this: *mut ConnectBuilder, pvxs_library: Arc<LoadLib>)
    -> u8 {
    let func: Symbol<unsafe extern "C" fn(*mut ConnectBuilder) -> u8> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"?exec@ConnectBuilder@client@pvxs@@QEAA?AV?$shared_ptr@UConnect@client@pvxs@@@std@@XZ"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs6client13ConnectBuilder4execESt10shared_ptrINS0_7ConnectEE"
        } else {
            panic!("Unsupported platform");
        })
        .expect("Failed to find symbol for ConnectBuilder::exec");

    func(this)
}

impl ConnectBuilder {
    pub fn new(ctx: StdSharedPtr, 
        pvname: StdString, 
        server: StdString, 
        on_conn: StdFunction, 
        on_dis: StdFunction,
        sync_cancel: bool,) -> Self {
        Self {
            ctx,
            _pvname: pvname,
            _server: server,
            _on_conn: on_conn,
            _on_dis: on_dis,
            _sync_cancel: sync_cancel,
        }
    }
    #[inline]
    pub unsafe fn exec(&mut self, pvxs_library: Arc<LoadLib>) -> u8 {
        pvxs_client_connect_builder_exec(self, pvxs_library)
    }
}