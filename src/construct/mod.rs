//! O(n) time and O(1) space SAIS algorithm for read-only byte string.
//!
//! Algorithm of the initial recursion level was described in [Ge Nong. 2013.
//! Practical linear-time O(1)-workspace suffix sorting for constant
//! alphabets.](https://dl.acm.org/citation.cfm?doid=2493175.2493180).
//! The algorithm used in the inner recursion levels was taken from
//! [Li, Z., Jian, L. and Huo, H. Optimal In-Place Suffix
//! Sorting](https://arxiv.org/abs/1610.08305).

mod sais_bytes;
mod sais_ints_mut;
mod utils;

pub fn construct(s: &[u8], sa: &mut [u32]) {
    assert!(s.len() <= sais_bytes::MAX_LENGTH);
    assert!(s.len() + 1 == sa.len());

    sais_bytes::sais_bytes(s, sa);
}
