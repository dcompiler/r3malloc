// Credit: https://gist.github.com/tstellanova/9b8ec9f1a6d4d928931d171c7b3b914a
use core::sync::atomic::{AtomicBool, Ordering};
use core::hint::spin_loop;

pub struct Mutex {
    lock: AtomicBool,
}
  
impl Mutex {
    pub fn acquire(&mut self) {
    	loop {
	        match self.lock.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Acquire) {
	        	Ok(_) => {
	        		while self.lock.load(Ordering::Relaxed) {
		                spin_loop();
		            }
	        	},
	        	Err(_) => (),
	        }
	    }
    }

    pub fn release(&mut self) {
        match self.lock.compare_exchange_weak(true, false, Ordering::Acquire, Ordering::Acquire) {
        	Ok(_) => (),
        	Err(_) => (),
        }
    }
  
}