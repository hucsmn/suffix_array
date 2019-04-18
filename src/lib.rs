//! Suffix array construction and searching algorithms balanced for indexing
//! small or median sized binary data in memory.
//!
//! The construction algorithm is based on SAIS as described in
//! [Two Efficient Algorithms for Linear Time Suffix Array
//! Construction](https://ieeexplore.ieee.org/document/5582081).
//! Several optimization tricks were taken from
//! [Optimal In-Place Suffix Sorting](https://arxiv.org/abs/1610.08305).

mod sais;
#[cfg(test)]
mod tests;
mod utils;

use sais::*;
use std::ops::Range;
use utils::*;

/// Suffix array for searching byte strings.
#[derive(Clone)]
pub struct SuffixArray<'s> {
    s: &'s [u8],
    sa: Vec<u32>,
}

impl<'s> SuffixArray<'s> {
    // Construct new suffix array for a byte string.
    pub fn new(s: &'s [u8]) -> Self {
        let mut sa = vec![0; s.len() + 1];
        construct(s, &mut sa[..]);
        SuffixArray { s, sa }
    }

    // Construct suffix array in place.
    pub fn set(&mut self, s: &'s [u8]) {
        self.sa.resize(s.len() + 1, 0);
        construct(s, &mut self.sa[..]);
    }

    // Release the unused memory of suffix array.
    pub fn fit(&mut self) {
        self.sa.shrink_to_fit()
    }

    /// Length of the underlying byte string.
    pub fn len(&self) -> usize {
        self.s.len()
    }

    /// Test if the underlying byte string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Test if contains given sub-string.
    pub fn contains(&self, sub: &[u8]) -> bool {
        self.sa
            .binary_search_by_key(&sub, |&i| truncate(&self.s[i as usize..], sub.len()))
            .is_ok()
    }

    /// Search for all the unsorted overlapping occurrence of given sub-string.
    pub fn search_all(&self, sub: &[u8]) -> &[u32] {
        let mut i = 0;
        let mut k = self.sa.len();
        while i < k {
            let m = i + (k - i) / 2;
            if sub > &self.s[self.sa[m] as usize..] {
                i = m + 1;
            } else {
                k = m;
            }
        }

        let mut j = i;
        k = self.sa.len();
        while j < k {
            let m = j + (k - j) / 2;
            if self.s[self.sa[m] as usize..].starts_with(sub) {
                j = m + 1;
            } else {
                k = m;
            }
        }

        &self.sa[i..j]
    }

    /// Search for one longest infix in the byte string that matches a prefix
    /// of the given pattern.
    pub fn search_prefix(&self, pat: &[u8]) -> Range<usize> {
        let point = self.sa.binary_search_by(|&i| self.s[i as usize..].cmp(pat));

        match point {
            Ok(i) => {
                let j = self.sa[i] as usize;
                j..self.s.len()
            }
            Err(i) => {
                if i > 0 && i < self.sa.len() {
                    let j = self.sa[i - 1] as usize;
                    let k = self.sa[i] as usize;
                    let a = common_prefix(pat, &self.s[j..]);
                    let b = common_prefix(pat, &self.s[k..]);
                    if a > b {
                        j..j + a
                    } else {
                        k..k + b
                    }
                } else if i == self.sa.len() {
                    let j = self.sa[i - 1] as usize;
                    let a = common_prefix(pat, &self.s[j..]);
                    j..j + a
                } else {
                    self.s.len()..self.s.len()
                }
            }
        }
    }

    // TODO: LehmerCode encode/decode?
}
