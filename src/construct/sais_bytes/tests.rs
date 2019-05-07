use super::sais_bytes;
use rand::prelude::*;
use std::ops::Range;

macro_rules! check_literal {
    ($($elem:expr),*) => {
        {
            let s0 = &mut [$($elem,)*];
            let mut sa0 = vec![0; s0.len()+1];
            sais_bytes(s0, &mut sa0[..]);

            let s1 = &[$($elem,)*];
            let mut sa1 = vec![0; s1.len()+1];
            naive_sort(s1, &mut sa1[..]);

            assert_eq!(sa0, sa1);
        }
    };
}

#[test]
fn sais_bytes_basic() {
    check_literal![];
    check_literal![0];
    check_literal![0, 0, 0, 0, 0, 0];
    check_literal![2, 1, 4, 1, 1, 4, 1, 3, 1];
    check_literal![2, 1, 1, 3, 3, 1, 1, 3, 3, 1, 2, 1];
    check_literal![2, 2, 1, 4, 4, 1, 4, 4, 1, 3, 3, 1, 1];
    check_literal![
        1, 2, 2, 1, 1, 0, 0, 1, 1, 2, 2, 0, 0, 2, 2, 0, 1, 0, 2, 0, 1, 1, 1, 1,
        2, 2, 0, 0, 2, 1, 2, 1, 1, 0, 2, 1, 2, 2, 0, 2, 1, 1, 2, 2, 2, 1, 2, 0,
        0, 1, 2, 0, 0, 0, 1, 2, 2, 2, 1, 1, 1, 1, 2, 0, 2, 1, 1, 1, 2, 1, 0, 1
    ];
}

#[test]
fn sais_bytes_small_random_samples() {
    const N: usize = 10000;
    const VALUE_RANGE: Range<u8> = 1..16;
    const LENGTH_RANGE: Range<usize> = 16..256;

    for _ in 0..N {
        let sample = generate_sample(Some(VALUE_RANGE), LENGTH_RANGE);
        let mut sa = vec![0; sample.len() + 1];
        let mut sa_naive = vec![0; sample.len() + 1];

        sais_bytes(&sample[..], &mut sa[..]);
        naive_sort(&sample[..], &mut sa_naive[..]);

        assert_eq!(sa, sa_naive);
    }
}

#[test]
fn sais_bytes_big_random_samples() {
    const N: usize = 3000;
    const LENGTH_RANGE: Range<usize> = 1024..2048;

    for _ in 0..N {
        let sample = generate_sample(None, LENGTH_RANGE);
        let mut sa = vec![0; sample.len() + 1];
        let mut sa_naive = vec![0; sample.len() + 1];

        sais_bytes(&sample[..], &mut sa[..]);
        naive_sort(&sample[..], &mut sa_naive[..]);

        assert_eq!(sa, sa_naive);
    }
}

/// Computes the order of suffixes using qsort.
fn naive_sort(s: &[u8], sa: &mut [u32]) {
    for (i, x) in (0..=s.len()).rev().enumerate() {
        sa[i] = x as u32;
    }

    (&mut sa[1..])
        .sort_by(|&i, &j| Ord::cmp(&s[i as usize..], &s[j as usize..]));
}

/// Generate sample data.
fn generate_sample(vr: Option<Range<u8>>, nr: Range<usize>) -> Vec<u8> {
    let n = thread_rng().gen_range(nr.start, nr.end);
    let mut sample = Vec::with_capacity(n);
    if let Some(vr) = vr {
        for _ in 0..n {
            sample.push(thread_rng().gen_range(vr.start, vr.end) as u8);
        }
    } else {
        for _ in 0..n {
            sample.push(thread_rng().gen::<u8>());
        }
    }

    sample
}
