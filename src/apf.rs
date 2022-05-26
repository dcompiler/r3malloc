use crate::pages::page_alloc_overcommit;
use crate::defines::parse_usize;
use core::{mem::size_of, slice::from_raw_parts_mut};
use core::cmp::{min, max};

const RS_CHUNK: usize = (1 as usize) << 15;
const RS_SIZE: usize = RS_CHUNK * size_of::<Reuse>();
const BOOST_LENGTH: usize = 20000;
const WINDOW_LENGTH: usize = match option_env!("WINDOW_LENGTH") {
	Some(wl) => parse_usize(wl),
	None => 2
};

struct Reuse<'a> {
	current_time: usize,
	num_intervals: usize,
	free_intervals: &'a mut [(usize, usize)],
	boost_count: usize,
	is_hibernating: bool,
	num_frees: usize,
	num_events: usize,
}

impl<'a> Reuse<'a> {
	pub const fn def() -> Self {
		Reuse {
			current_time: 0,
			num_intervals: 0,
			free_intervals: &mut [],
			boost_count: 0,
			is_hibernating: false,
			num_frees: 0,
			num_events: 0,
		}
	}

	pub fn init(&mut self) {
		unsafe {
			self.free_intervals = from_raw_parts_mut(page_alloc_overcommit::<(usize, usize)>(RS_SIZE), RS_CHUNK);
		}
	}

	pub fn on_allocation(&mut self) {
		if self.is_hibernating {
			return
		}

		if self.free_intervals[self.num_intervals].0 != 0 {
			self.free_intervals[self.num_intervals].1 = self.current_time;
			self.num_intervals += 1;
		}

		self.num_events += 1;
	}

	pub fn on_free(&mut self) {
		if self.is_hibernating {
			return
		}

		self.free_intervals[self.num_frees].0 = self.current_time;
		self.num_frees += 1;

		self.num_events += 1;
	}

	pub fn inc_timer(&mut self) {
		self.current_time += 1;

		if self.current_time == BOOST_LENGTH {
			if self.is_hibernating {
				self.is_hibernating = false;
				self.boost_count = 0;
			} else if self.boost_count == 1 {
				self.is_hibernating = true;
			} else {
				self.boost_count += 1;
			}

			self.current_time = 0;

			for i in 0..BOOST_LENGTH+1 {
				self.free_intervals[i] = (0, 0);
			}
			self.num_intervals = 0;
			self.num_frees = 0;
			self.num_events = 0;
		}
	}

	#[cfg(feature = "all_windows")]
	pub fn compute(&self) -> [f64; WINDOW_LENGTH] {
		let mut reuse = [0.0; WINDOW_LENGTH];

		for wl in 1..WINDOW_LENGTH+1 {
			let mut x = 0.0;
			let mut y = 0.0;
			let mut z = 0.0;

			if wl == 1 {
				for i in 0..self.num_intervals {
					if self.free_intervals[i].0 == self.free_intervals[i].1 {
						x += self.free_intervals[i].0 as f64;
						y += self.free_intervals[i].1 as f64;
						z += 1.0;
					}
				}
			} else {
				for i in 0..self.num_intervals {
					x += reuse[wl-2];
					y += reuse[wl-2];
					z += reuse[wl-2];

					if self.free_intervals[i].1 - self.free_intervals[i].0 + 1 == wl {
						x += min(self.num_events - wl, self.free_intervals[i].0) as f64;
						y += max(wl, self.free_intervals[i].1) as f64;
						z += wl as f64;
					}

					if self.free_intervals[i].0 as i64 >= self.num_events as i64 - (wl as i64 - 1) {
						x += 1.0;
					}
					if self.free_intervals[i].1 <= wl - 1 {
						y += 1.0;
					}
					if self.free_intervals[i].1 - self.free_intervals[i].0 < wl {
						z += 1.0;
					}
				}
			}

			reuse[wl - 1] = (x - y + z) / (self.num_events as f64 - wl as f64 + 1.0);
		}

		reuse
	}

	#[cfg(not(feature = "all_windows"))]
	pub fn compute(&self, wl: usize) -> f64 {
		let mut x = 0.0;
		let mut y = 0.0;
		let mut z = 0.0;

		for i in 0..self.num_intervals {
			if self.free_intervals[i].1 - self.free_intervals[i].0 <= wl {
				x += min(self.num_events as i64 - wl as i64, self.free_intervals[i].0 as i64) as f64;
				y += max(wl, self.free_intervals[i].1) as f64;
				z += wl as f64 + 1.0;
			}
		}

		(x - y + z) / (self.num_events as f64 - wl as f64 + 1.0)
	}
}

pub struct Apf<'a> {
	reuse: Reuse<'a>,
}

impl<'a> Apf<'a> {
	pub const fn new() -> Self {
		Apf { reuse: Reuse::def() }
	}

	pub fn init(&mut self) {
		self.reuse.init();
	}

	pub fn on_allocation(&mut self) {
		self.reuse.on_allocation();
	}

	pub fn on_free(&mut self) {
		self.reuse.on_free();
	}

	pub fn inc_timer(&mut self) {
		self.reuse.inc_timer();
	}

	#[cfg(feature = "all_windows")]
	pub fn demand(&self) -> [f64; WINDOW_LENGTH] {
		let mut reuse = self.reuse.compute();
		for wl in 1..WINDOW_LENGTH+1 {
			reuse[wl-1] = wl as f64 - reuse[wl - 1];
		}
		reuse
	}

	#[cfg(not(feature = "all_windows"))]
	pub fn demand(&self) -> f64 {
		WINDOW_LENGTH as f64 - self.reuse.compute(WINDOW_LENGTH)
	}
}

#[thread_local]
pub static mut APF_INIT: bool = false;

#[thread_local]
pub static mut APF: Apf = Apf::new();