//! Immutable compiled authority. Construction and public beat-input methods
//! preserve the link between structured programs and the recognized language.
use crate::{
    adapter,
    obligations::{self, Requirements},
    poem, structured, verse,
};
use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;
use watad_types::{Beat, FootId, Unit};

verus! {

pub struct Origin { circle: usize, rot: usize, reps: usize, feet: Vec<FootId> }

pub struct Profile {
    source: structured::Profile,
    lowered: verse::Profile,
    requirements: Requirements,
    origin: Option<Origin>,
}

pub open spec fn half_bound(slots: Seq<structured::FootSlot>, feet: Seq<FootId>) -> bool {
    slots.len() <= feet.len()
        && forall|i: int| 0 <= i < slots.len() ==> slots[i].units@ == feet[i].word_spec()
}

pub open spec fn basis_bound(source: &structured::Profile, feet: Seq<FootId>) -> bool {
    forall|i: int| 0 <= i < source.forms@.len() ==>
        half_bound(source.forms@[i].sadr@,feet) && half_bound(source.forms@[i].ajuz@,feet)
}

fn same_base(units: &[Unit], foot: FootId) -> (yes: bool)
    ensures yes == (units@ == foot.word_spec()),
{
    let (pattern,len) = foot.pattern();
    if units.len() != len { return false; }
    let mut i = 0;
    while i < len
        invariant i <= len, len <= 3, units.len() == len,
            pattern@.subrange(0,len as int) == foot.word_spec(),
            forall|j: int| 0 <= j < i ==> units@[j] == pattern@[j],
        decreases len-i,
    {
        assert(foot.word_spec()[i as int] == pattern@[i as int]);
        if units[i] != pattern[i] { return false; }
        i += 1;
    }
    assert(units@ =~= foot.word_spec());
    true
}

fn check_half(slots: &[structured::FootSlot], feet: &[FootId]) -> (yes: bool)
    ensures yes == half_bound(slots@,feet@),
{
    if slots.len() > feet.len() { return false; }
    let mut i = 0;
    while i < slots.len()
        invariant slots.len() <= feet.len(), i <= slots.len(),
            forall|j: int| 0 <= j < i ==> slots@[j].units@ == feet@[j].word_spec(),
        decreases slots.len()-i,
    {
        if !same_base(&slots[i].units,feet[i]) { return false; }
        i += 1;
    }
    true
}

fn check_basis(source: &structured::Profile, feet: &[FootId]) -> (yes: bool)
    ensures yes == basis_bound(source,feet@),
{
    let mut i = 0;
    while i < source.forms.len()
        invariant i <= source.forms.len(), forall|j: int| 0 <= j < i ==>
            half_bound(source.forms@[j].sadr@,feet@) && half_bound(source.forms@[j].ajuz@,feet@),
        decreases source.forms.len()-i,
    {
        if !check_half(&source.forms[i].sadr,feet) || !check_half(&source.forms[i].ajuz,feet) { return false; }
        i += 1;
    }
    true
}

pub open spec fn encoded(beats: Seq<Beat>) -> Seq<u8> {
    Seq::new(beats.len(),|i: int| adapter::beat_code(beats[i]))
}

fn encode(beats: &[Beat]) -> (codes: Vec<u8>)
    ensures codes@ == encoded(beats@),
{
    let codes = adapter::encode_beats(beats);
    assert(codes@ =~= encoded(beats@));
    codes
}

impl Profile {
    #[verifier::type_invariant]
    spec fn invariant(&self) -> bool {
        verse::valid(&self.lowered) && structured::projected(&self.source,&self.lowered)
            && match &self.origin {
                None => true,
                Some(o) => crate::layout::valid(o.circle,o.rot,o.reps,o.feet@) && basis_bound(&self.source,o.feet@),
            }
    }

    pub closed spec fn authority(&self) -> structured::Profile { self.source }
    pub closed spec fn language(&self) -> verse::Profile { self.lowered }
    pub closed spec fn policy(&self) -> Requirements { self.requirements }
    pub closed spec fn circle_bound(&self) -> bool { self.origin.is_some() }
    pub closed spec fn origin_model(&self) -> Option<(usize,usize,usize,Seq<FootId>)> {
        match &self.origin { None => None, Some(o) => Some((o.circle,o.rot,o.reps,o.feet@)) }
    }

    #[allow(clippy::question_mark)]
    pub fn new(source: structured::Profile, requirements: Requirements)
        -> (answer: Result<Self,structured::InvalidProfile>)
        ensures match answer { Err(_) => !structured::valid(&source), Ok(p) => structured::valid(&source) && p.authority() == source && p.policy() == requirements
            && verse::valid(&p.language()) && structured::projected(&source,&p.language()) },
    {
        let lowered = match structured::compile_profile(&source) { Err(e) => return Err(e), Ok(p) => p };
        Ok(Self { source,lowered,requirements,origin:None })
    }

    #[allow(clippy::question_mark)]
    pub fn new_bound(source: structured::Profile, requirements: Requirements,
        circle: usize, rot: usize, reps: usize, feet: Vec<FootId>)
        -> (answer: Result<Self,structured::InvalidProfile>)
        ensures match answer {
            Err(_) => !(structured::valid(&source) && crate::layout::valid(circle,rot,reps,feet@) && basis_bound(&source,feet@)),
            Ok(p) => p.circle_bound() && p.authority() == source && p.policy() == requirements
                && structured::valid(&source) && crate::layout::valid(circle,rot,reps,feet@) && basis_bound(&source,feet@)
                && verse::valid(&p.language()) && structured::projected(&source,&p.language()),
        },
    {
        if !crate::layout::check(circle,rot,reps,&feet) || !check_basis(&source,&feet) { return Err(structured::InvalidProfile); }
        let lowered = match structured::compile_profile(&source) { Err(e) => return Err(e), Ok(p) => p };
        Ok(Self { source,lowered,requirements,origin:Some(Origin { circle,rot,reps,feet }) })
    }

    pub fn is_circle_bound(&self) -> (yes: bool)
        ensures yes == self.circle_bound(),
    { self.origin.is_some() }

    pub fn origin(&self) -> (answer: Option<(usize,usize,usize,&[FootId])>)
        ensures match answer {
            None => self.origin_model().is_none(),
            Some((c,r,n,f)) => self.origin_model() == Some((c,r,n,f@))
                && crate::layout::valid(c,r,n,f@) && basis_bound(&self.authority(),f@),
        },
    {
        proof { use_type_invariant(self); }
        match &self.origin { None => None, Some(o) => Some((o.circle,o.rot,o.reps,o.feet.as_slice())) }
    }

    pub fn source(&self) -> (source: &structured::Profile)
        ensures *source == self.authority(),
    { &self.source }

    pub fn kernel(&self) -> (lowered: &verse::Profile)
        ensures *lowered == self.language(), verse::valid(lowered), structured::projected(&self.authority(),lowered),
    {
        proof { use_type_invariant(self); }
        &self.lowered
    }

    pub fn recognize(&self, scope: verse::Scope, sadr: &[Beat], ajuz: &[Beat])
        -> (answer: Result<Option<verse::Certificate>,verse::InvalidProfile>)
        ensures answer.is_ok(), match answer { Err(_) => false, Ok(result) =>
            result.is_some() == verse::derives(&self.language(),scope,encoded(sadr@),encoded(ajuz@),None)
                && match result { None => true, Some(c) => verse::legal(&self.language(),scope,encoded(sadr@),encoded(ajuz@),&c) } },
    {
        proof { use_type_invariant(self); }
        verse::recognize(&self.lowered,scope,&encode(sadr),&encode(ajuz),None)
    }

    pub fn check(&self, scope: verse::Scope, sadr: &[Beat], ajuz: &[Beat], c: &verse::Certificate) -> (yes: bool)
        ensures yes == verse::legal(&self.language(),scope,encoded(sadr@),encoded(ajuz@),c),
    {
        verse::check(&self.lowered,scope,&encode(sadr),&encode(ajuz),c)
    }

    pub fn recognize_poem(&self, input: &[poem::Input]) -> (answer: Result<Option<Vec<verse::Certificate>>,poem::Error>)
        ensures match answer {
            Err(poem::Error::InvalidProfile) => false,
            Err(poem::Error::EmptyPoem) => input@.len() == 0,
            Ok(result) => input@.len() > 0
                && result.is_some() == obligations::derives_from(&self.language(),input@,&self.policy(),0,None,Seq::empty())
                && match result { None => true, Some(c) => obligations::trace(&self.language(),input@,&self.policy(),0,None,Seq::empty(),c@) },
        },
    {
        proof { use_type_invariant(self); }
        obligations::recognize(&self.lowered,input,&self.requirements)
    }

    pub fn check_poem(&self, input: &[poem::Input], certificates: &[verse::Certificate]) -> (yes: bool)
        ensures yes == (input@.len() > 0 && obligations::trace(&self.language(),input@,&self.policy(),0,None,Seq::empty(),certificates@)),
    {
        proof { use_type_invariant(self); }
        obligations::check(&self.lowered,input,&self.requirements,certificates)
    }
}

} // verus!
