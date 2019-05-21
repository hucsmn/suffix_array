mod utils;

use criterion::{criterion_group, criterion_main, Criterion};
use suffix_array::SuffixArray;
use utils::*;

fn sa_construct(crit: &mut Criterion) {
    let dir = env!("CARGO_MANIFEST_DIR").to_owned() + "/benches/data";
    eprintln!("preparing data in {}...", dir);
    let (samples, _) = make_data(dir.as_ref()).unwrap();

    for sname in samples.into_iter() {
        eprintln!("loading sample {}...", sname);
        let sdata = load_data(dir.as_ref(), sname).unwrap();

        let bench_name =
            format!("sa_construct {} ({} bytes)", sname, sdata.len());

        let times;
        if sdata.len() <= 4096 {
            times = 200;
        } else if sdata.len() <= 1024 * 1024 {
            times = 20;
        } else if sdata.len() <= 16 * 1024 * 1024 {
            times = 8;
        } else {
            times = 3;
        }

        // dirty hack
        let mut tmp = std::mem::replace(crit, Criterion::default());
        tmp = tmp.sample_size(times);
        std::mem::replace(crit, tmp);

        crit.bench_function(bench_name.as_ref(), move |b| {
            b.iter(|| SuffixArray::new(&sdata[..]));
        });
    }
}

criterion_group!(sa_construct_benches, sa_construct);
criterion_main!(sa_construct_benches);
