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
fn sais<T>(s: &[T], scale: usize, sa: &mut [u32])
where
    T: Copy + Into<u32> + Ord,
{
    if s.len() == 0 {
        sa[0] = s.len() as u32;
        return;
    }

    let ty = &Types::calculate(s);
    let bkt = &mut Bucket::calculate(s, scale);

    place_sorted_lms(s, ty, sa, bkt);
    induce_by_lms(s, ty, sa, bkt);
}

/// Place lms-characters in the right place.
fn place_sorted_lms<T>(s: &[T], ty: &Types, sa: &mut [u32], bkt: &mut Bucket)
where
    T: Copy + Into<u32> + Ord,
{
    // try calculate the permutation of lms-characters
    let (n, is_permutation) = sort_or_permut_lms(s, ty, sa, bkt);

    if is_permutation {
        // collect lms-characters from left to right into the head of sa
        let mut m = 0;
        for i in 0..=s.len() {
            if ty.is_lms(i) {
                sa[m] = i as u32;
                m += 1;
            }
        }

        // place ordered lms-characters in the tail of sa
        for i in sa.len() - n..sa.len() {
            sa[i] = sa[to_usize(sa[i])];
        }
    }

    // move the lms-suffixes to the head at first
    unsafe {
        let src: *const u32 = &sa[sa.len() - n];
        let dst: *mut u32 = &mut sa[0];
        copy_nonoverlapping(src, dst, n);
    }

    // place lms-charaters in the correct place
    for i in (1..n).rev() {
        let c = to_usize(s[to_usize(sa[i])]);
        bkt[c].j -= 1;
        sa[to_usize(bkt[c].j)] = sa[i];
    }
}

/// Calculate the permutation or sorted lms-characters, storing in the tail of
/// sa. Return length and kind of result.
fn sort_or_permut_lms<T>(s: &[T], ty: &Types, sa: &mut [u32], bkt: &mut Bucket) -> (usize, bool)
where
    T: Copy + Into<u32> + Ord,
{
    // place lms-characters in corresponding bucket first
    sa[0] = s.len() as u32;
    for i in (1..s.len()).rev() {
        if ty.is_lms(i) {
            let sp = &mut bkt[to_usize(s[i])];
            sp.j -= 1;
            sa[to_usize(sp.j)] = i as u32;
        }
    }

    // sort lms-substrings
    induce_by_lms(s, ty, sa, bkt);

    // collect all the sorted lms-substrings into the tail of sa
    let mut lms_head = sa.len();
    for i in (0..sa.len()).rev() {
        if ty.is_lms(to_usize(sa[i])) {
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
        if !lms_substring_eq(s, ty, to_usize(last), to_usize(x)) {
            scale += 1;
        }
        head[i] = scale - 1;
        last = x;
    }

    let mut n = 0;
    for i in 0..head.len() {
        if head[i] == NIL {
            continue;
        }
        head[n] = head[i];
        n += 1;
    }

    if to_usize(scale) + 1 < lms.len() {
        // calculate the correct order of lms-suffixes in place
        let s1 = &head[..n];
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
fn induce_by_lms<T>(s: &[T], ty: &Types, sa: &mut [u32], bkt: &mut Bucket)
where
    T: Copy + Into<u32> + Ord,
{
    // TODO: disable bound checks? no

    // induce l characters by lms characters
    if s.len() > 0 {
        let c = to_usize(s[s.len() - 1]);
        bkt[c].push_front(sa, (s.len() - 1) as u32);
    }

    for c0 in 0..bkt.len() {
        let mut i = bkt[c0].head;
        while i < bkt[c0].i {
            let j = sa[to_usize(i)];
            if j > 0 && !ty[to_usize(j - 1)] {
                let c = to_usize(s[to_usize(j - 1)]);
                bkt[c].push_front(sa, j - 1);
            }
            i += 1;
        }

        i = bkt[c0].j;
        while i < bkt[c0].tail {
            let j = sa[to_usize(i)];
            let c = to_usize(s[to_usize(j - 1)]);
            bkt[c].push_front(sa, j - 1);
            i += 1;
        }
    }

    for i in 0..bkt.len() {
        bkt[i].reset_back();
    }

    // induce s characters by l characters
    for c0 in (0..bkt.len()).rev() {
        let mut i = bkt[c0].tail - 1;
        while i >= bkt[c0].j {
            let j = sa[to_usize(i)];
            if j > 0 && ty[to_usize(j - 1)] {
                let c = to_usize(s[to_usize(j - 1)]);
                bkt[c].push_back(sa, j - 1);
            }
            i -= 1;
        }

        i = bkt[c0].i - 1; // won't underflow
        while i >= bkt[c0].head {
            let j = sa[to_usize(i)];
            if j > 0 && ty[to_usize(j - 1)] {
                let c = to_usize(s[to_usize(j - 1)]);
                bkt[c].push_back(sa, j - 1);
            }
            i -= 1;
        }
    }

    for i in 0..bkt.len() {
        bkt[i].reset();
    }
}

/// Test if two lms-substrings are the same.
#[inline]
fn lms_substring_eq<T>(s: &[T], ty: &Types, i: usize, j: usize) -> bool
where
    T: Copy + Into<u32> + Ord,
{
    use std::iter::once;
    let xs = s[i..].iter().map(Some).chain(once(None));
    let ys = s[j..].iter().map(Some).chain(once(None));

    for (k, (x, y)) in Iterator::zip(xs, ys).enumerate() {
        if x != y || ty[i + k] != ty[j + k] {
            return false;
        }
        if k > 0 && (ty.is_lms(i + k) || ty.is_lms(j + k)) {
            return true;
        }
    }
    false
}
