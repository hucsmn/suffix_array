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
        let (_, sa_vec) = SuffixArray::new(&*s).into_parts();
        prop_assert!(SuffixArray::from_parts(&*s, sa_vec).is_some());
    }

    #[test]
    fn contains_correctness((s, pat) in bytes_with_pat(0..4096_usize)) {
        let sa = SuffixArray::new(&*s);
        let sa_result = sa.contains(&*pat);
        let naive_result = naive_contains(&*s, &*pat);
        prop_assert!(sa_result == naive_result);
    }

    #[test]
    fn search_all_correctness((s, pat) in bytes_with_pat(0..4096_usize)) {
        let sa = SuffixArray::new(&*s);
        let mut sa_result = Vec::from(sa.search_all(&*pat));
        sa_result.sort();
        let naive_result = naive_search_all(&*s, &*pat);
        prop_assert!(sa_result == naive_result);
    }

    #[test]
    fn search_lcp_correctness((s, pat) in bytes_with_pat(0..1024_usize)) {
        let sa = SuffixArray::new(&*s);
        let sa_result = &s[sa.search_lcp(&*pat)];
        let naive_result = naive_search_lcp(&*s, &*pat);
        prop_assert!(sa_result == naive_result);
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

fn naive_search_lcp<'s>(s: &[u8], sub: &'s [u8]) -> &'s [u8] {
    let mut matched = &sub[..0];
    for i in 0..=s.len() {
        let n = lcp(sub, &s[i..]);
        if n > matched.len() {
            matched = &sub[..n];
        }
    }
    matched
}
