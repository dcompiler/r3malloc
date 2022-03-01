use crate::defines::{page_ceiling, PAGE, PAGE_MASK};
use crate::heap::{Anchor, Descriptor, ProcHeap, SbState};
use crate::log_debug;
use crate::pagemap::{PageInfo, SPAGEMAP};
use crate::pages::page_alloc;
use crate::size_classes::{get_size_class, init_size_class, MAX_SZ, MAX_SZ_IDX, SIZE_CLASSES};
use crate::tcache::{TCacheBin, TCACHE};
use atomic::Ordering;
use core::ptr::null_mut;
use likely_stable::{likely, unlikely};

static mut MALLOC_INIT: bool = false;

// This is initialized using the Rust feature const_repeat_expr
// Details here: https://rust-lang.github.io/rfcs/2203-const-repeat-expr.html
const PROC_HEAP_INITIALIZER: ProcHeap = ProcHeap::const_new(0);
pub static mut HEAPS: [ProcHeap; MAX_SZ_IDX] = [PROC_HEAP_INITIALIZER; MAX_SZ_IDX];

fn update_page_map(heap: Option<&ProcHeap>, ptr: *mut u8, desc: &mut Descriptor, sc_idx: usize) {
    assert!(!ptr.is_null());

    let mut info = PageInfo::new();
    info.set_desc(desc as *mut Descriptor, sc_idx);

    match heap {
        Some(h) => {
            let sb_size = h.get_size_class().get_sb_size();
            assert_eq!((sb_size as usize) & PAGE_MASK, 0);
            for i in (0..sb_size).step_by(PAGE) {
                unsafe { SPAGEMAP.set_page_info(info, ptr.offset(i as isize)) }
            }
        }
        None => {
            unsafe { SPAGEMAP.set_page_info(info, ptr) };
        }
    }
}

fn register_desc(desc: &mut Descriptor) {
    let heap = desc.get_heap();
    let ptr = desc.get_superblock();
    let mut sc_idx = 0;
    if likely(!heap.is_null()) {
        sc_idx = unsafe { (*heap).get_sc_idx() };
    }

    update_page_map(unsafe { Some(&*heap) }, ptr, desc, sc_idx);
}

fn init_malloc() {
    log_debug!();

    // hard assumption that this can't be called concurrently
    unsafe {
        MALLOC_INIT = true;
    }

    // init size classes
    init_size_class();

    // init page map
    unsafe { SPAGEMAP.init() };

    // init heaps
    unsafe {
        for sz_idx in 0..MAX_SZ_IDX {
            HEAPS[sz_idx].set_sc_idx(sz_idx);
        }
    }
}

fn malloc_from_new_sb(sc_idx: usize, cache: &mut TCacheBin, block_num: usize) -> usize {
    let heap = unsafe { &mut HEAPS[sc_idx] };
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

    register_desc(desc);
    assert!(anchor.get_state() == SbState::Full);

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
        let pages = page_ceiling(size);
        let desc = Descriptor::alloc();
        // FIXME: ASSERT(desc); should we check for this???

        desc.set_heap(null_mut());
        desc.set_block_size(pages as u32);
        desc.set_maxcount(1);
        unsafe {
            desc.set_superblock(&mut *page_alloc::<u8>(pages));
        }

        let mut anchor = Anchor::new();
        anchor.set_avail(0);
        anchor.set_count(0);
        anchor.set_state(SbState::Full);

        desc.get_anchor().store(anchor, Ordering::SeqCst);

        register_desc(desc);

        let ptr = desc.get_superblock();
        log_debug!("Large, ptr: {}", ptr);
        return ptr;
    }

    let sc_idx = get_size_class(size);

    let cache = unsafe { &mut TCACHE[sc_idx] };

    if unlikely(cache.get_block_num() == 0) {
        fill_cache(sc_idx, cache);
    }

    cache.pop_block()
}
