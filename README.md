Suffix array
============

Suffix array construction and searching algorithms for in-memory binary data.

For text indexing, burntsushi's [suffix](https://crates.io/crates/suffix) featuring utf-8 support is preferred.

Currently, suffix array construction speed of this crate [2x-3x slower](https://gist.github.com/hucsmn/1e7848bed0c09a843291ae5afe0cd895) than Yuta Mori's [dissufsort](https://github.com/y-256/libdivsufsort) and Amos Wenger's [hand-ported divsufsort](https://crates.io/crates/divsufsort) in rust. For people interested in suffix array contruction for binary data, it is recommended to use the new [C-binding to divsufsort](https://crates.io/crates/cdivsufsort) by Amos Wenger.



TODO
=====
* [x] Benchmark using [Pizza&Chili Corpus](http://pizzachili.dcc.uchile.cl).
* [x] Compare LMS substrings in parallel.
* [ ] Speed up searching by LCP array (enhanced suffix array).
* [ ] <del>Add compressed suffix array support.</del>
* [x] Serialization/deserialization.
* [ ] **Rewrite suffix array construction algorithm (try to parallelize SACA-K according to
      [recent parallelization efforts on SAIS](https://link.springer.com/article/10.1007/s11227-018-2395-5),
      or simply switch the suffix array construction to Amos Wenger's [hand-ported divsufsort in rust](https://crates.io/crates/divsufsort).**
