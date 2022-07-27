use crate::apf::APF_INIT;
use crate::defines::{align_addr, align_val, page_ceiling, PAGE, PAGE_MASK};
use crate::heap::{Anchor, Descriptor, DescriptorNode, ProcHeap, SbState};
use crate::log_debug;
use crate::pagemap::{PageInfo, SPAGEMAP};
use crate::pages::{page_alloc, page_free};
use crate::size_classes::{
    compute_idx, get_size_class, init_size_class, MAX_SZ, MAX_SZ_IDX, SIZE_CLASSES,
};
use crate::tcache::{TCacheBin, TCACHE};
use atomic::Ordering;
use core::ptr::null_mut;
use likely_stable::{likely, unlikely};

static mut MALLOC_INIT: bool = false;

// This is initialized using the Rust feature const_repeat_expr
// Details here: https://rust-lang.github.io/rfcs/2203-const-repeat-expr.html
const PROC_HEAP_INITIALIZER: ProcHeap = ProcHeap::const_new(0);
pub static mut HEAPS: [ProcHeap; MAX_SZ_IDX] = [PROC_HEAP_INITIALIZER; MAX_SZ_IDX];

fn update_page_map(
    heap: Option<&ProcHeap>,
    ptr: *mut u8,
    desc: Option<&mut Descriptor>,
    sc_idx: usize,
) {
    assert!(!ptr.is_null());

    let mut info = PageInfo::new();
    match desc {
        Some(d) => info.set_desc(d as *mut Descriptor, sc_idx),
        None => info.set_desc(null_mut(), sc_idx),
    }

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

    update_page_map(unsafe { Some(&*heap) }, ptr, Some(desc), sc_idx);
}

fn unregister_desc(heap: Option<&ProcHeap>, superblock: *mut u8) {
    update_page_map(heap, superblock, None, 0);
}

pub fn heap_pop_partial<'a>(heap: &ProcHeap<'a>) -> *mut Descriptor<'a> {
    let list = heap.get_partial_list();
    let mut old_head;

    loop {
        old_head = list.load(Ordering::SeqCst);
        let old_desc = old_head.get_desc();
        if old_desc.is_null() {
            return null_mut();
        }
        let mut new_head = unsafe { (*old_desc).get_next_partial().load(Ordering::SeqCst) };
        let desc = new_head.get_desc();
        let counter = old_head.get_counter();
        new_head.set_desc(desc, counter);

        match list.compare_exchange_weak(old_head, new_head, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => {
                break;
            }
            _ => (),
        }
    }

    old_head.get_desc()
}

pub fn heap_push_partial(desc: *mut Descriptor) {
    let list = unsafe { (*(*desc).get_heap()).get_partial_list() };
    let mut new_head = DescriptorNode::new(null_mut());

    loop {
        let old_head = list.load(Ordering::SeqCst);
        new_head.set_desc(desc, old_head.get_counter() + 1);
        assert_ne!(old_head.get_desc(), new_head.get_desc());
        unsafe {
            (*new_head.get_desc())
                .get_next_partial()
                .store(old_head, Ordering::SeqCst)
        };

        match list.compare_exchange_weak(old_head, new_head, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => {
                break;
            }
            _ => (),
        }
    }
}

pub fn init_malloc() {
    log_debug!();

    // hard assumption that this can't be called concurrently
    unsafe {
        MALLOC_INIT = true;
    }

    // init page map
    unsafe { SPAGEMAP.init() };

    // init heaps
    unsafe {
        for sz_idx in 0..MAX_SZ_IDX {
            HEAPS[sz_idx].set_sc_idx(sz_idx);
        }
    }
}

pub fn thread_finalize() {
    for sc_idx in 1..MAX_SZ_IDX {
        flush_cache(sc_idx, unsafe{ &mut TCACHE[sc_idx] });
    }
}

