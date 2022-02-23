#![no_std] // Disables Rust heap
#![allow(dead_code)] // FIXME: have it here so there's no warning spam

mod defines;
mod heap;
mod log;
mod pages;
mod r3malloc;
mod size_classes;
mod tcache;

use heap::Anchor;
use libc_print::libc_println;

extern crate libc;

// FIXME: Dummy code as a POC (see tests/dummy.c)
#[no_mangle]
pub extern "C" fn test() -> u32 {
    let mut anch: Anchor = Anchor::new();

    anch.set_state(2);
    libc_println!("Hello from Rust: {}", anch.get_state());
    r3malloc::init_malloc();
    anch.get_state()
}

use core::panic::PanicInfo;

// Called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    libc_println!("{}", info);

    loop {}
}
