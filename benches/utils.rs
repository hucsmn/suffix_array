#![allow(unused)]

use rand::random;
use std::ops::Range;

pub fn gen_sample(
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

pub fn gen_bytes(len: Range<usize>, scale: Range<u8>) -> Vec<u8> {
    let n = len.start + random::<usize>() % (len.end - len.start);
    let k = scale.start + random::<u8>() % (scale.end - scale.start);
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(random::<u8>() % k);
    }
    v
}
