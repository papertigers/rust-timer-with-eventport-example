use libc::{c_int, clockid_t, sigevent};

/// TODO get these into the libc crate

#[repr(C)]
pub struct itimerspec {
    pub it_interval: libc::timespec,
    pub it_value: libc::timespec,
}

#[allow(non_camel_case_types)]
type timer_t = c_int;

extern "C" {
    pub fn timer_create(clock_id: clockid_t, evp: *mut sigevent, timerid: *mut timer_t) -> c_int;
    pub fn timer_settime(
        timerid: timer_t,
        flags: c_int,
        value: *const itimerspec,
        ovalue: *mut itimerspec,
    ) -> c_int;
    pub fn timer_delete(timerid: timer_t) -> c_int;
}
