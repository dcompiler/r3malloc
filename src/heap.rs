#[inline(always)]
const fn lg_max_block_num() -> u32 {
	31
}

#[inline(always)]
pub const fn max_block_num() -> u32 {
	2 << lg_max_block_num()
}
pub struct Anchor {
	// state is first 2 bits
	// avail is next lg_max_block_num (31) bits
	// count is next lg_max_block_num (31) bits
	anch: u64
}

impl Anchor {
	pub fn new() -> Anchor {
		Anchor {anch: 0x0}
	}

	pub fn set_state(&mut self, state: u32) {
		self.anch = (self.anch & 0x3FFFFFFFFFFFFFFF) | ((state as u64) << 2 * lg_max_block_num())
	}

	pub fn get_state(&self) -> u32 {
		(self.anch >> 2 * lg_max_block_num()) as u32
	}

	pub fn set_avail(&mut self, avail: u32) {
		self.anch = (self.anch & 0xC00000007FFFFFFF) | ((avail as u64) << lg_max_block_num() - 1)
	}

	pub fn get_avail(&self) -> u32 {
		((self.anch >> lg_max_block_num()) & 0x007FFFFFF) as u32
	}

	pub fn set_count(&mut self, count: u32) {
		self.anch = (self.anch & 0xFFFFFFFF80000000) | ((count & 0x7FFFFFFF) as u64)
	}

	pub fn get_count(&self) -> u32 {
		(self.anch & 0x07FFFFFFF) as u32
	}
}