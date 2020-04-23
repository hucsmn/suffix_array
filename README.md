Suffix array
============

Suffix array construction and searching algorithms for in-memory binary data.

For text indexing, burntsushi's [suffix](https://crates.io/crates/suffix) featuring utf-8 support is preferred.

Suffix array construction algorithm (SACA) in this crate uses the [C bindings by Amos Wenger](https://crates.io/crates/cdivsufsort) to [Yuta Mori's dissufsort](https://github.com/y-256/libdivsufsort), which is the fastest known SACA running in single thread with only O(1) additional workspace.

In addition, I have implemented a space-efficient parallel SACA named [pSACAK](https://ieeexplore.ieee.org/document/8371211) in [a separate crate](https://crates.io/crates/psacak). For now, it has not been thoroughly benchmarked as well as optimized. If it is proved to be valuable, I would like to make it an optional SACA for this crate in future.

TODO
=====
* [x] Benchmark using [Pizza&Chili Corpus](http://pizzachili.dcc.uchile.cl).
* [x] Compare LMS substrings in parallel.
* [ ] Speed up searching by LCP array (enhanced suffix array).
* [ ] <del>Add compressed suffix array support.</del>
* [x] Serialization/deserialization.
* [x] **Rewrite suffix array construction algorithm (try to parallelize SACA-K according to
      the recent parallelization efforts on [SAIS](https://link.springer.com/article/10.1007/s11227-018-2395-5) and
      SACK-K ([[1]](https://ieeexplore.ieee.org/document/8371211), [[2]](https://link.springer.com/chapter/10.1007%2F978-981-15-2767-8_30)) by Bin Lao.
      Or simply switch the suffix array construction slgorithm to Amos Wenger's [hand-ported](https://crates.io/crates/divsufsort) divsufsort or the [C binding](https://crates.io/crates/cdivsufsort).**
