/// Generic character type for the sais algorithm.
pub trait SaisChar: Sync + Copy + Ord {
    fn zero() -> Self;
}

impl SaisChar for u8 {
    #[inline]
    fn zero() -> u8 {
        0
    }
}

impl SaisChar for u32 {
    #[inline]
    fn zero() -> u32 {
        0
    }
}

/// Visit all the lms-suffixes (index, char) in reversed order.
/// Note that the index may equals to s.len() if sentinel enabled.
/// Length of the string must be greater than zero.
#[inline]
pub fn for_each_lms<T, F>(s: &[T], sentinel: bool, mut f: F)
where
    T: SaisChar,
    F: FnMut(usize, T),
{
    if sentinel {
        f(s.len(), SaisChar::zero());
    }

    let mut t = false;
    for i in (1..s.len() - 1).rev() {
        let (l, c, r) = (s[i - 1], s[i], s[i + 1]);
        if c < r {
            t = true;
        } else if c > r {
            t = false;
        }
        if l > c && t {
            f(i, c);
        }
    }
}

pub fn suffixes_from_substrs<'s, T, F>(s: &[T], head: &'s mut [u32], tail: &'s mut [u32], sort: F)
where
    T: SaisChar,
    F: FnOnce(&mut [u32], usize, &mut [u32]),
{
    debug_assert_eq!(s.len() + 1, head.len() + tail.len());

    // 1. rename lms substrings
    let k = rename_substrs(s, head, tail);

    // 2. sort lms suffixes
    if k + 1 == tail.len() {
        // lms substrings => lms suffixes
        unsafe {
            std::ptr::copy_nonoverlapping(
                &tail[0] as *const u32,
                &mut head[0] as *mut u32,
                tail.len(),
            );
        }
    } else {
        // construct sub-problem
        let mut t = 0;
        for i in 0..head.len() {
            if head[i] != 0xffffffff {
                head[t] = head[i];
                t += 1;
            }
        }

        // solve sub-problem
        sort(&mut head[..t], k, tail);

        // rearrange the lms suffixes
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
}

#[cfg(not(feature = "rayon"))]
fn rename_substrs<T: SaisChar>(s: &[T], head: &mut [u32], tail: &mut [u32]) -> usize {
    head.iter_mut().for_each(|i| *i = 0xffffffff);
    let mut k = 0;
    let mut j = tail[0];
    for &i in tail.iter().skip(1) {
        if !lms_substr_eq(s, i as usize, j as usize) {
            k += 1;
        }
        head[i as usize / 2] = k as u32 - 1;
        j = i;
    }
    k
}

#[cfg(feature = "rayon")]
fn rename_substrs<T: SaisChar>(s: &[T], head: &mut [u32], tail: &mut [u32]) -> usize {
    use rayon::prelude::*;

    // compare in parallel and mark result in the most significant bit of head
    head.iter_mut().for_each(|i| *i = 0xffffffff);
    head[1..].par_iter_mut()
        .zip(tail[1..].par_iter())
        .zip(tail.par_iter())
        .for_each(|((eq, i), j)| {
            if !lms_substr_eq(s, *i as usize, *j as usize) {
                *eq = 0x7fffffff;
            }
        });

    // rename lms substrings
    let mut k = 0;
    for p in 1..tail.len() {
        if head[p] & 0x80000000 == 0 {
            k += 1;
        }

        debug_assert!(k as u32 - 1 < 0x7fffffff);
        let i = tail[p] as usize / 2;
        if i == 0 || i >= tail.len() {
            head[i] = k as u32 - 1;
        } else {
            head[i] = (head[i] & 0x80000000) | (k as u32 - 1);
        }
    }
    for p in 1..tail.len() {
        let idx = head[p] & 0x7fffffff;
        if idx == 0x7fffffff {
            head[p] = 0xffffffff;
        } else {
            head[p] = idx;
        }
    }
    k
}

/// Test lms substring equality.
/// s[i..] and s[j..] must be lms substrings.
#[inline]
fn lms_substr_eq<T: SaisChar>(s: &[T], mut i: usize, mut j: usize) -> bool {
    if i > j {
        std::mem::swap(&mut i, &mut j)
    }
    if j > s.len() {
        panic!("index out of range")
    }
    if i == j {
        return true;
    }

    // compare s-type part and the peak part
    if j == s.len() || s[i] != s[j] {
        return false;
    }
    let mut last = s[i];
    i += 1;
    j += 1;
    while j < s.len() && s[i] >= last {
        if s[i] != s[j] {
            return false;
        }
        last = s[i];
        i += 1;
        j += 1;
    }

    // compare remaining l-type part until the valley
    while j < s.len() {
        let (p0, n0, t0) = peek(s, i);
        let (p1, n1, t1) = peek(s, j);
        if p0 != p1 || t0 != t1 {
            return false;
        }
        if t0 {
            // until the nearest lms character, they are the same
            return true;
        }
        if n0 != n1 {
            return false;
        }
        i += n0;
        j += n1;
    }
    false
}

/// Get (value, length, type) of the maximum slice since i, that all the
/// elements in this slice are the same.
#[inline]
fn peek<T: SaisChar>(s: &[T], mut i: usize) -> (T, usize, bool) {
    let p = s[i];
    let mut n = 1;
    i += 1;
    while i < s.len() {
        if s[i] > p {
            return (p, n, true);
        } else if s[i] < p {
            return (p, n, false);
        }
        n += 1;
        i += 1;
    }
    (p, n, false)
}
