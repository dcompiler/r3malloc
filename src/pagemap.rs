use crate::defines::LG_PAGE;
use crate::heap::Descriptor;
use crate::pages::page_alloc_overcommit;
use crate::size_classes::MAX_SZ_IDX;
use atomic::{Atomic, Ordering};
use core::{mem::size_of, slice::from_raw_parts};

const SC_MASK: usize = ((1 as usize) << 6) - 1;

const PM_NHS: usize = 14;
const PM_NLS: usize = LG_PAGE;
const PM_SB: usize = 64 - PM_NHS - PM_NLS;
const PM_KEY_SHIFT: usize = PM_NLS;
const PM_KEY_MASK: usize = ((1 as usize) << PM_SB) - 1;
const PM_NUM: usize = (1 as usize) << PM_SB;
const PM_SZ: usize = PM_NUM * size_of::<PageInfo>();

#[derive(Copy, Clone)]
pub struct PageInfo<'a> {
    desc: *mut Descriptor<'a>,
}

impl<'a> PageInfo<'a> {
    #[inline(always)]
    pub fn new() -> Self {
        PageInfo {
            desc: core::ptr::null_mut(),
        }
    }

    #[inline(always)]
    pub fn set_desc(&mut self, desc: *mut Descriptor, sc_idx: usize) {
        assert_eq!((desc as usize) & SC_MASK, 0);
        assert!(sc_idx < MAX_SZ_IDX);

        self.desc = ((desc as usize) | sc_idx) as *mut Descriptor;
    }

    #[inline(always)]
    pub fn get_desc(&self) -> *mut Descriptor<'a> {
        ((self.desc as usize) & !SC_MASK) as *mut Descriptor
    }

    #[inline(always)]
    pub fn get_sc_idx(&self) -> usize {
        (self.desc as usize) & SC_MASK
    }
}

pub struct PageMap<'a> {
    pagemap: &'a [Atomic<PageInfo<'a>>],
}

impl<'a> PageMap<'a> {
    pub const fn def() -> Self {
        PageMap { pagemap: &[] }
    }

    pub fn init(&mut self) {
        self.pagemap =
            unsafe { from_raw_parts(page_alloc_overcommit::<Atomic<PageInfo<'a>>>(PM_SZ), PM_NUM) };
    }

    #[inline(always)]
    fn addr_to_key(ptr: *mut u8) -> usize {
        ((ptr as usize) >> PM_KEY_SHIFT) & PM_KEY_MASK
    }

    pub fn get_page_info(&self, ptr: *mut u8) -> PageInfo<'a> {
        self.pagemap[Self::addr_to_key(ptr)].load(Ordering::SeqCst)
    }

    pub fn set_page_info(&mut self, info: PageInfo<'a>, ptr: *mut u8) {
        self.pagemap[Self::addr_to_key(ptr)].store(info, Ordering::SeqCst)
    }
}

pub static mut SPAGEMAP: PageMap = PageMap::def();
