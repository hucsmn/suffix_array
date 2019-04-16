use rand::random;

macro_rules! assert_sais_equals_naive {
    ($s:expr) => {{
        let s = &($s)[..];
        let mut sa0 = vec![0; s.len() + 1];
        let mut sa1 = vec![0; s.len() + 1];
        super::sais(s, 256, &mut sa0[..]);
        super::naive_sort(s, &mut sa1[..]);
        assert_eq!(sa0, sa1);
    }};
}

#[test]
fn sais_basic_correctness() {
    assert_sais_equals_naive!(b"");
    assert_sais_equals_naive!(b"\0");
    assert_sais_equals_naive!(b"\xff");
    assert_sais_equals_naive!(b"xxxxxxxx");
    assert_sais_equals_naive!(b"xxxxoooo");
    assert_sais_equals_naive!(b"baaccaaccaba");
    assert_sais_equals_naive!(b"mmississiippii");
}

#[test]
fn sais_random_samples() {
    const MIN_LEN: usize = 500;
    const MAX_LEN: usize = 1000;
    const SAMPLES: usize = 2000;

    let mut array = vec![0; MAX_LEN];
    for _ in 0..SAMPLES {
        let mut scale = 2;
        if random::<bool>() {
            // small alphabet
            scale += random::<u8>() % 4;
        } else {
            // large alphabet
            scale += 128 + random::<u8>() % 32;
        }

        // generate random data of random length
        let n = random::<usize>() % (MAX_LEN - MIN_LEN) + MIN_LEN;
        for i in 0..n {
            array[i] = random::<u8>() % scale;
        }

        assert_sais_equals_naive!(&array[..n]);
    }
}
