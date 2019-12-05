//! O(n) time and O(1) space SAIS algorithm for read-only byte string,
//! as described in the recursion level 1 of [Ge Nong. 2013.
//! Practical linear-time O(1)-workspace suffix sorting for constant
//! alphabets.](https://dl.acm.org/citation.cfm?doid=2493175.2493180).

#[cfg(test)]
mod tests;

use super::super::utils::*;

/// Special symbol.
const EMPTY: u32 = 0x80000000;

/// Maximum length of the input string.
pub const MAX_LENGTH: usize = 0x7fffffff;

/// O(n) time and O(1) space SAIS algorithm for writable integer string (s),
/// whose alphabet scale (k) is usually large enough comparing to the string
/// length.
pub fn sacak_u32s(s: &mut [u32], k: usize, sa: &mut [u32]) {
    debug_assert!(s.len() <= MAX_LENGTH);
    debug_assert!(s.len() + 1 == sa.len());
    if s.len() == 0 {
        sa[0] = 0;
        return;
    }

    transform_string(s, k, sa);
    sort_lms_suffixes(s, sa);
    induce_by_lms(s, sa);
}

/// Transform u32 string into an array of pointers to the corresponding bucket
/// head or tail position.
fn transform_string(s: &mut [u32], k: usize, sa: &mut [u32]) {
    debug_assert!(s.len() >= k);

    // 1. construct bucket array in the workspace
    let mut sum = 1;
    sa[..k + 1].iter_mut().for_each(|n| *n = 0);
    s.iter().for_each(|&c| sa[c as usize] += 1);
    sa[..k + 1].iter_mut().for_each(|acc| {
        let n = *acc;
        *acc = sum;
        sum += n;
    });

    // 2. transform the original string into a pointer array, while keep
    // the lexicographical order unchanged
    let mut t = false;
    let mut r = s[s.len() - 1];
    s[s.len() - 1] = sa[r as usize];
    for p in s.iter_mut().rev().skip(1) {
        if *p < r {
            t = true;
        } else if *p > r {
            t = false;
        }
        r = *p;
        if t {
            // set s-type character to corresponding bucket tail pointer
            *p = sa[*p as usize + 1] - 1;
        } else {
            // set l-type character to corresponding bucket head pointer
            *p = sa[*p as usize];
        }
    }
}

/// Sort all the lms suffixes in the workspace.
fn sort_lms_suffixes(s: &[u32], sa: &mut [u32]) {
    // 1. place lms characters
    sa.iter_mut().for_each(|p| *p = EMPTY);
    for_each_lms(s, false, |i, _| insert_tail(s, sa, None, i));
    sa[0] = s.len() as u32;
    finish_tail(sa);

    // 2. sort lms substrings by lms characters
    induce_by_lms(s, sa);

    // 3. collect the sorted lms substrings into the tail of workspace
    let mut h = sa.len();
    let mut q = s.len() as u32 + 1;
    let mut t = false;
    for p in (1..sa.len()).rev() {
        let i = sa[p] as usize;
        if s[i] != q {
            // (bucket tail) s[i] < s[i+1] <=> here comes a s-type bucket
            t = i < s.len() - 1 && s[i] < s[i + 1];
            q = s[i];
        }
        // s[i-1] > s[i] and is s-type <=> this is a lms substring
        if t && i > 0 && s[i - 1] > s[i] {
            h -= 1;
            sa[h] = sa[p];
        }
    }
    h -= 1;
    sa[h] = sa[0];

    // 4. get the sorted lms suffixes from sorted lms substrings
    let (head, tail) = sa.split_at_mut(h);
    let n = tail.len();
    suffixes_from_substrs(s, head, tail, sacak_u32s);
    sa[n..].iter_mut().for_each(|i| *i = EMPTY);

    // 5. place sorted lms suffixes to the bucket respectively
    if n > 1 {
        let mut r = n;
        let mut q = s[sa[n - 1] as usize] as usize;
        for l in (1..n).rev() {
            let i = sa[l - 1] as usize;
            let p = get_ptr(s, i);
            if p == q {
                continue;
            }

            let m = r - l;
            if m == 1 {
                let tmp = sa[l];
                sa[l] = EMPTY;
                sa[q] = tmp;
            } else {
                let t = q + 1 - m;
                sa.copy_within(l..l+m, t);
                sa[l..Ord::min(r, t)].iter_mut().for_each(|i| *i = EMPTY);
            }

            r = l;
            q = p;
        }
    }
}

