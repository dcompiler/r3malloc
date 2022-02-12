#![no_std] // No std disables Rust heap

mod heap;
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
	anch.get_state()
}

use core::panic::PanicInfo;

// Called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}