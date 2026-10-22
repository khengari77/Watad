//! Check the untrusted circle-to-feet derivation against the shared seed words.
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;
use watad_types::{FootId, Unit};

verus! {

pub open spec fn expected(word: Seq<Unit>, rot: usize, reps: usize) -> Seq<Unit> {
    watad_types::rotation::repeated(watad_types::rotation::rotated(word,rot as int),reps as nat)
}

pub open spec fn prefix(input: Seq<Unit>, at: int, word: Seq<Unit>) -> bool {
    0 <= at <= input.len() && at+word.len() <= input.len()
        && forall|j: int| 0 <= j < word.len() ==> word[j] == input[at+j]
}

pub open spec fn tiling(input: Seq<Unit>, feet: Seq<FootId>, i: int, at: int) -> bool
    decreases feet.len()-i,
{
    if i < 0 || i > feet.len() || at < 0 || at > input.len() { false }
    else if i == feet.len() { at == input.len() }
    else { prefix(input,at,feet[i].word_spec())
        && tiling(input,feet,i+1,at+feet[i].word_spec().len()) }
}

fn matches(word: &[Unit], rot: usize, _reps: usize, total: usize, at: usize, foot: FootId) -> (yes: bool)
    requires word.len() > 0, rot < word.len(), _reps > 0, total == word.len()*_reps,
    ensures yes == prefix(expected(word@,rot,_reps),at as int,foot.word_spec()),
{
    let (pattern,len) = foot.pattern();
    assert(foot.word_spec().len() == len);
    assert(expected(word@,rot,_reps).len() == total);
    if at > total || len > total-at { return false; }
    let mut j = 0;
    while j < len
        invariant word.len() > 0, rot < word.len(), _reps > 0, total == word.len()*_reps,
            at <= total, len <= total-at, j <= len, len <= 3,
            foot.word_spec().len() == len, expected(word@,rot,_reps).len() == total,
            pattern@.subrange(0,len as int) == foot.word_spec(),
            forall|k: int| 0 <= k < j ==> pattern@[k] == expected(word@,rot,_reps)[at as int+k],
        decreases len-j,
    {
        let phase = (at+j) % word.len();
        let index = if phase < word.len()-rot { rot+phase } else { phase-(word.len()-rot) };
        assert(expected(word@,rot,_reps)[at as int+j as int] == word@[index as int]);
        assert(foot.word_spec()[j as int] == pattern@[j as int]);
        if pattern[j] != word[index] { return false; }
        j += 1;
    }
    assert forall|k: int| 0 <= k < foot.word_spec().len() implies
        foot.word_spec()[k] == expected(word@,rot,_reps)[at as int+k] by {
        assert(foot.word_spec()[k] == pattern@[k]);
    }
    true
}

fn check_from(word: &[Unit], rot: usize, reps: usize, total: usize, feet: &[FootId], i: usize, at: usize) -> (yes: bool)
    requires word.len() > 0, rot < word.len(), reps > 0, total == word.len()*reps, i <= feet.len(),
    ensures yes == tiling(expected(word@,rot,reps),feet@,i as int,at as int),
    decreases feet.len()-i,
{
    if at > total { return false; }
    if i == feet.len() { return at == total; }
    let foot = feet[i];
    if !matches(word,rot,reps,total,at,foot) { return false; }
    let (_,len) = foot.pattern();
    check_from(word,rot,reps,total,feet,i+1,at+len)
}

pub open spec fn valid(circle: usize, rot: usize, reps: usize, feet: Seq<FootId>) -> bool {
    circle < 5 && rot < watad_types::circle_word_spec(circle).len() && reps > 0
        && watad_types::circle_word_spec(circle).len()*reps <= usize::MAX
        && tiling(expected(watad_types::circle_word_spec(circle),rot,reps),feet,0,0)
}

#[allow(clippy::question_mark)]
pub fn check(circle: usize, rot: usize, reps: usize, feet: &[FootId]) -> (yes: bool)
    ensures yes == valid(circle,rot,reps,feet@),
{
    if circle >= 5 || reps == 0 { return false; }
    let word = watad_types::circle_word(circle);
    if rot >= word.len() { return false; }
    let total = match word.len().checked_mul(reps) { None => return false, Some(n) => n };
    check_from(word,rot,reps,total,feet,0,0)
}

} // verus!
