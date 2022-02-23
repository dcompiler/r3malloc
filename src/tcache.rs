struct TCacheBinNode {
    next: Option<&'static mut TCacheBinNode>,
    block: Option<&'static u8>,
}

impl TCacheBinNode {
    pub fn new() -> Self {
        TCacheBinNode {
            next: None,
            block: None,
        }
    }
}

struct TCacheBin {
    head: TCacheBinNode, // dummy
    block_num: u32,
}

impl TCacheBin {
    pub fn new() -> Self {
        TCacheBin {
            head: TCacheBinNode::new(),
            block_num: 0,
        }
    }

    pub fn push_block(&mut self, block: Option<&'static u8>) {
        let mut new_head = TCacheBinNode::new();
        new_head.block = block;
        new_head.next = self.head.next.take();
        self.block_num += 1;
    }

    pub fn pop_block(&mut self) -> Option<&u8> {
        assert!(self.block_num > 0);

        self.block_num -= 1;
        match self.head.next.take() {
            Some(ret) => {
                self.head.next = ret.next.take();
                ret.block
            }
            None => {
                self.head.next = None;
                None
            }
        }
    }

    pub fn peek_block(&self) -> &Option<&u8> {
        &self.head.block
    }

    pub fn get_block_num(&self) -> u32 {
        self.block_num
    }
}
