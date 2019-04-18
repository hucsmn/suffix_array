mod utils;

use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use std::ops::Range;
use suffix_array::SuffixArray;
use utils::*;

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
    (str: $ss:tt .. $se:tt, scale: $scs:tt .. $sce:tt) => {
        |b| bench_by(b, $ss..$se, $scs..$sce)
    };
}

fn small_len_small_scale(c: &mut Criterion) {
    c.bench_function(
        "construct len-/scale-",
        bench_with_cfg!(str: 1..128, scale: 1..8),
    );
}

fn small_len_big_scale(c: &mut Criterion) {
    c.bench_function(
        "construct len-/scale+",
        bench_with_cfg!(str: 1..128, scale: 1..128),
    );
}

fn median_len_small_scale(c: &mut Criterion) {
    c.bench_function(
        "construct len=/scale-",
        bench_with_cfg!(str: 128..1024, scale: 4..16),
    );
}

fn median_len_big_scale(c: &mut Criterion) {
    c.bench_function(
        "construct len=/scale+",
        bench_with_cfg!(str: 128..1024, scale: 128..192),
    );
}

fn large_len_small_scale(c: &mut Criterion) {
    c.bench_function(
        "construct len+/scale-",
        bench_with_cfg!(str: 1024..2048, scale: 8..32),
    );
}

fn large_len_big_scale(c: &mut Criterion) {
    c.bench_function(
        "construct len+/scale+",
        bench_with_cfg!(str: 1024..2048, scale: 192..255),
    );
}

// TODO: test huge data

fn bench_by(b: &mut Bencher, len: Range<usize>, scale: Range<u8>) {
    let mut samples = Vec::with_capacity(LEN_SAMPLES);
    for _ in 0..LEN_SAMPLES {
        samples.push(gen_bytes(len.clone(), scale.clone()))
    }

    b.iter(|| {
        for s in samples.iter() {
            SuffixArray::new(&s[..]);
        }
    });
}
