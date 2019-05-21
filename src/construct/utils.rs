/// Generic character type for the sais algorithm.
pub trait SaisChar: Copy + Ord {
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

/// Test lms substring equality.
/// s[i..] and s[j..] must be lms substrings.
#[inline]
pub fn lms_substr_eq<T: SaisChar>(s: &[T], mut i: usize, mut j: usize) -> bool {
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
