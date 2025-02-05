use std::{ffi::CStr, sync::Arc};
use crate::pvxs_library::PvxsLibrary;

#[repr(C)]
pub struct Version {
}

impl Version {
    /// Resolve the version string from the PVXS library
    /// 
    /// Returns a rust string representation of the version
    pub unsafe fn version_str(pvxs_library: Arc<PvxsLibrary>) -> String{
        let str_ptr = pvxs_version_str(pvxs_library);
        if str_ptr.is_null() {
            return "Unknown PVXS version".to_string();
        }
        CStr::from_ptr(str_ptr).to_string_lossy().into_owned()
    }

    /// Resolve the version integer from the PVXS library
    /// 
    /// Returns the version as an unsigned long
    pub unsafe fn version_int(pvxs_library: Arc<PvxsLibrary>) -> ::std::os::raw::c_ulong {
        pvxs_version_int(pvxs_library)
    }

    /// Resolve the ABI version integer from the PVXS library
    /// 
    /// Returns the ABI version as an unsigned long
    pub unsafe fn version_abi_int(pvxs_library: Arc<PvxsLibrary>) -> ::std::os::raw::c_ulong {
        pvxs_version_abi_int(pvxs_library)
    }
}

pub unsafe fn pvxs_version_str(pvxs_library: Arc<PvxsLibrary>) -> *const ::std::os::raw::c_char {
    let func: libloading::Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"?version_str@pvxs@@YAPEBDXZ"
    } else if cfg!(target_os = "linux") {
        b"_ZN4pvxs11version_strEv"
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for pvxs::version_str");
    func()
}

pub unsafe fn pvxs_version_int(pvxs_library: Arc<PvxsLibrary>) -> ::std::os::raw::c_ulong {
    let func: libloading::Symbol<unsafe extern "C" fn() -> ::std::os::raw::c_ulong> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"?version_int@pvxs@@YAKXZ"
    } else if cfg!(target_os = "linux") {
        b"_ZN4pvxs11version_intEv"
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for pvxs::version_int");
    func()
}

pub unsafe fn pvxs_version_abi_int(pvxs_library: Arc<PvxsLibrary>) -> ::std::os::raw::c_ulong {
    let func: libloading::Symbol<unsafe extern "C" fn() -> ::std::os::raw::c_ulong> = 
    pvxs_library.lib
    .get(if cfg!(target_os = "windows") {
        b"?version_abi_int@pvxs@@YAKXZ"
    } else if cfg!(target_os = "linux") {
        b"_ZN4pvxs16version_abi_intEv"
    } else {
        panic!("Unsupported platform");
    })
    .expect("Failed to find symbol for pvxs::version_abi_int");
    func()
}

