//! Total bounded circle operations, shared by native Rust and Verus.
use crate::{Sabab, Unit};
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus! {

pub open spec fn index(n: int, start: int, i: int) -> int {
    if i < n - start { start + i } else { i - (n - start) }
}

pub open spec fn rotated(word: Seq<Unit>, start: int) -> Seq<Unit> {
    Seq::new(word.len(), |i: int| word[index(word.len() as int, start, i)])
}

pub fn rotate<const CAP: usize>(word: &[Unit], start: usize)
    -> (answer: Option<([Unit; CAP], usize)>)
    ensures answer.is_some() == (word.len() > 0 && start < word.len() && word.len() <= CAP),
        match answer { None => true, Some((a,n)) => n == word.len()
            && a@.subrange(0,n as int) == rotated(word@,start as int) },
{
    if word.is_empty() || start >= word.len() || word.len() > CAP { return None; }
    let mut out = [Unit::Sabab(Sabab::Khafif); CAP];
    let mut i = 0;
    while i < word.len()
        invariant start < word.len(), word.len() <= CAP, i <= word.len(),
            forall|j: int| 0 <= j < i ==> out@[j] == word@[index(word.len() as int,start as int,j)],
        decreases word.len() - i,
    {
        let at = if i < word.len() - start { start + i } else { i - (word.len() - start) };
        out[i] = word[at];
        i += 1;
    }
    assert(out@.subrange(0,word.len() as int) =~= rotated(word@,start as int));
    Some((out,word.len()))
}

/// Every rotated position has a unique original position, and vice versa.
pub proof fn permutation(n: int, start: int, i: int, j: int)
    requires n > 0, 0 <= start < n, 0 <= i < n, 0 <= j < n,
    ensures 0 <= index(n,start,i) < n,
        (index(n,start,i) == index(n,start,j)) == (i == j),
        index(n, if start == 0 { 0 } else { n-start }, index(n,start,i)) == i,
{}

pub proof fn identity(word: Seq<Unit>)
    ensures rotated(word,0) == word,
{
    assert(rotated(word,0) =~= word);
}

/// Rotation composition is addition in the finite cyclic group. The sum is
/// normalized in mathematical integers, so no machine overflow is assumed.
pub proof fn composition(word: Seq<Unit>, a: int, b: int)
    requires word.len() > 0, 0 <= a < word.len(), 0 <= b < word.len(),
    ensures rotated(rotated(word,a),b) == rotated(word,if a+b < word.len() { a+b } else { a+b-word.len() }),
{
    let n = word.len() as int;
    let sum = if a+b < n { a+b } else { a+b-n };
    assert forall|i: int| 0 <= i < n implies
        rotated(rotated(word,a),b)[i] == rotated(word,sum)[i] by {
        assert(0 <= index(n,b,i) < n);
        assert(index(n,a,index(n,b,i)) == index(n,sum,i));
    }
    assert(rotated(rotated(word,a),b) =~= rotated(word,sum));
}

pub open spec fn repeated(word: Seq<Unit>, reps: nat) -> Seq<Unit> {
    if word.len() == 0 || reps == 0 { Seq::empty() }
    else { Seq::new(word.len()*reps, |i: int| word[i % word.len() as int]) }
}

#[allow(clippy::question_mark)] // Keep the checked-overflow branch explicit for Verus.
pub fn repeat<const CAP: usize>(word: &[Unit], reps: usize)
    -> (answer: Option<([Unit; CAP], usize)>)
    ensures answer.is_some() == (word.len() * reps <= CAP),
        match answer { None => true, Some((a,n)) => n == word.len()*reps
            && a@.subrange(0,n as int) == repeated(word@,reps as nat) },
{
    let mut out = [Unit::Sabab(Sabab::Khafif); CAP];
    let n = match word.len().checked_mul(reps) { None => return None, Some(n) => n };
    if n > CAP { return None; }
    if word.is_empty() {
        assert(word@.len() == 0);
        assert(word.len() == 0);
        assert(word.len()*reps == 0) by(nonlinear_arith)
            requires word.len() == 0;
        assert(n == 0);
        assert(repeated(word@,reps as nat).len() == 0);
        assert(out@.subrange(0,0) =~= repeated(word@,reps as nat));
        return Some((out,n));
    }
    let mut i = 0;
    while i < n
        invariant n == word.len()*reps, n <= CAP, word.len() > 0, i <= n,
            forall|j: int| 0 <= j < i ==> out@[j] == word@[j % word.len() as int],
        decreases n - i,
    {
        out[i] = word[i % word.len()];
        i += 1;
    }
    assert(out@.subrange(0,n as int) =~= repeated(word@,reps as nat));
    Some((out,n))
}

} // verus!
