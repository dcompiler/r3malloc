// Credit to https://stackoverflow.com/questions/38088067/equivalent-of-func-or-function-in-rust
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
        use libc_print::{libc_println, libc_print};
        libc_print!("{:?}:{:?} {:?}", core::file!(), core::line!(), crate::function!());
        $(
            libc_print!(" {:?}", $x);
        )*
        libc_println!();
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
        use libc_print::{libc_println, libc_print};
        libc_eprint!("{}:{} {}", core::file!(), core::line!(), crate::function!());
        $(
            libc_eprint!(" {}", $x);
        )*
        libc_eprintln!();
    }};
}
