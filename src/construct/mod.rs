//! O(n) time and O(1) space SAIS algorithm for read-only byte string.
//!
//! The algorithm was described in [Ge Nong. 2013.
//! Practical linear-time O(1)-workspace suffix sorting for constant
//! alphabets.](https://dl.acm.org/citation.cfm?doid=2493175.2493180).

mod sacak;
mod utils;

pub use sacak::MAX_LENGTH;

/// Suffix array construction algorithm.
pub fn saca(s: &[u8], sa: &mut [u32]) {
    assert!(s.len() <= MAX_LENGTH);
    assert!(s.len() + 1 == sa.len());

    sacak::sacak(s, sa);
}