/// Induce the entire suffix array by sorted lms suffixes.
/// Or induce the order of lms substrings by the sorted lms characters.
fn induce_by_lms(s: &[u32], sa: &mut [u32]) {
    // 1. induce l-type characters by lms characters
    let mut p = 0;
    while p < sa.len() {
        if sa[p] < EMPTY {
            // if i == s.len(), i-1 must be l-type.
            // if s[i-1] > s[i], i-1 must be l-type.
            // if s[i-1] == s[i], directly invokes insert_head():
            //     if i-1 is l-type, it would be successfully inserted;
            //     if i-1 is s-type, it maps to a tail pointer, however would be
            //                       treated as a head pointer to either EMPTY
            //                       or lms character, both of these two cases
            //                       will be ignored by insert_head().
            let i = sa[p] as usize;
            if i == s.len() || (i > 0 && s[i - 1] >= s[i]) {
                insert_head(s, sa, Some(&mut p), i - 1);
            }
        }
        p += 1;
    }
    finish_head(sa);

    // 2. clear lms characters, except the sentinel
    clear_lms(s, sa);

    // 3. symmetrically induce s-type characters by l-type characters
    let mut p = sa.len() - 1;
    while p > 0 {
        if sa[p] < EMPTY {
            let i = sa[p] as usize;
            if i > 0 && get_ptr(s, i - 1) <= get_ptr(s, i) {
                insert_tail(s, sa, Some(&mut p), i - 1);
            }
        }
        p -= 1;
    }
    finish_tail(sa);
}

/// Safely get the bucket head/tail pointer (i <= s.len()).
#[inline]
fn get_ptr(s: &[u32], i: usize) -> usize {
    if i == s.len() {
        0
    } else {
        s[i] as usize
    }
}

/// Clear all the lms characters in bucket tail, keep the sentinel untouched.
#[inline]
fn clear_lms(s: &[u32], sa: &mut [u32]) {
    // mark and count lms characters
    for_each_lms(s, false, |_, c| {
        let p = c as usize;
        if sa[p] > EMPTY {
            sa[p] -= 1;
        } else {
            sa[p] = (-1i32) as u32;
        }
    });

    // fill EMPTY
    for p in (1..sa.len()).rev() {
        if sa[p] > EMPTY {
            let n = -(sa[p] as i32) as usize;
            sa[p + 1 - n..p + 1].iter_mut().for_each(|i| *i = EMPTY);
        }
    }
}

/// Insert l-type characters in corresponding bucket head.
/// Invalid insertion would be ignored (assuming that the bucket tail of s-type
/// characters is not in working state).
/// The cursor for suffix array scanning would be modified, when data chunk
/// containing the cursor is moved.
#[inline]
fn insert_head(
    s: &[u32],
    sa: &mut [u32],
    mut ptr: Option<&mut usize>,
    i: usize,
) {
    let p = s[i] as usize;

    if sa[p] < EMPTY {
        let lp = get_ptr(s, sa[p] as usize);
        if p > 0 && p != lp {
            // (-n, indexes[..-1]), <borrowed-by-left-indexes> => (indexes), E
            let mut q = p;
            while q < sa.len()
                && sa[q] < EMPTY
                && get_ptr(s, sa[q] as usize) == lp
            {
                q -= 1;
            }
            if sa[q] > EMPTY {
                let n = -(sa[q] as i32) as usize;
                sa_move(sa, q + 1, q, n, &mut ptr);
                sa[p] = EMPTY;
            }
        }
    }

    if sa[p] == EMPTY {
        if p + 1 >= sa.len() || sa[p + 1] != EMPTY {
            // E, <bound> => i, <bound>
            sa[p] = i as u32;
        } else {
            // E, E => -1, i
            sa[p] = (-1i32) as u32;
            sa[p + 1] = i as u32;
        }
    } else if sa[p] > EMPTY {
        let n = -(sa[p] as i32) as usize;
        if p + 1 + n >= sa.len() || sa[p + 1 + n] != EMPTY {
            // -n, indexes, <bound> => indexes, i, <bound>
            sa_move(sa, p + 1, p, n, &mut ptr);
            sa[p + n] = i as u32;
        } else {
            // -n, indexes, E => -(n+1), indexes, i
            sa[p + 1 + n] = i as u32;
            sa[p] -= 1;
        }
    }
}