fn malloc_from_partial(sc_idx: usize, cache: &mut TCacheBin, block_num: usize) -> usize {
    let heap = unsafe { &HEAPS[sc_idx] };

    let desc = heap_pop_partial(heap);
    if desc.is_null() {
        return 0;
    }

    // reserve blocks
    let mut old_anchor;
    let max_count = unsafe { (*desc).get_maxcount() };
    let block_size = unsafe { (*desc).get_block_size() };
    let superblock: *mut u8 = unsafe { (*desc).get_superblock() };

    loop {
        old_anchor = unsafe { (*desc).get_anchor().load(Ordering::SeqCst) };
        if old_anchor.state() == SbState::Empty as u32 {
            unsafe { (*desc).retire() }
            // retry
            return malloc_from_partial(sc_idx, cache, block_num);
        }

        // oldAnchor must be SB_PARTIAL
        // can't be SB_FULL because we *own* the block now
        // and it came from HeapPopPartial
        // can't be SB_EMPTY, we already checked
        // obviously can't be SB_ACTIVE
        assert_eq!(old_anchor.state(), SbState::Partial as u32);

        let mut new_anchor = old_anchor;
        new_anchor.set_count(0);
        // avail value doesn't actually matter
        new_anchor.set_avail(max_count);
        new_anchor.set_state(SbState::Full as u32);

        match unsafe {
            (*desc).get_anchor().compare_exchange_weak(
                old_anchor,
                new_anchor,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
        } {
            Ok(_) => {
                log_debug!("Filled on descriptor", desc, "anchors are", old_anchor, "and", new_anchor);
                break;
            }
            _ => (),
        }
    }

    // will take as many blocks as available from superblock
    // *AND* no thread can do malloc() using this superblock, we
    //  exclusively own it
    // if CAS fails, it just means another thread added more available blocks
    //  through FlushCache, which we can then use
    let blocks_taken = old_anchor.count();
    let avail = old_anchor.avail();

    assert!(avail < max_count);
    let block = unsafe { superblock.offset((avail * block_size) as isize) };

    // cache must be empty at this point
    // and the blocks are already organized as a list
    // so all we need do is "push" that list, a constant time op
    assert_eq!(cache.get_block_num(), 0);
    cache.push_list(block, blocks_taken);

    block_num + blocks_taken as usize
}

fn malloc_from_new_sb(sc_idx: usize, cache: &mut TCacheBin, block_num: usize) -> usize {
    let heap = unsafe { &mut HEAPS[sc_idx] };
    let sc = unsafe { &SIZE_CLASSES[sc_idx] };
    let desc = Descriptor::alloc();
    let block_size = sc.get_block_size();
    let maxcount = sc.get_block_num();

    desc.set_heap(heap);
    desc.set_block_size(block_size);
    desc.set_maxcount(maxcount);
    unsafe {
        desc.set_superblock(&mut *page_alloc::<u8>(sc.get_sb_size() as usize));
    }

    let mut anchor = Anchor::new();
    anchor.set_avail(maxcount);
    anchor.set_count(0);
    anchor.set_state(SbState::Full as u32);
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

    assert!(anchor.avail() < maxcount || anchor.state() == SbState::Full as u32);
    assert!(anchor.count() < maxcount);

    register_desc(desc);
    assert!(anchor.state() == SbState::Full as u32);

    block_num + maxcount as usize
}

fn fill_cache(sc_idx: usize, cache: &mut TCacheBin) {
    let mut block_num = 0;

    block_num = malloc_from_partial(sc_idx, cache, block_num);

    if block_num == 0 {
        block_num = malloc_from_new_sb(sc_idx, cache, block_num);
    }

    let sc = unsafe { &SIZE_CLASSES[sc_idx] };
    assert!(block_num > 0);
    assert!(block_num <= sc.get_cache_block_num() as usize);
}

fn flush_cache(sc_idx: usize, cache: &mut TCacheBin) {
    let heap = unsafe { &HEAPS[sc_idx] };
    let sc = unsafe { &SIZE_CLASSES[sc_idx] };
    let sb_size = sc.get_sb_size();
    let block_size = sc.get_block_size();
    let maxcount = sc.get_block_num();

    while cache.get_block_num() > 0 {
        let head = cache.peek_block();
        let mut tail = head;
        let info = unsafe { SPAGEMAP.get_page_info(head) };
        let desc = info.get_desc();
        let superblock = unsafe { (*desc).get_superblock() };
        let mut block_count = 1;

        while cache.get_block_num() > block_count {
            let ptr: *mut u8 = unsafe { *(tail as *mut *mut u8) };
            if unsafe {
                ptr.offset_from(superblock) < 0
                    || ptr.offset_from(superblock.offset(sb_size as isize)) >= 0
            } {
                break;
            }

            block_count += 1;
            tail = ptr;
        }

        cache.pop_list(unsafe { *(tail as *mut *mut u8) }, block_count);

        let idx = compute_idx(superblock, head, sc_idx);
        let mut old_anchor;
        let mut new_anchor;

        loop {
            old_anchor = unsafe { (*desc).get_anchor().load(Ordering::SeqCst) };
            new_anchor = old_anchor;
            let next: *mut u8 =
                unsafe { superblock.offset((old_anchor.avail() * block_size) as isize) };
            unsafe {
                *(tail as *mut *mut u8) = next;
            }

            new_anchor.set_avail(idx);

            if old_anchor.state() == SbState::Full as u32 {
                new_anchor.set_state(SbState::Partial as u32);
            }

            assert!(unsafe { old_anchor.count() < (*desc).get_maxcount() });
            if unsafe { old_anchor.count() + block_count == (*desc).get_maxcount() } {
                new_anchor.set_count(unsafe { (*desc).get_maxcount() - 1 });
                new_anchor.set_state(SbState::Empty as u32);
            } else {
                new_anchor.set_count(new_anchor.count() + block_count);
            }

            match unsafe {
                (*desc).get_anchor().compare_exchange_weak(
                    old_anchor,
                    new_anchor,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
            } {
                Ok(_) => {
                    log_debug!("Flushed on descriptor", desc, "anchors are", old_anchor, "and", new_anchor);
                    break;
                }
                _ => (),
            }
        }

        assert!(old_anchor.avail() < maxcount || old_anchor.state() == SbState::Full as u32);
        assert!(new_anchor.avail() < maxcount);
        assert!(new_anchor.count() < maxcount);

        if new_anchor.state() == SbState::Empty as u32 {
            unregister_desc(Some(heap), superblock);

            unsafe {
                page_free(superblock, heap.get_size_class().get_sb_size() as usize);
            }
        } else if old_anchor.state() == SbState::Full as u32 {
            heap_push_partial(desc);
        }
    }
}

fn cut_cache(sc_idx: usize, cache: &mut TCacheBin, cut_by: u32) {
    let heap = unsafe { &HEAPS[sc_idx] };
    let sc = unsafe { &SIZE_CLASSES[sc_idx] };
    let sb_size = sc.get_sb_size();
    let block_size = sc.get_block_size();
    let maxcount = sc.get_block_num();

        let head = cache.peek_block();
        let mut tail = head;
        let info = unsafe { SPAGEMAP.get_page_info(head) };
        let desc = info.get_desc();
        let superblock = unsafe { (*desc).get_superblock() };
        let mut block_count = 1;

        while block_count != cut_by {
            let ptr: *mut u8 = unsafe { *(tail as *mut *mut u8) };
            if unsafe {
                ptr.offset_from(superblock) < 0
                    || ptr.offset_from(superblock.offset(sb_size as isize)) >= 0
            } {
                break;
            }

            block_count += 1;
            tail = ptr;
        }

        cache.pop_list(unsafe { *(tail as *mut *mut u8) }, block_count);

        let idx = compute_idx(superblock, head, sc_idx);
        let mut old_anchor;
        let mut new_anchor;

        loop {
            old_anchor = unsafe { (*desc).get_anchor().load(Ordering::SeqCst) };
            new_anchor = old_anchor;
            let next: *mut u8 =
                unsafe { superblock.offset((old_anchor.avail() * block_size) as isize) };
            unsafe {
                *(tail as *mut *mut u8) = next;
            }

            new_anchor.set_avail(idx);

            if old_anchor.state() == SbState::Full as u32 {
                new_anchor.set_state(SbState::Partial as u32);
            }

            log_debug!(maxcount, new_anchor.count(), cut_by, block_count);
            assert!(unsafe { old_anchor.count() < (*desc).get_maxcount() });
            if unsafe { old_anchor.count() + block_count == (*desc).get_maxcount() } {
                new_anchor.set_count(unsafe { (*desc).get_maxcount() - 1 });
                new_anchor.set_state(SbState::Empty as u32);
            } else {
                new_anchor.set_count(new_anchor.count() + block_count);
            }

            match unsafe {
                (*desc).get_anchor().compare_exchange_weak(
                    old_anchor,
                    new_anchor,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
            } {
                Ok(_) => {
                    log_debug!("Cut on descriptor", desc, "anchors are", old_anchor, "and", new_anchor);
                    break;
                }
                _ => (),
            }
        }

        assert!(old_anchor.avail() < maxcount || old_anchor.state() == SbState::Full as u32);
        assert!(new_anchor.avail() < maxcount);
        assert!(new_anchor.count() < maxcount);

        if new_anchor.state() == SbState::Empty as u32 {
            unregister_desc(Some(heap), superblock);

            unsafe {
                page_free(superblock, heap.get_size_class().get_sb_size() as usize);
            }
        } else if old_anchor.state() == SbState::Full as u32 {
            heap_push_partial(desc);
        }
}

#[inline(always)]
pub fn do_malloc(size: usize) -> *mut u8 {
    // ensure malloc is initialized
    if unlikely(unsafe { !MALLOC_INIT }) {
        init_malloc();
    }

    // init size classes (here because APF analysis is per thread per sizeclass
    if unlikely(unsafe { !APF_INIT }) {
            init_size_class();
    }

    // large block allocation
    if unlikely(size > MAX_SZ) {
        let pages = page_ceiling(size);
        let desc = Descriptor::alloc();

        desc.set_heap(null_mut());
        desc.set_block_size(pages as u32);
        desc.set_maxcount(1);
        unsafe {
            desc.set_superblock(&mut *page_alloc::<u8>(pages));
        }

        let mut anchor = Anchor::new();
        anchor.set_avail(0);
        anchor.set_count(0);
        anchor.set_state(SbState::Full as u32);

        desc.get_anchor().store(anchor, Ordering::SeqCst);

        register_desc(desc);

        let ptr = desc.get_superblock();
        log_debug!("Large, ptr: ", ptr);
        return ptr;
    }

    let sc_idx = get_size_class(size);

    unsafe {
        SIZE_CLASSES[sc_idx].get_apf().on_allocation();
        SIZE_CLASSES[sc_idx].get_apf().inc_timer();
    }

    //unsafe { log_debug!("Demand: ", SIZE_CLASSES[sc_idx].get_apf().demand(None)); }

    let cache = unsafe { &mut TCACHE[sc_idx] };

    //unsafe { log_debug!("Thread cache: ", cache, " size class", SIZE_CLASSES[sc_idx]) };

    if unlikely(cache.get_block_num() == 0) {
        fill_cache(sc_idx, cache);
        unsafe { SIZE_CLASSES[sc_idx].get_apf().on_fetch(); }
    }

    cache.pop_block()
}

#[inline(always)]
pub fn do_aligned_alloc(alignment: usize, _size: usize) -> *mut u8 {
    if unlikely((alignment != 0) && !(alignment & (alignment - 1)) == 0) {
        return null_mut();
    }
    let mut size = align_val(_size, alignment);

    assert!(size > 0 && alignment > 0 && size >= alignment);

    // ensure malloc is initialized
    if unlikely(unsafe { !MALLOC_INIT }) {
        init_malloc();
    }

    // init size classes (here because APF analysis is per thread per sizeclass
    if unlikely(unsafe { !APF_INIT }) {
        init_size_class();
    }

    if unlikely(size > PAGE) {
        size = core::cmp::max(size, MAX_SZ + 1);

        let need_more_pages = alignment > PAGE;
        if unlikely(need_more_pages) {
            size += alignment;
        }

        let pages = page_ceiling(size);
        let desc = Descriptor::alloc();

        let mut ptr = unsafe { page_alloc::<u8>(pages) };

        desc.set_heap(null_mut());
        desc.set_block_size(pages as u32);
        desc.set_maxcount(1);
        desc.set_superblock(ptr);

        let mut anchor = Anchor::new();
        anchor.set_avail(0);
        anchor.set_count(0);
        anchor.set_state(SbState::Full as u32);

        desc.get_anchor().store(anchor, Ordering::SeqCst);

        register_desc(desc);

        if unlikely(need_more_pages) {
            ptr = align_addr(ptr, alignment);
            assert!(unsafe {
                ptr.offset(size as isize)
                    .offset_from(desc.get_superblock().offset(desc.get_block_size() as isize))
                    <= 0
            });

            update_page_map(None, ptr, Some(desc), 0);
        }

        log_debug!("Large, ptr: ", ptr);
        return ptr;
    }

    assert!(size <= PAGE);
    let sc_idx = get_size_class(size);

    let cache = unsafe { &mut TCACHE[sc_idx] };
    if unlikely(cache.get_block_num() == 0) {
        fill_cache(sc_idx, cache);
    }

    cache.pop_block()
}

#[inline(always)]
pub fn do_free(ptr: *mut u8) {
    if unlikely(ptr.is_null()) {
        return;
    }

    let info = unsafe { SPAGEMAP.get_page_info(ptr) };
    let desc = info.get_desc();

    let sc_idx = info.get_sc_idx();

    //log_debug!("Free: desc ", desc, ", ptr ", ptr);

    if unlikely(sc_idx == 0) {
        let superblock = unsafe { (*desc).get_superblock() };

        unregister_desc(None, superblock);
        if unlikely(ptr != superblock) {
            unregister_desc(None, ptr);
        }

        unsafe {
            page_free(superblock, (*desc).get_block_size() as usize);
            (*desc).retire();
        }

        return;
    }

    let cache = unsafe { &mut TCACHE[sc_idx] };
    let sc = unsafe { &SIZE_CLASSES[sc_idx] };

    unsafe { SIZE_CLASSES[sc_idx].get_apf().on_free(); }

    if unlikely(cache.get_block_num() >= sc.get_cache_block_num()) {
        flush_cache(sc_idx, cache);
    } else {
        unsafe {
            if let Some(num_slots) = (&mut SIZE_CLASSES[sc_idx]).get_apf().should_update_slots(cache.get_block_num() as usize) {
                log_debug!("Giving up", num_slots, "slots.");
                cut_cache(sc_idx, cache, num_slots as u32);
            }
        }
    }

    cache.push_block(ptr);
}
