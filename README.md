Suffix array
============

Suffix array construction and searching algorithms for in-memory binary
data, focusing on space efficiency.

TODO
=====
* [x] Benchmark using [Pizza&Chili Corpus](http://pizzachili.dcc.uchile.cl).
* [x] Compare LMS substrings in parallel.
* [ ] Speed up searching by LCP array (enhanced suffix array).
* [ ] <del>Add compressed suffix array support.</del>
* [x] Serialization/deserialization.
* [ ] **Rewrite suffix array construction algorithm (try to parallelize SACA-K according to
      [recent parallelization efforts on SAIS](https://link.springer.com/article/10.1007/s11227-018-2395-5),
      or look at the [parallel divsufsort](https://ieeexplore.ieee.org/document/7786147),
      whose original version was described in [this paper](https://arxiv.org/abs/1710.01896)).**
