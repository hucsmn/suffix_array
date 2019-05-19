//! Suffix array construction and searching algorithms for in-memory binary
//! data, focusing on space efficiency.
//!
//! The suffix array construction algorithm is O(n) time and O(1) space, which
//! combined the recursion level 0 of SACA-K as described in [Ge Nong. 2013.
//! Practical linear-time O(1)-workspace suffix sorting for constant
//! alphabets.](https://dl.acm.org/citation.cfm?doid=2493175.2493180) and a
//! SAIS variant used in inner recursion levels that was developed by
//! [Li, Z., Jian, L. and Huo, H. Optimal In-Place Suffix
//! Sorting](https://arxiv.org/abs/1610.08305).
//! Both of these two SAIS algorithms ran in linear-time and allocated constant
//! workspace. There are no heap allocations, except for a bucket array (about
//! 3k memory) in the recursion level 0 for speeding up sorting.

mod construct;
mod utils;

#[cfg(test)]
mod tests;

pub use construct::MAX_LENGTH;
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
        construct::saca(s, &mut sa[..]);
        SuffixArray { s, sa }
    }

    // Construct suffix array in place.
    pub fn set(&mut self, s: &'s [u8]) {
        self.sa.resize(s.len() + 1, 0);
        construct::saca(s, &mut self.sa[..]);
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

    /// Take out the suffix array and its corresponding byte string.
    pub fn into_parts(self) -> (&'s [u8], Vec<u32>) {
        (self.s, self.sa)
    }

    /// Compose existed suffix array and its corresponding byte string
    /// together, and checks the integrity.
    pub fn from_parts(s: &'s [u8], sa: Vec<u32>) -> Option<Self> {
        let compose = SuffixArray { s, sa };
        if compose.check_integrity() {
            Some(compose)
        } else {
            None
        }
    }

    /// Compose existed suffix array and its corresponding byte string
    /// together without integrity checking.
    pub unsafe fn from_parts_unchecked(s: &'s [u8], sa: Vec<u32>) -> Self {
        SuffixArray { s, sa }
    }

    fn check_integrity(&self) -> bool {
        if self.s.len() + 1 != self.sa.len() {
            return false;
        }
        for i in 1..self.sa.len() {
            let x = &self.s[self.sa[i - 1] as usize..];
            let y = &self.s[self.sa[i] as usize..];
            if x >= y {
                return false;
            }
        }
        true
    }

    /// Test if contains given sub-string.
    pub fn contains(&self, sub: &[u8]) -> bool {
        self.sa
            .binary_search_by_key(&sub, |&i| {
                trunc(&self.s[i as usize..], sub.len())
            })
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

    /// Search for one sub-string that has the longest common prefix of the
    /// given pattern.
    pub fn search_lcp(&self, pat: &[u8]) -> Range<usize> {
        let point =
            self.sa.binary_search_by(|&i| self.s[i as usize..].cmp(pat));

        match point {
            Ok(i) => {
                let j = self.sa[i] as usize;
                j..self.s.len()
            }
            Err(i) => {
                if i > 0 && i < self.sa.len() {
                    let j = self.sa[i - 1] as usize;
                    let k = self.sa[i] as usize;
                    let a = lcp(pat, &self.s[j..]);
                    let b = lcp(pat, &self.s[k..]);
                    if a > b {
                        j..j + a
                    } else {
                        k..k + b
                    }
                } else if i == self.sa.len() {
                    let j = self.sa[i - 1] as usize;
                    let a = lcp(pat, &self.s[j..]);
                    j..j + a
                } else {
                    self.s.len()..self.s.len()
                }
            }
        }
    }
}

impl<'s> From<SuffixArray<'s>> for Vec<u32> {
    fn from(sa: SuffixArray<'s>) -> Vec<u32> {
        sa.sa
    }
}

impl<'s> AsRef<[u8]> for SuffixArray<'s> {
    fn as_ref(&self) -> &[u8] {
        self.s
    }
}
