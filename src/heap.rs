use crate::defines::{align_addr, CACHELINE, CACHELINE_MASK, PAGE};
use crate::pages::page_alloc;
use crate::size_classes::{SizeClassData, SIZE_CLASSES};
use crate::lock::Mutex;
use atomic::Atomic;
use core::{mem::size_of, ptr::null_mut, ops::Deref};
use core::cell::{Ref, RefCell};
use c2rust_bitfields::BitfieldStruct;

pub const LG_MAX_BLOCK_NUM: u32 = 31;
pub const MAX_BLOCK_NUM: u64 = (2 as u64) << LG_MAX_BLOCK_NUM;

pub const LOCK_BLOCK_SZ: usize = 4*PAGE;
pub const DESCRIPTOR_BLOCK_SZ: usize = 16 * PAGE;

#[derive(PartialEq, Debug)]
pub enum SbState {
    Full = 0,
    Partial = 1,
    Empty = 2,
}


#[derive(Debug, Clone, Copy, BitfieldStruct)]
pub struct Anchor {
    #[bitfield(name = "state", ty = "u32", bits = "0..=1")]
    #[bitfield(name = "avail", ty = "u32", bits = "2..=32")]
    #[bitfield(name = "count", ty = "u32", bits = "33..=63")]
    anch: [u8; 8],
}

impl Anchor {
    pub fn new() -> Self {
        Anchor { anch: [0; 8] }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DescriptorNode<'a> {
    desc: *mut Descriptor<'a>,
}

// * DescriptorNode needs to always have a valid (non-null) pointer to a Descriptor
impl<'a> DescriptorNode<'a> {
    pub fn new(desc: *mut Descriptor<'a>) -> Self {
        // todo: make sure desc is cacheline aligned
        DescriptorNode { desc: desc }
    }

    pub fn set_desc(&mut self, desc: *mut Descriptor<'a>, counter: usize) {
        assert_eq!((desc as usize) & CACHELINE_MASK, 0);

        self.desc = ((desc as usize) | (counter & CACHELINE_MASK)) as *mut Descriptor
    }

    pub fn get_desc(&self) -> *mut Descriptor<'a> {
        ((self.desc as usize) & !CACHELINE_MASK) as *mut Descriptor
    }

    pub fn get_counter(&self) -> usize {
        (self.desc as usize) & CACHELINE_MASK
    }
}

#[derive(Debug)]
pub struct Descriptor<'a> {
    // used in free descriptor list
    next_free: DescriptorNode<'a>,
    // used in partial descriptor list
    next_partial: DescriptorNode<'a>,

    anchor: Atomic<Anchor>,
    superblock: *mut u8,
    heap: *mut ProcHeap<'a>,
    block_size: u32,
    maxcount: u32,
}

static mut AVAIL_DESC: DescriptorNode = DescriptorNode { desc: null_mut() };
pub static mut POOL_LOCK: Mutex = Mutex::new();

