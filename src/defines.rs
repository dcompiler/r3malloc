pub const LG_PAGE: usize = 12;
const LG_CACHELINE: usize = 6;

pub const PAGE: usize = (1 as usize) << LG_PAGE;
pub const PAGE_MASK: usize = PAGE - 1;
pub const CACHELINE: usize = (1 as usize) << LG_CACHELINE;
pub const CACHELINE_MASK: usize = CACHELINE - 1;

// return smallest page size multiple that is >= s
#[inline(always)]
pub fn page_ceiling(s: usize) -> usize {
    (s + (PAGE - 1)) & !(PAGE - 1)
}

// returns smallest address >= addr with alignment align
#[inline(always)]
pub fn align_addr<T>(addr: *mut T, align: usize) -> *mut T {
    (((addr as usize) + (align - 1)) & (!align + 1)) as *mut T
}
