#![no_std] // Disables Rust heap
#![allow(dead_code)] // FIXME: have it here so there's no warning spam

mod defines;
mod heap;
mod log;
mod pagemap;
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
    let _dummy = heap::Descriptor::alloc();
    let _dummy2 = heap::Descriptor::alloc();

    use heap::Descriptor;
    use r3malloc::{heap_pop_partial, heap_push_partial, init_malloc, HEAPS};
    init_malloc();

    // test heap push/pop
    let heap = unsafe { &mut HEAPS[1] };
    let mut stuff: *mut Descriptor = heap_pop_partial(heap);
    let list = heap.get_partial_list();
    libc_println!("\ninitial list of heap: {:?}\n", list);
    _dummy.set_heap(heap);
    _dummy.set_block_size(10);
    _dummy.set_maxcount(20);
    libc_println!("desc to add: {:?}\n", _dummy);
    libc_println!("1st pop is null: {}\n", stuff.is_null());
    heap_push_partial(_dummy);
    stuff = heap_pop_partial(heap);
    libc_println!("2nd pop is null: {}", stuff.is_null());
    libc_println!("2nd pop object: {:?}\n", unsafe { &mut *stuff });
    stuff = heap_pop_partial(heap);
    libc_println!("3rd pop is null: {}\n", stuff.is_null());

    anch.get_avail()
}

#[no_mangle]
pub extern "C" fn malloc(size: usize) -> *mut libc::c_void {
    r3malloc::do_malloc(size) as *mut libc::c_void
}

#[no_mangle]
pub extern "C" fn free(ptr: *mut libc::c_void) {
    r3malloc::do_free(ptr as *mut u8)
}

use core::panic::PanicInfo;

// Called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    libc_println!("{}", info);

    loop {}
}
