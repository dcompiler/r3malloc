use crate::defines::PAGE;
use crate::heap::MAX_BLOCK_NUM;
use core::assert;

#[derive(Copy, Clone)]
pub struct SizeClassData {
    block_size: u32,
    sb_size: u32,
    block_num: u32,
    cache_block_num: u32,
}

pub const MAX_SZ_IDX: usize = 40;
const MAX_SZ: usize = (1 << 13) + (1 << 11) * 3;

pub enum SizeClassOpt {
    Yes,
    No,
}

fn size_class_filter_init(
    (_index, lg_grp, lg_delta, ndelta, _psz, bin, pgs, _lg_delta_lookup): &(
        u32,
        u32,
        u32,
        u32,
        SizeClassOpt,
        SizeClassOpt,
        u32,
        u32,
    ),
) -> Option<SizeClassData> {
    match bin {
        SizeClassOpt::Yes => Some(SizeClassData {
            block_size: ((1 as u32) << lg_grp) + (ndelta << lg_delta),
            sb_size: pgs * (PAGE as u32),
            block_num: 0,
            cache_block_num: 0,
        }),
        SizeClassOpt::No => None,
    }
}

fn size_classes() -> [SizeClassData; MAX_SZ_IDX] {
    let mut size_classes = [SizeClassData {
        block_size: 0,
        sb_size: 0,
        block_num: 0,
        cache_block_num: 0,
    }; MAX_SZ_IDX];

    let mut i: usize = 1;
    let mut j: usize = 0;
    while i < MAX_SZ_IDX {
        match size_class_filter_init(&SIZE_CLASSES_TABLE[j]) {
            Some(sc) => {
                size_classes[i] = sc;
                i += 1
            }
            None => (),
        };
        j += 1;
    }
    size_classes
}

// NOTE: this is initialized in init_size_class
// maybe we want to wrap it into an Option to ensure initialization?
pub static mut SIZE_CLASSES: [SizeClassData; MAX_SZ_IDX] = [SizeClassData {
    block_size: 0,
    sb_size: 0,
    block_num: 0,
    cache_block_num: 0,
}; MAX_SZ_IDX];
static mut SIZE_CLASS_LOOKUP: [usize; MAX_SZ + 1] = [0; MAX_SZ + 1];

pub fn init_size_class() {
    unsafe {
        SIZE_CLASSES = size_classes();
    }

    // each superblock has to contain several block *perfectly*
    for sc_idx in 1..MAX_SZ_IDX {
        let sc = unsafe { SIZE_CLASSES[sc_idx] };
        let block_size = sc.block_size;
        let mut sb_size = sc.sb_size;

        if sb_size > block_size && (sb_size % block_size) == 0 {
            continue;
        }

        while block_size >= sb_size {
            sb_size += sc.sb_size;
        }

        // update value in SIZE_CLASSES
        unsafe {
            SIZE_CLASSES[sc_idx].sb_size = sc.sb_size;
        }
    }

    for sc_idx in 1..MAX_SZ_IDX {
        let mut sc = unsafe { SIZE_CLASSES[sc_idx] };
        let mut sb_size = sc.sb_size;

        // increase superblock size if needed
        // 64 KB
        while sb_size < (16 * PAGE) as u32 {
            sb_size += sc.sb_size;
        }
        sc.sb_size = sb_size;

        // fill block_num and cache_block_num
        sc.block_num = sc.sb_size / sc.block_size;
        sc.cache_block_num = sc.block_num * 1;

        assert!(sc.block_num > 0);
        assert!((sc.block_num as u64) < MAX_BLOCK_NUM);
        assert!(sc.block_num >= sc.cache_block_num);

        // update value in SIZE_CLASSES
        unsafe {
            SIZE_CLASSES[sc_idx] = sc;
        }
    }

    // first size class reserved for large allocations
    let mut lookup_idx: usize = 0;
    for sc_idx in 1..MAX_SZ_IDX {
        let sc = unsafe { SIZE_CLASSES[sc_idx] };
        while lookup_idx <= sc.block_size as usize {
            unsafe {
                SIZE_CLASS_LOOKUP[lookup_idx] = sc_idx;
            }
            lookup_idx += 1;
        }
    }
}

