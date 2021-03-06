#[macro_use]
extern crate rental;

mod utils;

use criterion::{criterion_group, criterion_main, Criterion};
use std::rc::Rc;
use suffix_array::SuffixArray;
use utils::*;

// workaround for the static lifetime bound
rental! {
    mod owned_sa {
        use suffix_array::SuffixArray;

        #[rental]
        pub struct OwnedSA {
            data: Vec<u8>,
            sa: SuffixArray<'data>,
        }
    }
}
use owned_sa::OwnedSA;

macro_rules! search_method_bench {
    ($name:ident, $label:expr, $method:ident) => {
        fn $name(crit: &mut Criterion) {
            let dir = env!("CARGO_MANIFEST_DIR").to_owned() + "/benches/data";
            eprintln!("preparing data in {}...", dir);
            let (samples, patterns) = make_data(dir.as_ref()).unwrap();

            for sname in samples.into_iter() {
                eprint!("loading sample {}...", sname);
                let sdata;
                if let Ok(tmp) = load_data(dir.as_ref(), sname) {
                    sdata = tmp;
                    eprintln!("yes");
                } else {
                    eprintln!("pass");
                    continue;
                }
                let slen = sdata.len();

                eprintln!("constructing sa...");
                let osa = Rc::new(OwnedSA::new(sdata, |sdata| {
                    let mut sa = SuffixArray::new(&sdata[..]);
                    sa.enable_buckets();
                    sa
                }));

                for &pname in patterns.iter() {
                    eprint!("loading pattern {}...", pname);
                    let pdata;
                    if let Ok(tmp) = load_pattern(dir.as_ref(), sname, pname) {
                        pdata = tmp;
                        eprintln!("yes");
                    } else {
                        eprintln!("pass");
                        continue;
                    }
                    let plen = pdata.len();

                    let bench_name = format!("{} {}~{}", $label, sname, pname);

                    set_criterion_samples(crit, calc_samples(slen, plen));

                    let osa = osa.clone();
                    crit.bench_function(bench_name.as_ref(), move |b| {
                        b.iter(|| {
                            osa.rent(|sa| {
                                sa.$method(&pdata[..]);
                            })
                        })
                    });
                }
            }
        }
    };
}

fn calc_samples(slen: usize, plen: usize) -> usize {
    if slen <= 4096 {
        1000
    } else if slen <= 1024 * 1024 {
        100
    } else if slen <= 16 * 1024 * 1024 {
        if plen <= 4096 {
            100
        } else if slen <= 1024 * 1024 {
            10
        } else {
            3
        }
    } else {
        if plen <= 4096 {
            50
        } else if slen <= 1024 * 1024 {
            5
        } else {
            2
        }
    }
}

search_method_bench!(sa_contains, "constains", contains);
search_method_bench!(sa_search_all, "search_all", search_all);
search_method_bench!(sa_search_lcp, "search_lcp", search_lcp);

criterion_group!(sa_search_benches, sa_contains, sa_search_all, sa_search_lcp);
criterion_main!(sa_search_benches);
