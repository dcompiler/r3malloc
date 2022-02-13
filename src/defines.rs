#[inline(always)]
pub fn lg_page() -> usize {
	12
}

#[inline(always)]
pub fn page() -> usize {
	(1 as usize) << lg_page()
}

#[inline(always)]
pub fn page_mask() -> usize {
	page() - 1
}