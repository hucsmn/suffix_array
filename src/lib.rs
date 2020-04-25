//! Suffix array construction and searching algorithms for in-memory binary
//! data, focusing on space efficiency.
//!
//! The suffix array construction algorithm is guaranteed to be O(n) time
//! and O(1) space.
//!
//! For now, This crate uses the Amos Wenger's [C bindings](https://crates.io/crates/cdivsufsort)
//! to Yuta Mori's [dissufsort](https://github.com/y-256/libdivsufsort) to construct suffix array,
//! which is the fastest known suffix array construction algorithm (SACA) running in single thread
//! that uses merely O(1) additional workspace.
//!
//! # Examples
//!
//! Test if the data contains given pattern.
//!
//! ```rust
//! use suffix_array::SuffixArray;
//!
//! let sa = SuffixArray::new(b"splendid splendor");
//! assert!(sa.contains(b"splend"));
//! ```
//!
//! Search for all the occurrences of given pattern in data.
//!
//! ```rust
//! use suffix_array::SuffixArray;
//!
//! let sa = SuffixArray::new(b"splendid splendor");
//! assert_eq!(sa.search_all(b"splend"), &[0, 9]);
//! ```
//!
//! Search for a longest common prefix of given pattern that matches somewhere in the data.
//!
//! ```rust
//! use suffix_array::SuffixArray;
//!
//! let s = b"splendid splendor";
//! let sa = SuffixArray::new(s);
//! let lcp = sa.search_lcp(b"splash");
//! assert_eq!(&s[lcp], b"spl");
//! ```

#[cfg(feature = "pack")]
mod packed_sa;
mod sa;
mod saca;
mod utils;

#[cfg(test)]
mod tests;

pub use self::sa::SuffixArray;
pub use self::saca::MAX_LENGTH;
