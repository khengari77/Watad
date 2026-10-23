//! Exact concatenation with exclusions on the selected original programs.
use crate::language::{self, Slot, Step};
use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus! {

/// Each flag records deletion of the rule's original letter by that alternative.
pub struct Rule {
    pub left: usize, pub right: usize,
    pub left_deleted: Vec<bool>, pub right_deleted: Vec<bool>,
}

pub open spec fn valid(slots: Seq<Slot>, rules: Seq<Rule>) -> bool {
    forall|i: int| 0 <= i < rules.len() ==> (
        rules[i].left < slots.len() && rules[i].right < slots.len()
        && rules[i].left_deleted@.len() == slots[rules[i].left as int].words@.len()
        && rules[i].right_deleted@.len() == slots[rules[i].right as int].words@.len())
}

pub fn validate(slots: &[Slot], rules: &[Rule]) -> (yes: bool)
    ensures yes == valid(slots@,rules@),
{
    let mut i = 0;
    while i < rules.len()
        invariant i <= rules.len(), forall|j: int| 0 <= j < i ==> (
            rules@[j].left < slots.len() && rules@[j].right < slots.len()
            && rules@[j].left_deleted@.len() == slots@[rules@[j].left as int].words@.len()
            && rules@[j].right_deleted@.len() == slots@[rules@[j].right as int].words@.len()),
        decreases rules.len()-i,
    {
        let r = &rules[i];
        if r.left >= slots.len() || r.right >= slots.len()
            || r.left_deleted.len() != slots[r.left].words.len()
            || r.right_deleted.len() != slots[r.right].words.len() { return false; }
        i += 1;
    }
    true
}

pub open spec fn permits_rule(rule: &Rule, steps: Seq<Step>) -> bool {
    rule.left < steps.len() && rule.right < steps.len()
        && steps[rule.left as int].choice < rule.left_deleted@.len()
        && steps[rule.right as int].choice < rule.right_deleted@.len()
        && !(rule.left_deleted@[steps[rule.left as int].choice as int]
            && rule.right_deleted@[steps[rule.right as int].choice as int])
}

pub open spec fn permits(rules: Seq<Rule>, steps: Seq<Step>) -> bool {
    forall|i: int| 0 <= i < rules.len() ==> permits_rule(&rules[i],steps)
}

pub fn check_rules(rules: &[Rule], steps: &[Step]) -> (yes: bool)
    ensures yes == permits(rules@,steps@),
{
    let mut i = 0;
    while i < rules.len()
        invariant i <= rules.len(), forall|j: int| 0 <= j < i ==> permits_rule(&rules@[j],steps@),
        decreases rules.len()-i,
    {
        let r = &rules[i];
        if r.left >= steps.len() || r.right >= steps.len() { return false; }
        let l = steps[r.left].choice;
        let a = steps[r.right].choice;
        if l >= r.left_deleted.len() || a >= r.right_deleted.len()
            || (r.left_deleted[l] && r.right_deleted[a]) { return false; }
        i += 1;
    }
    true
}

pub open spec fn trace(slots: Seq<Slot>, rules: Seq<Rule>, input: Seq<u8>, steps: Seq<Step>) -> bool {
    language::trace(slots,input,0,0,steps) && permits(rules,steps)
}

pub fn check(slots: &[Slot], rules: &[Rule], input: &[u8], steps: &[Step]) -> (yes: bool)
    ensures yes == trace(slots@,rules@,input@,steps@),
{
    language::check(slots,input,steps) && check_rules(rules,steps)
}

/// Independent finite derivation relation; constraints apply to the whole
/// selected program sequence, including alternatives with equal beat surfaces.
pub open spec fn derives_from(slots: Seq<Slot>, rules: Seq<Rule>, input: Seq<u8>,
    i: int, at: int, selected: Seq<Step>) -> bool
    decreases slots.len()-i,
{
    if i < 0 || i > slots.len() || at < 0 || at > input.len() || selected.len() != i { false }
    else if i == slots.len() { at == input.len() && permits(rules,selected) }
    else { exists|c: int| 0 <= c < slots[i].words@.len()
        && language::prefix(input,at,#[trigger] slots[i].words@[c]@)
        && derives_from(slots,rules,input,i+1,at+slots[i].words@[c]@.len(),
            selected.push(Step { choice:c as usize,start:at as usize,end:(at+slots[i].words@[c]@.len()) as usize })) }
}

pub open spec fn derives(slots: Seq<Slot>, rules: Seq<Rule>, input: Seq<u8>) -> bool {
    derives_from(slots,rules,input,0,0,Seq::empty())
}

proof fn trace_from_derives(slots: Seq<Slot>, rules: Seq<Rule>, input: Seq<u8>,
    i: int, at: int, selected: Seq<Step>, steps: Seq<Step>)
    requires selected.len() == i, language::trace(slots,input,i,at,steps), permits(rules,selected+steps),
    ensures derives_from(slots,rules,input,i,at,selected),
    decreases slots.len()-i,
{
    if i < slots.len() {
        let step = steps[0];
        let tail = steps.subrange(1,steps.len() as int);
        assert(selected.push(step)+tail =~= selected+steps);
        trace_from_derives(slots,rules,input,i+1,step.end as int,selected.push(step),tail);
        assert(exists|c: int| 0 <= c < slots[i].words@.len()
            && language::prefix(input,at,#[trigger] slots[i].words@[c]@)
            && derives_from(slots,rules,input,i+1,at+slots[i].words@[c]@.len(),
                selected.push(Step { choice:c as usize,start:at as usize,end:(at+slots[i].words@[c]@.len()) as usize })));
    }
}

pub proof fn trace_is_derivation(slots: Seq<Slot>, rules: Seq<Rule>, input: Seq<u8>, steps: Seq<Step>)
    requires trace(slots,rules,input,steps),
    ensures derives(slots,rules,input),
{
    assert(Seq::<Step>::empty()+steps =~= steps);
    trace_from_derives(slots,rules,input,0,0,Seq::empty(),steps);
}

fn extend(selected: &[Step], step: Step) -> (out: Vec<Step>)
    ensures out@ == selected@.push(step),
{
    let mut out = Vec::new();
    let mut i = 0;
    while i < selected.len()
        invariant i <= selected.len(), out@ == selected@.subrange(0,i as int),
        decreases selected.len()-i,
    { out.push(selected[i]); i += 1; }
    assert(out@ =~= selected@);
    out.push(step);
    out
}

fn recognize_from(slots: &[Slot], rules: &[Rule], input: &[u8], i: usize, at: usize, selected: &[Step])
    -> (answer: Option<Vec<Step>>)
    requires i <= slots.len(), at <= input.len(), selected.len() == i,
    ensures answer.is_some() == derives_from(slots@,rules@,input@,i as int,at as int,selected@),
        match answer { None => true, Some(steps) => language::trace(slots@,input@,i as int,at as int,steps@)
            && permits(rules@,selected@+steps@) },
    decreases slots.len()-i,
{
    if i == slots.len() {
        return if at == input.len() && check_rules(rules,selected) { Some(Vec::new()) } else { None };
    }
    let words = &slots[i].words;
    let mut c = 0;
    while c < words.len()
        invariant i < slots.len(), at <= input.len(), selected.len() == i,
            words@ == slots@[i as int].words@, c <= words.len(),
            forall|k: int| 0 <= k < c ==> !(
                language::prefix(input@,at as int,#[trigger] words@[k]@)
                && derives_from(slots@,rules@,input@,i+1,at+words@[k]@.len(),
                    selected@.push(Step { choice:k as usize,start:at,end:(at+words@[k]@.len()) as usize }))),
        decreases words.len()-c,
    {
        let word = &words[c];
        if language::word_matches(input,at,word) {
            let end = at+word.len();
            let step = Step { choice:c,start:at,end };
            let next = extend(selected,step);
            if let Some(mut rest) = recognize_from(slots,rules,input,i+1,end,&next) {
                let ghost previous = rest@;
                rest.insert(0,step);
                proof {
                    previous.insert_ensures(0,step);
                    assert(rest@.subrange(1,rest@.len() as int) =~= previous);
                    assert(selected@+rest@ =~= next@+previous);
                    trace_from_derives(slots@,rules@,input@,i as int,at as int,selected@,rest@);
                }
                return Some(rest);
            }
        }
        c += 1;
    }
    None
}

pub fn recognize(slots: &[Slot], rules: &[Rule], input: &[u8]) -> (answer: Option<Vec<Step>>)
    ensures answer.is_some() == derives(slots@,rules@,input@),
        match answer { None => true, Some(steps) => trace(slots@,rules@,input@,steps@) },
{
    recognize_from(slots,rules,input,0,0,&Vec::new())
}

} // verus!
