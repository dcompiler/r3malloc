use crate::pages::page_alloc_overcommit;
use crate::defines::{parse_usize};
use core::{mem::size_of, ptr::null_mut};
use core::cmp::{min, max};
use crate::{log_debug, PAGE};
use c2rust_bitfields::BitfieldStruct;

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
const REUSE_COMPUTE_INTERVAL: usize = match option_env!("REUSE_COMPUTE_INTERVAL") {
	Some(n) => parse_usize(n),
	None => 10,
};
const NUM_FREE_INTERVALS: usize = match option_env!("NUM_FREE_INTERVALS") {
	Some(n) => parse_usize(n),
	None => 250,
};

#[derive(Debug, Clone, Copy, BitfieldStruct)]
pub struct Xyz {
	#[bitfield(name = "init", ty = "bool", bits = "0..=0")]
	#[bitfield(name = "x", ty = "u32", bits = "1..=21")]
	#[bitfield(name = "y", ty = "u32", bits = "22..=42")]
	#[bitfield(name = "z", ty = "u32", bits = "43..=63")]
	xyz: [u8; 8],
}

impl Xyz {
	pub fn new() -> Self {
		Xyz { xyz: [0; 8] }
	}
}

#[derive(Debug)]
struct Reuse {
	current_time: usize,
	num_intervals: usize,
	free_intervals: *mut (usize, usize),
	all_reuses: *mut Xyz,
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
			all_reuses: null_mut(),
			boost_count: 0,
			is_hibernating: false,
			num_frees: 0,
			num_events: 0,
		}
	}

	pub fn init(&mut self) {
		unsafe {
			//self.free_intervals = page_alloc_overcommit::<(usize, usize)>(RS_SIZE);
			let f_sz = (NUM_FREE_INTERVALS * size_of::<(usize, usize)>()) as f64 / (PAGE as f64);
			// dumb replacement for f64 ceil(), which the rust linker does not like for some reason!
			if (f_sz as u64) as f64 == f_sz {
				self.free_intervals = page_alloc_overcommit::<(usize, usize)>(f_sz as usize * PAGE);
			} else {
				self.free_intervals = page_alloc_overcommit::<(usize, usize)>((f_sz as usize + 1) * PAGE);
			}

			let r_sz = (TARGET_APF * size_of::<Xyz>()) as f64 / (PAGE as f64);
			// dumb replacement for f64 ceil(), which the rust linker does not like for some reason!
			if (r_sz as u64) as f64 == r_sz {
				self.all_reuses = page_alloc_overcommit::<Xyz>(r_sz as usize * PAGE);
			} else {
				self.all_reuses = page_alloc_overcommit::<Xyz>((r_sz as usize + 1) * PAGE);
			}
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
				self.num_intervals = (self.num_intervals + 1) % NUM_FREE_INTERVALS;
			}
		}

		self.num_events = (self.num_events + 1) % NUM_FREE_INTERVALS;
	}

	pub fn on_free(&mut self) {
		if self.is_hibernating {
			return
		}

		log_debug!("num_frees", self.num_frees);
		unsafe { (*self.free_intervals.add(self.num_frees)).0 = self.current_time; }
		self.num_frees = (self.num_frees + 1) % NUM_FREE_INTERVALS;

		self.num_events = (self.num_events + 1) % NUM_FREE_INTERVALS;
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
				for i in 0..NUM_FREE_INTERVALS + 1 {
					*self.free_intervals.add(i) = (0, 0);
				}
			}
			self.num_intervals = 0;
			self.num_frees = 0;
			self.num_events = 0;
		}
	}

	#[inline(always)]
	fn compute_slow(&mut self, wl: usize) -> f64 {
		let mut x: u64 = 0;
		let mut y: u64 = 0;
		let mut z: u64 = 0;

		for i in (0..self.num_intervals).rev() {
			let interval = unsafe { *self.free_intervals.add(i) };
			if interval.1 >= interval.0 && interval.1 - interval.0 < wl {
				unsafe {
					x = x.unchecked_add(min(self.num_events as i64 - wl as i64, interval.0 as i64) as u64);
					y = y.unchecked_add(max(wl, interval.1) as u64);
					z = z.unchecked_add(wl as u64);
				}
			}
		}

		if wl < TARGET_APF {
			let mut xyz = Xyz::new();
			xyz.set_init(true); xyz.set_x(x as u32); xyz.set_y(y as u32); xyz.set_z(z as u32);
			unsafe { *self.all_reuses.add(wl) = xyz; }
		}

		(x.checked_sub(y).unwrap_or(0).checked_add(z).unwrap_or(u64::MAX)) as f64 / (self.num_events as f64 - wl as f64 + 1.0)
	}

	#[inline(always)]
	fn compute_fast(&mut self, wl: usize) -> f64 {
		let lower_bound = if wl <= REUSE_COMPUTE_INTERVAL {
			0
		} else {
			wl - REUSE_COMPUTE_INTERVAL
		};
		let mut reuse = Xyz::new();
		let mut lowest_computed = lower_bound;

		for i in (lower_bound..wl+1).rev() {
			reuse = unsafe { *self.all_reuses.add(i) };
			if reuse.init() {
				lowest_computed = i;
				break;
			}
		}

		if !reuse.init() {
			self.compute_slow(lowest_computed);
		}

		// always recompute the given element
		if lowest_computed == wl {
			lowest_computed = wl - 1;
		}

		for r in lowest_computed+1..wl+1 {
			let prev_reuse = unsafe { *self.all_reuses.add(r-1) };
			let mut x: u64 = 0;
			let mut y: u64 = 0;
			let mut z: u64 = 0;

			if r == 0 {
				for i in (0..self.num_intervals).rev() {
					let interval = unsafe { *self.free_intervals.add(i) };
					if interval.1 >= interval.0 && interval.0 == interval.1 {
						x = x.checked_add(interval.0 as u64).unwrap_or(u64::MAX);
						y = y.checked_add(interval.1 as u64).unwrap_or(u64::MAX);
						z = z.checked_add(1).unwrap_or(u64::MAX);
					}
				}
			} else {
				x += prev_reuse.x() as u64;
				y += prev_reuse.y() as u64;
				z += prev_reuse.z() as u64;
				for i in 0..self.num_intervals {
					let interval = unsafe { *self.free_intervals.add(i) };

					unsafe {
						if interval.1 >= interval.0 && interval.1 - interval.0 + 1 == r {
							x = x.unchecked_add(min(self.num_events - r, interval.0) as u64);
							y = y.unchecked_add(max(r, interval.1) as u64);
							z = z.unchecked_add(r as u64);
						}

						if interval.0 as i64 >= self.num_events as i64 - (r as i64 - 1) {
							x = x.unchecked_add(1);
						}
						if interval.1 <= r - 1 {
							y = y.unchecked_add(1);
						}

						if interval.1 >= interval.0 && interval.1 - interval.0 < r - 1 {
							z = z.unchecked_add(1);
						}
					}
				}
			}

			let mut xyz = Xyz::new();
			xyz.set_init(true); xyz.set_x(x as u32); xyz.set_y(y as u32); xyz.set_z(z as u32);
			unsafe { *self.all_reuses.add(r) = xyz; }
			if r == wl {
				return (x.checked_sub( y).unwrap_or(0).checked_add(z).unwrap_or(u64::MAX)) as f64 / (self.num_events as f64 - wl as f64 + 1.0)
			}
		}

		0.0
	}

	pub fn compute(&mut self, wl: usize) -> f64 {
		if wl >= TARGET_APF {
			self.compute_slow(wl)
		} else {
			self.compute_fast(wl)
		}
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

	pub fn on_fetch(&mut self) { self.num_fetches += 1; log_debug!("Number of fetches", self.num_fetches); }

	pub fn inc_timer(&mut self) {
		self.reuse.inc_timer();
	}

	pub fn demand(&mut self, wl: Option<usize>) -> f64 {
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

	pub fn should_update_slots(&mut self, available_slots: usize) -> Option<usize> {
		self.update_apf();
		let demand = self.demand(Some(self.current_apf)) as usize;
		match (demand as u64).checked_mul(2) {
			Some(res) => if available_slots >= res as usize + 1 {
				Some(demand + 1)
			} else {
				None
			},
			None => None,
		}
	}
}

#[thread_local]
pub static mut APF_INIT: bool = false;