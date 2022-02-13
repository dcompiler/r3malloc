#![no_std] // Disables Rust heap
#![allow(dead_code)] // FIXME: have it here so there's no warning spam

mod heap;
mod pages;
mod defines;

use heap::Anchor;
use libc_print::libc_println;

extern crate libc;

// FIXME: Dummy code as a POC (see tests/dummy.c)
#[no_mangle]
pub extern "C" fn test() -> u32
{
    let mut anch: Anchor = Anchor::new();

	anch.set_state(2);
	libc_println!("Hello from Rust: {}", anch.get_state());
	let _alloc = unsafe {pages::page_alloc(2);};
	anch.get_state()
}

use core::panic::PanicInfo;

// Called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}