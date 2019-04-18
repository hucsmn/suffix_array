#[cfg(test)]
mod tests;
mod utils;

use std::ptr::copy_nonoverlapping;
use utils::*;

const THRESHOLD: usize = 128;
const MAX_LENGTH: usize = std::u32::MAX as usize - 2;
const NIL: u32 = std::u32::MAX;

/// Sort the byte string and calculate its suffix array.
/// The algorithm is not designed for sorting huge data in memeory.
/// Thus u32 is choosen instead of usize for reducing memory usage in x64.
pub fn construct(s: &[u8], sa: &mut [u32]) {
    assert_eq!(s.len() + 1, sa.len());
    assert!(s.len() < MAX_LENGTH);
    assert!(std::usize::MAX as u64 >= std::u32::MAX as u64);

    general_sort(s, 256, sa);
}

/// Sort u32 slice and calculate its suffix array.
#[inline]
fn general_sort<T>(s: &[T], scale: usize, sa: &mut [u32])
where
    T: Copy + Into<u32> + Ord,
{
    if s.len() < THRESHOLD {
        naive_sort(s, sa);
    } else {
        sais(s, scale, sa);
    }
}

/// The simplest but time costing way to calculate suffix array.
/// No auxilary data structure is required.
#[inline]
fn naive_sort<T: Ord>(s: &[T], sa: &mut [u32]) {
    for (i, x) in (0..=s.len()).rev().enumerate() {
        sa[i] = x as u32;
    }

    (&mut sa[1..]).sort_by(|&i, &j| Ord::cmp(&s[to_usize(i)..], &s[to_usize(j)..]));
}

/// Suffix array induced-sorting algorithm.
/// The fast (O(n)) and relatively space efficient (O(n+n/log(n))) way to
/// calculate suffix array.
/// Use bucket and type bitmap to speed up construction.
pub fn sais<T>(s: &[T], scale: usize, sa: &mut [u32])
where
    T: Copy + Into<u32> + Ord,
{
    if s.len() == 0 {
        sa[0] = 0;
        return;
    }

    let mut ctx = SaisContext::new(s, scale);
    ctx.sort(sa);
}

/// SAIS excution context.
struct SaisContext<'s, T>
where
    T: Copy + Into<u32> + Ord,
{
    s: &'s [T],
    ty: Types,
    bkt: Bucket,
}

