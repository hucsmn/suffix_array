mod utils;

use criterion::{criterion_group, criterion_main, Criterion};
use suffix_array::SuffixArray;
use utils::*;

fn sa_construct(crit: &mut Criterion) {
    let dir = env!("CARGO_MANIFEST_DIR").to_owned() + "/benches/data";
    eprintln!("preparing data in {}...", dir);
    let (samples, _) = make_data(dir.as_ref()).unwrap();

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

        let bench_name = format!("saca {}", sname);

        set_criterion_samples(crit, calc_samples(sdata.len()));

        crit.bench_function(bench_name.as_ref(), move |b| {
            b.iter(|| SuffixArray::new(&sdata[..]));
        });
    }
}

fn calc_samples(slen: usize) -> usize {
    if slen <= 4096 {
        100
    } else if slen <= 1024 * 1024 {
        10
    } else if slen <= 16 * 1024 * 1024 {
        3
    } else {
        2
    }
}

criterion_group!(sa_construct_benches, sa_construct);
criterion_main!(sa_construct_benches);
