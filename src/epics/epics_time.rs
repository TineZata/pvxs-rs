use crate::std_types::{StdLogicError, StdRuntimeError};
use super::epics_type::EpicsUInt32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _FILETIME {
    pub dw_low_date_time: ::std::os::raw::c_ulong,
    pub dw_high_date_time: ::std::os::raw::c_ulong,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of _FILETIME"][::std::mem::size_of::<_FILETIME>() - 8usize];
    ["Alignment of _FILETIME"][::std::mem::align_of::<_FILETIME>() - 4usize];
    ["Offset of field: _FILETIME::dwLowDateTime"]
        [::std::mem::offset_of!(_FILETIME, dw_low_date_time) - 0usize];
    ["Offset of field: _FILETIME::dwHighDateTime"]
        [::std::mem::offset_of!(_FILETIME, dw_high_date_time) - 4usize];
};

#[doc = " \\struct timeval\n \\brief BSD and SRV5 Unix timestamp\n\n BSD and SRV5 Unix timestamp. It has two fields:\n \\li <tt>time_t tv_sec</tt> - Number of seconds since 1970 (The POSIX epoch)\n \\li <tt>time_t tv_nsec</tt> - nanoseconds within a second"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Timeval {
    pub tv_sec: ::std::os::raw::c_long,
    pub tv_usec: ::std::os::raw::c_long,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of timeval"][::std::mem::size_of::<Timeval>() - 8usize];
    ["Alignment of timeval"][::std::mem::align_of::<Timeval>() - 4usize];
    ["Offset of field: timeval::tv_sec"][::std::mem::offset_of!(Timeval, tv_sec) - 0usize];
    ["Offset of field: timeval::tv_usec"][::std::mem::offset_of!(Timeval, tv_usec) - 4usize];
};

#[doc = " \\brief EPICS time stamp, for use from C code.\n\n Because it uses an unsigned 32-bit integer to hold the seconds count, an\n epicsTimeStamp can safely represent time stamps until the year 2106."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EpicsTimeStamp {
    #[doc = "< \\brief seconds since 0000 Jan 1, 1990"]
    pub sec_past_epoch: EpicsUInt32,
    #[doc = "< \\brief nanoseconds within second"]
    pub nsec: EpicsUInt32,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of epicsTimeStamp"][::std::mem::size_of::<EpicsTimeStamp>() - 8usize];
    ["Alignment of epicsTimeStamp"][::std::mem::align_of::<EpicsTimeStamp>() - 4usize];
    ["Offset of field: epicsTimeStamp::secPastEpoch"]
        [::std::mem::offset_of!(EpicsTimeStamp, sec_past_epoch) - 0usize];
    ["Offset of field: epicsTimeStamp::nsec"]
        [::std::mem::offset_of!(EpicsTimeStamp, nsec) - 4usize];
};

#[doc = " \\brief C++ time stamp object\n\n Holds an EPICS time stamp, and provides conversion functions for both\n input and output from/to other types.\n\n \\note Time conversions: The epicsTime implementation will properly\n convert between the various formats from the beginning of the EPICS\n epoch until at least 2038. Unless the underlying architecture support\n has defective POSIX, BSD/SRV5, or standard C time support the EPICS\n implementation should be valid until 2106."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EpicsTime {
    pub ts: EpicsTimeStamp,
}
#[doc = " \\brief Exception: Time provider problem"]
pub type EpicsTimeUnableToFetchCurrentTime = StdRuntimeError;
#[doc = " \\brief Exception: Bad field(s) in <tt>struct tm</tt>"]
pub type EpicsTimeFormatProblemWithStructTm = StdLogicError;
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of epicsTime"][::std::mem::size_of::<EpicsTime>() - 8usize];
    ["Alignment of epicsTime"][::std::mem::align_of::<EpicsTime>() - 4usize];
    ["Offset of field: epicsTime::ts"][::std::mem::offset_of!(EpicsTime, ts) - 0usize];
};
unsafe extern "C" {
    #[doc = " \\brief Get current clock time\n\n Returns an epicsTime containing the current time. For example:\n \\code{.cpp}\n   epicsTime now = epicsTime::getCurrent();\n \\endcode"]
    #[link_name = "\u{1}?getCurrent@epicsTime@@SA?AV1@XZ"]
    pub fn epicsTime_getCurrent() -> EpicsTime;
}
unsafe extern "C" {
    #[doc = " \\brief Construct from epicsTimeStamp"]
    #[link_name = "\u{1}??0epicsTime@@QEAA@AEBUepicsTimeStamp@@@Z"]
    pub fn epicsTime_epicsTime(this: *mut EpicsTime, replace: *const EpicsTimeStamp);
}
unsafe extern "C" {
    #[doc = " \\brief Construct from <tt>struct timeval</tt>"]
    #[link_name = "\u{1}??0epicsTime@@QEAA@AEBUtimeval@@@Z"]
    pub fn epicsTime_epicsTime1(this: *mut EpicsTime, replace: *const Timeval);
}
unsafe extern "C" {
    #[doc = " \\brief Construct from Windows <tt>struct _FILETIME</tt>"]
    #[link_name = "\u{1}??0epicsTime@@QEAA@AEBU_FILETIME@@@Z"]
    pub fn epicsTime_epicsTime2(this: *mut EpicsTime, arg1: *const _FILETIME);
}
impl EpicsTime {
    #[inline]
    pub unsafe fn getCurrent() -> EpicsTime {
        epicsTime_getCurrent()
    }
    #[inline]
    pub unsafe fn new(replace: *const EpicsTimeStamp) -> Self {
        let mut __bindgen_tmp = ::std::mem::MaybeUninit::uninit();
        epicsTime_epicsTime(__bindgen_tmp.as_mut_ptr(), replace);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new1(replace: *const Timeval) -> Self {
        let mut __bindgen_tmp = ::std::mem::MaybeUninit::uninit();
        epicsTime_epicsTime1(__bindgen_tmp.as_mut_ptr(), replace);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new2(arg1: *const _FILETIME) -> Self {
        let mut __bindgen_tmp = ::std::mem::MaybeUninit::uninit();
        epicsTime_epicsTime2(__bindgen_tmp.as_mut_ptr(), arg1);
        __bindgen_tmp.assume_init()
    }
}