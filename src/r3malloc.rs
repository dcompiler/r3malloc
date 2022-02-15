use crate::log_debug;
use crate::size_classes::init_size_class;

static mut MALLOC_INIT: bool = false;

pub fn init_malloc() {
    log_debug!();

    // hard assumption that this can't be called concurrently
    unsafe {
        MALLOC_INIT = true;
    }

    // init size classes
    init_size_class();

    // FIXME: init page map

    // FIXME: init heaps
}
