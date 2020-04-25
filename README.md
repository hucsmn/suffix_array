Suffix array
============

Suffix array construction and searching algorithms for in-memory binary data.

To index plain texts, burntsushi's [suffix](https://crates.io/crates/suffix) featuring utf-8 support is a better choice.

This crate uses the Amos Wenger's [C bindings](https://crates.io/crates/cdivsufsort) to Yuta Mori's [dissufsort](https://github.com/y-256/libdivsufsort) to construct suffix array, which is the fastest known suffix array construction algorithm (SACA) running in single thread that uses merely O(1) additional workspace.

In addition, I have implemented the space-efficient parallel SACA [pSACAK](https://ieeexplore.ieee.org/document/8371211) in a separate [crate](https://crates.io/crates/psacak). For now, it has not been thoroughly benchmarked as well as optimized.

TODO
=====
* [x] Benchmark using [Pizza&Chili Corpus](http://pizzachili.dcc.uchile.cl).

* [ ] Rewrite the benchmarks.

* [ ] <del>Speed up searching by LCP array, a.k.a. construct enhanced suffix array (ESA).</del>
      There are two major classes of LCP array construction algorithms (LACA):
      The first class produces LCP array from suffix array. AFAIK, these LACAs have to allocate additional auxiliary arrays (such as [ISA](https://dl.acm.org/doi/10.5555/647820.736222), [PLCP](https://doi.org/10.1007/978-3-642-02441-2_17), and [BWT](https://dl.acm.org/doi/10.5555/2790248.2790251)) to construct the LCP array.
      The second class constructs the suffix array together with its LCP array.
      They are fast and space-efficient but require sophisticated modifications on kind of SACAs based on induce copying (such as [sais-lite](https://arxiv.org/abs/1101.3448), [saca-k](https://dl.acm.org/doi/10.1016/j.ipl.2016.09.010), and [divsufsort](https://arxiv.org/abs/1710.01896)).
      This crate would not provide ESA support until I figure out a proper way to implement it.

* [x] Speed up searching by bucket pointers, inspired by [this paper](https://dl.acm.org/doi/10.1145/1376616.1376683) (which uses a trie as index, interleaved arrays and text fingerprints to improve locality for ESA searching on disk).
      See `SuffixArray::enable_buckets()`.

* [ ] <del>Add compressed suffix array (CSA) support.</del>
      CSA safcrificed the speed quite a lot to gain some space efficiency, which is not that necessary.

* [x] Serialization/deserialization. Enable the optional `pack` feature to use those APIs. This feature is based on Paul Masurel's [bitpacking](https://crates.io/crates/bitpacking).

* [x] Rewrite suffix array construction algorithm.
      Currently, this crate uses [dissufsort](https://github.com/y-256/libdivsufsort) to construct all the suffix arrays.

* [ ] Look into other one's efforts on improving libdivsufsort and multikey quick sort: [1](http://panthema.net/2013/parallel-string-sorting/), [2](https://github.com/akamiru/sort/wiki/General-Description-of-DAware), and [3](https://github.com/jlabeit/parallel-divsufsort).