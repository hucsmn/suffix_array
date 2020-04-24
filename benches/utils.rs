#![allow(dead_code)]

use self::DataLength::*;
use self::PatternLength::*;
use criterion::Criterion;
use flate2::read::GzDecoder;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::fs;
use std::io;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::path::PathBuf;
use suffix_array::MAX_LENGTH;

const SEPARATOR: &'static str = "~";

static RANDOM_DATA_STATS: &[(&str, u8, usize)] = &[
    // (sample_name, alphabet, length)
    ("random-128k", 255, 128 * 1024),
    ("random-50m", 255, 50 * 1024 * 1024),
    ("random-200m", 255, 200 * 1024 * 1024),
];

static PIZZA_CHILI_URLS: &[(&str, &str, bool)] = &[
    (
        "dna-50m",
        "http://pizzachili.dcc.uchile.cl/texts/dna/dna.50MB.gz",
        true,
    ),
    (
        "dna-200m",
        "http://pizzachili.dcc.uchile.cl/texts/dna/dna.200MB.gz",
        true,
    ),
    (
        "english-50m",
        "http://pizzachili.dcc.uchile.cl/texts/nlang/english.50MB.gz",
        true,
    ),
    (
        "english-200m",
        "http://pizzachili.dcc.uchile.cl/texts/nlang/english.200MB.gz",
        true,
    ),
];

static PATTERN_SCHEMES: &[(&str, DataLength, PatternLength, PatternLength)] = &[
    // (pattern_name, sample_length_filter, final_pattern_length, random_trailing_junk)
    ("select-32b", Unlimited, Fixed(32), Fixed(0)),
    ("hybrid-32b", Till(1024 * 1024), Fixed(32), Fixed(16)),
    ("select-4k", Unlimited, Fixed(4 * 1024), Fixed(0)),
    (
        "hybrid-4k",
        Till(1024 * 1024),
        Fixed(4 * 1024),
        Fixed(2 * 1024),
    ),
    ("select-5%", Since(1024 * 1024), Ratio(0.05), Fixed(0)),
    ("hybrid-5%", Since(1024 * 1024), Ratio(0.05), Ratio(0.5)),
];

#[derive(Clone, Copy)]
enum DataLength {
    Unlimited,
    Since(usize),
    Till(usize),
    Ranged(usize, usize),
}

impl DataLength {
    pub fn contains(&self, dlen: usize) -> bool {
        match self {
            &Unlimited => true,
            &Since(x) => dlen >= x,
            &Till(x) => dlen <= x,
            &Ranged(x, y) => dlen >= x && dlen <= y,
        }
    }
}

#[derive(Clone, Copy)]
enum PatternLength {
    Fixed(usize),
    Ratio(f64),
}

impl PatternLength {
    pub fn calculate(&self, accord: usize) -> usize {
        match self {
            &Fixed(n) => Ord::min(accord, n),
            &Ratio(mut r) => {
                if r > 1.0 {
                    r = 1.0;
                }
                if r < 0.0 {
                    r = 0.0;
                }
                let n = (r * accord as f64) as usize;
                Ord::min(accord, n)
            }
        }
    }
}

pub fn load_data(dir: &str, name: &str) -> Result<Vec<u8>> {
    let fpath = PathBuf::from(dir).join(name);
    let flen = fs::metadata(&fpath)?.len();
    if flen > MAX_LENGTH as u64 {
        return Err(Error::new(ErrorKind::Other, "file too long"));
    }

    let mut file = fs::File::open(fpath)?;
    let mut data = Vec::with_capacity(flen as usize);
    file.read_to_end(&mut data)?;
    Ok(data)
}

pub fn load_pattern(dir: &str, name: &str, pattern: &str) -> Result<Vec<u8>> {
    let full_name = String::from(name) + SEPARATOR + pattern;
    load_data(dir, full_name.as_ref())
}

pub fn make_data(dir: &str) -> Result<(Vec<&'static str>, Vec<&'static str>)> {
    let mut samples = Vec::new();
    let mut patterns = Vec::new();

    if fs::metadata(dir).is_err() {
        fs::create_dir_all(dir)?;
    }

    // Generates random data if not already created.
    for &(name, scale, mut length) in RANDOM_DATA_STATS.iter() {
        let fpath = PathBuf::from(dir).join(name);
        if fs::metadata(&fpath).is_err() {
            eprintln!("generating sample {}", name);
            let mut file = fs::File::create(&fpath)?;
            let mut buf = vec![0; 4096];
            while length > 0 {
                let n = Ord::min(length, buf.len());
                random_bytes(scale, &mut buf[..n]);
                file.write_all(&buf[..n])?;
                length -= n;
            }
        }
        samples.push(name);
    }

    // Download Pizza&Chili's Corpus (each 50M) if not cached.
    for &(name, url, gz) in PIZZA_CHILI_URLS.iter() {
        let fpath = PathBuf::from(dir).join(name);
        if fs::metadata(&fpath).is_err() {
            eprintln!("downloading sample {}", name);
            match reqwest::get(url) {
                Ok(mut resp) => {
                    let mut file = fs::File::create(&fpath)?;
                    if gz {
                        let mut uncompressed = GzDecoder::new(&mut resp);
                        io::copy(&mut uncompressed, &mut file)?;
                    } else {
                        io::copy(&mut resp, &mut file)?;
                    }
                }
                Err(e) => {
                    return Err(Error::new(ErrorKind::Other, e));
                }
            }
        }
        samples.push(name);
    }

    for &(name, _, _, _) in PATTERN_SCHEMES.iter() {
        patterns.push(name);
    }

    // Generate patterns for each sample if not already created.
    for &sname in samples.iter() {
        let sdata = load_data(dir, sname)?;
        for &(pname, dlen, plen, jlen) in PATTERN_SCHEMES.iter() {
            if !dlen.contains(sdata.len()) {
                continue;
            }

            let full_name = String::from(sname) + SEPARATOR + pname;
            let fpath = PathBuf::from(dir).join(full_name);
            if fs::metadata(&fpath).is_err() {
                eprintln!("generating pattern {}~{}", sname, pname);
                let mut file = fs::File::create(fpath)?;
                let total = plen.calculate(sdata.len());
                let b = jlen.calculate(total);
                let a = total - b;

                let mut pdata = Vec::with_capacity(total);
                pdata.extend_from_slice(random_select(a, &sdata[..]));
                let mut rng = thread_rng();
                for _ in 0..b {
                    pdata.push(rng.gen::<u8>());
                }
                file.write_all(&pdata[..])?;
            }
        }
    }

    Ok((samples, patterns))
}

fn random_bytes(scale: u8, buf: &mut [u8]) {
    let uni = Uniform::from(0..=scale);
    let mut rng = thread_rng();
    buf.iter_mut().for_each(|c| *c = rng.sample(uni));
}

fn random_select(len: usize, buf: &[u8]) -> &[u8] {
    let uni = Uniform::from(0..=buf.len() - len);
    let mut rng = thread_rng();
    let i = rng.sample(uni);
    &buf[i..i + len]
}

// dirty hack
pub fn set_criterion_samples(crit: &mut Criterion, mut n: usize) {
    if n < 2 {
        n = 2;
    }

    let mut tmp = std::mem::replace(crit, Criterion::default());
    tmp = tmp.sample_size(n);
    std::mem::replace(crit, tmp);
}