// size class data, from jemalloc 5.0
const SIZE_CLASSES_TABLE: [(u32, u32, u32, u32, SizeClassOpt, SizeClassOpt, u32, u32); 235] = [
    /* index, lg_grp, lg_delta, ndelta, psz, bin, pgs, lg_delta_lookup */
    (0, 3, 3, 0, SizeClassOpt::No, SizeClassOpt::Yes, 1, 3),
    (1, 3, 3, 1, SizeClassOpt::No, SizeClassOpt::Yes, 1, 3),
    (2, 3, 3, 2, SizeClassOpt::No, SizeClassOpt::Yes, 3, 3),
    (3, 3, 3, 3, SizeClassOpt::No, SizeClassOpt::Yes, 1, 3),
    (4, 5, 3, 1, SizeClassOpt::No, SizeClassOpt::Yes, 5, 3),
    (5, 5, 3, 2, SizeClassOpt::No, SizeClassOpt::Yes, 3, 3),
    (6, 5, 3, 3, SizeClassOpt::No, SizeClassOpt::Yes, 7, 3),
    (7, 5, 3, 4, SizeClassOpt::No, SizeClassOpt::Yes, 1, 3),
    (8, 6, 4, 1, SizeClassOpt::No, SizeClassOpt::Yes, 5, 4),
    (9, 6, 4, 2, SizeClassOpt::No, SizeClassOpt::Yes, 3, 4),
    (10, 6, 4, 3, SizeClassOpt::No, SizeClassOpt::Yes, 7, 4),
    (11, 6, 4, 4, SizeClassOpt::No, SizeClassOpt::Yes, 1, 4),
    (12, 7, 5, 1, SizeClassOpt::No, SizeClassOpt::Yes, 5, 5),
    (13, 7, 5, 2, SizeClassOpt::No, SizeClassOpt::Yes, 3, 5),
    (14, 7, 5, 3, SizeClassOpt::No, SizeClassOpt::Yes, 7, 5),
    (15, 7, 5, 4, SizeClassOpt::No, SizeClassOpt::Yes, 1, 5),
    (16, 8, 6, 1, SizeClassOpt::No, SizeClassOpt::Yes, 5, 6),
    (17, 8, 6, 2, SizeClassOpt::No, SizeClassOpt::Yes, 3, 6),
    (18, 8, 6, 3, SizeClassOpt::No, SizeClassOpt::Yes, 7, 6),
    (19, 8, 6, 4, SizeClassOpt::No, SizeClassOpt::Yes, 1, 6),
    (20, 9, 7, 1, SizeClassOpt::No, SizeClassOpt::Yes, 5, 7),
    (21, 9, 7, 2, SizeClassOpt::No, SizeClassOpt::Yes, 3, 7),
    (22, 9, 7, 3, SizeClassOpt::No, SizeClassOpt::Yes, 7, 7),
    (23, 9, 7, 4, SizeClassOpt::No, SizeClassOpt::Yes, 1, 7),
    (24, 10, 8, 1, SizeClassOpt::No, SizeClassOpt::Yes, 5, 8),
    (25, 10, 8, 2, SizeClassOpt::No, SizeClassOpt::Yes, 3, 8),
    (26, 10, 8, 3, SizeClassOpt::No, SizeClassOpt::Yes, 7, 8),
    (27, 10, 8, 4, SizeClassOpt::No, SizeClassOpt::Yes, 1, 8),
    (28, 11, 9, 1, SizeClassOpt::No, SizeClassOpt::Yes, 5, 9),
    (29, 11, 9, 2, SizeClassOpt::No, SizeClassOpt::Yes, 3, 9),
    (30, 11, 9, 3, SizeClassOpt::No, SizeClassOpt::Yes, 7, 9),
    (31, 11, 9, 4, SizeClassOpt::Yes, SizeClassOpt::Yes, 1, 9),
    (32, 12, 10, 1, SizeClassOpt::No, SizeClassOpt::Yes, 5, 0),
    (33, 12, 10, 2, SizeClassOpt::No, SizeClassOpt::Yes, 3, 0),
    (34, 12, 10, 3, SizeClassOpt::No, SizeClassOpt::Yes, 7, 0),
    (35, 12, 10, 4, SizeClassOpt::Yes, SizeClassOpt::Yes, 2, 0),
    (36, 13, 11, 1, SizeClassOpt::No, SizeClassOpt::Yes, 5, 0),
    (37, 13, 11, 2, SizeClassOpt::Yes, SizeClassOpt::Yes, 3, 0),
    (38, 13, 11, 3, SizeClassOpt::No, SizeClassOpt::Yes, 7, 0),
    (39, 13, 11, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (40, 14, 12, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (41, 14, 12, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (42, 14, 12, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (43, 14, 12, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (44, 15, 13, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (45, 15, 13, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (46, 15, 13, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (47, 15, 13, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (48, 16, 14, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (49, 16, 14, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (50, 16, 14, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (51, 16, 14, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (52, 17, 15, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (53, 17, 15, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (54, 17, 15, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (55, 17, 15, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (56, 18, 16, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (57, 18, 16, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (58, 18, 16, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (59, 18, 16, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (60, 19, 17, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (61, 19, 17, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (62, 19, 17, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (63, 19, 17, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (64, 20, 18, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (65, 20, 18, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (66, 20, 18, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (67, 20, 18, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (68, 21, 19, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (69, 21, 19, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (70, 21, 19, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (71, 21, 19, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (72, 22, 20, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (73, 22, 20, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (74, 22, 20, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (75, 22, 20, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (76, 23, 21, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (77, 23, 21, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (78, 23, 21, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (79, 23, 21, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (80, 24, 22, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (81, 24, 22, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (82, 24, 22, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (83, 24, 22, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (84, 25, 23, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (85, 25, 23, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (86, 25, 23, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (87, 25, 23, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (88, 26, 24, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (89, 26, 24, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (90, 26, 24, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (91, 26, 24, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (92, 27, 25, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (93, 27, 25, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (94, 27, 25, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (95, 27, 25, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (96, 28, 26, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (97, 28, 26, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (98, 28, 26, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (99, 28, 26, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (100, 29, 27, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (101, 29, 27, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (102, 29, 27, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (103, 29, 27, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (104, 30, 28, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (105, 30, 28, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (106, 30, 28, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (107, 30, 28, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (108, 31, 29, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (109, 31, 29, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (110, 31, 29, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (111, 31, 29, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (112, 32, 30, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (113, 32, 30, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (114, 32, 30, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (115, 32, 30, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (116, 33, 31, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (117, 33, 31, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (118, 33, 31, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (119, 33, 31, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (120, 34, 32, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (121, 34, 32, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (122, 34, 32, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (123, 34, 32, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (124, 35, 33, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (125, 35, 33, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (126, 35, 33, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (127, 35, 33, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (128, 36, 34, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (129, 36, 34, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (130, 36, 34, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (131, 36, 34, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (132, 37, 35, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (133, 37, 35, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (134, 37, 35, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (135, 37, 35, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (136, 38, 36, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (137, 38, 36, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (138, 38, 36, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (139, 38, 36, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (140, 39, 37, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (141, 39, 37, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (142, 39, 37, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (143, 39, 37, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (144, 40, 38, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (145, 40, 38, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (146, 40, 38, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (147, 40, 38, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (148, 41, 39, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (149, 41, 39, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (150, 41, 39, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (151, 41, 39, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (152, 42, 40, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (153, 42, 40, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (154, 42, 40, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (155, 42, 40, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (156, 43, 41, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (157, 43, 41, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (158, 43, 41, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (159, 43, 41, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (160, 44, 42, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (161, 44, 42, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (162, 44, 42, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (163, 44, 42, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (164, 45, 43, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (165, 45, 43, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (166, 45, 43, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (167, 45, 43, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (168, 46, 44, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (169, 46, 44, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (170, 46, 44, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (171, 46, 44, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (172, 47, 45, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (173, 47, 45, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (174, 47, 45, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (175, 47, 45, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (176, 48, 46, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (177, 48, 46, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (178, 48, 46, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (179, 48, 46, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (180, 49, 47, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (181, 49, 47, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (182, 49, 47, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (183, 49, 47, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (184, 50, 48, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (185, 50, 48, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (186, 50, 48, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (187, 50, 48, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (188, 51, 49, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (189, 51, 49, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (190, 51, 49, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (191, 51, 49, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (192, 52, 50, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (193, 52, 50, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (194, 52, 50, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (195, 52, 50, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (196, 53, 51, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (197, 53, 51, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (198, 53, 51, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (199, 53, 51, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (200, 54, 52, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (201, 54, 52, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (202, 54, 52, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (203, 54, 52, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (204, 55, 53, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (205, 55, 53, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (206, 55, 53, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (207, 55, 53, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (208, 56, 54, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (209, 56, 54, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (210, 56, 54, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (211, 56, 54, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (212, 57, 55, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (213, 57, 55, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (214, 57, 55, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (215, 57, 55, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (216, 58, 56, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (217, 58, 56, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (218, 58, 56, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (219, 58, 56, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (220, 59, 57, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (221, 59, 57, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (222, 59, 57, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (223, 59, 57, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (224, 60, 58, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (225, 60, 58, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (226, 60, 58, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (227, 60, 58, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (228, 61, 59, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (229, 61, 59, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (230, 61, 59, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (231, 61, 59, 4, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (232, 62, 60, 1, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (233, 62, 60, 2, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
    (234, 62, 60, 3, SizeClassOpt::Yes, SizeClassOpt::No, 0, 0),
];
