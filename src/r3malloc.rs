use crate::heap::{Anchor, Descriptor, ProcHeap, SbState};
use crate::log_debug;
use crate::pages::page_alloc;
use crate::size_classes::{init_size_class, get_size_class, MAX_SZ_IDX, SIZE_CLASSES, MAX_SZ};
use crate::tcache::{TCacheBin, TCACHE};
use atomic::Ordering;
use likely_stable::unlikely;

static mut MALLOC_INIT: bool = false;

// This is initialized using the Rust feature const_repeat_expr
// Details here: https://rust-lang.github.io/rfcs/2203-const-repeat-expr.html
const PROC_HEAP_INITIALIZER: ProcHeap = ProcHeap::const_new(0);
pub static mut HEAPS: [ProcHeap; MAX_SZ_IDX] = [PROC_HEAP_INITIALIZER; MAX_SZ_IDX];

fn init_malloc() {
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

fn malloc_from_new_sb(sc_idx: usize, cache: &mut TCacheBin, block_num: usize) -> usize {
    let heap = unsafe { &HEAPS[sc_idx] };
    let sc = unsafe { &SIZE_CLASSES[sc_idx] };
    let desc = Descriptor::alloc();
    let block_size = sc.get_block_size();
    let maxcount = sc.get_block_num();

    // FIXME: ASSERT(desc); should we check for this???

    desc.set_heap(heap);
    desc.set_block_size(block_size);
    desc.set_maxcount(maxcount);
    unsafe {
        desc.set_superblock(&mut *page_alloc::<u8>(sc.get_sb_size() as usize));
    }

    let mut anchor = Anchor::new();
    anchor.set_avail(maxcount);
    anchor.set_count(0);
    anchor.set_state(SbState::Full);
    desc.get_anchor().store(anchor, Ordering::SeqCst);

    let superblock: *mut u8 = desc.get_superblock() as *mut u8;

    for i in 0..maxcount - 1 {
        unsafe {
            let block = superblock.offset((i * block_size) as isize);
            let next = superblock.offset(((i + 1) * block_size) as isize);
            *(block as *mut *mut u8) = next;
        }
    }

    cache.push_list(superblock, maxcount);

    assert!(anchor.get_avail() < maxcount || anchor.get_state() == SbState::Full);
    assert!(anchor.get_count() < maxcount);

    // FIXME: register_desc(desc)
    // FIXME: assert_eq!(anchor.get_state() == SbState::Full)

    block_num + maxcount as usize
}

fn fill_cache(sc_idx: usize, cache: &mut TCacheBin) {
    let mut block_num = 0;

    // FIXME: malloc from partial sb

    if block_num == 0 {
        block_num = malloc_from_new_sb(sc_idx, cache, block_num);
    }

    let sc = unsafe { &SIZE_CLASSES[sc_idx] };
    assert!(block_num > 0);
    assert!(block_num <= sc.get_cache_block_num() as usize);
}

pub fn do_malloc(size: usize) -> *mut u8 {
    // ensure malloc is initialized
    if unlikely(unsafe { !MALLOC_INIT }) {
        init_malloc();
    }

    // large block allocation
    if unlikely(size > MAX_SZ) {
        todo!();
    }

    let sc_idx = get_size_class(size);

    let cache = unsafe { &mut TCACHE[sc_idx] };

    if unlikely(cache.get_block_num() == 0) {
        fill_cache(sc_idx, cache);
    }

    cache.pop_block()
}