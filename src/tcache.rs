use crate::size_classes::MAX_SZ_IDX;
use core::ptr::null_mut;

#[derive(Clone, Copy)]
pub struct TCacheBin {
    block: *mut u8,
    block_num: u32,
}

impl TCacheBin {
    pub fn new() -> Self {
        TCacheBin {
            block: null_mut(),
            block_num: 0,
        }
    }

    pub fn get_block_num(&self) -> u32 {
        self.block_num
    }

    pub fn peek_block(&self) -> *mut u8 {
        self.block
    }

    #[inline(always)]
    pub fn push_block(&mut self, block: *mut u8) {
        unsafe { *(block as *mut *mut u8) = self.block };
        self.block = block;
        self.block_num += 1;
    }

    #[inline(always)]
    pub fn push_list(&mut self, block: *mut u8, length: u32) {
        assert_eq!(self.block_num, 0);

        self.block = block;
        self.block_num = length;
    }

    #[inline(always)]
    pub fn pop_block(&mut self) -> *mut u8 {
        assert!(self.block_num > 0);

        let ret = self.block;
        self.block = unsafe { *(self.block as *mut *mut u8) };
        self.block_num -= 1;
        ret
    }

    #[inline(always)]
    pub fn pop_list(&mut self, block: *mut u8, length: u32) {
        assert!(self.block_num >= length);

        self.block = block;
        self.block_num -= length;
    }
}

pub static mut TCACHE: [TCacheBin; MAX_SZ_IDX] = [TCacheBin {
    block: null_mut(),
    block_num: 0,
}; MAX_SZ_IDX];
