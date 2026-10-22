//! Structural, simultaneous original-coordinate edits over typed units.
use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;
use watad_types::Unit;

verus! {

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Letter { pub beat: u8, pub owner: usize, pub offset: usize, pub sabab: bool }

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action { Keep, Delete, Silence }

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Zihaf, Terminal, Initial,
    /// Explicit profile licence for an illa behaving as nonbinding variation.
    /// Unlike Zihaf, this does not assert that all edits are sabab-owned.
    Licensed,
}

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position { pub initial: bool, pub terminal: bool }

#[derive(Debug)]
pub struct Program {
    pub scope: Scope,
    pub required_final: Option<Unit>,
    pub actions: Vec<Action>,
    pub expected: Vec<u8>,
    pub suffix: Vec<u8>,
}

#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidProgram;

/// Byte projection of the actual shared primitive words; no second unit table.
pub open spec fn unit_word(u: Unit) -> Seq<u8> {
    let beats = u.beat_spec();
    Seq::new(beats.len(),|i: int| crate::adapter::beat_code(beats[i]))
}

pub open spec fn sabab(u: Unit) -> bool { matches!(u, Unit::Sabab(_)) }

pub fn unit_size(u: Unit) -> (n: usize)
    ensures n == unit_word(u).len(), 2 <= n <= 3,
{
    u.beats().len()
}

fn unit_letter(u: Unit, j: usize) -> (beat: u8)
    requires j < unit_word(u).len(),
    ensures beat == unit_word(u)[j as int], beat <= 1,
{
    crate::adapter::encode_beat(u.beats()[j])
}

pub open spec fn unit_letters(u: Unit, owner: usize) -> Seq<Letter> {
    Seq::new(unit_word(u).len(), |j: int| Letter {
        beat: unit_word(u)[j], owner, offset: j as usize, sabab: sabab(u),
    })
}

fn expand_unit(u: Unit, owner: usize) -> (letters: Vec<Letter>)
    ensures letters@ == unit_letters(u, owner),
{
    let n = unit_size(u);
    let is_sabab = matches!(u, Unit::Sabab(_));
    let mut letters = Vec::new();
    let mut j = 0;
    while j < n
        invariant n == unit_word(u).len(), j <= n, is_sabab == sabab(u),
            letters@.len() == j,
            forall|k: int| 0 <= k < j ==> #[trigger] letters@[k] == unit_letters(u, owner)[k],
        decreases n - j,
    {
        letters.push(Letter { beat: unit_letter(u, j), owner, offset: j, sabab: is_sabab });
        j += 1;
    }
    assert(letters@ =~= unit_letters(u, owner));
    letters
}

pub open spec fn expanded(units: Seq<Unit>, n: int) -> Seq<Letter>
    decreases n,
{
    if n <= 0 || n > units.len() { Seq::empty() }
    else { expanded(units, n - 1) + unit_letters(units[n - 1], (n - 1) as usize) }
}

pub fn expand(units: &[Unit]) -> (letters: Vec<Letter>)
    ensures letters@ == expanded(units@, units@.len() as int),
{
    let mut letters = Vec::new();
    let mut i = 0;
    while i < units.len()
        invariant i <= units.len(), letters@ == expanded(units@, i as int),
        decreases units.len() - i,
    {
        let mut next = expand_unit(units[i], i);
        letters.append(&mut next);
        i += 1;
    }
    letters
}

pub open spec fn change_valid(letter: Letter, action: Action, expected: u8, scope: Scope) -> bool {
    action == Action::Keep || (
        expected == letter.beat && (action != Action::Silence || expected == 1)
        && (scope != Scope::Zihaf || (letter.sabab && letter.offset == 1)))
}

fn check_change(letter: Letter, action: Action, expected: u8, scope: Scope) -> (yes: bool)
    ensures yes == change_valid(letter, action, expected, scope),
{
    if action == Action::Keep { return true; }
    if expected != letter.beat || (action == Action::Silence && expected != 1) { return false; }
    scope != Scope::Zihaf || (letter.sabab && letter.offset == 1)
}

pub open spec fn eligible(p: &Program, position: Position) -> bool {
    match p.scope {
        Scope::Zihaf => p.suffix@.len() == 0,
        Scope::Terminal => position.terminal,
        Scope::Initial => position.initial && p.suffix@.len() == 0,
        Scope::Licensed => true,
    }
}

pub open spec fn final_matches(units: Seq<Unit>, expected: Option<Unit>) -> bool {
    match expected { None => true, Some(u) => units.len() > 0 && units.last() == u }
}

pub open spec fn valid(units: Seq<Unit>, p: &Program, position: Position) -> bool {
    let letters = expanded(units, units.len() as int);
    &&& units.len() > 0
    &&& eligible(p, position)
    &&& final_matches(units, p.required_final)
    &&& p.actions@.len() == letters.len() && p.expected@.len() == letters.len()
    &&& forall|i: int| 0 <= i < letters.len() ==>
        change_valid(letters[i], p.actions@[i], p.expected@[i], p.scope)
    &&& forall|i: int| 0 <= i < p.suffix@.len() ==> p.suffix@[i] <= 1
}

