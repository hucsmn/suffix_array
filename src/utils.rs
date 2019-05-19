/// Calculate the longest common prefix length of two string.
pub fn lcp(xs: &[u8], ys: &[u8]) -> usize {
    Iterator::zip(xs.iter(), ys.iter())
        .take_while(|(&x, &y)| x == y)
        .count()
}

/// Truncate string.
pub fn trunc(s: &[u8], max: usize) -> &[u8] {
    &s[..Ord::min(s.len(), max)]
}
