use super::utils::lcp;
use super::SuffixArray;

use proptest::prelude::*;

macro_rules! bytes {
    ($range:expr) => {
        prop::collection::vec(any::<u8>(), $range)
    };
}

proptest! {
    #[test]
    fn conversion_correctness(s in bytes!(0..4096_usize)) {
        let (_, sa_vec) = SuffixArray::new(&s[..]).into_parts();
        prop_assert!(SuffixArray::from_parts(&s[..], sa_vec).is_some());
    }

    #[test]
    fn contains_correctness((s, pat) in bytes_with_pat(0..4096_usize)) {
        let naive_result = naive_contains(&s[..], &pat[..]);

        let mut sa = SuffixArray::new(&s[..]);
        let sa_result_simple = sa.contains(&pat[..]);
        prop_assert_eq!(sa_result_simple, naive_result);

        sa.enable_buckets();
        let sa_result_bucket = sa.contains(&pat[..]);
        prop_assert_eq!(sa_result_bucket, naive_result);
    }

    #[test]
    fn search_all_correctness((s, pat) in bytes_with_pat(0..4096_usize)) {
        let mut naive_result = naive_search_all(&s[..], &pat[..]);
        naive_result.sort();

        let mut sa = SuffixArray::new(&s[..]);
        let mut sa_result_simple = Vec::from(sa.search_all(&pat[..]));
        sa_result_simple.sort();
        prop_assert_eq!(&sa_result_simple[..], &naive_result[..]);

        sa.enable_buckets();
        let mut sa_result_bucket = Vec::from(sa.search_all(&pat[..]));
        sa_result_bucket.sort();
        prop_assert_eq!(&sa_result_bucket[..], &naive_result[..]);
    }

    #[test]
    fn search_lcp_correctness((s, pat) in bytes_with_pat(0..1024_usize)) {
        let naive_result = naive_search_lcp(&s[..], &pat[..]);

        let mut sa = SuffixArray::new(&s[..]);
        let sa_result_simple = &s[sa.search_lcp(&pat[..])];
        prop_assert_eq!(sa_result_simple, naive_result);

        sa.enable_buckets();
        let sa_result_bucket = &s[sa.search_lcp(&pat[..])];
        prop_assert_eq!(sa_result_bucket, naive_result);
    }

    #[cfg(feature = "pack")]
    #[test]
    fn pack_correctness(s in bytes!(0..4096_usize)) {
        use std::io::Cursor;

        let sa1 = SuffixArray::new(&s[..]);
        let bytes1 = sa1.dump_bytes().unwrap();
        let mut bytes2 = Vec::with_capacity(bytes1.len());
        sa1.dump(Cursor::new(&mut bytes2)).unwrap();
        let sa2 = SuffixArray::load_bytes(&s[..], &*bytes1).unwrap();

        let (_, sa1) = sa1.into_parts();
        let (_, sa2) = sa2.into_parts();
        prop_assert_eq!(sa1, sa2);
        prop_assert_eq!(bytes1, bytes2);
    }
}

fn bytes_with_pat(
    len: impl Strategy<Value = usize>,
) -> impl Strategy<Value = (Vec<u8>, Vec<u8>)> {
    (len, 0.0..1.0).prop_flat_map(|(n, pat_ratio)| {
        let m = (n as f64 * pat_ratio) as usize;

        let no_junk = (0..=n - m, bytes!(n..=n)).prop_map(move |(i, s)| {
            let pat = Vec::from(&s[i..i + m]);
            (s, pat)
        });

        let trail_junk = (0..=n - m, bytes!(n..=n), bytes!(0..=m)).prop_map(
            move |(i, s, mut junk)| {
                let mut pat = Vec::from(&s[i..i + (m - junk.len())]);
                pat.append(&mut junk);
                (s, pat)
            },
        );

        let all_junk = (bytes!(n..=n), bytes!(m..=m));

        prop_oneof!(no_junk, trail_junk, all_junk)
    })
}

fn naive_contains(s: &[u8], pat: &[u8]) -> bool {
    for i in 0..=s.len().saturating_sub(pat.len()) {
        if pat == &s[i..Ord::min(s.len(), i + pat.len())] {
            return true;
        }
    }
    false
}

fn naive_search_all(s: &[u8], pat: &[u8]) -> Vec<u32> {
    let mut result = Vec::new();
    for i in 0..=s.len().saturating_sub(pat.len()) {
        if pat == &s[i..Ord::min(s.len(), i + pat.len())] {
            result.push(i as u32);
        }
    }
    result
}

fn naive_search_lcp<'s>(s: &[u8], pat: &'s [u8]) -> &'s [u8] {
    let mut matched = &pat[..0];
    for i in 0..=s.len() {
        let n = lcp(pat, &s[i..]);
        if n > matched.len() {
            matched = &pat[..n];
        }
    }
    matched
}
