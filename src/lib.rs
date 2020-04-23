//! Suffix array construction and searching algorithms for in-memory binary
//! data, focusing on space efficiency.
//!
//! The suffix array construction algorithm is guaranteed to be O(n) time
//! and O(1) space.
//! 
//! For now, this crate uses the [C bindings by Amos Wenger](https://crates.io/crates/cdivsufsort)
//! to [Yuta Mori's dissufsort](https://github.com/y-256/libdivsufsort),
//! which is the fastest known SACA running in single thread with only O(1) additional workspace.

mod saca;
#[cfg(feature = "pack")]
mod packed_sa;
mod sa;
#[cfg(test)]
mod tests;
mod utils;

pub use self::saca::MAX_LENGTH;
pub use self::sa::SuffixArray;
