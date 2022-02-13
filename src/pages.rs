use libc::*;
use crate::defines::page_mask;

pub unsafe fn page_alloc(size: usize) -> *mut c_void {
	core::assert_eq!(size & page_mask(), 0);

	let mut ptr = mmap(0 as *mut c_void, size, PROT_READ | PROT_WRITE,
		MAP_PRIVATE | MAP_ANON, -1, 0);
	if ptr == MAP_FAILED {
		ptr = 0 as *mut c_void
	}

	ptr
}

pub unsafe fn page_alloc_overcommit(size: usize) -> *mut c_void {
	core::assert_eq!(size & page_mask(), 0);

	let mut ptr = mmap(0 as *mut c_void, size, PROT_READ | PROT_WRITE,
		MAP_PRIVATE | MAP_ANON | MAP_NORESERVE, -1, 0);
	if ptr == MAP_FAILED {
		ptr = 0 as *mut c_void
	}

	ptr	
}

pub unsafe fn page_free(ptr: *mut c_void, size: usize) {
	core::assert_eq!(size & page_mask(), 0);
	let ret = munmap(ptr, size);
	core::assert_eq!(ret, 0);	
}