use libloading::Symbol;
use std::sync::Arc;
use crate::bin::LoadLib;

use crate::std_types::{StdRuntimeError, StdString, StdString32};
use crate::epics::epics_time::EpicsTime;

/// For monitor only.  Subscription has (re)connected.
#[repr(C)]
#[derive(Debug)]
pub struct Connected {
    pub _base: StdRuntimeError,
    pub peer_name: StdString32,
    pub time: EpicsTime,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of pvxs_client_Connected"][::std::mem::size_of::<Connected>() - 64usize];
    ["Alignment of pvxs_client_Connected"]
        [::std::mem::align_of::<Connected>() - 8usize];
    ["Offset of field: pvxs_client_Connected::peerName"]
        [::std::mem::offset_of!(Connected, peer_name) - 24usize];
    ["Offset of field: pvxs_client_Connected::time"]
        [::std::mem::offset_of!(Connected, time) - 56usize];
};

pub unsafe fn pvxs_client_connected_connected(this: *mut Connected, peer_name: *const StdString, pvxs_library: Arc<LoadLib>) -> Connected {
    // Load the symbol for `Connected`
    let func: Symbol<unsafe extern "C" fn(*mut Connected, *const StdString) -> Connected> = 
        pvxs_library.lib
        .get(if cfg!(target_os = "windows") {
            b"??0Connected@client@pvxs@@QEAA@AEBV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@@Z"
        } else if cfg!(target_os = "linux") {
            b"_ZN4pvxs6client8ConnectedC1ERKSt6string"
        } else {
            panic!("Unsupported platform");
        })
        .expect("Failed to find symbol for Connected::Connected");
    func(this, peer_name)
}

impl Connected {
    #[inline]
    pub unsafe fn new(peer_name: *const StdString, pvxs_library: Arc<LoadLib>) -> Self {
        let mut tmp = ::std::mem::MaybeUninit::uninit();
        pvxs_client_connected_connected(tmp.as_mut_ptr(), peer_name, pvxs_library);
        tmp.assume_init()
    }
}
unsafe extern "C" {
    #[link_name = "\u{1}??1Connected@client@pvxs@@UEAA@XZ"]
    pub fn pvxs_client_Connected_Connected_destructor(this: *mut Connected);
}