impl<'s, T> SaisContext<'s, T>
where
    T: Copy + Into<u32> + Ord,
{
    /// Initialize context.
    pub fn new(s: &'s [T], scale: usize) -> Self {
        let ty = Types::calculate(s);
        let bkt = Bucket::calculate(s, scale);
        SaisContext { s, ty, bkt }
    }

    /// Start sorting.
    pub fn sort(&mut self, sa: &mut [u32]) {
        self.place_sorted_lms(sa);
        self.induce_by_lms(sa);
    }

    /// Place lms-characters in the right place.
    fn place_sorted_lms(&mut self, sa: &mut [u32]) {
        // try calculate the permutation of lms-characters
        let (n, is_permut) = self.sort_or_permut_lms(sa);

        if is_permut {
            // collect lms-characters from left to right into the head of sa
            let mut m = 0;
            for i in 0..=self.len() {
                if self.is_lms(i) {
                    sa[m] = i as u32;
                    m += 1;
                }
            }

            // place ordered lms-characters in the tail of sa
            for i in sa.len() - n..sa.len() {
                sa[i] = sa[to_usize(sa[i])];
            }
        }

        // move the lms-suffixes to the head of sa
        unsafe {
            let src: *const u32 = &sa[sa.len() - n];
            let dst: *mut u32 = &mut sa[0];
            copy_nonoverlapping(src, dst, n);
        }

        // place lms-charaters in the correct place
        for i in (1..n).rev() {
            self.span_mut(self.sa_char_at(sa, i)).push_back(sa, sa[i]);
        }
    }

    /// Calculate the permutation or sorted lms-characters, storing in the tail
    /// of sa. Return length and kind of result.
    fn sort_or_permut_lms(&mut self, sa: &mut [u32]) -> (usize, bool) {
        // place lms-characters in corresponding bucket first
        sa[0] = self.len() as u32;
        for i in (1..self.len()).rev() {
            if self.is_lms(i) {
                self.push_schar(sa, i);
            }
        }

        // sort lms-substrings
        self.induce_by_lms(sa);

        // collect all the sorted lms-substrings into the tail of sa
        let mut lms_head = sa.len();
        for i in (0..sa.len()).rev() {
            if self.is_lms(to_usize(sa[i])) {
                lms_head -= 1;
                sa[lms_head] = sa[i];
            }
        }

        // construct the sub-problem in place
        let (head, lms) = sa.split_at_mut(lms_head);
        let mut scale = 0;
        let mut last = lms[0];

        for x in head.iter_mut() {
            *x = NIL
        }
        for &x in lms.iter().skip(1) {
            let i = to_usize(x / 2); // in-place trick here
            if !self.lms_substring_eq(to_usize(last), to_usize(x)) {
                scale += 1;
            }
            head[i] = scale - 1;
            last = x;
        }

        let mut s1_tail = 0;
        for i in 0..head.len() {
            if head[i] == NIL {
                continue;
            }
            head[s1_tail] = head[i];
            s1_tail += 1;
        }

        if to_usize(scale) + 1 < lms.len() {
            // calculate the correct order of lms-suffixes in place
            let s1 = &head[..s1_tail];
            let sa1 = lms;
            general_sort(s1, to_usize(scale), sa1);

            // finally computes a permutation here
            (sa1.len(), true)
        } else {
            // the lms-suffixes are already sorted
            (lms.len(), false)
        }
    }

    /// Induced sort the whole suffix array by sorted lms-suffixes.
    /// Or induced sort the order of lms-substrings by randomly placing
    /// lms-characters in corresponding bucket.
    fn induce_by_lms(&mut self, sa: &mut [u32]) {
        // induce l characters by lms characters
        if self.len() > 0 {
            self.push_lchar(sa, self.len() - 1);
        }

        for c in 0..self.scale() {
            let mut i = self.bkt[c].head;
            while i < self.bkt[c].i {
                let j = to_usize(sa[to_usize(i)]);
                if j > 0 && !self.is_schar(j - 1) {
                    self.push_lchar(sa, j - 1);
                }
                i += 1;
            }

            i = self.bkt[c].j;
            while i < self.bkt[c].tail {
                let j = to_usize(sa[to_usize(i)]);
                self.push_lchar(sa, j - 1);
                i += 1;
            }
        }

        for c in 0..self.scale() {
            self.bkt[c].reset_back();
        }

        // induce s characters by l characters
        for c in (0..self.scale()).rev() {
            let mut i = self.bkt[c].tail - 1;
            while i >= self.bkt[c].j {
                let j = to_usize(sa[to_usize(i)]);
                if j > 0 && self.is_schar(j - 1) {
                    self.push_schar(sa, j - 1);
                }
                i -= 1;
            }

            i = self.bkt[c].i - 1; // won't underflow
            while i >= self.bkt[c].head {
                let j = to_usize(sa[to_usize(i)]);
                if j > 0 && self.is_schar(j - 1) {
                    self.push_schar(sa, j - 1);
                }
                i -= 1;
            }
        }

        for c in 0..self.scale() {
            self.bkt[c].reset();
        }
    }

    /// Length of byte string.
    #[inline]
    fn len(&self) -> usize {
        self.s.len()
    }

    /// Length of bucket.
    #[inline]
    fn scale(&self) -> usize {
        self.bkt.len()
    }

    /// Get type of character.
    #[inline]
    fn is_schar(&self, i: usize) -> bool {
        self.ty[i]
    }

    /// Get type of character.
    #[inline]
    fn is_lms(&self, i: usize) -> bool {
        self.ty.is_lms(i)
    }

    /// Write span of character.
    #[inline]
    fn span_mut(&mut self, ch: T) -> &mut Span {
        &mut self.bkt[ch.into() as usize]
    }

    /// Get type of character.
    #[inline]
    fn sa_char_at(&self, sa: &mut [u32], i: usize) -> T {
        self.s[sa[i] as usize]
    }

    /// Push l-character in s[i].
    #[inline]
    fn push_lchar(&mut self, sa: &mut [u32], i: usize) {
        self.span_mut(self.s[i]).push_front(sa, i as u32);
    }

    /// Push s-character in s[i].
    #[inline]
    fn push_schar(&mut self, sa: &mut [u32], i: usize) {
        self.span_mut(self.s[i]).push_back(sa, i as u32);
    }

    /// Test if two lms-substrings are the same.
    #[inline]
    fn lms_substring_eq(&self, i: usize, j: usize) -> bool {
        use std::iter::once;
        let xs = self.s[i..].iter().map(Some).chain(once(None));
        let ys = self.s[j..].iter().map(Some).chain(once(None));

        for (k, (x, y)) in Iterator::zip(xs, ys).enumerate() {
            if x != y || self.ty[i + k] != self.ty[j + k] {
                return false;
            }
            if k > 0 && (self.ty.is_lms(i + k) || self.ty.is_lms(j + k)) {
                return true;
            }
        }
        false
    }
}