#[allow(clippy::collapsible_if)] // Explicit option branch keeps the proof readable.
fn validate(units: &[Unit], letters: &[Letter], p: &Program, position: Position) -> (yes: bool)
    requires letters@ == expanded(units@, units@.len() as int),
    ensures yes == valid(units@, p, position),
{
    if units.is_empty() { return false; }
    match p.scope {
        Scope::Zihaf => if !p.suffix.is_empty() { return false; },
        Scope::Terminal => if !position.terminal { return false; },
        Scope::Initial => if !position.initial || !p.suffix.is_empty() { return false; },
        Scope::Licensed => {},
    }
    if let Some(expected) = p.required_final {
        if units[units.len() - 1] != expected { return false; }
    }
    if p.actions.len() != letters.len() || p.expected.len() != letters.len() { return false; }
    let mut i = 0;
    while i < letters.len()
        invariant i <= letters.len(), units.len() > 0, eligible(p, position),
            final_matches(units@, p.required_final),
            letters@ == expanded(units@, units@.len() as int),
            p.actions.len() == letters.len(), p.expected.len() == letters.len(),
            forall|j: int| 0 <= j < i ==>
                change_valid(letters@[j], p.actions@[j], p.expected@[j], p.scope),
        decreases letters.len() - i,
    {
        if !check_change(letters[i], p.actions[i], p.expected[i], p.scope) { return false; }
        i += 1;
    }
    let mut j = 0;
    while j < p.suffix.len()
        invariant j <= p.suffix.len(), units.len() > 0, eligible(p, position),
            final_matches(units@, p.required_final),
            letters@ == expanded(units@, units@.len() as int),
            p.actions.len() == letters.len(), p.expected.len() == letters.len(),
            forall|k: int| 0 <= k < letters.len() ==>
                change_valid(letters@[k], p.actions@[k], p.expected@[k], p.scope),
            forall|k: int| 0 <= k < j ==> p.suffix@[k] <= 1,
        decreases p.suffix.len() - j,
    {
        if p.suffix[j] > 1 { return false; }
        j += 1;
    }
    true
}

pub open spec fn changed(letters: Seq<Letter>, actions: Seq<Action>, n: int) -> Seq<u8>
    decreases n,
{
    if n <= 0 || n > letters.len() || n > actions.len() { Seq::empty() }
    else {
        let prefix = changed(letters, actions, n - 1);
        match actions[n - 1] {
            Action::Keep => prefix.push(letters[n - 1].beat),
            Action::Delete => prefix,
            Action::Silence => prefix.push(0u8),
        }
    }
}

/// Apply only after all original-letter conditions have been checked. Earlier
/// deletions never change the coordinate of a later action.
pub fn apply(units: &[Unit], p: &Program, position: Position)
    -> (answer: Result<Vec<u8>, InvalidProgram>)
    ensures match answer {
        Err(_) => !valid(units@, p, position),
        Ok(word) => valid(units@, p, position)
            && word@ == changed(expanded(units@, units@.len() as int), p.actions@,
                                p.actions@.len() as int) + p.suffix@,
    },
{
    let letters = expand(units);
    if !validate(units, &letters, p, position) { return Err(InvalidProgram); }
    let mut word = Vec::new();
    let mut i = 0;
    while i < letters.len()
        invariant valid(units@, p, position),
            letters@ == expanded(units@, units@.len() as int),
            i <= letters.len(), p.actions.len() == letters.len(),
            word@ == changed(letters@, p.actions@, i as int),
        decreases letters.len() - i,
    {
        match p.actions[i] {
            Action::Keep => word.push(letters[i].beat),
            Action::Delete => {},
            Action::Silence => word.push(0),
        }
        i += 1;
    }
    let mut suffix = Vec::new();
    let mut j = 0;
    while j < p.suffix.len()
        invariant j <= p.suffix.len(), suffix@ == p.suffix@.subrange(0, j as int),
            valid(units@, p, position),
            word@ == changed(expanded(units@, units@.len() as int), p.actions@, p.actions@.len() as int),
        decreases p.suffix.len() - j,
    {
        suffix.push(p.suffix[j]);
        j += 1;
    }
    word.append(&mut suffix);
    Ok(word)
}

/// A valid zihaf can never delete or silence any letter owned by a watad.
pub proof fn watad_is_protected(units: Seq<Unit>, p: &Program, position: Position, i: int)
    requires valid(units, p, position), p.scope == Scope::Zihaf,
        0 <= i < expanded(units, units.len() as int).len(),
        !expanded(units, units.len() as int)[i].sabab,
    ensures p.actions@[i] == Action::Keep,
{}

} // verus!
