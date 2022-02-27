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

    anch.set_avail(128);
    libc_println!("Hello from Rust: {}", anch.get_avail());
    r3malloc::init_malloc();
    let _dummy = heap::Descriptor::alloc();
    let _dummy2 = heap::Descriptor::alloc();
    anch.get_avail()
}

use core::panic::PanicInfo;

// Called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    libc_println!("{}", info);

    loop {}
}
