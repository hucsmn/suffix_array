use super::sacak;
use proptest::prelude::*;

macro_rules! bytes {
    ($range:expr) => {
        prop::collection::vec(any::<u8>(), $range)
    };
}

proptest! {
    #[test]
    fn sacak_correctness(s in bytes!(0..8192_usize)) {
        prop_assert!(check(&s[..]));
    }
}

fn check(s: &[u8]) -> bool {
    let mut sa = vec![0; s.len() + 1];
    sacak(s, &mut sa[..]);
    for i in 1..sa.len() {
        let x = &s[sa[i - 1] as usize..];
        let y = &s[sa[i] as usize..];
        if x >= y {
            return false;
        }
    }
    true
}
