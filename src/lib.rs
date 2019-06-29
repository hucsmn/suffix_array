//! Suffix array construction and searching algorithms for in-memory binary
//! data, focusing on space efficiency.
//!
//! The suffix array construction algorithm is O(n) time and O(1) space, as
//! described in [Ge Nong. 2013.
//! Practical linear-time O(1)-workspace suffix sorting for constant
//! alphabets.](https://dl.acm.org/citation.cfm?doid=2493175.2493180).
//! There are no heap allocations, except for a bucket array (about
//! 3k memory) in the recursion level 0 for speeding up sorting.

mod construct;
#[cfg(feature = "pack")]
mod packed_sa;
mod sa;
#[cfg(test)]
mod tests;
mod utils;

pub use self::construct::MAX_LENGTH;
pub use self::sa::SuffixArray;
