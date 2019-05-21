#![allow(dead_code)]

use self::Length::{Fixed, Ratio};
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
    ("random-small-s", 8, 128),
    ("random-ascii-s", 127, 128),
    ("random-bin-s", 255, 128),
    /*
        ("random-small-m", 8, 1024),
        ("random-ascii-m", 127, 1024),
        ("random-bin-m", 255, 1024),
    */
    ("random-small-l", 8, 65536),
    ("random-ascii-l", 127, 65536),
    ("random-bin-l", 255, 65536),
    ("random-small-xl", 8, 4 * 1024 * 1024),
    ("random-ascii-xl", 127, 4 * 1024 * 1024),
    ("random-bin-xl", 255, 4 * 1024 * 1024),
    ("random-small-2xl", 8, 16 * 1024 * 1024),
    ("random-ascii-2xl", 127, 16 * 1024 * 1024),
    ("random-bin-2xl", 255, 16 * 1024 * 1024),
    ("random-small-3xl", 8, 50 * 1024 * 1024),
    ("random-ascii-3xl", 127, 50 * 1024 * 1024),
    ("random-bin-3xl", 255, 50 * 1024 * 1024),
    /*
        ("random-small-4xl", 8, 100 * 1024 * 1024),
        ("random-ascii-4xl", 127, 100 * 1024 * 1024),
        ("random-bin-4xl", 255, 100 * 1024 * 1024),
    */
];

static PIZZA_CHILI_URLS: &[(&str, &str, bool)] = &[
    (
        "dna-50m",
        "http://pizzachili.dcc.uchile.cl/texts/dna/dna.50MB.gz",
        true,
    ),
    (
        "proteins-50m",
        "http://pizzachili.dcc.uchile.cl/texts/protein/proteins.50MB.gz",
        true,
    ),
    (
        "english-50m",
        "http://pizzachili.dcc.uchile.cl/texts/nlang/english.50MB.gz",
        true,
    ),
    (
        "xml-50m",
        "http://pizzachili.dcc.uchile.cl/texts/xml/dblp.xml.50MB.gz",
        true,
    ),
    (
        "sources-50m",
        "http://pizzachili.dcc.uchile.cl/texts/code/sources.50MB.gz",
        true,
    ),
    (
        "pitches-50m",
        "http://pizzachili.dcc.uchile.cl/texts/music/pitches.50MB.gz",
        true,
    ),
];

static PATTERN_SCHEMES: &[(&str, Length, Length)] = &[
    ("s-0junk", Fixed(8), Fixed(0)),
    ("s-50junk", Fixed(8), Fixed(4)),
    ("s-100junk", Fixed(8), Fixed(8)),
    ("m-0junk", Fixed(128), Fixed(0)),
    ("m-50junk", Fixed(128), Fixed(64)),
    ("m-100junk", Fixed(128), Fixed(128)),
    /*
        ("l-0junk", Fixed(4096), Fixed(0)),
        ("l-50junk", Fixed(4096), Fixed(2048)),
        ("l-100junk", Fixed(4096), Fixed(4096)),

        ("xl-0junk", Ratio(0.1), Fixed(0)),
        ("xl-50junk", Ratio(0.1), Ratio(0.5)),
        ("xl-100junk", Ratio(0.1), Ratio(1.0)),

        ("2xl-0junk", Ratio(0.33), Fixed(0)),
        ("2xl-50junk", Ratio(0.33), Ratio(0.5)),
        ("2xl-100junk", Ratio(0.33), Ratio(1.0)),

        ("3xl-0junk", Ratio(0.66), Fixed(0)),
        ("3xl-50junk", Ratio(0.66), Ratio(0.5)),
        ("3xl-100junk", Ratio(0.66), Ratio(1.0)),

        ("4xl-0junk", Ratio(0.95), Fixed(0)),
        ("4xl-50junk", Ratio(0.95), Ratio(0.5)),
        ("4xl-100junk", Ratio(0.95), Ratio(1.0)),
    */
];

#[derive(Clone, Copy)]
enum Length {
    Fixed(usize),
    Ratio(f64),
}

impl Length {
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

    for &(name, _, _) in PATTERN_SCHEMES.iter() {
        patterns.push(name);
    }

    // Generate patterns for each sample if not already created.
    for &sname in samples.iter() {
        let sdata = load_data(dir, sname)?;
        for &(pname, plen, jlen) in PATTERN_SCHEMES.iter() {
            let full_name = String::from(sname) + SEPARATOR + pname;
            let fpath = PathBuf::from(dir).join(full_name);
            if fs::metadata(&fpath).is_err() {
                eprintln!("generating pattern {} for {}", pname, sname);
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
