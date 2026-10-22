//! Exact backtracking for profile-declared poem-wide existence requirements.
//! A requirement names forms of which at least one must occur somewhere.
use crate::{
    poem::Input,
    verse::{self, BindingPolicy, Certificate, DarbClass, Profile, Scope},
};
use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus! {

pub struct Requirements { pub groups: Vec<Vec<usize>> }

pub open spec fn member(xs: Seq<usize>, x: usize) -> bool {
    exists|i: int| 0 <= i < xs.len() && xs[i] == x
}

fn contains(xs: &[usize], x: usize) -> (yes: bool)
    ensures yes == member(xs@, x),
{
    let mut i = 0;
    while i < xs.len()
        invariant i <= xs.len(), forall|j: int| 0 <= j < i ==> xs@[j] != x,
        decreases xs.len() - i,
    {
        if xs[i] == x { return true; }
        i += 1;
    }
    false
}

pub open spec fn group_seen(group: Seq<usize>, selected: Seq<usize>) -> bool {
    exists|i: int| 0 <= i < group.len() && member(selected, group[i])
}

fn seen(group: &[usize], selected: &[usize]) -> (yes: bool)
    ensures yes == group_seen(group@, selected@),
{
    let mut i = 0;
    while i < group.len()
        invariant i <= group.len(), forall|j: int| 0 <= j < i ==> !member(selected@, group@[j]),
        decreases group.len() - i,
    {
        if contains(selected, group[i]) { return true; }
        i += 1;
    }
    false
}

pub open spec fn satisfied(r: &Requirements, selected: Seq<usize>) -> bool {
    forall|g: int| 0 <= g < r.groups@.len() ==> group_seen(r.groups@[g]@, selected)
}

pub fn check_requirements(r: &Requirements, selected: &[usize]) -> (yes: bool)
    ensures yes == satisfied(r, selected@),
{
    let mut i = 0;
    while i < r.groups.len()
        invariant i <= r.groups.len(), forall|g: int| 0 <= g < i ==> group_seen(r.groups@[g]@, selected@),
        decreases r.groups.len() - i,
    {
        if !seen(&r.groups[i], selected) { return false; }
        i += 1;
    }
    true
}

pub open spec fn next_filter(p: &Profile, filter: Option<DarbClass>, f: int) -> Option<DarbClass> {
    if p.binding == BindingPolicy::FixedDarbClass && filter.is_none() { Some(p.forms@[f].class) } else { filter }
}

fn advance_filter(p: &Profile, filter: Option<DarbClass>, f: usize) -> (next: Option<DarbClass>)
    requires f < p.forms.len(),
    ensures next == next_filter(p, filter, f as int),
{
    if p.binding == BindingPolicy::FixedDarbClass && filter.is_none() { Some(p.forms[f].class) } else { filter }
}

fn class_matches(class: DarbClass, filter: Option<DarbClass>) -> (yes: bool)
    ensures yes == verse::class_matches(class, filter),
{
    match filter { None => true, Some(c) => class == c }
}

/// Independent finite assignment relation, including the entire prefix history.
pub open spec fn derives_from(p: &Profile, input: Seq<Input>, r: &Requirements,
    at: int, filter: Option<DarbClass>, selected: Seq<usize>) -> bool
    decreases input.len() - at,
{
    if at < 0 || at > input.len() { false }
    else if at == input.len() { satisfied(r, selected) }
    else { exists|f: int| 0 <= f < p.forms@.len()
        && verse::class_matches(p.forms@[f].class, filter)
        && verse::form_derives(p, Scope { context: input[at].context, initial: at == 0 }, input[at].sadr@, input[at].ajuz@, f)
        && derives_from(p, input, r, at + 1, next_filter(p, filter, f), selected.push(f as usize)) }
}

pub open spec fn trace(p: &Profile, input: Seq<Input>, r: &Requirements,
    at: int, filter: Option<DarbClass>, selected: Seq<usize>, certificates: Seq<Certificate>) -> bool
    decreases input.len() - at,
{
    if at < 0 || at > input.len() { false }
    else if at == input.len() { certificates.len() == 0 && satisfied(r, selected) }
    else { certificates.len() > 0 && certificates[0].form < p.forms@.len()
        && verse::class_matches(p.forms@[certificates[0].form as int].class, filter)
        && verse::legal(p, Scope { context: input[at].context, initial: at == 0 }, input[at].sadr@, input[at].ajuz@, &certificates[0])
        && trace(p, input, r, at + 1, next_filter(p, filter, certificates[0].form as int),
            selected.push(certificates[0].form), certificates.drop_first()) }
}

fn check_from(p: &Profile, input: &[Input], r: &Requirements, at: usize,
    filter: Option<DarbClass>, selected: Vec<usize>, certificates: &[Certificate]) -> (yes: bool)
    requires verse::valid(p), at <= input.len(),
    ensures yes == trace(p, input@, r, at as int, filter, selected@, certificates@),
    decreases input.len() - at,
{
    if at == input.len() { return certificates.is_empty() && check_requirements(r, &selected); }
    if certificates.is_empty() { return false; }
    let c = &certificates[0];
    if !verse::check(p, Scope { context: input[at].context, initial: at == 0 }, input[at].sadr, input[at].ajuz, c) { return false; }
    if !class_matches(p.forms[c.form].class, filter) { return false; }
    let mut next = selected.clone();
    next.push(c.form);
    check_from(p, input, r, at + 1, advance_filter(p, filter, c.form), next, &certificates[1..])
}

pub fn check(p: &Profile, input: &[Input], r: &Requirements, certificates: &[Certificate]) -> (yes: bool)
    ensures yes == (verse::valid(p) && input@.len() > 0
        && trace(p, input@, r, 0, None, Seq::empty(), certificates@)),
{
    if !verse::validate(p) || input.is_empty() { return false; }
    check_from(p, input, r, 0, None, Vec::new(), certificates)
}

pub proof fn trace_is_derivation(p: &Profile, input: Seq<Input>, r: &Requirements,
    at: int, filter: Option<DarbClass>, selected: Seq<usize>, certificates: Seq<Certificate>)
    requires trace(p, input, r, at, filter, selected, certificates),
    ensures derives_from(p, input, r, at, filter, selected),
    decreases input.len() - at,
{
    if at < input.len() {
        let c = certificates[0];
        verse::certificate_derives(p, Scope { context: input[at].context, initial: at == 0 }, input[at].sadr@, input[at].ajuz@, &c);
        trace_is_derivation(p, input, r, at + 1, next_filter(p, filter, c.form as int), selected.push(c.form), certificates.drop_first());
    }
}

#[allow(clippy::collapsible_if)]
fn search(p: &Profile, input: &[Input], r: &Requirements,
    at: usize, filter: Option<DarbClass>, selected: Vec<usize>) -> (answer: Option<Vec<Certificate>>)
    requires verse::valid(p), at <= input.len(),
    ensures answer.is_some() == derives_from(p, input@, r, at as int, filter, selected@),
        match answer { None => true, Some(c) => trace(p, input@, r, at as int, filter, selected@, c@) },
    decreases input.len() - at,
{
    if at == input.len() {
        return if check_requirements(r, &selected) { Some(Vec::new()) } else { None };
    }
    let mut f = 0;
    while f < p.forms.len()
        invariant verse::valid(p), at < input.len(), f <= p.forms.len(),
            forall|j: int| 0 <= j < f ==> !(
                verse::class_matches(p.forms@[j].class, filter)
                && verse::form_derives(p, Scope { context: input@[at as int].context, initial: at == 0 }, input@[at as int].sadr@, input@[at as int].ajuz@, j)
                && derives_from(p, input@, r, at as int + 1, next_filter(p, filter, j), selected@.push(j as usize))),
        decreases p.forms.len() - f,
    {
        if class_matches(p.forms[f].class, filter) {
            if let Some(c) = verse::recognize_form(p, Scope { context: input[at].context, initial: at == 0 }, input[at].sadr, input[at].ajuz, f) {
                let mut next = selected.clone();
                next.push(f);
                let next_class = advance_filter(p, filter, f);
                if let Some(mut tail) = search(p, input, r, at + 1, next_class, next) {
                    let ghost previous = tail@;
                    let ghost head = c;
                    tail.insert(0, c);
                    proof {
                        previous.insert_ensures(0, head);
                        assert(tail@.drop_first() =~= previous);
                    }
                    assert(exists|j: int| 0 <= j < p.forms@.len()
                        && verse::class_matches(p.forms@[j].class, filter)
                        && verse::form_derives(p, Scope { context: input@[at as int].context, initial: at == 0 }, input@[at as int].sadr@, input@[at as int].ajuz@, j)
                        && derives_from(p, input@, r, at as int + 1, next_filter(p, filter, j), selected@.push(j as usize)));
                    return Some(tail);
                }
            }
        }
        f += 1;
    }
    None
}

pub fn recognize(p: &Profile, input: &[Input], r: &Requirements)
    -> (answer: Result<Option<Vec<Certificate>>, crate::poem::Error>)
    ensures match answer {
        Err(crate::poem::Error::InvalidProfile) => !verse::valid(p),
        Err(crate::poem::Error::EmptyPoem) => verse::valid(p) && input@.len() == 0,
        Ok(result) => verse::valid(p) && input@.len() > 0
            && result.is_some() == derives_from(p, input@, r, 0, None, Seq::empty())
            && match result { None => true, Some(c) => trace(p, input@, r, 0, None, Seq::empty(), c@) },
    },
{
    if !verse::validate(p) { return Err(crate::poem::Error::InvalidProfile); }
    if input.is_empty() { return Err(crate::poem::Error::EmptyPoem); }
    Ok(search(p, input, r, 0, None, Vec::new()))
}

} // verus!
