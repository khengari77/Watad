//! Finite facts about the actual shared circle seeds, rather than a copied table.
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus! {

pub proof fn primitive(c: usize, a: usize, b: usize)
    requires c < 5, a < crate::circle_word_spec(c).len(), b < crate::circle_word_spec(c).len(),
    ensures crate::rotation::rotated(crate::circle_word_spec(c),a as int)
        == crate::rotation::rotated(crate::circle_word_spec(c),b as int) ==> a == b,
{
    let word = crate::circle_word_spec(c);
    let x = crate::rotation::rotated(word,a as int);
    let y = crate::rotation::rotated(word,b as int);
    if x == y {
        assert(x[0] == y[0]);
        assert(x[1] == y[1]);
        if word.len() > 2 { assert(x[2] == y[2]); }
        if word.len() > 3 { assert(x[3] == y[3]); }
        if word.len() > 4 { assert(x[4] == y[4]); }
        if word.len() > 5 { assert(x[5] == y[5]); }
        if word.len() > 6 { assert(x[6] == y[6]); }
        if word.len() > 7 { assert(x[7] == y[7]); }
        if word.len() > 8 { assert(x[8] == y[8]); }
        assert(a == b);
    }
}

pub proof fn disjoint(c: usize, d: usize, a: usize, b: usize)
    requires c < 5, d < 5, c != d,
        a < crate::circle_word_spec(c).len(), b < crate::circle_word_spec(d).len(),
    ensures crate::rotation::rotated(crate::circle_word_spec(c),a as int)
        != crate::rotation::rotated(crate::circle_word_spec(d),b as int),
{
    let x = crate::rotation::rotated(crate::circle_word_spec(c),a as int);
    let y = crate::rotation::rotated(crate::circle_word_spec(d),b as int);
    if x == y {
        assert(x.len() == y.len());
        assert(x[0] == y[0]);
        assert(x[1] == y[1]);
        if x.len() > 2 { assert(x[2] == y[2]); }
        if x.len() > 3 { assert(x[3] == y[3]); }
        if x.len() > 4 { assert(x[4] == y[4]); }
        if x.len() > 5 { assert(x[5] == y[5]); }
        if x.len() > 6 { assert(x[6] == y[6]); }
        if x.len() > 7 { assert(x[7] == y[7]); }
        if x.len() > 8 { assert(x[8] == y[8]); }
        assert(false);
    }
}

pub proof fn twenty_two()
    ensures crate::circle_word_spec(0).len() + crate::circle_word_spec(1).len()
        + crate::circle_word_spec(2).len() + crate::circle_word_spec(3).len()
        + crate::circle_word_spec(4).len() == 22,
{}

} // verus!
