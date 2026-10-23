//! Generic, versioned finite meter profiles, terminal branches and certificates.
//! The core has no meter-name or named-operation dispatch.

use crate::choice;
use crate::language::{Slot, Step};
use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus! {

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileId { pub family: u16, pub revision: u16 }

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DarbClass(pub u16);

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingPolicy { FixedDarbClass, Unconstrained }

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scope { pub context: u16, pub initial: bool }

/// One permitted layout/terminal pairing. Context is a profile-defined label.
/// Multiple branches may have the same surface language and different classes.
pub struct Form {
    pub context: u16,
    pub initial_only: bool,
    pub class: DarbClass,
    pub sadr: Vec<Slot>,
    pub ajuz: Vec<Slot>,
    pub sadr_rules: Vec<choice::Rule>,
    pub ajuz_rules: Vec<choice::Rule>,
}

pub struct Profile {
    pub id: ProfileId,
    pub binding: BindingPolicy,
    pub forms: Vec<Form>,
}

/// Plain claims. The caller supplies the expected profile and context separately.
pub struct Certificate {
    pub profile: ProfileId,
    pub context: u16,
    pub form: usize,
    pub sadr: Vec<Step>,
    pub ajuz: Vec<Step>,
}

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidProfile;

pub open spec fn binary(word: Seq<u8>) -> bool {
    word.len() > 0 && forall|i: int| 0 <= i < word.len() ==> word[i] <= 1
}

fn check_word(word: &[u8]) -> (yes: bool)
    ensures yes == binary(word@),
{
    if word.is_empty() { return false; }
    let mut i = 0;
    while i < word.len()
        invariant word.len() > 0, i <= word.len(),
            forall|j: int| 0 <= j < i ==> word@[j] <= 1,
        decreases word.len() - i,
    {
        if word[i] > 1 { return false; }
        i += 1;
    }
    true
}

pub open spec fn alternatives_valid(words: Seq<Vec<u8>>) -> bool {
    words.len() > 0 && forall|i: int| 0 <= i < words.len() ==> binary(words[i]@)
}

fn check_alternatives(words: &[Vec<u8>]) -> (yes: bool)
    ensures yes == alternatives_valid(words@),
{
    if words.is_empty() { return false; }
    let mut i = 0;
    while i < words.len()
        invariant words.len() > 0, i <= words.len(),
            forall|j: int| 0 <= j < i ==> binary(words@[j]@),
        decreases words.len() - i,
    {
        if !check_word(&words[i]) { return false; }
        i += 1;
    }
    true
}

pub open spec fn slots_valid(slots: Seq<Slot>) -> bool {
    slots.len() > 0 && forall|i: int| 0 <= i < slots.len()
        ==> alternatives_valid(slots[i].words@)
}

fn check_slots(slots: &[Slot]) -> (yes: bool)
    ensures yes == slots_valid(slots@),
{
    if slots.is_empty() { return false; }
    let mut i = 0;
    while i < slots.len()
        invariant slots.len() > 0, i <= slots.len(),
            forall|j: int| 0 <= j < i ==> alternatives_valid(slots@[j].words@),
        decreases slots.len() - i,
    {
        if !check_alternatives(&slots[i].words) { return false; }
        i += 1;
    }
    true
}

pub open spec fn valid(p: &Profile) -> bool {
    p.id.family != 0 && p.id.revision != 0 && p.forms@.len() > 0
        && forall|i: int| 0 <= i < p.forms@.len()
            ==> slots_valid(p.forms@[i].sadr@)
                && (p.forms@[i].ajuz@.len() == 0 || slots_valid(p.forms@[i].ajuz@))
                && choice::valid(p.forms@[i].sadr@, p.forms@[i].sadr_rules@)
                && choice::valid(p.forms@[i].ajuz@, p.forms@[i].ajuz_rules@)
}

/// Schema validation is mathematical; historical authority is an external fact.
pub fn validate(p: &Profile) -> (yes: bool)
    ensures yes == valid(p),
{
    if p.id.family == 0 || p.id.revision == 0 || p.forms.is_empty() { return false; }
    let mut i = 0;
    while i < p.forms.len()
        invariant p.id.family != 0, p.id.revision != 0,
            p.forms.len() > 0, i <= p.forms.len(),
            forall|j: int| 0 <= j < i ==>
                slots_valid(p.forms@[j].sadr@)
                    && (p.forms@[j].ajuz@.len() == 0 || slots_valid(p.forms@[j].ajuz@))
                    && choice::valid(p.forms@[j].sadr@, p.forms@[j].sadr_rules@)
                    && choice::valid(p.forms@[j].ajuz@, p.forms@[j].ajuz_rules@),
        decreases p.forms.len() - i,
    {
        if !check_slots(&p.forms[i].sadr)
            || (!p.forms[i].ajuz.is_empty() && !check_slots(&p.forms[i].ajuz))
            || !choice::validate(&p.forms[i].sadr, &p.forms[i].sadr_rules)
            || !choice::validate(&p.forms[i].ajuz, &p.forms[i].ajuz_rules) { return false; }
        i += 1;
    }
    true
}

pub open spec fn legal(p: &Profile, scope: Scope, sadr: Seq<u8>, ajuz: Seq<u8>,
                      c: &Certificate) -> bool {
    valid(p) && c.profile == p.id && c.context == scope.context
        && c.form < p.forms@.len() && p.forms@[c.form as int].context == scope.context
        && (!p.forms@[c.form as int].initial_only || scope.initial)
        && choice::trace(p.forms@[c.form as int].sadr@, p.forms@[c.form as int].sadr_rules@, sadr, c.sadr@)
        && choice::trace(p.forms@[c.form as int].ajuz@, p.forms@[c.form as int].ajuz_rules@, ajuz, c.ajuz@)
}

pub fn check(p: &Profile, scope: Scope, sadr: &[u8], ajuz: &[u8], c: &Certificate)
    -> (yes: bool)
    ensures yes == legal(p, scope, sadr@, ajuz@, c),
{
    if !validate(p) || c.profile != p.id || c.context != scope.context || c.form >= p.forms.len() {
        return false;
    }
    let f = &p.forms[c.form];
    f.context == scope.context && (!f.initial_only || scope.initial)
        && choice::check(&f.sadr, &f.sadr_rules, sadr, &c.sadr)
        && choice::check(&f.ajuz, &f.ajuz_rules, ajuz, &c.ajuz)
}

pub open spec fn class_matches(class: DarbClass, filter: Option<DarbClass>) -> bool {
    match filter { None => true, Some(wanted) => class == wanted }
}

fn matches_class(class: DarbClass, filter: Option<DarbClass>) -> (yes: bool)
    ensures yes == class_matches(class, filter),
{
    match filter { None => true, Some(wanted) => class == wanted }
}

pub open spec fn derives(p: &Profile, scope: Scope, sadr: Seq<u8>, ajuz: Seq<u8>,
                        filter: Option<DarbClass>) -> bool {
    valid(p) && exists|i: int| 0 <= i < p.forms@.len()
        && p.forms@[i].context == scope.context && (!p.forms@[i].initial_only || scope.initial) && class_matches(p.forms@[i].class, filter)
        && choice::derives(p.forms@[i].sadr@, p.forms@[i].sadr_rules@, sadr)
        && choice::derives(p.forms@[i].ajuz@, p.forms@[i].ajuz_rules@, ajuz)
}

pub open spec fn form_derives(p: &Profile, scope: Scope, sadr: Seq<u8>, ajuz: Seq<u8>, f: int) -> bool {
    valid(p) && 0 <= f < p.forms@.len()
        && p.forms@[f].context == scope.context && (!p.forms@[f].initial_only || scope.initial)
        && choice::derives(p.forms@[f].sadr@, p.forms@[f].sadr_rules@, sadr)
        && choice::derives(p.forms@[f].ajuz@, p.forms@[f].ajuz_rules@, ajuz)
}

#[allow(clippy::question_mark)] // Verus checks each explicit early-return branch.
pub fn recognize_form(p: &Profile, scope: Scope, sadr: &[u8], ajuz: &[u8], f: usize)
    -> (answer: Option<Certificate>)
    ensures answer.is_some() == form_derives(p, scope, sadr@, ajuz@, f as int),
        match answer { None => true, Some(c) => c.form == f && legal(p, scope, sadr@, ajuz@, &c) },
{
    if !validate(p) || f >= p.forms.len() { return None; }
    let form = &p.forms[f];
    if form.context != scope.context || (form.initial_only && !scope.initial) { return None; }
    let s = match choice::recognize(&form.sadr, &form.sadr_rules, sadr) { None => return None, Some(s) => s };
    let a = match choice::recognize(&form.ajuz, &form.ajuz_rules, ajuz) { None => return None, Some(a) => a };
    Some(Certificate { profile: p.id, context: scope.context, form: f, sadr: s, ajuz: a })
}

pub proof fn certificate_derives(p: &Profile, scope: Scope, sadr: Seq<u8>, ajuz: Seq<u8>, c: &Certificate)
    requires legal(p, scope, sadr, ajuz, c),
    ensures form_derives(p, scope, sadr, ajuz, c.form as int),
{
    choice::trace_is_derivation(p.forms@[c.form as int].sadr@, p.forms@[c.form as int].sadr_rules@, sadr, c.sadr@);
    choice::trace_is_derivation(p.forms@[c.form as int].ajuz@, p.forms@[c.form as int].ajuz_rules@, ajuz, c.ajuz@);
}

/// Exact recognition, optionally constrained by a poem's selected binding class.
/// An invalid profile is an error, never a negative recognition result.
#[allow(clippy::collapsible_if)] // Keep recursive witness branches explicit for the proof.
pub fn recognize(p: &Profile, scope: Scope, sadr: &[u8], ajuz: &[u8],
                 filter: Option<DarbClass>) -> (answer: Result<Option<Certificate>, InvalidProfile>)
    ensures match answer {
        Err(_) => !valid(p),
        Ok(result) => valid(p) && result.is_some() == derives(p, scope, sadr@, ajuz@, filter)
            && match result {
                None => true,
                Some(c) => legal(p, scope, sadr@, ajuz@, &c)
                    && class_matches(p.forms@[c.form as int].class, filter),
            },
    },
{
    if !validate(p) { return Err(InvalidProfile); }
    let mut i = 0;
    while i < p.forms.len()
        invariant valid(p), i <= p.forms.len(),
            forall|j: int| 0 <= j < i ==> !(
                p.forms@[j].context == scope.context && (!p.forms@[j].initial_only || scope.initial) && class_matches(p.forms@[j].class, filter)
                && choice::derives(p.forms@[j].sadr@, p.forms@[j].sadr_rules@, sadr@)
                && choice::derives(p.forms@[j].ajuz@, p.forms@[j].ajuz_rules@, ajuz@)),
        decreases p.forms.len() - i,
    {
        let f = &p.forms[i];
        if f.context == scope.context && (!f.initial_only || scope.initial) && matches_class(f.class, filter) {
            if let Some(s) = choice::recognize(&f.sadr, &f.sadr_rules, sadr) {
                if let Some(a) = choice::recognize(&f.ajuz, &f.ajuz_rules, ajuz) {
                    let c = Certificate { profile: p.id, context: scope.context, form: i, sadr: s, ajuz: a };
                    assert(exists|j: int| 0 <= j < p.forms@.len()
                        && p.forms@[j].context == scope.context && (!p.forms@[j].initial_only || scope.initial) && class_matches(p.forms@[j].class, filter)
                        && choice::derives(p.forms@[j].sadr@, p.forms@[j].sadr_rules@, sadr@)
                        && choice::derives(p.forms@[j].ajuz@, p.forms@[j].ajuz_rules@, ajuz@));
                    return Ok(Some(c));
                }
            }
        }
        i += 1;
    }
    Ok(None)
}

} // verus!
