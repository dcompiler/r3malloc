use crate::pages::page_alloc_overcommit;
use crate::defines::parse_usize;
use core::{mem::size_of, ptr::null_mut};
use core::cmp::{min, max};

const RS_CHUNK: usize = (1 as usize) << 15;
const RS_SIZE: usize = RS_CHUNK * size_of::<Reuse>();
const BOOST_LENGTH: usize = 20000;
const WINDOW_LENGTH: usize = match option_env!("WINDOW_LENGTH") {
	Some(wl) => parse_usize(wl),
	None => 2
};
// default target apf is 1000
const TARGET_APF: usize = match option_env!("TARGET_APF") {
	Some(apf) => parse_usize(apf),
	None => 1000
};

#[derive(Debug)]
struct Reuse {
	current_time: usize,
	num_intervals: usize,
	free_intervals: *mut (usize, usize),
	boost_count: usize,
	is_hibernating: bool,
	num_frees: usize,
	num_events: usize,
}

impl Reuse {
	pub const fn def() -> Self {
		Reuse {
			current_time: 0,
			num_intervals: 0,
			free_intervals: null_mut(),
			boost_count: 0,
			is_hibernating: false,
			num_frees: 0,
			num_events: 0,
		}
	}

	pub fn init(&mut self) {
		unsafe {
			self.free_intervals = page_alloc_overcommit::<(usize, usize)>(RS_SIZE);
		}
	}

	pub fn get_time(&self) -> usize {
		self.current_time
	}

	pub fn on_allocation(&mut self) {
		if self.is_hibernating {
			return
		}

		unsafe {
			let interval = self.free_intervals.add(self.num_intervals);

			if (*interval).0 != 0 {
				(*interval).1 = self.current_time;
				self.num_intervals += 1;
			}
		}

		self.num_events += 1;
	}

	pub fn on_free(&mut self) {
		if self.is_hibernating {
			return
		}

		unsafe { (*self.free_intervals.add(self.num_frees)).0 = self.current_time; }
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

			unsafe {
				for i in 0..BOOST_LENGTH + 1 {
					*self.free_intervals.add(i) = (0, 0);
				}
			}
			self.num_intervals = 0;
			self.num_frees = 0;
			self.num_events = 0;
		}
	}

	#[cfg(feature = "all_windows")]
	pub fn compute(&self) -> [f64; WINDOW_LENGTH] {
		let mut xyz = [(0.0, 0.0, 0.0); WINDOW_LENGTH];
		let mut reuse = [0.0; WINDOW_LENGTH];

		for wl in 1..WINDOW_LENGTH+1 {
			let mut x = 0.0;
			let mut y = 0.0;
			let mut z = 0.0;

			if wl == 1 {
				for i in 0..self.num_intervals {
					let interval = unsafe { *self.free_intervals.add(i) };
					if interval.0 == interval.1 {
						x += interval.0 as f64;
						y += interval.1 as f64;
						z += 1.0;
					}
				}
			} else {
				x += xyz[wl-2].0;
				y += xyz[wl-2].1;
				z += xyz[wl-2].2;
				for i in 0..self.num_intervals {
					let interval = unsafe { *self.free_intervals.add(i) };

					if interval.1 - interval.0 + 1 == wl {
						x += min(self.num_events - wl, interval.0) as f64;
						y += max(wl, interval.1) as f64;
						z += wl as f64;
					}

					if interval.0 as i64 >= self.num_events as i64 - (wl as i64 - 1) {
						x += 1.0;
					}
					if interval.1 <= wl - 1 {
						y += 1.0;
					}

					if interval.1 - interval.0 < wl - 1 {
						z += 1.0;
					}
				}
			}

			xyz[wl - 1] = (x, y, z);
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
			let interval = unsafe { *self.free_intervals.add(i) };
			if interval.1 - interval.0 < wl {
				x += min(self.num_events as i64 - wl as i64, interval.0 as i64) as f64;
				y += max(wl, interval.1) as f64;
				z += wl as f64;
			}
		}

		(x - y + z) / (self.num_events as f64 - wl as f64 + 1.0)
	}
}

#[derive(Debug)]
pub struct Apf {
	reuse: Reuse,
	num_fetches: usize,
	current_apf: usize,
}

impl Apf {
	pub const fn new() -> Self {
		Apf { reuse: Reuse::def(), num_fetches: 0, current_apf: 0 }
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

	pub fn on_fetch(&mut self) { self.num_fetches += 1; }

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
	pub fn demand(&self, wl: Option<usize>) -> f64 {
		match wl {
			Some(wl) => wl as f64 - self.reuse.compute(wl),
			None => WINDOW_LENGTH as f64 - self.reuse.compute(WINDOW_LENGTH)
		}
	}

	pub fn update_apf(&mut self) {
		let current_time = self.reuse.get_time();
		if TARGET_APF * (self.num_fetches + 1) <= current_time {
			self.current_apf = TARGET_APF;
		} else {
			self.current_apf = TARGET_APF * (self.num_fetches + 1) - current_time;
		}
	}

	#[cfg(not(feature = "all_windows"))]
	pub fn should_update_slots(&mut self, available_slots: usize) -> Option<usize> {
		let demand = self.demand(Some(self.current_apf)) as usize;
		if available_slots == 2 * demand + 1 {
			Some(demand + 1)
		} else {
			None
		}
	}
}

#[thread_local]
pub static mut APF_INIT: bool = false;