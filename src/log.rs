use libc::c_char;
use crate::defines::parse_usize;

// Credit to https://stackoverflow.com/questions/38088067/equivalent-of-func-or-function-in-rust
pub const LOG: bool = match option_env!("LOG") {
    Some(_) => true,
    None => false,
};
pub const SAVE_LOG: bool = match option_env!("SAVE_LOG") {
    Some(_) => true,
    None => false,
};
// by default, save on every invocation of save_log
pub const SAVE_PERIOD: usize = match option_env!("SAVE_PERIOD") {
    Some(sp) => parse_usize(sp),
    None => 0,
};
// would be nice to make it compile-time inputted (as above)
// but I cannot figure out how to null-terminate the input &str
pub const FILE_PATH: &str = "log.txt\0";
static mut CURR_PERIOD: usize = 0;

#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            core::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! log_debug {
    ( $( $x: expr ), * ) => {{
        use crate::log::LOG;
        if LOG {
            use libc_print::{libc_println, libc_print};
            libc_print!("{}: {} {}", core::file!(), core::line!(), crate::function!());
            $(
                libc_print!(" {:?}", $x);
            )*
            libc_println!();
        }
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! log_debug {
    ( $( $x: expr ), * ) => {{}};
}

#[macro_export]
macro_rules! log_err {
    ( $( $x: expr ), * ) => {{
        use crate::log::LOG;
        if LOG {
            use libc_print::{libc_println, libc_print};
            libc_eprint!("{}:{} {}", core::file!(), core::line!(), crate::function!());
            $(
                libc_eprint!(" {}", $x);
            )*
            libc_eprintln!();
        }
    }};
}

#[inline(never)]
pub fn save_log(log: &str) {
    if !SAVE_LOG {
        return;
    }

    unsafe {
        if SAVE_PERIOD != 0 {
            if CURR_PERIOD != SAVE_PERIOD {
                CURR_PERIOD += 1;
                return;
            } else {
                CURR_PERIOD = 0;
            }
        }

        let fp = FILE_PATH.as_ptr().cast::<core::ffi::c_void>() as *mut c_char;
        let fd = libc::open(fp, libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND, libc::S_IWRITE | libc::S_IREAD);
        if fd < 0 {
            panic!("Failed to open log file.");
        }

        libc::write(fd,log.as_ptr().cast::<core::ffi::c_void>(), log.len());

        libc::close(fd);
    }
}