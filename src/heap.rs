use atomic::Atomic;
use crate::size_classes::{SizeClassData, SIZE_CLASSES};

pub const LG_MAX_BLOCK_NUM: u32 = 31;
pub const MAX_BLOCK_NUM: u64 = (2 as u64) << LG_MAX_BLOCK_NUM;

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

    pub fn set_state(&mut self, state: u32) {
        self.anch = (self.anch & 0x3FFFFFFFFFFFFFFF) | ((state as u64) << 2 * LG_MAX_BLOCK_NUM)
    }

    pub fn get_state(&self) -> u32 {
        (self.anch >> 2 * LG_MAX_BLOCK_NUM) as u32
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

#[derive(Debug, Clone, Copy)]
pub struct DescriptorNode<'a> {
    desc: &'a Descriptor<'a>,
}

impl<'a> DescriptorNode<'a> {
    pub fn new(desc: &'a Descriptor<'a>) -> Self {
        // todo: make sure desc is cacheline aligned
        DescriptorNode { desc: desc }
    }

    pub fn set_desc(&mut self, desc: &'a Descriptor<'a>, counter: u64) {
        // todo: make sure desc is cacheline aligned
        self.desc = desc;
    }

    pub fn get_desc(&self) -> &'a Descriptor<'a> { &self.desc }

    pub fn get_counter() -> u64 {
        todo!();
    }
}

#[derive(Debug)]
pub struct Descriptor<'a> {
    // used in free descriptor list
    next_free: Atomic<DescriptorNode<'a>>,
    // used in partial descriptor list
    next_partial: Atomic<DescriptorNode<'a>>,

    anchor: Atomic<Anchor>,
    superblock: &'a u8,
    heap: &'a ProcHeap<'a>,
    block_size: u32,
    maxcount: u32,
}

impl<'a> Descriptor<'a> {
    pub fn new() -> Self {
        todo!();
    }

    pub fn get_next_free(&self) -> &Atomic<DescriptorNode<'a>> { &self.next_free }

    pub fn get_next_partial(&self) -> &Atomic<DescriptorNode<'a>> { &self.next_partial }

    pub fn get_anchor(&self) -> &Atomic<Anchor> { &self.anchor }

    pub fn get_superblock(&self) -> &'a u8 { self.superblock }

    pub fn get_heap(&self) -> &'a ProcHeap { self.heap }

    pub fn get_block_size(&self) -> u32 { self.block_size }

    pub fn get_maxcount(&self) -> u32 { self.maxcount }
}

#[derive(Debug)]
pub struct ProcHeap<'a> {
    partial_list: Option<Atomic<DescriptorNode<'a>>>,
    sc_idx: usize,
}

impl<'a> ProcHeap<'a> {
    pub fn new(sc_idx: usize) -> Self {
        ProcHeap { partial_list: None, sc_idx: sc_idx }
    }

    // pub fn set_sc_idx(&self, sc_idx: usize) {
    //     self.sc_idx = sc_idx;
    // }

    pub fn get_sc_idx(&self) -> usize {
        self.sc_idx
    }

    pub fn get_size_class(&self) -> &SizeClassData {
        unsafe {
            &SIZE_CLASSES[self.sc_idx]
        }
    }
}
