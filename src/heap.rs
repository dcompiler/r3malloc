use crate::defines::{align_addr, CACHELINE, CACHELINE_MASK, PAGE};
use crate::pages::page_alloc;
use crate::size_classes::{SizeClassData, SIZE_CLASSES};
use atomic::{Atomic, Ordering};
use core::{mem::size_of, ptr::null_mut};

pub const LG_MAX_BLOCK_NUM: u32 = 31;
pub const MAX_BLOCK_NUM: u64 = (2 as u64) << LG_MAX_BLOCK_NUM;

pub const DESCRIPTOR_BLOCK_SZ: usize = 16 * PAGE;

#[derive(PartialEq)]
pub enum SbState {
    Full,
    Partial,
    Empty,
}

#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    // state is first 2 bits
    // avail is next lg_max_block_num (31) bits
    // count is next lg_max_block_num (31) bits
    anch: u64,
}

impl Anchor {
    pub fn new() -> Anchor {
        Anchor { anch: 0x0 }
    }

    pub fn set_state(&mut self, state: SbState) {
        let st: u64 = match state {
            SbState::Full => 0,
            SbState::Partial => 1,
            SbState::Empty => 2,
        };
        self.anch = (self.anch & 0x3FFFFFFFFFFFFFFF) | ((st) << 2 * LG_MAX_BLOCK_NUM)
    }

    pub fn get_state(&self) -> SbState {
        match self.anch >> 2 * LG_MAX_BLOCK_NUM {
            0 => SbState::Full,
            1 => SbState::Partial,
            2 => SbState::Empty,
            _ => panic!("Anchor state was not full, partial or empty"),
        }
    }

    pub fn set_avail(&mut self, avail: u32) {
        self.anch = (self.anch & 0xC00000007FFFFFFF) | ((avail as u64) << LG_MAX_BLOCK_NUM - 1)
    }

    pub fn get_avail(&self) -> u32 {
        ((self.anch >> LG_MAX_BLOCK_NUM) & 0x007FFFFFF) as u32
    }

    pub fn set_count(&mut self, count: u32) {
        self.anch = (self.anch & 0xFFFFFFFF80000000) | ((count & 0x7FFFFFFF) as u64)
    }

    pub fn get_count(&self) -> u32 {
        (self.anch & 0x07FFFFFFF) as u32
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
    next_free: Atomic<Option<DescriptorNode<'a>>>,
    // used in partial descriptor list
    next_partial: Atomic<Option<DescriptorNode<'a>>>,

    anchor: Atomic<Anchor>,
    superblock: *mut u8,
    heap: *mut ProcHeap<'a>,
    block_size: u32,
    maxcount: u32,
}

static mut AVAIL_DESC: Atomic<Option<DescriptorNode>> = Atomic::new(None);

impl<'a> Descriptor<'a> {
    pub fn get_next_free(&self) -> &Atomic<Option<DescriptorNode<'a>>> {
        &self.next_free
    }

    pub fn get_next_partial(&self) -> &Atomic<Option<DescriptorNode<'a>>> {
        &self.next_partial
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
        let old_head = unsafe { AVAIL_DESC.load(Ordering::SeqCst) };
        loop {
            match old_head {
                Some(old_desc) => {
                    let desc: &mut Descriptor = unsafe { old_desc.get_desc().as_mut().unwrap() };
                    let new_head = desc.get_next_free().load(Ordering::SeqCst);
                    new_head
                        .unwrap()
                        .set_desc(new_head.unwrap().get_desc(), old_desc.get_counter());

                    match unsafe {
                        AVAIL_DESC.compare_exchange_weak(
                            old_head,
                            new_head,
                            Ordering::SeqCst,
                            Ordering::SeqCst,
                        )
                    } {
                        Ok(_) => {
                            assert_eq!(desc.get_block_size(), 0);
                            return desc;
                        }
                        _ => (),
                    }
                }
                None => {
                    let ptr = unsafe { page_alloc::<u8>(DESCRIPTOR_BLOCK_SZ) };
                    let ret = ptr as *mut Descriptor;

                    let mut curr_ptr: *mut u8 =
                        unsafe { ptr.offset(size_of::<Descriptor>() as isize) };
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
                                    .get_next_free()
                                    .store(Some(DescriptorNode::new(&mut *curr)), Ordering::SeqCst)
                            };
                        }

                        prev = curr;
                        curr_ptr = unsafe { curr_ptr.offset(size_of::<Descriptor>() as isize) };
                        curr_ptr = align_addr(curr_ptr, CACHELINE);
                    }

                    unsafe { (*prev).get_next_free().store(None, Ordering::SeqCst) };

                    let old_head = unsafe { AVAIL_DESC.load(Ordering::SeqCst) };
                    let mut new_head: Option<DescriptorNode> = None;
                    loop {
                        unsafe {
                            (*prev).get_next_free().store(old_head, Ordering::SeqCst);
                            match new_head {
                                Some(mut d) => {
                                    d.set_desc(&mut *first, old_head.unwrap().get_counter() + 1);
                                }
                                None => {
                                    new_head = Some(DescriptorNode::new(&mut *first));
                                }
                            }

                            match AVAIL_DESC.compare_exchange_weak(
                                old_head,
                                new_head,
                                Ordering::SeqCst,
                                Ordering::SeqCst,
                            ) {
                                Ok(_) => {
                                    break;
                                }
                                _ => (),
                            }
                        }
                    }

                    return unsafe { &mut *ret };
                }
            };
        }
    }

    pub fn retire(&'static mut self) {
        self.block_size = 0;
        let old_head = unsafe { AVAIL_DESC.load(Ordering::SeqCst) };
        let mut new_head: Option<DescriptorNode> = None;
        loop {
            self.get_next_free().store(old_head, Ordering::SeqCst);
            match new_head {
                Some(mut d) => {
                    d.set_desc(self, old_head.unwrap().get_counter() + 1);
                }
                None => {
                    new_head = Some(DescriptorNode::new(self));
                }
            }

            unsafe {
                match AVAIL_DESC.compare_exchange_weak(
                    old_head,
                    new_head,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => {
                        break;
                    }
                    _ => (),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ProcHeap<'a> {
    partial_list: Atomic<Option<DescriptorNode<'a>>>,
    sc_idx: usize,
}

impl<'a> ProcHeap<'a> {
    pub const fn const_new(sc_idx: usize) -> Self {
        ProcHeap {
            partial_list: Atomic::new(None),
            sc_idx: sc_idx,
        }
    }

    pub fn set_sc_idx(&mut self, sc_idx: usize) {
        self.sc_idx = sc_idx;
    }

    pub fn get_sc_idx(&self) -> usize {
        self.sc_idx
    }

    pub fn get_partial_list(&self) -> &Atomic<Option<DescriptorNode<'a>>> {
        &self.partial_list
    }

    pub fn get_size_class(&self) -> &SizeClassData {
        unsafe { &SIZE_CLASSES[self.sc_idx] }
    }
}
