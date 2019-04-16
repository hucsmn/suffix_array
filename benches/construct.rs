use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use rand::random;
use std::ops::Range;
use suffix_array::SuffixArray;

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

fn small_len_small_scale(c: &mut Criterion) {
    c.bench_function("len-/scale-", |b| bench_by(b, 1..128, 1..8));
}

fn small_len_big_scale(c: &mut Criterion) {
    c.bench_function("len-/scale+", |b| bench_by(b, 1..128, 1..128));
}

fn median_len_small_scale(c: &mut Criterion) {
    c.bench_function("len=/scale-", |b| bench_by(b, 128..1024, 4..16));
}

fn median_len_big_scale(c: &mut Criterion) {
    c.bench_function("len=/scale+", |b| bench_by(b, 128..1024, 128..192));
}

fn large_len_small_scale(c: &mut Criterion) {
    c.bench_function("len+/scale-", |b| bench_by(b, 1024..2048, 8..32));
}

fn large_len_big_scale(c: &mut Criterion) {
    c.bench_function("len+/scale+", |b| bench_by(b, 1024..2048, 192..255));
}

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

fn gen_bytes(len: Range<usize>, scale: Range<u8>) -> Vec<u8> {
    let n = len.start + random::<usize>() % (len.end - len.start);
    let k = scale.start + random::<u8>() % (scale.end - scale.start);
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(random::<u8>() % k);
    }
    v
}
