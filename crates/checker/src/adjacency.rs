//! Exclusions between original letters in two foot slots.
use crate::{
    morphology::{Action, Program},
    structured::FootSlot,
};
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus! {
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule { pub left: usize, pub left_letter: usize, pub right: usize, pub right_letter: usize }

pub open spec fn compatible(p: &Program, q: &Program, l: usize, r: usize) -> bool {
    l < p.actions@.len() && r < q.actions@.len()
        && !(p.actions@[l as int] == Action::Delete && q.actions@[r as int] == Action::Delete)
}

pub(crate) fn check_pair(p: &Program, q: &Program, l: usize, r: usize) -> (yes: bool)
    ensures yes == compatible(p,q,l,r),
{
    l < p.actions.len() && r < q.actions.len()
        && !(p.actions[l] == Action::Delete && q.actions[r] == Action::Delete)
}

/// Every referenced slot and original-letter index exists. Conflicting
/// alternatives are allowed here; selected-program compatibility is checked by
/// `valid_rule` and `choice::permits`.
pub open spec fn rule_scope_ok(slots: Seq<FootSlot>, rule: Rule) -> bool { rule.left < slots.len() && rule.right < slots.len()
        && (forall|i: int| 0 <= i < slots[rule.left as int].programs@.len()
            ==> rule.left_letter < slots[rule.left as int].programs@[i].actions@.len())
        && (forall|j: int| 0 <= j < slots[rule.right as int].programs@.len()
            ==> rule.right_letter < slots[rule.right as int].programs@[j].actions@.len())
}

pub fn check_rule_scope_ok(slots: &[FootSlot], rule: Rule) -> (yes: bool)
    ensures yes == rule_scope_ok(slots@,rule),
{
    if rule.left >= slots.len() || rule.right >= slots.len() { return false; }
    let left = &slots[rule.left].programs;
    let right = &slots[rule.right].programs;
    let mut i = 0;
    while i < left.len()
        invariant rule.left < slots.len(), rule.right < slots.len(),
            left@ == slots@[rule.left as int].programs@,
            i <= left.len(),
            forall|a: int| 0 <= a < i ==> rule.left_letter < left@[a].actions@.len(),
        decreases left.len()-i,
    {
        if rule.left_letter >= left[i].actions.len() { return false; }
        i += 1;
    }
    let mut j = 0;
    while j < right.len()
        invariant rule.left < slots.len(), rule.right < slots.len(),
            right@ == slots@[rule.right as int].programs@,
            j <= right.len(),
            forall|b: int| 0 <= b < j ==> rule.right_letter < right@[b].actions@.len(),
        decreases right.len()-j,
    {
        if rule.right_letter >= right[j].actions.len() { return false; }
        j += 1;
    }
    true
}

pub open spec fn valid_rule(slots: Seq<FootSlot>, rule: Rule) -> bool {
    rule_scope_ok(slots,rule)
        && forall|i: int,j: int| 0 <= i < slots[rule.left as int].programs@.len()
            && 0 <= j < slots[rule.right as int].programs@.len() ==>
            compatible(&slots[rule.left as int].programs@[i],&slots[rule.right as int].programs@[j],rule.left_letter,rule.right_letter)
}

fn check_rule(slots: &[FootSlot], rule: Rule) -> (yes: bool)
    ensures yes == valid_rule(slots@,rule),
{
    if !check_rule_scope_ok(slots,rule) { return false; }
    let left = &slots[rule.left].programs;
    let right = &slots[rule.right].programs;
    let mut i = 0;
    while i < left.len()
        invariant rule.left < slots.len(), rule.right < slots.len(),
            left@ == slots@[rule.left as int].programs@, right@ == slots@[rule.right as int].programs@,
            i <= left.len(),
            forall|a: int,b: int| 0 <= a < i && 0 <= b < right.len() ==>
                compatible(&left@[a],&right@[b],rule.left_letter,rule.right_letter),
        decreases left.len()-i,
    {
        let mut j = 0;
        while j < right.len()
            invariant rule.left < slots.len(), rule.right < slots.len(),
                left@ == slots@[rule.left as int].programs@, right@ == slots@[rule.right as int].programs@,
                i < left.len(), j <= right.len(),
                forall|a: int,b: int| 0 <= a < i && 0 <= b < right.len() ==>
                    compatible(&left@[a],&right@[b],rule.left_letter,rule.right_letter),
                forall|b: int| 0 <= b < j ==> compatible(&left@[i as int],&right@[b],rule.left_letter,rule.right_letter),
            decreases right.len()-j,
        {
            if !check_pair(&left[i],&right[j],rule.left_letter,rule.right_letter) { return false; }
            j += 1;
        }
        i += 1;
    }
    true
}

pub open spec fn valid(slots: Seq<FootSlot>, rules: Seq<Rule>) -> bool {
    forall|i: int| 0 <= i < rules.len() ==> valid_rule(slots,rules[i])
}

pub fn check(slots: &[FootSlot], rules: &[Rule]) -> (yes: bool)
    ensures yes == valid(slots@,rules@),
{
    let mut i = 0;
    while i < rules.len()
        invariant i <= rules.len(), forall|j: int| 0 <= j < i ==> valid_rule(slots@,rules@[j]),
        decreases rules.len()-i,
    {
        if !check_rule(slots,rules[i]) { return false; }
        i += 1;
    }
    true
}
} // verus!
