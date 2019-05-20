//! O(n) time and O(1) space SAIS algorithm for writable integers mentioned in
//! the Section 3 of [Li, Z., Jian, L. and Huo, H. Optimal In-Place Suffix
//! Sorting](https://arxiv.org/abs/1610.08305).

#[cfg(test)]
mod tests;

use super::utils::*;

/// Special symbols.
const EMPTY: u32 = 0xffffffff;
const UNIQUE: u32 = 0xfffffffe;
const MULTI: u32 = 0xfffffffd;

/// Smallest symbol value.
const MIN_SYMBOL: u32 = 0xfffffffd;

/// Maximum length of the input string.
pub const MAX_LENGTH: usize = 0xfffffffc;

/// O(n) time and O(1) space SAIS algorithm for writable integer string (s),
/// whose alphabet scale (k) is usually large enough comparing to the string
/// length.
pub fn llhsais(s: &mut [u32], k: usize, sa: &mut [u32]) {
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
    prepare_lms(s, sa);
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
    let (head, tail) = sa.split_at_mut(h);

    // 4. rename lms substrings and get alphabet scale of the sub-problem
    head.iter_mut().for_each(|i| *i = EMPTY);
    let mut k = 0;
    let mut j = tail[0];
    for &i in tail.iter().skip(1) {
        if !lms_substr_eq(s, i as usize, j as usize) {
            k += 1;
        }
        head[i as usize / 2] = k as u32 - 1;
        j = i;
    }

    // 5. sort then move the lms suffixes into the head of workspace
    if k + 1 == tail.len() {
        // 5.a. the order of lms suffixes is exactly the same as lms substrings
        unsafe {
            std::ptr::copy_nonoverlapping(
                &tail[0] as *const u32,
                &mut head[0] as *mut u32,
                tail.len(),
            );
        }
    } else {
        // 5.b.1 construct and solve the sub-problem recursively
        let mut t = 0;
        for i in 0..h {
            if head[i] != EMPTY {
                head[t] = head[i];
                t += 1;
            }
        }
        llhsais(&mut head[..t], k, tail);

        // 5.b.2 rearrange the lms suffixes
        unsafe {
            std::ptr::copy_nonoverlapping(
                &tail[0] as *const u32,
                &mut head[0] as *mut u32,
                tail.len(),
            );
        }
        let mut h = tail.len();
        for_each_lms(s, true, |i, _| {
            h -= 1;
            tail[h] = i as u32;
        });
        for p in 0..tail.len() {
            let i = head[p] as usize;
            head[p] = tail[i];
        }
    }

    // 6. place sorted lms suffixes to the bucket respectively
    let n = tail.len();
    sa[n..].iter_mut().for_each(|i| *i = EMPTY);
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
                unsafe {
                    std::ptr::copy(
                        &sa[l] as *const u32,
                        &mut sa[t] as *mut u32,
                        m,
                    );
                }
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
    prepare_typed(s, sa, false);
    while p < sa.len() {
        if sa[p] == MULTI {
            // skip MULTI symbol and counter
            p += 2;
            continue;
        } else if sa[p] != EMPTY && sa[p] != UNIQUE {
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
    prepare_typed(s, sa, true);
    while p > 0 {
        if sa[p] == MULTI {
            p -= 2;
            continue;
        } else if sa[p] != EMPTY && sa[p] != UNIQUE {
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

/// Set UNIQUE/MULTI in bucket tails/heads.
#[inline]
fn prepare_typed(s: &[u32], sa: &mut [u32], t: bool) {
    for_each_typed(s, |_, ti, c| {
        let p = c as usize;
        if ti == t {
            match sa[p] {
                EMPTY => sa[p] = UNIQUE,
                UNIQUE => sa[p] = MULTI,
                _ => (),
            }
        }
    });
}

/// Set UNIQUE/MULTI in bucket tails of lms characters.
#[inline]
fn prepare_lms(s: &[u32], sa: &mut [u32]) {
    for_each_lms(s, true, |_, c| {
        let p = c as usize;
        match sa[p] {
            EMPTY => sa[p] = UNIQUE,
            UNIQUE => sa[p] = MULTI,
            _ => (),
        }
    });
}

/// Clear all the lms characters in bucket tail, keep the sentinel untouched.
#[inline]
fn clear_lms(s: &[u32], sa: &mut [u32]) {
    // mark and count lms characters
    for_each_lms(s, false, |_, c| {
        let p = c as usize;
        match sa[p] {
            UNIQUE => {
                sa[p] = MULTI;
                sa[p - 1] = 2;
            }
            MULTI => sa[p - 1] += 1,
            _ => sa[p] = UNIQUE,
        }
    });

    // fill EMPTY
    for p in (1..sa.len()).rev() {
        if sa[p] == UNIQUE {
            sa[p] = EMPTY;
        } else if sa[p] == MULTI {
            let n = sa[p - 1] as usize;
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
fn insert_head(s: &[u32], sa: &mut [u32], ptr: Option<&mut usize>, i: usize) {
    let p = s[i] as usize;

    if sa[p] == UNIQUE {
        // U => i
        sa[p] = i as u32;
    } else if sa[p] == MULTI && sa[p + 1] == EMPTY {
        if p + 2 < sa.len() && sa[p + 2] == EMPTY {
            // M, E, E => M, 1, i
            sa[p + 1] = 1;
            sa[p + 2] = i as u32;
        } else {
            // M, E <bound> => i, E
            sa[p] = i as u32;
            sa[p + 1] = EMPTY;
        }
    } else if sa[p] == MULTI && sa[p + 1] != EMPTY {
        let n = sa[p + 1] as usize;
        if p + n + 2 < sa.len() && sa[p + n + 2] == EMPTY {
            // M, n, indexes, E => M, n+1, indexes, i
            sa[p + 1] += 1;
            sa[p + n + 2] = i as u32;
        } else {
            // M, n, indexes <bound> => indexes, i, E
            unsafe {
                std::ptr::copy(&sa[p + 2], &mut sa[p], n);
            }
            sa[p + n] = i as u32;
            sa[p + n + 1] = EMPTY;

            // move the cursor if needed
            if let Some(cursor) = ptr {
                if *cursor >= p + 2 && *cursor < p + n + 2 {
                    *cursor -= 2;
                }
            }
        }
    } else if sa[p] != EMPTY {
        // indexes, E => indexes, i
        let mut q = p;
        while q < sa.len()
            && sa[q] < MIN_SYMBOL
            && get_ptr(s, sa[q] as usize) == p
        {
            q += 1;
        }
        if q < sa.len() && sa[q] == EMPTY {
            sa[q] = i as u32;
        }
        // do nothing if nowhere to insert
    }
}

/// Insert s-type characters in corresponding bucket tail.
/// Invalid insertion would be ignored (assuming that the bucket head of l-type
/// characters is not in working state).
/// The cursor for suffix array scanning would be modified, when data chunk
/// containing the cursor is moved.
#[inline]
fn insert_tail(s: &[u32], sa: &mut [u32], ptr: Option<&mut usize>, i: usize) {
    let p = s[i] as usize;

    if sa[p] == UNIQUE {
        // U => i
        sa[p] = i as u32;
    } else if sa[p] == MULTI && sa[p - 1] == EMPTY {
        if p >= 2 && sa[p - 2] == EMPTY {
            // E, E, M => i, 1, M
            sa[p - 1] = 1;
            sa[p - 2] = i as u32;
        } else {
            // <bound> E, M => E, i
            sa[p] = i as u32;
            sa[p - 1] = EMPTY;
        }
    } else if sa[p] == MULTI && sa[p - 1] != EMPTY {
        let n = sa[p - 1] as usize;
        if p >= n + 2 && sa[p - n - 2] == EMPTY {
            // E, indexes, n, M => i, indexes, n+1, M
            sa[p - 1] += 1;
            sa[p - n - 2] = i as u32;
        } else {
            // <bound> indexes, n, M => E, i, indexes
            unsafe {
                std::ptr::copy(&sa[p - n - 1], &mut sa[p - n + 1], n);
            }
            sa[p - n] = i as u32;
            sa[p - n - 1] = EMPTY;

            // move the cursor if needed
            if let Some(cursor) = ptr {
                if *cursor <= p - 2 && *cursor > p - n - 2 {
                    *cursor += 2;
                }
            }
        }
    } else if sa[p] != EMPTY {
        // E, indexes => i, indexes
        let mut q = p;
        while q > 0 && sa[q] < MIN_SYMBOL && get_ptr(s, sa[q] as usize) == p {
            q -= 1;
        }
        if q > 0 && sa[q] == EMPTY {
            sa[q] = i as u32;
        }
        // do nothing if nowhere to insert
    }
}

/// Clean up the remaining MULTIs in the bucket tail.
#[inline]
fn finish_tail(sa: &mut [u32]) {
    // indexes, n, M => E, E, indexes
    for p in (0..sa.len()).rev() {
        if sa[p] == MULTI {
            let n = sa[p - 1] as usize;
            unsafe {
                std::ptr::copy(&sa[p - n - 1], &mut sa[p - n + 1], n);
            }
            sa[p - n] = EMPTY;
            sa[p - n - 1] = EMPTY;
        }
    }
}

/// Clean up the remaining MULTIs in the bucket head.
#[inline]
fn finish_head(sa: &mut [u32]) {
    // M, n, indexes => indexes, E, E
    for p in 0..sa.len() {
        if sa[p] == MULTI {
            let n = sa[p + 1] as usize;
            unsafe {
                std::ptr::copy(&sa[p + 2], &mut sa[p], n);
            }
            sa[p + n] = EMPTY;
            sa[p + n + 1] = EMPTY;
        }
    }
}
