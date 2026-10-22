//! Lower typed bases and original-coordinate programs without dropping choices.
use crate::adjacency;
use crate::verse::{self, BindingPolicy, DarbClass, ProfileId};
use crate::{
    language::Slot,
    morphology::{self as m, InvalidProgram, Position, Program},
};
use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;
use watad_types::Unit;

verus! {

#[derive(Debug)]
pub struct FootSlot { pub units: Vec<Unit>, pub programs: Vec<Program> }

pub struct Form {
    pub context: u16, pub initial_only: bool, pub class: DarbClass,
    pub sadr: Vec<FootSlot>, pub ajuz: Vec<FootSlot>,
    pub sadr_constraints: Vec<adjacency::Rule>, pub ajuz_constraints: Vec<adjacency::Rule>,
}

pub struct Profile { pub id: ProfileId, pub binding: BindingPolicy, pub forms: Vec<Form> }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidProfile;

pub open spec fn watads(units: Seq<Unit>, n: int) -> nat
{
    watad_types::watad_count(units,n)
}

pub fn one_watad(units: &[Unit]) -> (yes: bool)
    ensures yes == (watads(units@, units.len() as int) == 1),
{
    watad_types::has_exactly_one_watad(units)
}

pub open spec fn compilable(s: &FootSlot, position: Position) -> bool {
    &&& watads(s.units@, s.units@.len() as int) == 1
    &&& s.programs@.len() > 0
    &&& forall|i: int| 0 <= i < s.programs@.len() ==> m::valid(s.units@, &s.programs@[i], position)
}

pub open spec fn output(s: &FootSlot, i: int) -> Seq<u8> {
    let p = s.programs@[i];
    m::changed(m::expanded(s.units@, s.units@.len() as int), p.actions@, p.actions@.len() as int) + p.suffix@
}

/// Each output index preserves the corresponding source program, including
/// distinct programs with equal surfaces. Invalid programs reject the slot.
pub fn compile(s: &FootSlot, position: Position) -> (answer: Result<Slot, InvalidProgram>)
    ensures match answer {
        Err(_) => !compilable(s, position),
        Ok(slot) => compilable(s, position)
            && slot.words@.len() == s.programs@.len()
            && forall|i: int| 0 <= i < slot.words@.len() ==> slot.words@[i]@ == output(s, i),
    },
{
    if !one_watad(&s.units) || s.programs.is_empty() { return Err(InvalidProgram); }
    let mut words: Vec<Vec<u8>> = Vec::new();
    let mut i = 0;
    while i < s.programs.len()
        invariant i <= s.programs.len(), s.programs.len() > 0,
            watads(s.units@, s.units@.len() as int) == 1,
            words@.len() == i,
            forall|j: int| 0 <= j < i ==> m::valid(s.units@, &s.programs@[j], position),
            forall|j: int| 0 <= j < i ==> words@[j]@ == output(s, j),
        decreases s.programs.len() - i,
    {
        match m::apply(&s.units, &s.programs[i], position) {
            Err(_) => return Err(InvalidProgram),
            Ok(word) => words.push(word),
        }
        i += 1;
    }
    Ok(Slot { words })
}

pub open spec fn projected_slots(source: Seq<FootSlot>, target: Seq<Slot>) -> bool {
    source.len() == target.len()
        && forall|i: int| 0 <= i < source.len() ==> (
            compilable(&source[i], Position { initial: i == 0, terminal: i + 1 == source.len() })
            && target[i].words@.len() == source[i].programs@.len()
            && forall|j: int| 0 <= j < target[i].words@.len() ==> target[i].words@[j]@ == output(&source[i], j))
}

pub open spec fn structural_slots(source: Seq<FootSlot>) -> bool {
    forall|i: int| 0 <= i < source.len() ==>
        compilable(&source[i],Position { initial:i == 0, terminal:i+1 == source.len() })
}

pub open spec fn valid_slots(source: Seq<FootSlot>) -> bool {
    structural_slots(source) && forall|i: int,j: int| 0 <= i < source.len()
        && 0 <= j < source[i].programs@.len() ==> verse::binary(output(&source[i],j))
}

pub open spec fn valid(source: &Profile) -> bool {
    source.id.family != 0 && source.id.revision != 0 && source.forms@.len() > 0
        && forall|i: int| 0 <= i < source.forms@.len() ==> (
            source.forms@[i].sadr@.len() > 0
            && valid_slots(source.forms@[i].sadr@) && valid_slots(source.forms@[i].ajuz@)
            && adjacency::valid(source.forms@[i].sadr@,source.forms@[i].sadr_constraints@)
            && adjacency::valid(source.forms@[i].ajuz@,source.forms@[i].ajuz_constraints@))
}

proof fn slots_equivalent(source: Seq<FootSlot>, target: Seq<Slot>)
    requires projected_slots(source,target),
    ensures verse::slots_valid(target) == (source.len() > 0 && valid_slots(source)),
{
    assert forall|i: int| 0 <= i < source.len() implies
        verse::alternatives_valid(target[i].words@)
            == (forall|j: int| 0 <= j < source[i].programs@.len() ==> verse::binary(output(&source[i],j))) by {
        assert(compilable(&source[i],Position { initial:i == 0, terminal:i+1 == source.len() }));
        assert(target[i].words@.len() == source[i].programs@.len());
        assert(source[i].programs@.len() > 0);
        assert forall|j: int| 0 <= j < source[i].programs@.len() implies
            verse::binary(target[i].words@[j]@) == verse::binary(output(&source[i],j)) by {
            assert(target[i].words@[j]@ == output(&source[i],j));
        }
        if verse::alternatives_valid(target[i].words@) {
            assert forall|j: int| 0 <= j < source[i].programs@.len() implies verse::binary(output(&source[i],j)) by {
                assert(verse::binary(target[i].words@[j]@));
                assert(target[i].words@[j]@ == output(&source[i],j));
            }
        }
        if forall|j: int| 0 <= j < source[i].programs@.len() ==> verse::binary(output(&source[i],j)) {
            assert forall|j: int| 0 <= j < target[i].words@.len() implies verse::binary(target[i].words@[j]@) by {
                assert(verse::binary(output(&source[i],j)));
                assert(target[i].words@[j]@ == output(&source[i],j));
            }
            assert(verse::alternatives_valid(target[i].words@));
        }
    }
}

fn compile_slots(source: &[FootSlot]) -> (answer: Result<Vec<Slot>, InvalidProfile>)
    ensures match answer { Err(_) => !structural_slots(source@), Ok(target) => projected_slots(source@, target@) },
{
    let mut target: Vec<Slot> = Vec::new();
    let mut i = 0;
    while i < source.len()
        invariant i <= source.len(), target@.len() == i,
            forall|k: int| 0 <= k < i ==> (
                compilable(&source@[k], Position { initial: k == 0, terminal: k + 1 == source.len() })
                && target@[k].words@.len() == source@[k].programs@.len()
                && forall|j: int| 0 <= j < target@[k].words@.len() ==> target@[k].words@[j]@ == output(&source@[k], j)),
        decreases source.len() - i,
    {
        match compile(&source[i], Position { initial: i == 0, terminal: i == source.len() - 1 }) {
            Err(_) => return Err(InvalidProfile),
            Ok(slot) => target.push(slot),
        }
        i += 1;
    }
    Ok(target)
}

pub open spec fn projected(source: &Profile, target: &verse::Profile) -> bool {
    &&& target.id == source.id && target.binding == source.binding
    &&& target.forms@.len() == source.forms@.len()
    &&& forall|i: int| 0 <= i < source.forms@.len() ==> (
        target.forms@[i].context == source.forms@[i].context
        && target.forms@[i].initial_only == source.forms@[i].initial_only
        && target.forms@[i].class == source.forms@[i].class
        && adjacency::valid(source.forms@[i].sadr@,source.forms@[i].sadr_constraints@)
        && adjacency::valid(source.forms@[i].ajuz@,source.forms@[i].ajuz_constraints@)
        && projected_slots(source.forms@[i].sadr@, target.forms@[i].sadr@)
        && projected_slots(source.forms@[i].ajuz@, target.forms@[i].ajuz@))
}

pub proof fn projection_equivalent(source: &Profile, target: &verse::Profile)
    requires projected(source,target),
    ensures valid(source) == verse::valid(target),
{
    assert forall|i: int| 0 <= i < source.forms@.len() implies (
        verse::slots_valid(target.forms@[i].sadr@)
            == (source.forms@[i].sadr@.len() > 0 && valid_slots(source.forms@[i].sadr@)))
        && ((target.forms@[i].ajuz@.len() == 0 || verse::slots_valid(target.forms@[i].ajuz@))
            == valid_slots(source.forms@[i].ajuz@)) by {
        slots_equivalent(source.forms@[i].sadr@,target.forms@[i].sadr@);
        slots_equivalent(source.forms@[i].ajuz@,target.forms@[i].ajuz@);
    }
}

/// Compile all branches and alternatives, retaining their indices. Structural
/// failures or empty/nonbinary surface words reject the entire profile.
pub fn compile_profile(source: &Profile) -> (answer: Result<verse::Profile, InvalidProfile>)
    ensures match answer { Err(_) => !valid(source), Ok(target) => valid(source) && verse::valid(&target) && projected(source, &target) },
{
    let mut forms: Vec<verse::Form> = Vec::new();
    let mut i = 0;
    while i < source.forms.len()
        invariant i <= source.forms.len(), forms@.len() == i,
            forall|j: int| 0 <= j < i ==> (
                forms@[j].context == source.forms@[j].context
                && forms@[j].initial_only == source.forms@[j].initial_only
                && forms@[j].class == source.forms@[j].class
                && adjacency::valid(source.forms@[j].sadr@,source.forms@[j].sadr_constraints@)
                && adjacency::valid(source.forms@[j].ajuz@,source.forms@[j].ajuz_constraints@)
                && projected_slots(source.forms@[j].sadr@, forms@[j].sadr@)
                && projected_slots(source.forms@[j].ajuz@, forms@[j].ajuz@)),
        decreases source.forms.len() - i,
    {
        let f = &source.forms[i];
        if !adjacency::check(&f.sadr,&f.sadr_constraints) || !adjacency::check(&f.ajuz,&f.ajuz_constraints) { return Err(InvalidProfile); }
        let sadr = match compile_slots(&f.sadr) { Err(_) => return Err(InvalidProfile), Ok(v) => v };
        let ajuz = match compile_slots(&f.ajuz) { Err(_) => return Err(InvalidProfile), Ok(v) => v };
        forms.push(verse::Form { context: f.context, initial_only: f.initial_only, class: f.class, sadr, ajuz });
        i += 1;
    }
    let target = verse::Profile { id: source.id, binding: source.binding, forms };
    proof { projection_equivalent(source,&target); }
    if !verse::validate(&target) { return Err(InvalidProfile); }
    Ok(target)
}

} // verus!