/// Insert s-type characters in corresponding bucket tail.
/// Invalid insertion would be ignored (assuming that the bucket head of l-type
/// characters is not in working state).
/// The cursor for suffix array scanning would be modified, when data chunk
/// containing the cursor is moved.
#[inline]
fn insert_tail(
    s: &[u32],
    sa: &mut [u32],
    mut ptr: Option<&mut usize>,
    i: usize,
) {
    let p = s[i] as usize;

    if sa[p] < EMPTY {
        let rp = get_ptr(s, sa[p] as usize);
        if p > 0 && p != rp {
            // <borrowed-by-left-indexes>, (indexes[1..], -n) => i, (indexes)
            let mut q = p;
            while q < sa.len()
                && sa[q] < EMPTY
                && get_ptr(s, sa[q] as usize) == rp
            {
                q += 1;
            }
            if sa[q] > EMPTY {
                let n = -(sa[q] as i32) as usize;
                sa_move(sa, p, p + 1, n, &mut ptr);
                sa[p] = EMPTY;
            }
        }
    }

    if sa[p] == EMPTY {
        if p <= 1 || sa[p - 1] != EMPTY {
            // <bound>, E =>  <bound>, i
            sa[p] = i as u32;
        } else {
            // E, E => i, -1
            sa[p] = (-1i32) as u32;
            sa[p - 1] = i as u32;
        }
    } else if sa[p] > EMPTY {
        let n = -(sa[p] as i32) as usize;
        if p - 1 <= n || sa[p - n - 1] != EMPTY {
            // <bound>, indexes, -n => <bound>, i, indexes
            sa_move(sa, p - n, p - n + 1, n, &mut ptr);
            sa[p - n] = i as u32;
        } else {
            // E, indexes, -n => i, indexes, -(n+1)
            sa[p - n - 1] = i as u32;
            sa[p] -= 1;
        }
    }
}

/// Clean up the remaining MULTIs in the bucket head.
#[inline]
fn finish_head(sa: &mut [u32]) {
    for p in 1..sa.len() {
        if sa[p] > EMPTY {
            let n = -(sa[p] as i32) as usize;
            sa.copy_within(p+1..p+n+1, p);
            sa[p + n] = EMPTY;
        }
    }
}

/// Clean up the remaining MULTIs in the bucket tail.
#[inline]
fn finish_tail(sa: &mut [u32]) {
    for p in (1..sa.len()).rev() {
        if sa[p] > EMPTY {
            let n = -(sa[p] as i32) as usize;
            sa.copy_within(p-n..p, p-n+1);
            sa[p - n] = EMPTY;
        }
    }
}

/// Move slice of workspace forward or backward.
fn sa_move(
    sa: &mut [u32],
    src: usize,
    dst: usize,
    n: usize,
    ptr: &mut Option<&mut usize>,
) {
    sa.copy_within(src..src+n, dst);

    if let Some(p) = std::mem::replace(ptr, None) {
        if *p >= src && *p < src + n {
            if dst >= src {
                *p += dst - src;
            } else {
                *p -= src - dst;
            }
        }
        std::mem::replace(ptr, Some(p));
    }
}
