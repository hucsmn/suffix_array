//! O(n) time and O(1) space SAIS algorithm for read-only byte string,
//! as described in the recursion level 0 of [Ge Nong. 2013.
//! Practical linear-time O(1)-workspace suffix sorting for constant
//! alphabets.](https://dl.acm.org/citation.cfm?doid=2493175.2493180).

mod bucket;

#[cfg(test)]
mod tests;

use self::bucket::Bucket;
use super::sacak1::sacak1;
use super::utils::*;

// The empty symbol in workspace.
const EMPTY: u32 = 0xffffffff;

/// Maximum length of the input string.
pub const MAX_LENGTH: usize = 0xfffffffc;

/// O(n) time and O(1) space SAIS algorithm for read-only byte string.
pub fn sacak0(s: &[u8], sa: &mut [u32]) {
    debug_assert!(s.len() <= MAX_LENGTH);
    debug_assert!(s.len() + 1 == sa.len());
    if s.len() == 0 {
        sa[0] = 0;
        return;
    }

    // allocate about 3k memory for bucket array to speed up recursion level 0
    let mut bkt = Bucket::compute(s);

    sort_lms_suffixes(s, sa, bkt.as_mut());
    induce_by_lms(s, sa, bkt.as_mut());
}

fn sort_lms_suffixes(s: &[u8], sa: &mut [u32], bkt: &mut Bucket) {
    // 1. place lms characters
    sa.iter_mut().for_each(|p| *p = EMPTY);
    for_each_lms(s, false, |i, _| bkt.insert_tail(s, sa, i));
    sa[0] = s.len() as u32;

    // 2. sort lms substrings by lms characters
    induce_by_lms(s, sa, bkt);

    // 3. collect the sorted lms substrings into the tail of workspace
    let mut h = sa.len();
    for c in (0..=255).rev() {
        let t = bkt.get_tail_ptr(c);
        let q = bkt.get_s_ptr(c);
        for i in (q..t).rev() {
            let j = sa[i] as usize;
            if j > 0 && s[j - 1] > s[j] {
                h -= 1;
                sa[h] = sa[i];
            }
        }
    }
    h -= 1;
    sa[h] = sa[0];

    // 4. get the sorted lms suffixes from sorted lms substrings
    let (head, tail) = sa.split_at_mut(h);
    let n = tail.len();
    suffixes_from_substrs(s, head, tail, sacak1);
    sa[n..].iter_mut().for_each(|i| *i = EMPTY);

    // 5. place sorted lms suffixes to the bucket respectively
    bkt.reset_l_ptrs();
    bkt.reset_s_ptrs();
    for i in (1..n).rev() {
        let j = sa[i] as usize;
        sa[i] = EMPTY;
        bkt.insert_tail(s, sa, j);
    }
}

fn induce_by_lms(s: &[u8], sa: &mut [u32], bkt: &mut Bucket) {
    // 1. induce l-type from sorted lms characters/suffixes
    for i in 0..sa.len() {
        if sa[i] == EMPTY || sa[i] == 0 {
            continue;
        }

        // j == s.len() || s[sa[i]-1] >= s[sa[i]] <=> sa[i]-1 is l-type,
        // becuase the workspace contains only l-type/lms characters
        let j = sa[i] as usize;
        if j == s.len() || s[j - 1] >= s[j] {
            bkt.insert_head(s, sa, j - 1);
        }
    }

    // 2. clear lms characters, except the sentinel
    bkt.clear_tails(sa);

    // 3. symmetrically induce s-type characters by l-type characters
    for i in (1..sa.len()).rev() {
        if sa[i] == EMPTY || sa[i] == 0 {
            continue;
        }

        // if s[sa[i]-1] < s[sa[i]]: sa[i]-1 is s-type.
        // if s[sa[i]-1] == s[sa[i]]:
        //     sa[i]-1 is located in the same bucket of c = s[sa[i]];
        //     all the l-type characters are already placed before l-pointer;
        //     thus, l-pointer(c) < i <=> sa[i]-1 is s-type.
        //     (l-pointer(c) won't be equal to i)
        let j = sa[i] as usize;
        if s[j - 1] < s[j] || (s[j - 1] == s[j] && bkt.get_l_ptr(s[j]) < i) {
            bkt.insert_tail(s, sa, j - 1);
        }
    }
}
