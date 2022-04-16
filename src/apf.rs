use crate::pages::page_alloc;
use core::{mem::size_of, slice::from_raw_parts_mut};
use core::cmp::{min, max};

const LV_CHUNK: usize = (1 as usize) << 32;
const LV_SIZE: usize = LV_CHUNK * size_of::<Liveness>();
const RS_CHUNK: usize = (1 as usize) << 32;
const RS_SIZE: usize = RS_CHUNK * size_of::<Reuse>();
const BOOST_LENGTH: usize = 20000;
const WINDOW_LENGTH: usize = 2;

struct Liveness<'a> {
	current_time: usize,
	num_objects: usize,
	sum_allocations: &'a mut [usize],
	sum_frees: &'a mut [usize],
	num_allocations: &'a mut [usize],
	num_frees: &'a mut [usize],
}

impl<'a> Liveness<'a> {
	pub const fn def() -> Self {
		Liveness {
			current_time: 0,
			num_objects: 0,
			sum_allocations: &mut [],
			sum_frees: &mut [],
			num_allocations: &mut [],
			num_frees: &mut [],
		}
	}

	pub fn init(&mut self) {
		unsafe {
			self.sum_allocations = from_raw_parts_mut(page_alloc::<usize>(LV_SIZE), LV_CHUNK);
			self.sum_frees = from_raw_parts_mut(page_alloc::<usize>(LV_SIZE), LV_CHUNK);
			self.num_allocations = from_raw_parts_mut(page_alloc::<usize>(LV_SIZE), LV_CHUNK);
			self.num_frees = from_raw_parts_mut(page_alloc::<usize>(LV_SIZE), LV_CHUNK);
		}
	}

	pub fn on_allocation(&mut self) {
		self.sum_allocations[self.current_time] += self.current_time;
		self.num_allocations[self.current_time] += 1;
		self.num_objects += 1;
	}

	pub fn on_free(&mut self) {
		self.sum_frees[self.current_time] += self.current_time;
		self.num_frees[self.current_time] += 1;
	}

	fn allocate_more(&mut self) {
		todo!();
	}

	pub fn inc_timer(&mut self) {
		self.current_time += 1;

		if self.current_time == self.sum_allocations.len() {
			self.allocate_more();
		}

		self.sum_allocations[self.current_time] = self.sum_allocations[self.current_time - 1];
		self.num_allocations[self.current_time] = self.num_allocations[self.current_time - 1];
		self.sum_frees[self.current_time] = self.sum_frees[self.current_time - 1];
		self.num_frees[self.current_time] = self.num_frees[self.current_time - 1];
	}

	pub fn compute(&self, window_length: usize) -> usize {
		let i = self.current_time - window_length + 1;
		let tmp_1 = (self.num_objects - self.num_frees[i]) * i + self.sum_frees[i];
		let tmp_2 = self.num_allocations[window_length] + self.sum_allocations[self.current_time] - self.sum_allocations[window_length];
		(tmp_1 - tmp_2 + self.num_objects * window_length) / i
	}
}

struct Reuse<'a> {
	current_time: usize,
	num_intervals: usize,
	free_intervals: &'a mut [(usize, usize)],
	boost_count: usize,
	is_hibernating: bool,
}

impl<'a> Reuse<'a> {
	pub const fn def() -> Self {
		Reuse {
			current_time: 0,
			num_intervals: 0,
			free_intervals: &mut [],
			boost_count: 0,
			is_hibernating: false,
		}
	}

	pub fn init(&mut self) {
		unsafe {
			self.free_intervals = from_raw_parts_mut(page_alloc::<(usize, usize)>(RS_SIZE), RS_CHUNK);
		}
	}

	fn allocate_more(&mut self) {
		todo!();
	}

	pub fn on_allocation(&mut self) {
		if self.is_hibernating {
			return
		}

		self.free_intervals[self.num_intervals].1 = self.current_time;
	}

	pub fn on_free(&mut self) {
		if self.is_hibernating {
			return
		}

		self.free_intervals[self.num_intervals].0 = self.current_time;
		self.num_intervals += 1;

		if self.num_intervals == self.free_intervals.len() {
			self.allocate_more()
		}
	}

	pub fn inc_timer(&mut self) {
		self.current_time += 1;

		if self.current_time == BOOST_LENGTH {
			if self.is_hibernating {
				self.is_hibernating = false;
				self.boost_count = 0;
			} else if self.boost_count == 2 {
				self.is_hibernating = true;
			} else {
				self.boost_count += 1;
			}

			self.current_time = 0;
			self.num_intervals = 0;
		}
	}

	pub fn compute(&self) -> usize {
		let mut x = 0;
		let mut y = 0;
		let mut z = 0;

		for i in 0..self.num_intervals {
			if self.free_intervals[i].1 - self.free_intervals[i].0 <= WINDOW_LENGTH {
				x += min(self.current_time - WINDOW_LENGTH, self.free_intervals[i].0);
				y += max(WINDOW_LENGTH, self.free_intervals[i].1);
				z += WINDOW_LENGTH + 1;
			} 
		}

		(x - y + z) / (self.current_time - WINDOW_LENGTH + 1)
	}
}

pub struct Apf<'a> {
	liveness: Liveness<'a>,
	reuse: Reuse<'a>,
}

impl<'a> Apf<'a> {
	pub const fn new() -> Self {
		Apf { liveness: Liveness::def(), reuse: Reuse::def() }
	}

	pub fn init(&mut self) {
		self.liveness.init();
		self.reuse.init();
	}

	pub fn on_allocation(&mut self) {
		self.liveness.on_allocation();
		self.reuse.on_allocation();
	}

	pub fn on_free(&mut self) {
		self.liveness.on_free();
		self.reuse.on_free();
	}

	pub fn inc_timer(&mut self) {
		self.liveness.inc_timer();
		self.reuse.inc_timer();
	}

	pub fn demand(&self) -> usize {
		self.liveness.compute(WINDOW_LENGTH) - self.liveness.compute(0) - self.reuse.compute()
	}
}

#[thread_local]
pub static mut APF: Apf = Apf::new();