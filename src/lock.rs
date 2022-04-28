use core::sync::atomic::{AtomicBool, Ordering, fence};

pub struct Mutex {
    lock: AtomicBool,
}

impl Mutex {
	pub const fn new() -> Self {
		Mutex { lock: AtomicBool::new(false) }
	}

    pub fn acquire(&mut self) {
	    while self.lock.compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed).is_err() {}
	    fence(Ordering::Acquire);
    }

    pub fn release(&mut self) {
        self.lock.store(false, Ordering::Release)
    }

}
