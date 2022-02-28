pub const LG_PAGE: usize = 12;
const LG_CACHELINE: usize = 6;

pub const PAGE: usize = (1 as usize) << LG_PAGE;
pub const PAGE_MASK: usize = PAGE - 1;
pub const CACHELINE: usize = (1 as usize) << LG_CACHELINE;
pub const CACHELINE_MASK: usize = CACHELINE - 1;
