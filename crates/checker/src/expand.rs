//! Finite expansion of independent slot alternatives into complete branches.
//!
//! A half-profile is a finite sequence of slots, each carrying an ordered list of
//! compiled programs. Expansion selects exactly one program index per slot and
//! keeps only assignments satisfying every declared adjacency rule. The relation
//! below is independent of the executable Cartesian product: production may
//! filter early or late, but the accepted index tuples are the same.
use crate::{
    adjacency::{self, Rule},
    structured::FootSlot,
};
use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus! {

/// A partial assignment has one in-range source index per already-decided slot.
pub open spec fn partial(slots: Seq<FootSlot>, choice: Seq<usize>) -> bool {
    choice.len() <= slots.len()
        && forall|i: int| 0 <= i < choice.len()
            ==> (choice[i] as int) < slots[i].programs@.len()
}

/// One declared rule is satisfied by a complete (or sufficiently long) choice.
pub open spec fn rule_satisfied(slots: Seq<FootSlot>, rule: Rule, choice: Seq<usize>) -> bool {
    rule.left < slots.len() && rule.right < slots.len()
        && rule.left < choice.len() && rule.right < choice.len()
        && (choice[rule.left as int] as int) < slots[rule.left as int].programs@.len()
        && (choice[rule.right as int] as int) < slots[rule.right as int].programs@.len()
        && adjacency::compatible(
            &slots[rule.left as int].programs@[choice[rule.left as int] as int],
            &slots[rule.right as int].programs@[choice[rule.right as int] as int],
            rule.left_letter,
            rule.right_letter,
        )
}

/// The independent expansion relation: one alternative per slot, all rules met.
pub open spec fn complete(slots: Seq<FootSlot>, rules: Seq<Rule>, choice: Seq<usize>) -> bool {
    choice.len() == slots.len()
        && partial(slots, choice)
        && forall|r: int| 0 <= r < rules.len() ==> rule_satisfied(slots, rules[r], choice)
}


/// The independent lexicographic order on complete source-index tuples: compare
/// at the first slot where the two alternatives differ.
pub open spec fn extends_seq(a: Seq<usize>, prefix: Seq<usize>) -> bool {
    a.len() >= prefix.len()
        && forall|i: int| 0 <= i < prefix.len() ==> a[i] == prefix[i]
}

pub open spec fn same_prefix(a: Seq<usize>, b: Seq<usize>, k: int) -> bool {
    forall|i: int| 0 <= i < k ==> a[i] == b[i]
}

pub open spec fn lex_less(a: Seq<usize>, b: Seq<usize>) -> bool {
    a.len() == b.len()
        && exists|k: int| 0 <= k < a.len()
            && same_prefix(a, b, k)
            && a[k] < b[k]
}

/// Strict lexicographic order of a whole output list.
pub open spec fn ordered_vecs(xs: Seq<Vec<usize>>) -> bool {
    forall|i: int, j: int| 0 <= i < j < xs.len() ==> lex_less(xs[i]@, xs[j]@)
}

/// No two output positions carry the same source-index tuple.
pub open spec fn distinct_vecs(xs: Seq<Vec<usize>>) -> bool {
    forall|i: int, j: int| 0 <= i < j < xs.len() ==> xs[i]@ != xs[j]@
}

proof fn lex_less_neq(a: Seq<usize>, b: Seq<usize>)
    requires lex_less(a, b),
    ensures a != b,
{
    let k = choose|k: int| 0 <= k < a.len() && same_prefix(a, b, k) && a[k] < b[k];
    if a == b {
        assert(a[k] == b[k]);
    }
}

proof fn ordered_implies_distinct(xs: Seq<Vec<usize>>)
    requires ordered_vecs(xs),
    ensures distinct_vecs(xs),
{
    assert forall|i: int, j: int| 0 <= i < j < xs.len() implies xs[i]@ != xs[j]@ by {
        lex_less_neq(xs[i]@, xs[j]@);
    }
}

proof fn ordered_append(before: Seq<Vec<usize>>, after: Seq<Vec<usize>>)
    requires ordered_vecs(before), ordered_vecs(after),
        forall|i: int, j: int| 0 <= i < before.len() && 0 <= j < after.len()
            ==> lex_less(before[i]@, after[j]@),
    ensures ordered_vecs(before + after),
{
    assert forall|i: int, j: int| 0 <= i < j < (before + after).len()
        implies lex_less((before + after)[i]@, (before + after)[j]@) by {
        if j < before.len() {
            assert((before + after)[i]@ == before[i]@);
            assert((before + after)[j]@ == before[j]@);
        } else if i >= before.len() {
            assert((before + after)[i]@ == after[i - before.len()]@);
            assert((before + after)[j]@ == after[j - before.len()]@);
        } else {
            assert((before + after)[i]@ == before[i]@);
            assert((before + after)[j]@ == after[j - before.len()]@);
        }
    }
}

proof fn distinct_append(before: Seq<Vec<usize>>, after: Seq<Vec<usize>>)
    requires distinct_vecs(before), distinct_vecs(after),
        forall|i: int, j: int| 0 <= i < before.len() && 0 <= j < after.len()
            ==> before[i]@ != after[j]@,
    ensures distinct_vecs(before + after),
{
    assert forall|i: int, j: int| 0 <= i < j < (before + after).len()
        implies (before + after)[i]@ != (before + after)[j]@ by {
        if j < before.len() {
            assert((before + after)[i]@ == before[i]@);
            assert((before + after)[j]@ == before[j]@);
        } else if i >= before.len() {
            assert((before + after)[i]@ == after[i - before.len()]@);
            assert((before + after)[j]@ == after[j - before.len()]@);
        } else {
            assert((before + after)[i]@ == before[i]@);
            assert((before + after)[j]@ == after[j - before.len()]@);
        }
    }
}


/// A tuple extending `prefix` at slot `k` with a smaller alternative precedes
/// one extending it with the current loop alternative.
proof fn lex_less_across_prefix(
    a: Seq<usize>, b: Seq<usize>, prefix: Seq<usize>, k: int, c: usize)
    requires 0 <= k < a.len(), a.len() == b.len(),
        prefix.len() == k, extends_seq(a, prefix), extends_seq(b, prefix),
        (a[k] as int) < c as int, b[k] == c,
    ensures lex_less(a, b),
{
    assert(same_prefix(a, b, k)) by {
        assert forall|i: int| 0 <= i < k implies a[i] == b[i] by {
            assert(a[i] == prefix[i]);
            assert(b[i] == prefix[i]);
        }
    }
    assert((a[k] as int) < (b[k] as int));
}

fn check_complete(slots: &[FootSlot], rules: &[Rule], choice: &[usize]) -> (yes: bool)
    ensures yes == complete(slots@, rules@, choice@),
{
    if choice.len() != slots.len() { return false; }
    let mut i = 0;
    while i < slots.len()
        invariant i <= slots.len(), choice@.len() == slots@.len(),
            forall|j: int| 0 <= j < i ==> (choice@[j] as int) < slots@[j].programs@.len(),
        decreases slots.len() - i,
    {
        if choice[i] >= slots[i].programs.len() { return false; }
        i += 1;
    }
    let mut r = 0;
    while r < rules.len()
        invariant r <= rules.len(),
            choice@.len() == slots@.len(),
            forall|j: int| 0 <= j < slots@.len()
                ==> (choice@[j] as int) < slots@[j].programs@.len(),
            forall|q: int| 0 <= q < r ==> rule_satisfied(slots@, rules@[q], choice@),
        decreases rules.len() - r,
    {
        let rule = rules[r];
        if rule.left >= slots.len() || rule.right >= slots.len() { return false; }
        if !adjacency::check_pair(
            &slots[rule.left].programs[choice[rule.left]],
            &slots[rule.right].programs[choice[rule.right]],
            rule.left_letter,
            rule.right_letter,
        ) {
            return false;
        }
        assert(rule_satisfied(slots@, rule, choice@));
        r += 1;
    }
    true
}

fn copy_prefix(prefix: &[usize]) -> (out: Vec<usize>)
    ensures out@ == prefix@, out.len() == prefix.len(),
{
    let mut out = Vec::new();
    let mut i = 0;
    while i < prefix.len()
        invariant i <= prefix.len(), out@ == prefix@.subrange(0, i as int),
        decreases prefix.len() - i,
    {
        out.push(prefix[i]);
        i += 1;
    }
    out
}

proof fn subrange_push(choice: Seq<usize>, k: int)
    requires 0 <= k < choice.len(),
    ensures choice.subrange(0, k).push(choice[k]) == choice.subrange(0, k + 1),
{
    assert(choice.subrange(0, k).push(choice[k]) =~= choice.subrange(0, k + 1));
}

/// Enumerate all complete extensions of a partial prefix. Source order is the
/// slot order, so the result is lexicographic by source alternative index.
fn enumerate(slots: &[FootSlot], rules: &[Rule], prefix: Vec<usize>)
    -> (out: Vec<Vec<usize>>)
    requires prefix.len() <= slots.len(),
    ensures
        forall|i: int| 0 <= i < out@.len() ==> complete(slots@, rules@, out@[i]@),
        forall|choice: Seq<usize>| complete(slots@, rules@, choice)
            && choice.subrange(0, prefix@.len() as int) == prefix@
            ==> exists|i: int| 0 <= i < out@.len() && out@[i]@ == choice,
        ordered_vecs(out@), distinct_vecs(out@),
        forall|i: int| 0 <= i < out@.len() ==> extends_seq(out@[i]@, prefix@),
    decreases slots.len() - prefix.len(),
{
    let k: usize = prefix.len();
    if k > slots.len() {
        return Vec::new();
    }
    if k == slots.len() {
        let mut out = Vec::new();
        if check_complete(slots, rules, &prefix) {
            out.push(copy_prefix(&prefix));
            assert(out@[0]@ == prefix@);
            assert(extends_seq(out@[0]@, prefix@));
        }
        assert(ordered_vecs(out@));
        assert(distinct_vecs(out@));
        assert forall|i: int| 0 <= i < out@.len() implies extends_seq(out@[i]@, prefix@) by {
            assert(out@[i]@ == prefix@);
        }
        return out;
    }
    let mut out: Vec<Vec<usize>> = Vec::new();
    let mut c = 0;
    while c < slots[k].programs.len()
        invariant c <= slots@[k as int].programs@.len(), k < slots.len(),
            slots.len() == slots@.len(), prefix.len() == k,
            prefix@.len() == k as int,
            forall|j: int| 0 <= j < out@.len() ==> complete(slots@, rules@, out@[j]@),
            forall|choice: Seq<usize>| complete(slots@, rules@, choice)
                && choice.subrange(0, k as int) == prefix@
                && (choice[k as int] as int) < c
                ==> exists|j: int| 0 <= j < out@.len() && out@[j]@ == choice,
            ordered_vecs(out@), distinct_vecs(out@),
            forall|j: int| 0 <= j < out@.len() ==> extends_seq(out@[j]@, prefix@),
            forall|j: int| 0 <= j < out@.len()
                ==> (out@[j]@[k as int] as int) < c,
        decreases slots@[k as int].programs@.len() - c,
    {
        let mut next = copy_prefix(&prefix);
        next.push(c);
        assert(next@ == prefix@.push(c as usize));
        assert(next@.len() == prefix@.len() + 1);
        assert(next.len() == prefix.len() + 1);
        assert(next.len() == k + 1);
        assert(prefix.len() < slots.len());
        assert(prefix.len() + 1 <= slots.len());
        assert(next.len() <= slots.len());
        let mut tail = enumerate(slots, rules, next);
        let ghost before = out@;
        let ghost tail_view = tail@;
        out.append(&mut tail);
        assert(out@ =~= before + tail_view);
        assert forall|i: int, j: int| 0 <= i < before.len() && 0 <= j < tail_view.len()
            implies lex_less(before[i]@, tail_view[j]@) by {
            assert(before[i]@.len() == slots@.len());
            assert(tail_view[j]@.len() == slots@.len());
            assert(tail_view[j]@[k as int] == c);
            lex_less_across_prefix(before[i]@, tail_view[j]@, prefix@, k as int, c);
        }
        proof {
            ordered_append(before, tail_view);
        }
        assert forall|i: int, j: int| 0 <= i < before.len() && 0 <= j < tail_view.len()
            implies before[i]@ != tail_view[j]@ by {
            assert(lex_less(before[i]@, tail_view[j]@));
            lex_less_neq(before[i]@, tail_view[j]@);
        }
        proof {
            distinct_append(before, tail_view);
        }
        assert(ordered_vecs(out@));
        assert(distinct_vecs(out@));
        assert forall|j: int| 0 <= j < out@.len() implies extends_seq(out@[j]@, prefix@) by {
            if j < before.len() {
                assert(out@[j]@ == before[j]@);
            } else {
                assert(out@[j]@ == tail_view[j - before.len()]@);
                assert(extends_seq(tail_view[j - before.len()]@, next@));
                assert forall|i: int| 0 <= i < prefix@.len()
                    implies tail_view[j - before.len()]@[i] == prefix@[i] by {
                    assert(tail_view[j - before.len()]@[i] == next@[i]);
                    assert(next@[i] == prefix@[i]);
                }
            }
        }
        assert forall|j: int| 0 <= j < out@.len()
            implies (out@[j]@[k as int] as int) < c + 1 by {
            if j < before.len() {
                assert(out@[j]@ == before[j]@);
                assert((out@[j]@[k as int] as int) < c);
            } else {
                assert(out@[j]@ == tail_view[j - before.len()]@);
                assert(extends_seq(tail_view[j - before.len()]@, next@));
                assert(tail_view[j - before.len()]@[k as int] == c);
            }
        }
        assert forall|choice: Seq<usize>| complete(slots@, rules@, choice)
            && choice.subrange(0, k as int) == prefix@
            && (choice[k as int] as int) < c + 1
            implies exists|j: int| 0 <= j < out@.len() && out@[j]@ == choice by {
            if (choice[k as int] as int) < c {
                assert(exists|j: int| 0 <= j < before.len() && before[j]@ == choice);
                let j = choose|j: int| 0 <= j < before.len() && before[j]@ == choice;
                assert(0 <= j < out@.len());
                assert(out@[j]@ == choice);
            } else {
                assert(choice[k as int] == c as usize);
                assert(choice.subrange(0, k + 1) == next@) by {
                    subrange_push(choice, k as int);
                    assert(choice.subrange(0, k as int) == prefix@);
                    assert(choice.subrange(0, k as int).push(choice[k as int]) == next@);
                }
                assert(next@.len() == k + 1);
                assert(next.len() == k + 1);
                assert(choice.subrange(0, next@.len() as int) == next@);
                assert(exists|j: int| 0 <= j < tail_view.len() && tail_view[j]@ == choice);
                let j = choose|j: int| 0 <= j < tail_view.len() && tail_view[j]@ == choice;
                assert(0 <= before.len() + j < out@.len());
                assert(out@[before.len() + j]@ == choice);
            }
        }
        c += 1;
    }
    assert forall|choice: Seq<usize>| complete(slots@, rules@, choice)
        && choice.subrange(0, k as int) == prefix@
        implies exists|j: int| 0 <= j < out@.len() && out@[j]@ == choice by {
        assert((choice[k as int] as int) < slots@[k as int].programs@.len());
    }
    out
}

/// All complete index tuples for a half, in lexicographic source order.
/// Repeated source programs remain distinct because their indices are distinct.
pub fn assignments(slots: &[FootSlot], rules: &[Rule]) -> (out: Vec<Vec<usize>>)
    ensures
        forall|i: int| 0 <= i < out@.len() ==> complete(slots@, rules@, out@[i]@),
        forall|choice: Seq<usize>| complete(slots@, rules@, choice)
            ==> exists|i: int| 0 <= i < out@.len() && out@[i]@ == choice
                && (forall|j: int| 0 <= j < out@.len() && out@[j]@ == choice ==> j == i),
        ordered_vecs(out@), distinct_vecs(out@),
{
    let prefix: Vec<usize> = Vec::new();
    assert(prefix.len() == 0);
    assert(prefix@ == Seq::<usize>::empty());
    let out = enumerate(slots, rules, prefix);
    assert forall|choice: Seq<usize>| complete(slots@, rules@, choice)
        implies choice.subrange(0, 0) == Seq::<usize>::empty() by {
        assert(choice.subrange(0, 0) == Seq::<usize>::empty());
    }
    assert forall|choice: Seq<usize>| complete(slots@, rules@, choice)
        implies exists|i: int| 0 <= i < out@.len() && out@[i]@ == choice
            && (forall|j: int| 0 <= j < out@.len() && out@[j]@ == choice ==> j == i) by {
        assert(exists|i: int| 0 <= i < out@.len() && out@[i]@ == choice);
        let i = choose|i: int| 0 <= i < out@.len() && out@[i]@ == choice;
        assert forall|j: int| 0 <= j < out@.len() && out@[j]@ == choice implies j == i by {
            if j != i {
                if j < i {
                    assert(out@[j]@ != out@[i]@);
                } else {
                    assert(out@[i]@ != out@[j]@);
                }
                assert(false);
            }
        }
    }
    out
}

} // verus!
