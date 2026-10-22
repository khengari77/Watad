//! Exact poem consistency over complete verse hypotheses. No greedy class choice.
use crate::verse::{self, BindingPolicy, Certificate, DarbClass, Profile, Scope};
use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus! {

pub struct Input<'a> {
    pub context: u16,
    pub sadr: &'a [u8],
    pub ajuz: &'a [u8],
}

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { InvalidProfile, EmptyPoem }

pub open spec fn all_derivable(p: &Profile, input: Seq<Input>, filter: Option<DarbClass>) -> bool {
    forall|i: int| 0 <= i < input.len() ==>
        verse::derives(p, Scope { context: input[i].context, initial: i == 0 }, input[i].sadr@, input[i].ajuz@, filter)
}

pub open spec fn legal(p: &Profile, input: Seq<Input>, certificates: Seq<Certificate>) -> bool {
    &&& verse::valid(p)
    &&& input.len() > 0
    &&& certificates.len() == input.len()
    &&& forall|i: int| 0 <= i < input.len() ==>
        verse::legal(p, Scope { context: input[i].context, initial: i == 0 }, input[i].sadr@, input[i].ajuz@, &certificates[i])
    &&& p.binding == BindingPolicy::Unconstrained || (
        certificates[0].form < p.forms@.len()
        && forall|i: int| #![trigger certificates[i]] 0 <= i < certificates.len() ==>
            certificates[i].form < p.forms@.len()
            && p.forms@[certificates[i].form as int].class ==
               p.forms@[certificates[0].form as int].class)
}

/// Independent consistency relation: one shared declared class, or explicitly
/// unconstrained choices. The first verse is never presumed unambiguous.
pub open spec fn derives(p: &Profile, input: Seq<Input>) -> bool {
    verse::valid(p) && input.len() > 0 && if p.binding == BindingPolicy::Unconstrained {
        all_derivable(p, input, None)
    } else {
        exists|f: int| 0 <= f < p.forms@.len()
            && all_derivable(p, input, Some(p.forms@[f].class))
    }
}

pub fn check(p: &Profile, input: &[Input], certificates: &[Certificate]) -> (yes: bool)
    ensures yes == legal(p, input@, certificates@),
{
    if !verse::validate(p) || input.is_empty() || certificates.len() != input.len() {
        return false;
    }
    if !verse::check(p, Scope { context: input[0].context, initial: true }, input[0].sadr, input[0].ajuz, &certificates[0]) {
        return false;
    }
    let class = p.forms[certificates[0].form].class;
    let mut i = 1;
    while i < input.len()
        invariant
            verse::valid(p), input.len() > 0, certificates.len() == input.len(),
            1 <= i <= input.len(),
            certificates@[0].form < p.forms@.len(),
            class == p.forms@[certificates@[0].form as int].class,
            forall|j: int| 0 <= j < i ==>
                verse::legal(p, Scope { context: input@[j].context, initial: j == 0 }, input@[j].sadr@, input@[j].ajuz@, &certificates@[j]),
            p.binding == BindingPolicy::Unconstrained || forall|j: int| 0 <= j < i ==>
                certificates@[j].form < p.forms@.len()
                && p.forms@[certificates@[j].form as int].class == class,
        decreases input.len() - i,
    {
        if !verse::check(p, Scope { context: input[i].context, initial: i == 0 }, input[i].sadr, input[i].ajuz, &certificates[i]) {
            return false;
        }
        if p.binding == BindingPolicy::FixedDarbClass
            && p.forms[certificates[i].form].class != class { return false; }
        i += 1;
    }
    true
}

fn recognize_class(p: &Profile, input: &[Input], filter: Option<DarbClass>)
    -> (answer: Option<Vec<Certificate>>)
    requires verse::valid(p), input.len() > 0,
    ensures
        answer.is_some() == all_derivable(p, input@, filter),
        match answer {
            None => true,
            Some(c) => c@.len() == input@.len()
                && (forall|i: int| 0 <= i < input@.len() ==>
                    verse::legal(p, Scope { context: input@[i].context, initial: i == 0 }, input@[i].sadr@, input@[i].ajuz@, &c@[i])
                    && verse::class_matches(p.forms@[c@[i].form as int].class, filter)),
        },
{
    let mut certificates = Vec::new();
    let mut i = 0;
    while i < input.len()
        invariant
            verse::valid(p), input.len() > 0, i <= input.len(), certificates.len() == i,
            forall|j: int| 0 <= j < i ==>
                verse::legal(p, Scope { context: input@[j].context, initial: j == 0 }, input@[j].sadr@, input@[j].ajuz@, &certificates@[j])
                && verse::class_matches(p.forms@[certificates@[j].form as int].class, filter),
            forall|j: int| 0 <= j < i ==>
                verse::derives(p, Scope { context: input@[j].context, initial: j == 0 }, input@[j].sadr@, input@[j].ajuz@, filter),
        decreases input.len() - i,
    {
        match verse::recognize(p, Scope { context: input[i].context, initial: i == 0 }, input[i].sadr, input[i].ajuz, filter) {
            Ok(Some(c)) => certificates.push(c),
            _ => return None,
        }
        i += 1;
    }
    Some(certificates)
}

/// Complete class search. Empty input and malformed profiles are errors; only a
/// proved exhausted finite search is reported as `Ok(None)`.
pub fn recognize(p: &Profile, input: &[Input])
    -> (answer: Result<Option<Vec<Certificate>>, Error>)
    ensures match answer {
        Err(Error::InvalidProfile) => !verse::valid(p),
        Err(Error::EmptyPoem) => verse::valid(p) && input@.len() == 0,
        Ok(result) => verse::valid(p) && input@.len() > 0
            && result.is_some() == derives(p, input@)
            && match result { None => true, Some(c) => legal(p, input@, c@) },
    },
{
    if !verse::validate(p) { return Err(Error::InvalidProfile); }
    if input.is_empty() { return Err(Error::EmptyPoem); }
    if p.binding == BindingPolicy::Unconstrained {
        return Ok(recognize_class(p, input, None));
    }
    let mut f = 0;
    while f < p.forms.len()
        invariant
            verse::valid(p), input.len() > 0,
            p.binding == BindingPolicy::FixedDarbClass, f <= p.forms.len(),
            forall|j: int| 0 <= j < f ==> !all_derivable(p, input@, Some(p.forms@[j].class)),
        decreases p.forms.len() - f,
    {
        let class = p.forms[f].class;
        if let Some(certificates) = recognize_class(p, input, Some(class)) {
            assert(exists|j: int| 0 <= j < p.forms@.len()
                && all_derivable(p, input@, Some(p.forms@[j].class)));
            return Ok(Some(certificates));
        }
        f += 1;
    }
    Ok(None)
}

} // verus!
