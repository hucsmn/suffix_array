use super::utils::common_prefix;
use super::SuffixArray;
use rand::random;
use std::ops::Range;

macro_rules! assert_contains_correct {
    ($pat:expr => $s:expr) => {{
        let s = $s;
        let pat = $pat;
        let sa = SuffixArray::new(s);
        assert_eq!(sa.contains(pat), naive_contains(s, pat));
    }};
}

macro_rules! assert_search_all_correct {
    ($pat:expr => $s:expr) => {{
        let s = $s;
        let pat = $pat;
        let sa = SuffixArray::new(s);

        let mut sa_result: Vec<_> = sa.search_all(pat).into();
        sa_result.sort();
        let naive_result = naive_search_all(s, pat);
        assert_eq!(sa_result, naive_result);
    }};
}

macro_rules! assert_search_prefix_correct {
    ($pat:expr => $s:expr) => {{
        let s = $s;
        let pat = $pat;
        let sa = SuffixArray::new(s);

        let sa_result = &s[sa.search_prefix(pat)];
        let naive_result = naive_search_prefix(s, pat);
        assert_eq!(sa_result, naive_result);
    }};
}

#[test]
fn suffix_array_contains_basic() {
    assert_contains_correct!(b"" => b"");
    assert_contains_correct!(b"" => b"");
    assert_contains_correct!(b"" => b"x");
    assert_contains_correct!(b"x" => b"");
    assert_contains_correct!(b"x" => b"x");
    assert_contains_correct!(b"11" => b"31112113");
    assert_contains_correct!(b"112" => b"31112113");
    assert_contains_correct!(b"114" => b"31112113");
}

#[test]
fn suffix_array_contains_random_samples() {
    const SAMPLES: usize = 1000;
    const BYTES_LEN: Range<usize> = 0..200;
    const PATTERN_LEN: Range<usize> = 0..16;
    const TRAILING_LEN: Range<usize> = 0..2;
    const SCALE: Range<u8> = 1..16;

    for _ in 0..SAMPLES {
        let (sample_s, sample_pat) = gen_sample(BYTES_LEN, PATTERN_LEN, TRAILING_LEN, SCALE);
        assert_contains_correct!(&sample_pat[..] => &sample_s[..]);
    }
}

#[test]
fn suffix_array_search_all_basic() {
    assert_search_all_correct!(b"" => b"");
    assert_search_all_correct!(b"" => b"x");
    assert_search_all_correct!(b"x" => b"");
    assert_search_all_correct!(b"x" => b"x");
    assert_search_all_correct!(b"11" => b"31112113");
    assert_search_all_correct!(b"112" => b"31112113");
    assert_search_all_correct!(b"114" => b"31112113");
}

#[test]
fn suffix_array_search_all_random_samples() {
    const SAMPLES: usize = 200;
    const BYTES_LEN: Range<usize> = 0..200;
    const PATTERN_LEN: Range<usize> = 0..4;
    const TRAILING_LEN: Range<usize> = 0..2;
    const SCALE: Range<u8> = 1..6;

    for _ in 0..SAMPLES {
        let (sample_s, sample_pat) = gen_sample(BYTES_LEN, PATTERN_LEN, TRAILING_LEN, SCALE);
        assert_search_all_correct!(&sample_pat[..] => &sample_s[..]);
    }
}

#[test]
fn suffix_array_search_prefix_basic() {
    assert_search_prefix_correct!(b"" => b"");
    assert_search_prefix_correct!(b"" => b"x");
    assert_search_prefix_correct!(b"x" => b"");
    assert_search_prefix_correct!(b"x" => b"x");
    assert_search_prefix_correct!(b"11" => b"31112113");
    assert_search_prefix_correct!(b"112" => b"31112113");
    assert_search_prefix_correct!(b"114" => b"31112113");
}

#[test]
fn suffix_array_search_prefix_random_samples() {
    const SAMPLES: usize = 500;
    const BYTES_LEN: Range<usize> = 0..500;
    const PATTERN_LEN: Range<usize> = 0..8;
    const TRAILING_LEN: Range<usize> = 0..8;
    const SCALE: Range<u8> = 1..8;

    for _ in 0..SAMPLES {
        let (sample_s, sample_pat) = gen_sample(BYTES_LEN, PATTERN_LEN, TRAILING_LEN, SCALE);
        assert_search_prefix_correct!(&sample_pat[..] => &sample_s[..]);
    }
}

fn naive_contains(s: &[u8], sub: &[u8]) -> bool {
    for i in 0..=s.len().saturating_sub(sub.len()) {
        if sub == &s[i..Ord::min(s.len(), i + sub.len())] {
            return true;
        }
    }
    false
}

fn naive_search_all(s: &[u8], sub: &[u8]) -> Vec<u32> {
    let mut result = Vec::new();
    for i in 0..=s.len().saturating_sub(sub.len()) {
        if sub == &s[i..Ord::min(s.len(), i + sub.len())] {
            result.push(i as u32);
        }
    }
    result
}

fn naive_search_prefix<'s>(s: &[u8], sub: &'s [u8]) -> &'s [u8] {
    let mut matched = &sub[..0];
    for i in 0..=s.len() {
        let n = common_prefix(sub, &s[i..]);
        if n > matched.len() {
            matched = &sub[..n];
        }
    }
    matched
}

fn gen_sample(
    s: Range<usize>,
    p: Range<usize>,
    t: Range<usize>,
    scale: Range<u8>,
) -> (Vec<u8>, Vec<u8>) {
    let bytes = gen_bytes(s, scale.clone());
    let plen = p.start + random::<usize>() % (p.end - p.start);
    let pstart = random::<usize>() % (bytes.len().saturating_sub(plen) + 1);
    let mut pat: Vec<_> = bytes[pstart..Ord::min(bytes.len(), pstart + plen)].into();
    pat.append(&mut gen_bytes(t, scale));
    (bytes, pat)
}

fn gen_bytes(len: Range<usize>, scale: Range<u8>) -> Vec<u8> {
    let n = len.start + random::<usize>() % (len.end - len.start);
    let k = scale.start + random::<u8>() % (scale.end - scale.start);
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(random::<u8>() % k);
    }
    v
}
