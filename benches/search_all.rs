mod utils;

use utils::*;
use std::ops::Range;
use suffix_array::SuffixArray;
use criterion::{criterion_group, criterion_main, Bencher, Criterion};

const LEN_SAMPLES: usize = 20;

criterion_group!(
    benches,
    small_len_small_scale,
    small_len_big_scale,
    median_len_small_scale,
    median_len_big_scale,
    large_len_small_scale,
    large_len_big_scale,
);
criterion_main!(benches);

macro_rules! bench_with_cfg {
    (str: $ss:tt .. $se:tt, pat: $ps:tt .. $pe:tt +  $ts:tt .. $te:tt, scale: $scs:tt .. $sce:tt) => {
        |b| bench_by(b, $ss..$se, $ps..$pe, $ts..$te, $scs..$sce)
    };
}

fn small_len_small_scale(c: &mut Criterion) {
    c.bench_function("len-/scale-", bench_with_cfg!(str: 1..128, pat: 4..16 + 0..2, scale: 1..8));
}

fn small_len_big_scale(c: &mut Criterion) {
    c.bench_function("len-/scale+", bench_with_cfg!(str: 1..128, pat: 4..16 + 0..2, scale: 1..128));
}

fn median_len_small_scale(c: &mut Criterion) {
    c.bench_function("len=/scale-", bench_with_cfg!(str: 128..1024, pat: 8..32 + 0..2, scale: 4..16));
}

fn median_len_big_scale(c: &mut Criterion) {
    c.bench_function("len=/scale+", bench_with_cfg!(str: 128..1024, pat: 8..32 + 0..2, scale: 128..192));
}

fn large_len_small_scale(c: &mut Criterion) {
    c.bench_function("len+/scale-", bench_with_cfg!(str: 1024..2048, pat: 16..64 + 0..2, scale: 8..32));
}

fn large_len_big_scale(c: &mut Criterion) {
    c.bench_function("len+/scale+", bench_with_cfg!(str: 1024..2048, pat: 16..64 + 0..2, scale: 192..255));
}

// TODO: test huge data

fn bench_by(b: &mut Bencher, slen: Range<usize>, plen: Range<usize>, tlen: Range<usize>, scale: Range<u8>) {
    let mut data = Vec::with_capacity(LEN_SAMPLES);
    for _ in 0..LEN_SAMPLES {
        let (s, p) = gen_sample(slen.clone(), plen.clone(), tlen.clone(), scale.clone());
        data.push((s, p));
    }

    let mut samples = Vec::with_capacity(LEN_SAMPLES);
    for x in data.iter() {
        samples.push((SuffixArray::new(&x.0), &x.1));
    }

    b.iter(|| {
        for (sa, pat) in samples.iter() {
            sa.search_all(pat);
        }
    });
}
