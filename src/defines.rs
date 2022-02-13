#[inline(always)]
pub const fn lg_page() -> usize {
	12
}

#[inline(always)]
pub const fn page() -> usize {
	(1 as usize) << lg_page()
}

#[inline(always)]
pub const fn page_mask() -> usize {
	page() - 1
}