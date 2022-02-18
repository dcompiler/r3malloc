use crate::defines::PAGE_MASK;
use libc::*;

pub unsafe fn page_alloc<T>(size: usize) -> *mut T {
    core::assert_eq!(size & PAGE_MASK, 0);

    let ptr = mmap(
        0 as *mut c_void,
        size,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANON,
        -1,
        0,
    );
    if ptr == MAP_FAILED {
        return core::ptr::null_mut();
    }

    ptr as *mut T
}

pub unsafe fn page_alloc_overcommit<T>(size: usize) -> *mut T {
    core::assert_eq!(size & PAGE_MASK, 0);

    let ptr = mmap(
        0 as *mut c_void,
        size,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANON | MAP_NORESERVE,
        -1,
        0,
    );
    if ptr == MAP_FAILED {
        return core::ptr::null_mut();
    }

    ptr as *mut T
}

pub unsafe fn page_free(ptr: *mut c_void, size: usize) {
    core::assert_eq!(size & PAGE_MASK, 0);
    let ret = munmap(ptr, size);
    core::assert_eq!(ret, 0);
}
