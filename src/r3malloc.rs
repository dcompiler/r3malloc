use crate::log_debug;
use crate::size_classes::{init_size_class, MAX_SZ_IDX};
use crate::heap::{ProcHeap};

static mut MALLOC_INIT: bool = false;

// This is initialized using the Rust feature const_repeat_expr
// Details here: https://rust-lang.github.io/rfcs/2203-const-repeat-expr.html
const PROC_HEAP_INITIALIZER: ProcHeap = ProcHeap::const_new(0);
pub static mut HEAPS: [ProcHeap; MAX_SZ_IDX] = [PROC_HEAP_INITIALIZER; MAX_SZ_IDX];

pub fn init_malloc() {
    log_debug!();

    // hard assumption that this can't be called concurrently
    unsafe {
        MALLOC_INIT = true;
    }

    // init size classes
    init_size_class();

    // FIXME: init page map

    // init heaps
    unsafe {
        for sz_idx in 0..MAX_SZ_IDX {
            HEAPS[sz_idx].set_sc_idx(sz_idx);
        }
    }
}
