use super::EMPTY;

/// Bucket array with bi-directional pointers.
pub struct Bucket {
    bounds: [u32; 257],
    ps: [u32; 256],
    qs: [u32; 256],
}

impl Bucket {
    /// Construct the bucket array.
    #[inline]
    pub fn compute(s: &[u8]) -> Box<Self> {
        let mut bkt = Box::new(Bucket {
            bounds: [0; 257],
            ps: [0; 256],
            qs: [0; 256],
        });

        let mut sum = 1;
        s.iter().for_each(|&ch| bkt.bounds[ch as usize] += 1);
        bkt.bounds.iter_mut().for_each(|acc| {
            let n = *acc;
            *acc = sum;
            sum += n;
        });
        bkt.reset_l_ptrs();
        bkt.reset_s_ptrs();
        bkt
    }

    /// Insert l-type characters in corresponding bucket head.
    #[inline]
    pub fn insert_head(&mut self, s: &[u8], sa: &mut [u32], i: usize) {
        let c = s[i] as usize;
        sa[self.ps[c] as usize] = i as u32;
        self.ps[c] += 1;
    }

    /// Insert s-type characters in corresponding bucket tail.
    #[inline]
    pub fn insert_tail(&mut self, s: &[u8], sa: &mut [u32], i: usize) {
        let c = s[i] as usize;
        self.qs[c] -= 1;
        sa[self.qs[c] as usize] = i as u32;
    }

    /// Clear content of all the bucket tails and reset all the s-pointers.
    #[inline]
    pub fn clear_tails(&mut self, sa: &mut [u32]) {
        for c in 0..=255 {
            let t = self.get_tail_ptr(c);
            let q = self.get_s_ptr(c);
            sa[q..t].iter_mut().for_each(|i| *i = EMPTY);
        }
        self.reset_s_ptrs();
    }

    /// Get the bucket tail.
    #[inline]
    pub fn get_tail_ptr(&self, c: u8) -> usize {
        self.bounds[c as usize + 1] as usize
    }

    /// Get the l-pointer.
    #[inline]
    pub fn get_l_ptr(&self, c: u8) -> usize {
        self.ps[c as usize] as usize
    }

    /// Get the s-pointer.
    #[inline]
    pub fn get_s_ptr(&self, c: u8) -> usize {
        self.qs[c as usize] as usize
    }

    /// Reset all the l-pointers.
    #[inline]
    pub fn reset_l_ptrs(&mut self) {
        self.ps.copy_from_slice(&mut self.bounds[..256]);
    }

    /// Reset all the s-pointers.
    #[inline]
    pub fn reset_s_ptrs(&mut self) {
        self.qs.copy_from_slice(&mut self.bounds[1..257]);
    }
}
