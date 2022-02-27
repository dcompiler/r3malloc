pub struct TCacheBin {
    block: Option<&'static mut u8>,
    block_num: u32,
}

impl TCacheBin {
    pub fn new() -> Self {
        TCacheBin {
            block: None,
            block_num: 0,
        }
    }

    pub fn push_list(&mut self, block: Option<&'static mut u8>, length: u32) {
        assert_eq!(self.block_num, 0);

        self.block = block;
        self.block_num = length;
    }
}
