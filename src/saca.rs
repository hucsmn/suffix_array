use std::slice::from_raw_parts_mut;

use cdivsufsort::sort_in_place as dss;

/// Maximum length of the input string.
pub const MAX_LENGTH: usize = std::i32::MAX as usize;

/// Wrapper of the underlying suffix array construction algorithm.
pub fn saca(s: &[u8], sa: &mut [u32]) {
    assert!(s.len() <= MAX_LENGTH);
    assert_eq!(s.len() + 1, sa.len());

    sa[0] = s.len() as u32;
    dss(s, as_signed_integer_slice(&mut sa[1..]));
}

fn as_signed_integer_slice(sa: &mut [u32]) -> &mut [i32] {
    unsafe {
        let len = sa.len();
        let data = sa.as_mut_ptr() as *mut i32;
        from_raw_parts_mut(data, len)
    }
}
