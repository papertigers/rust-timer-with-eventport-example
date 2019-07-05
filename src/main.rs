mod ffi;

use ffi::{itimerspec, timer_create, timer_delete, timer_settime};

const SIGEV_PORT: i32 = 4;

macro_rules! err_exit (
    ($msg:tt) => { {
        eprintln!("{}", $msg);
        std::process::exit(1);
    } }
);

#[repr(C)]
struct PortNotify {
    port: i32,
    user: *const libc::c_void,
}

fn source_to_str(s: libc::c_ushort) -> &'static str {
    match i32::from(s) {
        libc::PORT_SOURCE_AIO => "AIO",
        libc::PORT_SOURCE_TIMER => "TIMER",
        libc::PORT_SOURCE_USER => "USER",
        libc::PORT_SOURCE_FD => "FD",
        libc::PORT_SOURCE_ALERT => "ALERT",
        libc::PORT_SOURCE_MQ => "MQ",
        libc::PORT_SOURCE_FILE => "FILE",
        _ => panic!("found unknown event port source"),
    }
}

fn debug_port_event(pe: &libc::port_event) {
    println!("  events: {}", pe.portev_events);
    println!("  source: {}", source_to_str(pe.portev_source));
    println!("  object: {}", pe.portev_object);
    println!("  user: {:p}", pe.portev_user);
}

fn main() {
    let mut timerid: i32 = 0;

    let pfd = match unsafe { libc::port_create() } {
        -1 => err_exit!("failed to open port fd"),
        fd => fd,
    };

    // setup port_notify_t
    let mut notify = PortNotify {
        port: pfd,
        user: std::ptr::null(),
    };

    // setup the sigevent
    let mut evp: libc::sigevent = unsafe { std::mem::zeroed() };
    evp.sigev_notify = SIGEV_PORT;
    evp.sigev_value = libc::sigval {
        sival_ptr: &mut notify as *mut _ as *mut libc::c_void,
    };

    // create the timer
    if unsafe { timer_create(libc::CLOCK_REALTIME, &mut evp, &mut timerid) } == -1 {
        let _ = unsafe { libc::close(pfd) };
        err_exit!("failed to create timer");
    }

    let ts = itimerspec {
        it_interval: libc::timespec {
            tv_sec: 1,
            tv_nsec: 0,
        },
        it_value: libc::timespec {
            tv_sec: 1,
            tv_nsec: 0,
        },
    };

    if unsafe { timer_settime(timerid, libc::TIMER_RELTIME, &ts, std::ptr::null_mut()) } == -1 {
        let _ = unsafe { libc::close(pfd) };
        let _ = unsafe { timer_delete(timerid) };
        err_exit!("failed to set timer");
    }

    loop {
        let mut pe: libc::port_event = unsafe { std::mem::zeroed() };

        // for now consider any errno a failure
        if unsafe { libc::port_get(pfd, &mut pe, std::ptr::null_mut()) } != 0 {
            let _ = unsafe { libc::close(pfd) };
            let _ = unsafe { timer_delete(timerid) };
            err_exit!("failed to call port_get");
        }

        println!("event returned:");
        debug_port_event(&pe);
        println!();
    }
}
