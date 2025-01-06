use std::os::raw::c_char;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("pvxs_wrapper.h");

        // Bind the pvxs::version_str() function
        unsafe fn pvxs_version_str() -> *const c_char;
    }
}

pub  use ffi::*;
