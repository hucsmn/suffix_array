use std::ops::{Index, IndexMut};

/// Type bitmap of input string.
#[derive(Debug, Clone)]
pub struct Types {
    len: usize,
    map: Vec<u32>,
}

impl Types {
    pub fn new(len: usize, t: bool) -> Self {
        let chunks = if len % 32 == 0 {
            len / 32
        } else {
            len / 32 + 1
        };
        let map = vec![if t { std::u32::MAX } else { 0 }; chunks];
        Types { len, map }
    }

    pub fn calculate<T>(s: &[T]) -> Self
    where
        T: Ord,
    {
        let mut types = Types::new(s.len(), false);

        for i in (0..s.len() - 1).rev() {
            use std::cmp::Ordering::*;
            match Ord::cmp(&s[i], &s[i + 1]) {
                Less => types.set(i, true),
                Greater => (),
                Equal => types.set(i, types[i + 1]),
            }
        }
        types
    }

    fn set(&mut self, i: usize, t: bool) {
        debug_assert!(i < self.len);
        if t {
            self.map[i / 32] |= 1 << (i % 32);
        } else {
            self.map[i / 32] &= !(1 << (i % 32));
        }
    }

    pub fn is_lms(&self, i: usize) -> bool {
        debug_assert!(i <= self.len);
        if i == self.len {
            true
        } else if i == 0 {
            false
        } else {
            self[i] && !self[i - 1]
        }
    }
}

impl Index<usize> for Types {
    type Output = bool;
    fn index(&self, i: usize) -> &bool {
        debug_assert!(i < self.len);
        let chunk = self.map[i / 32];
        if chunk & (1 << (i % 32)) != 0 {
            &true
        } else {
            &false
        }
    }
}

/// Non-overlapping span of suffix array with double direction pointers.
#[derive(Debug, Copy, Clone)]
pub struct Span {
    pub head: u32,
    pub tail: u32,
    pub i: u32,
    pub j: u32,
}

impl Span {
    pub fn new() -> Span {
        Span {
            head: 0,
            tail: 0,
            i: 0,
            j: 0,
        }
    }

    pub fn reset(&mut self) {
        self.reset_front();
        self.reset_back();
    }

    pub fn reset_front(&mut self) {
        self.i = self.head;
    }

    pub fn reset_back(&mut self) {
        self.j = self.tail;
    }

    pub fn push_front(&mut self, sa: &mut [u32], n: u32) {
        sa[self.i as usize] = n;
        self.i += 1;
    }

    pub fn push_back(&mut self, sa: &mut [u32], n: u32) {
        self.j -= 1;
        sa[self.j as usize] = n;
    }
}

/// Bucket of spans.
#[derive(Debug)]
pub struct Bucket(Vec<Span>);

impl Bucket {
    pub fn calculate<T>(s: &[T], scale: usize) -> Self
    where
        T: Copy + Into<u32>,
    {
        let mut bkt = vec![Span::new(); scale];

        for &c in s.iter() {
            bkt[to_usize(c)].tail += 1;
        }

        let mut offset = 1;
        for sp in bkt.iter_mut() {
            sp.head += offset;
            offset += sp.tail;
            sp.tail += sp.head;
            sp.reset();
        }
        Bucket(bkt)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<usize> for Bucket {
    type Output = Span;
    fn index(&self, i: usize) -> &Span {
        &self.0[i]
    }
}

impl IndexMut<usize> for Bucket {
    fn index_mut(&mut self, i: usize) -> &mut Span {
        &mut self.0[i]
    }
}

/// Convert integers to u32.
pub fn to_usize<T: Copy + Into<u32>>(x: T) -> usize {
    x.into() as usize
}
