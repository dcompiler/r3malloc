const LG_PAGE: usize = 12;
pub const PAGE: usize = (1 as usize) << LG_PAGE;
pub const PAGE_MASK: usize = PAGE - 1;