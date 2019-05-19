use super::sacak0;
use proptest::prelude::*;

macro_rules! bytes {
    ($range:expr) => {
        prop::collection::vec(any::<u8>(), $range)
    };
}

proptest! {
    #[test]
    fn sacak0_correctness(s in bytes!(0..8192_usize)) {
        prop_assert!(check(&s[..]));
    }
}

fn check(s: &[u8]) -> bool {
    let mut sa = vec![0; s.len() + 1];
    sacak0(s, &mut sa[..], naive_sort);
    for i in 1..sa.len() {
        let x = &s[sa[i - 1] as usize..];
        let y = &s[sa[i] as usize..];
        if x >= y {
            return false;
        }
    }
    true
}

fn naive_sort(s: &mut [u32], _: usize, sa: &mut [u32]) {
    for (i, x) in (0..=s.len()).rev().enumerate() {
        sa[i] = x as u32;
    }

    (&mut sa[1..])
        .sort_by(|&i, &j| Ord::cmp(&s[i as usize..], &s[j as usize..]));
}