impl<'a> Descriptor<'a> {
    pub fn get_next_free(&mut self) -> &mut DescriptorNode<'a> {
        &mut self.next_free
    }

    pub fn set_next_free(&mut self, next_free: DescriptorNode<'a>) {
        self.next_free = next_free
    }

    pub fn get_next_partial(&mut self) -> &mut DescriptorNode<'a> {
        &mut self.next_partial
    }

    pub fn set_next_partial(&mut self, next_partial: DescriptorNode<'a>) {
        self.next_partial = next_partial
    }

    pub fn get_anchor(&self) -> &Atomic<Anchor> {
        &self.anchor
    }

    pub fn get_superblock(&self) -> *mut u8 {
        self.superblock
    }

    pub fn get_heap(&self) -> *mut ProcHeap<'a> {
        self.heap
    }

    pub fn set_heap(&mut self, heap: *mut ProcHeap<'a>) {
        self.heap = heap
    }

    pub fn get_block_size(&self) -> u32 {
        self.block_size
    }

    pub fn get_maxcount(&self) -> u32 {
        self.maxcount
    }

    pub fn set_block_size(&mut self, block_size: u32) {
        self.block_size = block_size
    }

    pub fn set_maxcount(&mut self, maxcount: u32) {
        self.maxcount = maxcount
    }

    pub fn set_superblock(&mut self, superblock: *mut u8) {
        self.superblock = superblock
    }

    // FIXME: not static lifetime?
    pub fn alloc() -> &'static mut Self {
        unsafe { POOL_LOCK.acquire() };
        let old_head = unsafe { AVAIL_DESC };
        
        let desc: *mut Descriptor = old_head.get_desc();
        if !desc.is_null() {
            let new_head : &mut DescriptorNode<'_> = unsafe { (*desc).get_next_free() };
            new_head.set_desc(new_head.get_desc(), old_head.get_counter());

            unsafe {
                AVAIL_DESC = *new_head;
                POOL_LOCK.release();
            }

            assert_eq!(unsafe { (*desc).get_block_size() }, 0);
            return unsafe { &mut *desc };
        } else {
            // block of descriptors
            let ptr = unsafe { page_alloc::<u8>(DESCRIPTOR_BLOCK_SZ) };
            let ret = ptr as *mut Descriptor;

            let mut curr_ptr: *mut u8 = unsafe { ptr.offset(size_of::<Descriptor>() as isize) };
            curr_ptr = align_addr(curr_ptr, CACHELINE);
            let first: *mut Descriptor = curr_ptr as *mut Descriptor;
            let mut prev: *mut Descriptor = null_mut();

            while unsafe {
                (curr_ptr.offset(size_of::<Descriptor>() as isize))
                    .offset_from(ptr.offset(DESCRIPTOR_BLOCK_SZ as isize))
                    < 0
            } {
                let curr = curr_ptr as *mut Descriptor;
                if !prev.is_null() {
                    unsafe {
                        (*prev)
                            .set_next_free(DescriptorNode::new(&mut *curr))
                    };
                }

                prev = curr;
                curr_ptr = unsafe { curr_ptr.offset(size_of::<Descriptor>() as isize) };
                curr_ptr = align_addr(curr_ptr, CACHELINE);
            }

            unsafe {
                (*prev)
                    .set_next_free(DescriptorNode::new(null_mut()))
            };

            let old_head = unsafe { AVAIL_DESC };
            let mut new_head: DescriptorNode = DescriptorNode::new(null_mut());
            unsafe { (*prev).set_next_free(old_head) };
            new_head.set_desc(first, old_head.get_counter() + 1);
            unsafe { POOL_LOCK.release() };

            return unsafe { &mut *ret };
        }
    }

    pub fn retire(&'static mut self) {
        self.block_size = 0;
        unsafe { POOL_LOCK.acquire() };
        let old_head = unsafe { AVAIL_DESC };
        let mut new_head: DescriptorNode = DescriptorNode::new(null_mut());

        self.set_next_free(old_head);
        new_head.set_desc(self, old_head.get_counter() + 1);
        
        unsafe {
            AVAIL_DESC = new_head;
            POOL_LOCK.release();
        }
    }
}

pub struct PartialListGuard<'a> {
    guard: Ref<'a, DescriptorNode<'a>>
}

impl <'a> Deref for PartialListGuard<'a> {
    type Target = DescriptorNode<'a>;

    fn deref(&self) -> &DescriptorNode<'a> {
        &self.guard
    }
}

#[derive(Debug)]
pub struct ProcHeap<'a> {
    partial_list: RefCell<DescriptorNode<'a>>,
    sc_idx: usize,
}

impl<'a> ProcHeap<'a> {
    pub const fn const_new(sc_idx: usize) -> Self {
        ProcHeap {
            partial_list: RefCell::new(DescriptorNode { desc: null_mut() }),
            sc_idx: sc_idx,
        }
    }

    pub fn set_sc_idx(&mut self, sc_idx: usize) {
        self.sc_idx = sc_idx;
    }

    pub fn get_sc_idx(&self) -> usize {
        self.sc_idx
    }

    pub fn get_partial_list(&'a self) -> PartialListGuard<'a> {
        PartialListGuard { guard: self.partial_list.borrow() }
    }

    pub fn set_partial_list(&self, list: DescriptorNode<'a>) {
        *(self.partial_list.borrow_mut()) = list
    }

    pub fn get_size_class(&self) -> &SizeClassData {
        unsafe { &SIZE_CLASSES[self.sc_idx] }
    }
}
