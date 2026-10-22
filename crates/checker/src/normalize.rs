//! Verified construction of original-coordinate programs from named recipes.
//!
//! Normalization is a pure function of the original base and the requested
//! changes. It never consults a meter, profile or position: source eligibility
//! remains a separate boundary. Every successful edit is checked against the
//! actual letter state and ownership before it is recorded.
#[cfg(verus_keep_ghost)]
use crate::named::Operation;
use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;
use watad_types::{FootId, Unit};

use crate::morphology::{self as m, Action, Program, Scope};
use crate::named::{Change, Illa, Zihaf};

verus! {

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    OutOfRange,
    Conflict,
    Scope,
    Structure,
    Precondition,
}

#[cfg(verus_keep_ghost)]
pub struct Normalized {
    pub scope: Scope,
    pub required_final: Option<Unit>,
    pub actions: Seq<Action>,
    pub expected: Seq<u8>,
    pub suffix: Seq<u8>,
}

pub open spec fn program_view(p: &Program) -> Normalized {
    Normalized {
        scope: p.scope,
        required_final: p.required_final,
        actions: p.actions@,
        expected: p.expected@,
        suffix: p.suffix@,
    }
}

pub open spec fn update_action(s: Seq<Action>, at: int, action: Action) -> Seq<Action> {
    Seq::new(s.len(), |i: int| if i == at { action } else { s[i] })
}

pub open spec fn update_beat(s: Seq<u8>, at: int, beat: u8) -> Seq<u8> {
    Seq::new(s.len(), |i: int| if i == at { beat } else { s[i] })
}

pub open spec fn initial_norm(base: FootId) -> Normalized {
    let units = base.word_spec();
    let letters = m::expanded(units, units.len() as int);
    Normalized {
        scope: Scope::Zihaf,
        required_final: None,
        actions: Seq::new(letters.len(), |i: int| Action::Keep),
        expected: Seq::new(letters.len(), |i: int| letters[i].beat),
        suffix: Seq::empty(),
    }
}

pub open spec fn edit_norm(
    p: Normalized,
    letters: Seq<m::Letter>,
    at: int,
    expected: u8,
    action: Action,
) -> Result<Normalized, Error> {
    if at < 0 || at >= p.actions.len() || at >= letters.len() {
        Err(Error::OutOfRange)
    } else if letters[at].beat != expected {
        Err(Error::Precondition)
    } else if p.actions[at] != Action::Keep
        && (p.actions[at] != action || p.expected[at] != expected)
    {
        Err(Error::Conflict)
    } else {
        Ok(Normalized {
            scope: p.scope,
            required_final: p.required_final,
            actions: update_action(p.actions, at, action),
            expected: update_beat(p.expected, at, expected),
            suffix: p.suffix,
        })
    }
}

pub open spec fn compose_scope(a: Scope, b: Scope) -> Result<Scope, Error> {
    match (a, b) {
        (Scope::Zihaf, s) => Ok(s),
        (s, Scope::Zihaf) => Ok(s),
        (Scope::Terminal, Scope::Terminal) => Ok(Scope::Terminal),
        (Scope::Terminal, Scope::Licensed) => Ok(Scope::Terminal),
        (Scope::Licensed, Scope::Terminal) => Ok(Scope::Terminal),
        (Scope::Licensed, Scope::Licensed) => Ok(Scope::Licensed),
        (Scope::Initial, Scope::Initial) => Ok(Scope::Initial),
        _ => Err(Error::Scope),
    }
}

pub open spec fn set_required(
    units: Seq<Unit>,
    p: Normalized,
    required: Option<Unit>,
) -> Result<Normalized, Error> {
    match required {
        None => Ok(p),
        Some(u) => {
            if units.len() == 0 || units.last() != u {
                Err(Error::Structure)
            } else if p.required_final.is_some() && p.required_final != Some(u) {
                Err(Error::Structure)
            } else {
                Ok(Normalized {
                    scope: p.scope,
                    required_final: Some(u),
                    actions: p.actions,
                    expected: p.expected,
                    suffix: p.suffix,
                })
            }
        }
    }
}

pub open spec fn append_suffix(p: Normalized, suffix: Seq<u8>) -> Normalized {
    Normalized {
        scope: p.scope,
        required_final: p.required_final,
        actions: p.actions,
        expected: p.expected,
        suffix: p.suffix + suffix,
    }
}

pub open spec fn changes_scope(p: Normalized, scope: Scope) -> Result<Normalized, Error> {
    match compose_scope(p.scope, scope) {
        Err(e) => Err(e),
        Ok(s) => Ok(Normalized {
            scope: s,
            required_final: p.required_final,
            actions: p.actions,
            expected: p.expected,
            suffix: p.suffix,
        }),
    }
}

pub open spec fn first_watad(units: Seq<Unit>, i: int) -> Option<int>
    decreases units.len() - i,
{
    if i < 0 || i >= units.len() {
        None
    } else if units[i] == Unit::W {
        Some(i)
    } else {
        first_watad(units, i + 1)
    }
}

pub open spec fn beats_before(units: Seq<Unit>, i: int) -> int
    decreases i,
{
    if i <= 0 {
        0
    } else {
        beats_before(units, i - 1) + m::unit_word(units[i - 1]).len()
    }
}

pub open spec fn suffix_tarfil() -> Seq<u8> {
    Seq::new(2, |i: int| if i == 0 { 1u8 } else { 0u8 })
}

pub open spec fn suffix_sakin() -> Seq<u8> {
    Seq::new(1, |_i: int| 0u8)
}

#[cfg_attr(verus_keep_ghost, verifier::opaque)]
pub open spec fn illa_edits(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    i: Illa,
) -> Result<Normalized, Error> {
    let n = letters.len();
    match i {
        Illa::Tarfil => {
            if p.suffix.len() != 0 {
                Err(Error::Conflict)
            } else {
                Ok(append_suffix(p, suffix_tarfil()))
            }
        }
        Illa::Tadhyil | Illa::Tasbigh => {
            if p.suffix.len() != 0 {
                Err(Error::Conflict)
            } else {
                Ok(append_suffix(p, suffix_sakin()))
            }
        }
        Illa::Qat => {
            if n < 2 {
                Err(Error::OutOfRange)
            } else {
                match edit_norm(p, letters, n - 1, 0, Action::Delete) {
                    Err(e) => Err(e),
                    Ok(q) => edit_norm(q, letters, n - 2, 1, Action::Silence),
                }
            }
        }
        Illa::Hadhadh | Illa::Salm => {
            if n < 3 {
                Err(Error::OutOfRange)
            } else {
                match edit_norm(p, letters, n - 3, letters[n - 3].beat, Action::Delete) {
                    Err(e) => Err(e),
                    Ok(q) => match edit_norm(q, letters, n - 2, letters[n - 2].beat, Action::Delete) {
                        Err(e) => Err(e),
                        Ok(r) => edit_norm(r, letters, n - 1, letters[n - 1].beat, Action::Delete),
                    },
                }
            }
        }
        Illa::Hadhf | Illa::Qatf | Illa::Batr => {
            if n < 2 {
                Err(Error::OutOfRange)
            } else {
                match edit_norm(p, letters, n - 2, 1, Action::Delete) {
                    Err(e) => Err(e),
                    Ok(q) => match edit_norm(q, letters, n - 1, 0, Action::Delete) {
                        Err(e) => Err(e),
                        Ok(r) => {
                            let s = if i == Illa::Qatf {
                                edit_norm(r, letters, 4, 1, Action::Silence)
                            } else {
                                Ok(r)
                            };
                            match s {
                                Err(e) => Err(e),
                                Ok(t) => {
                                    if i != Illa::Batr {
                                        Ok(t)
                                    } else if units.len() < 2
                                        || units[units.len() - 2] != Unit::W
                                    {
                                        Err(Error::Structure)
                                    } else {
                                        match edit_norm(t, letters, n - 3, 0, Action::Delete) {
                                            Err(e) => Err(e),
                                            Ok(u) => {
                                                edit_norm(u, letters, n - 4, 1, Action::Silence)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }
        }
        Illa::Qasr => {
            if n < 2 {
                Err(Error::OutOfRange)
            } else {
                match edit_norm(p, letters, n - 1, 0, Action::Delete) {
                    Err(e) => Err(e),
                    Ok(q) => edit_norm(q, letters, n - 2, 1, Action::Silence),
                }
            }
        }
        Illa::Waqf => edit_norm(p, letters, 6, 1, Action::Silence),
        Illa::Kasf => edit_norm(p, letters, 6, 1, Action::Delete),
        Illa::Kharm => {
            if units.len() == 0 || units[0] != Unit::W {
                Err(Error::Structure)
            } else {
                edit_norm(p, letters, 0, 1, Action::Delete)
            }
        }
    }
}

pub open spec fn apply_illa(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    i: Illa,
    nonbinding: bool,
) -> Result<Normalized, Error> {
    let desired = if nonbinding {
        Scope::Licensed
    } else if i == Illa::Kharm {
        Scope::Initial
    } else {
        Scope::Terminal
    };
    match changes_scope(p, desired) {
        Err(e) => Err(e),
        Ok(p1) => match set_required(units, p1, crate::named::illa_required(i)) {
            Err(e) => Err(e),
            Ok(p2) => illa_edits(units, letters, p2, i),
        },
    }
}

#[cfg_attr(verus_keep_ghost, verifier::opaque)]
pub open spec fn apply_change(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    change: Change,
) -> Result<Normalized, Error> {
    match change {
        Change::Unchanged => Ok(p),
        Change::Zihaf(z) => {
            let at = crate::named::zihaf_index(z);
            if at < 0 || at >= letters.len() {
                Err(Error::OutOfRange)
            } else if !letters[at].sabab || letters[at].offset != 1 {
                Err(Error::Structure)
            } else {
                let expected = crate::named::zihaf_expected(z);
                if letters[at].beat != expected {
                    Err(Error::Precondition)
                } else {
                    let action = match crate::named::zihaf_action(z) {
                        Operation::Delete => Action::Delete,
                        Operation::Silence => Action::Silence,
                    };
                    edit_norm(p, letters, at, expected, action)
                }
            }
        }
        Change::Illa(i) => apply_illa(units, letters, p, i, false),
        Change::NonbindingIlla(i) => apply_illa(units, letters, p, i, true),
        Change::Tashith => match compose_scope(p.scope, Scope::Terminal) {
            Err(e) => Err(e),
            Ok(s) => match first_watad(units, 0) {
                None => Err(Error::Structure),
                Some(owner) => edit_norm(
                    Normalized {
                        scope: s,
                        required_final: p.required_final,
                        actions: p.actions,
                        expected: p.expected,
                        suffix: p.suffix,
                    },
                    letters,
                    beats_before(units, owner),
                    1,
                    Action::Delete,
                ),
            },
        },
    }
}

#[cfg_attr(verus_keep_ghost, verifier::opaque)]
pub open spec fn initial_match(base: FootId, initial: Normalized) -> bool {
    initial == initial_norm(base)
}

#[cfg_attr(verus_keep_ghost, verifier::opaque)]
pub open spec fn letters_match(units: Seq<Unit>, letters: Seq<m::Letter>) -> bool {
    letters == m::expanded(units, units.len() as int)
}

pub open spec fn seen(changes: Seq<Change>, i: int, change: Change) -> bool
    decreases i,
{
    if i <= 0 {
        false
    } else if changes[i - 1] == change {
        true
    } else {
        seen(changes, i - 1, change)
    }
}

#[cfg_attr(verus_keep_ghost, verifier::opaque)]
pub open spec fn normalize_steps(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    changes: Seq<Change>,
    i: int,
) -> Result<Normalized, Error>
    decreases changes.len() - i,
{
    if i >= changes.len() {
        Ok(p)
    } else if seen(changes, i, changes[i]) {
        normalize_steps(units, letters, p, changes, i + 1)
    } else {
        match apply_change(units, letters, p, changes[i]) {
            Err(e) => Err(e),
            Ok(q) => normalize_steps(units, letters, q, changes, i + 1),
        }
    }
}

proof fn normalize_steps_unchanged(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    changes: Seq<Change>,
    i: int,
)
    requires 0 <= i < changes.len(), changes[i] == Change::Unchanged,
    ensures normalize_steps(units, letters, p, changes, i)
        == normalize_steps(units, letters, p, changes, i + 1),
{
    reveal(normalize_steps);
    reveal(apply_change);
}

proof fn normalize_steps_duplicate(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    changes: Seq<Change>,
    i: int,
)
    requires 0 <= i < changes.len(), seen(changes, i, changes[i]),
    ensures normalize_steps(units, letters, p, changes, i)
        == normalize_steps(units, letters, p, changes, i + 1),
{
    reveal(normalize_steps);
}

proof fn normalize_steps_step_ok(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    q: Normalized,
    changes: Seq<Change>,
    i: int,
)
    requires 0 <= i < changes.len(), !seen(changes, i, changes[i]),
        apply_change(units, letters, p, changes[i]) == Ok(q),
    ensures normalize_steps(units, letters, p, changes, i)
        == normalize_steps(units, letters, q, changes, i + 1),
{
    reveal(normalize_steps);
}

proof fn normalize_steps_step_err(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    changes: Seq<Change>,
    i: int,
    e: Error,
)
    requires 0 <= i < changes.len(), !seen(changes, i, changes[i]),
        apply_change(units, letters, p, changes[i]) == Err(e),
    ensures normalize_steps(units, letters, p, changes, i) == Err(e),
{
    reveal(normalize_steps);
}

proof fn normalize_steps_done(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    changes: Seq<Change>,
    i: int,
)
    requires i >= changes.len(),
    ensures normalize_steps(units, letters, p, changes, i) == Ok(p),
{
    reveal(normalize_steps);
}

pub open spec fn normalize_spec(base: FootId, changes: Seq<Change>) -> Result<Normalized, Error> {
    let units = base.word_spec();
    let letters = m::expanded(units, units.len() as int);
    normalize_steps(units, letters, initial_norm(base), changes, 0)
}

/// Proof-only invariant carried by successful normalization toward
/// `morphology::valid`. It is not a second validator and is not consulted by
/// the executable normalizer.
pub open spec fn norm_valid_invariant(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
) -> bool {
    &&& p.actions.len() == letters.len()
    &&& p.expected.len() == letters.len()
    &&& forall|j: int| 0 <= j < letters.len() ==> #[trigger] p.expected[j] == letters[j].beat
    &&& forall|j: int| 0 <= j < letters.len() && #[trigger] p.actions[j] == Action::Silence
        ==> p.expected[j] == 1
    &&& (p.scope == Scope::Zihaf ==> forall|j: int|
        0 <= j < letters.len() && #[trigger] p.actions[j] != Action::Keep
            ==> letters[j].sabab && letters[j].offset == 1)
    &&& m::final_matches(units, p.required_final)
    &&& forall|j: int| 0 <= j < p.suffix.len() ==> #[trigger] p.suffix[j] <= 1
}

proof fn edit_norm_invariant(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    q: Normalized,
    at: int,
    expected: u8,
    action: Action,
)
    requires
        norm_valid_invariant(units, letters, p),
        0 <= at < letters.len(),
        letters[at].beat == expected,
        edit_norm(p, letters, at, expected, action) == Ok(q),
        action == Action::Silence ==> expected == 1,
        q.scope == Scope::Zihaf && action != Action::Keep
            ==> letters[at].sabab && letters[at].offset == 1,
    ensures norm_valid_invariant(units, letters, q),
{
    reveal(edit_norm);
    assert(q.actions == update_action(p.actions, at, action));
    assert(q.expected == update_beat(p.expected, at, expected));
    assert forall|j: int| 0 <= j < letters.len()
        implies #[trigger] q.expected[j] == letters[j].beat by {
        if j == at {
            assert(q.expected[j] == expected);
        } else {
            assert(q.expected[j] == p.expected[j]);
        }
    }
    assert forall|j: int| 0 <= j < letters.len() && #[trigger] q.actions[j] == Action::Silence
        implies q.expected[j] == 1 by {
        if j == at {
            assert(action == Action::Silence);
        } else {
            assert(q.actions[j] == p.actions[j]);
        }
    }
    assert forall|j: int| 0 <= j < letters.len() && q.scope == Scope::Zihaf
        && #[trigger] q.actions[j] != Action::Keep
        implies letters[j].sabab && letters[j].offset == 1 by {
        if j == at {
            assert(action != Action::Keep);
        } else {
            assert(q.actions[j] == p.actions[j]);
        }
    }
}

proof fn append_suffix_invariant(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    suffix: Seq<u8>,
)
    requires
        norm_valid_invariant(units, letters, p),
        forall|j: int| 0 <= j < suffix.len() ==> #[trigger] suffix[j] <= 1,
    ensures norm_valid_invariant(units, letters, append_suffix(p, suffix)),
{
    let q = append_suffix(p, suffix);
    assert(q.suffix == p.suffix + suffix);
    assert forall|j: int| 0 <= j < q.suffix.len() implies #[trigger] q.suffix[j] <= 1 by {
        if j < p.suffix.len() {
            assert(q.suffix[j] == p.suffix[j]);
        } else {
            assert(0 <= j - p.suffix.len() < suffix.len());
            assert((p.suffix + suffix)[j] == suffix[j - p.suffix.len()]);
        }
    }
}

proof fn compose_scope_not_zihaf(a: Scope, b: Scope, s: Scope)
    requires b != Scope::Zihaf, compose_scope(a, b) == Ok(s),
    ensures s != Scope::Zihaf,
{
    reveal(compose_scope);
}

proof fn changes_scope_invariant(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    q: Normalized,
    scope: Scope,
)
    requires
        norm_valid_invariant(units, letters, p),
        scope != Scope::Zihaf,
        changes_scope(p, scope) == Ok(q),
    ensures norm_valid_invariant(units, letters, q),
{
    reveal(changes_scope);
    compose_scope_not_zihaf(p.scope, scope, q.scope);
}

proof fn set_required_invariant(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    q: Normalized,
    required: Option<Unit>,
)
    requires norm_valid_invariant(units, letters, p), set_required(units, p, required) == Ok(q),
    ensures norm_valid_invariant(units, letters, q),
{
    reveal(set_required);
    match required {
        None => {},
        Some(u) => {
            assert(units.len() > 0 && units.last() == u);
        },
    }
}

proof fn edit_norm_non_zihaf(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    q: Normalized,
    at: int,
    expected: u8,
    action: Action,
)
    requires
        norm_valid_invariant(units, letters, p),
        p.scope != Scope::Zihaf,
        edit_norm(p, letters, at, expected, action) == Ok(q),
        action == Action::Silence ==> expected == 1,
    ensures norm_valid_invariant(units, letters, q),
{
    reveal(edit_norm);
    assert(0 <= at < letters.len());
    assert(letters[at].beat == expected);
    assert(q.scope == p.scope);
    edit_norm_invariant(units, letters, p, q, at, expected, action);
}

proof fn illa_edits_invariant(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    q: Normalized,
    i: Illa,
)
    requires
        norm_valid_invariant(units, letters, p),
        p.scope != Scope::Zihaf,
        illa_edits(units, letters, p, i) == Ok(q),
    ensures norm_valid_invariant(units, letters, q),
{
    reveal(illa_edits);
    let n = letters.len();
    match i {
        Illa::Tarfil => {
            assert(q == append_suffix(p, suffix_tarfil()));
            assert forall|j: int| 0 <= j < suffix_tarfil().len()
                implies #[trigger] suffix_tarfil()[j] <= 1 by {}
            append_suffix_invariant(units, letters, p, suffix_tarfil());
        },
        Illa::Tadhyil | Illa::Tasbigh => {
            assert(q == append_suffix(p, suffix_sakin()));
            assert forall|j: int| 0 <= j < suffix_sakin().len()
                implies #[trigger] suffix_sakin()[j] <= 1 by {}
            append_suffix_invariant(units, letters, p, suffix_sakin());
        },
        Illa::Qat => {
            if n < 2 {
                assert(false);
            } else {
                match edit_norm(p, letters, n - 1, 0, Action::Delete) {
                    Err(_) => assert(false),
                    Ok(q1) => {
                        edit_norm_non_zihaf(
                            units, letters, p, q1, n - 1, 0, Action::Delete,
                        );
                        match edit_norm(q1, letters, n - 2, 1, Action::Silence) {
                            Err(_) => assert(false),
                            Ok(q2) => {
                                edit_norm_non_zihaf(
                                    units, letters, q1, q2, n - 2, 1, Action::Silence,
                                );
                                assert(q2 == q);
                            },
                        }
                    },
                }
            }
        },
        Illa::Hadhadh | Illa::Salm => {
            if n < 3 {
                assert(false);
            } else {
                match edit_norm(p, letters, n - 3, letters[n - 3].beat, Action::Delete) {
                    Err(_) => assert(false),
                    Ok(q1) => {
                        edit_norm_non_zihaf(
                            units, letters, p, q1, n - 3, letters[n - 3].beat, Action::Delete,
                        );
                        match edit_norm(q1, letters, n - 2, letters[n - 2].beat, Action::Delete) {
                            Err(_) => assert(false),
                            Ok(q2) => {
                                edit_norm_non_zihaf(
                                    units, letters, q1, q2, n - 2, letters[n - 2].beat,
                                    Action::Delete,
                                );
                                match edit_norm(q2, letters, n - 1, letters[n - 1].beat, Action::Delete) {
                                    Err(_) => assert(false),
                                    Ok(q3) => {
                                        edit_norm_non_zihaf(
                                            units, letters, q2, q3, n - 1, letters[n - 1].beat,
                                            Action::Delete,
                                        );
                                        assert(q3 == q);
                                    },
                                }
                            },
                        }
                    },
                }
            }
        },
        Illa::Hadhf | Illa::Qatf | Illa::Batr => {
            if n < 2 {
                assert(false);
            } else {
                match edit_norm(p, letters, n - 2, 1, Action::Delete) {
                    Err(_) => assert(false),
                    Ok(q1) => {
                        edit_norm_non_zihaf(
                            units, letters, p, q1, n - 2, 1, Action::Delete,
                        );
                        match edit_norm(q1, letters, n - 1, 0, Action::Delete) {
                            Err(_) => assert(false),
                            Ok(q2) => {
                                edit_norm_non_zihaf(
                                    units, letters, q1, q2, n - 1, 0, Action::Delete,
                                );
                                let q3 = if i == Illa::Qatf {
                                    match edit_norm(q2, letters, 4, 1, Action::Silence) {
                                        Err(_) => {
                                            assert(false);
                                            q2
                                        },
                                        Ok(q3) => {
                                            edit_norm_non_zihaf(
                                                units, letters, q2, q3, 4, 1, Action::Silence,
                                            );
                                            q3
                                        },
                                    }
                                } else {
                                    q2
                                };
                                if i != Illa::Batr {
                                    assert(q3 == q);
                                } else if units.len() < 2
                                    || units[units.len() - 2] != Unit::W
                                {
                                    assert(false);
                                } else {
                                    match edit_norm(q3, letters, n - 3, 0, Action::Delete) {
                                        Err(_) => assert(false),
                                        Ok(q4) => {
                                            edit_norm_non_zihaf(
                                                units, letters, q3, q4, n - 3, 0, Action::Delete,
                                            );
                                            match edit_norm(q4, letters, n - 4, 1, Action::Silence) {
                                                Err(_) => assert(false),
                                                Ok(q5) => {
                                                    edit_norm_non_zihaf(
                                                        units, letters, q4, q5, n - 4, 1,
                                                        Action::Silence,
                                                    );
                                                    assert(q5 == q);
                                                },
                                            }
                                        },
                                    }
                                }
                            },
                        }
                    },
                }
            }
        },
        Illa::Qasr => {
            if n < 2 {
                assert(false);
            } else {
                match edit_norm(p, letters, n - 1, 0, Action::Delete) {
                    Err(_) => assert(false),
                    Ok(q1) => {
                        edit_norm_non_zihaf(
                            units, letters, p, q1, n - 1, 0, Action::Delete,
                        );
                        match edit_norm(q1, letters, n - 2, 1, Action::Silence) {
                            Err(_) => assert(false),
                            Ok(q2) => {
                                edit_norm_non_zihaf(
                                    units, letters, q1, q2, n - 2, 1, Action::Silence,
                                );
                                assert(q2 == q);
                            },
                        }
                    },
                }
            }
        },
        Illa::Waqf => {
            edit_norm_non_zihaf(units, letters, p, q, 6, 1, Action::Silence);
        },
        Illa::Kasf => {
            edit_norm_non_zihaf(units, letters, p, q, 6, 1, Action::Delete);
        },
        Illa::Kharm => {
            if units.len() == 0 || units[0] != Unit::W {
                assert(false);
            } else {
                edit_norm_non_zihaf(units, letters, p, q, 0, 1, Action::Delete);
            }
        },
    }
}

proof fn scope_only_invariant(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    scope: Scope,
)
    requires norm_valid_invariant(units, letters, p), scope != Scope::Zihaf,
    ensures norm_valid_invariant(units, letters, Normalized {
        scope,
        required_final: p.required_final,
        actions: p.actions,
        expected: p.expected,
        suffix: p.suffix,
    }),
{
    let q = Normalized {
        scope,
        required_final: p.required_final,
        actions: p.actions,
        expected: p.expected,
        suffix: p.suffix,
    };
    assert forall|j: int| 0 <= j < letters.len() && q.scope == Scope::Zihaf
        && #[trigger] q.actions[j] != Action::Keep
        implies letters[j].sabab && letters[j].offset == 1 by {}
}

proof fn apply_illa_invariant(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    q: Normalized,
    i: Illa,
    nonbinding: bool,
)
    requires
        norm_valid_invariant(units, letters, p),
        apply_illa(units, letters, p, i, nonbinding) == Ok(q),
    ensures norm_valid_invariant(units, letters, q),
{
    reveal(apply_illa);
    let desired = if nonbinding {
        Scope::Licensed
    } else if i == Illa::Kharm {
        Scope::Initial
    } else {
        Scope::Terminal
    };
    assert(desired != Scope::Zihaf);
    match changes_scope(p, desired) {
        Err(_) => assert(false),
        Ok(p1) => {
            changes_scope_invariant(units, letters, p, p1, desired);
            compose_scope_not_zihaf(p.scope, desired, p1.scope);
            match set_required(units, p1, crate::named::illa_required(i)) {
                Err(_) => assert(false),
                Ok(p2) => {
                    set_required_invariant(
                        units, letters, p1, p2, crate::named::illa_required(i),
                    );
                    assert(p2.scope == p1.scope);
                    illa_edits_invariant(units, letters, p2, q, i);
                },
            }
        },
    }
}

proof fn apply_change_invariant(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    q: Normalized,
    change: Change,
)
    requires
        norm_valid_invariant(units, letters, p),
        apply_change(units, letters, p, change) == Ok(q),
    ensures norm_valid_invariant(units, letters, q),
{
    reveal(apply_change);
    match change {
        Change::Unchanged => {
            assert(q == p);
        },
        Change::Zihaf(z) => {
            let at = crate::named::zihaf_index(z);
            if at < 0 || at >= letters.len() {
                assert(false);
            } else if !letters[at].sabab || letters[at].offset != 1 {
                assert(false);
            } else {
                let expected = crate::named::zihaf_expected(z);
                if letters[at].beat != expected {
                    assert(false);
                } else {
                    let action = match crate::named::zihaf_action(z) {
                        Operation::Delete => Action::Delete,
                        Operation::Silence => Action::Silence,
                    };
                    assert(action == Action::Silence ==> expected == 1);
                    edit_norm_invariant(
                        units, letters, p, q, at, expected, action,
                    );
                }
            }
        },
        Change::Illa(i) => {
            apply_illa_invariant(units, letters, p, q, i, false);
        },
        Change::NonbindingIlla(i) => {
            apply_illa_invariant(units, letters, p, q, i, true);
        },
        Change::Tashith => {
            match compose_scope(p.scope, Scope::Terminal) {
                Err(_) => assert(false),
                Ok(s) => {
                    compose_scope_not_zihaf(p.scope, Scope::Terminal, s);
                    scope_only_invariant(units, letters, p, s);
                    let r = Normalized {
                        scope: s,
                        required_final: p.required_final,
                        actions: p.actions,
                        expected: p.expected,
                        suffix: p.suffix,
                    };
                    match first_watad(units, 0) {
                        None => assert(false),
                        Some(owner) => {
                            match edit_norm(
                                r,
                                letters,
                                beats_before(units, owner),
                                1,
                                Action::Delete,
                            ) {
                                Err(_) => assert(false),
                                Ok(q1) => {
                                    assert(q1 == q);
                                    reveal(edit_norm);
                                    assert(0 <= beats_before(units, owner) < letters.len());
                                    edit_norm_invariant(
                                        units, letters, r, q1,
                                        beats_before(units, owner), 1, Action::Delete,
                                    );
                                },
                            }
                        },
                    }
                },
            }
        },
    }
}

proof fn normalize_steps_invariant(
    units: Seq<Unit>,
    letters: Seq<m::Letter>,
    p: Normalized,
    changes: Seq<Change>,
    i: int,
)
    requires norm_valid_invariant(units, letters, p),
    ensures match normalize_steps(units, letters, p, changes, i) {
        Err(_) => true,
        Ok(q) => norm_valid_invariant(units, letters, q),
    },
    decreases changes.len() - i,
{
    reveal(normalize_steps);
    if i >= changes.len() {
    } else if seen(changes, i, changes[i]) {
        normalize_steps_invariant(units, letters, p, changes, i + 1);
    } else {
        match apply_change(units, letters, p, changes[i]) {
            Err(_) => {},
            Ok(q) => {
                apply_change_invariant(units, letters, p, q, changes[i]);
                normalize_steps_invariant(units, letters, q, changes, i + 1);
            },
        }
    }
}

proof fn initial_norm_invariant(base: FootId)
    ensures norm_valid_invariant(
        base.word_spec(),
        m::expanded(base.word_spec(), base.word_spec().len() as int),
        initial_norm(base),
    ),
{
    reveal(initial_norm);
}

proof fn normalize_spec_invariant(base: FootId, changes: Seq<Change>, q: Normalized)
    requires normalize_spec(base, changes) == Ok(q),
    ensures norm_valid_invariant(
        base.word_spec(),
        m::expanded(base.word_spec(), base.word_spec().len() as int),
        q,
    ),
{
    reveal(normalize_spec);
    initial_norm_invariant(base);
    normalize_steps_invariant(
        base.word_spec(),
        m::expanded(base.word_spec(), base.word_spec().len() as int),
        initial_norm(base),
        changes,
        0,
    );
}

proof fn foot_word_nonempty(base: FootId)
    ensures base.word_spec().len() > 0,
{
    base.one_watad();
    reveal(watad_types::watad_count);
    if base.word_spec().len() == 0 {
        assert(watad_types::watad_count(base.word_spec(), 0) == 0);
    }
}

/// Successful normalization produces a mechanically valid program whenever the
/// external position is eligible for the resulting scope. This is validity, not
/// profile or source permission.
pub proof fn normalize_spec_valid(
    base: FootId,
    changes: Seq<Change>,
    p: &Program,
    position: m::Position,
)
    requires
        normalize_spec(base, changes) == Ok(program_view(p)),
        m::eligible(p, position),
    ensures m::valid(base.word_spec(), p, position),
{
    let units = base.word_spec();
    let letters = m::expanded(units, units.len() as int);
    normalize_spec_invariant(base, changes, program_view(p));
    foot_word_nonempty(base);
    assert(p.actions@ == program_view(p).actions);
    assert(p.expected@ == program_view(p).expected);
    assert(p.suffix@ == program_view(p).suffix);
    assert(p.scope == program_view(p).scope);
    assert(p.required_final == program_view(p).required_final);
    assert forall|j: int| 0 <= j < letters.len()
        implies m::change_valid(
            letters[j], p.actions@[j], p.expected@[j], p.scope,
        ) by {
        if p.actions@[j] != Action::Keep {
            assert(p.expected@[j] == letters[j].beat);
            assert(p.actions@[j] != Action::Silence || p.expected@[j] == 1);
            assert(p.scope != Scope::Zihaf
                || (letters[j].sabab && letters[j].offset == 1));
        }
    }
    assert(m::valid(units, p, position));
}

fn illa_required_exec(i: Illa) -> (required: Option<Unit>)
    ensures required == crate::named::illa_required(i),
{
    match i {
        Illa::Tarfil | Illa::Tadhyil | Illa::Qat | Illa::Hadhadh => Some(Unit::W),
        Illa::Tasbigh | Illa::Hadhf | Illa::Qatf | Illa::Qasr | Illa::Batr => Some(Unit::S),
        Illa::Salm | Illa::Waqf | Illa::Kasf => Some(Unit::M),
        Illa::Kharm => None,
    }
}

fn edit(
    mut p: Program,
    letters: &[m::Letter],
    at: usize,
    expected: u8,
    action: Action,
) -> (answer: Result<Program, Error>)
    requires p.actions.len() == p.expected.len(),
        p.actions.len() == letters.len(), p.expected.len() == letters.len(),
    ensures match answer {
        Err(e) => edit_norm(program_view(&p), letters@, at as int, expected, action) == Err(e),
        Ok(q) => edit_norm(program_view(&p), letters@, at as int, expected, action) == Ok(program_view(&q))
            && q.actions.len() == letters.len() && q.expected.len() == letters.len(),
    },
{
    let ghost before = program_view(&p);
    if at >= p.actions.len() || at >= p.expected.len() || at >= letters.len() {
        assert(edit_norm(before, letters@, at as int, expected, action) == Err(Error::OutOfRange));
        return Err(Error::OutOfRange);
    }
    if letters[at].beat != expected {
        assert(edit_norm(before, letters@, at as int, expected, action) == Err(Error::Precondition));
        return Err(Error::Precondition);
    }
    let old_action = p.actions[at];
    if old_action != Action::Keep && (old_action != action || p.expected[at] != expected) {
        assert(edit_norm(before, letters@, at as int, expected, action) == Err(Error::Conflict));
        return Err(Error::Conflict);
    }
    p.actions[at] = action;
    p.expected[at] = expected;
    assert(p.actions@ =~= update_action(before.actions, at as int, action));
    assert(p.expected@ =~= update_beat(before.expected, at as int, expected));
    assert(program_view(&p) == Normalized {
        scope: before.scope,
        required_final: before.required_final,
        actions: update_action(before.actions, at as int, action),
        expected: update_beat(before.expected, at as int, expected),
        suffix: before.suffix,
    });
    Ok(p)
}

fn scope_step(a: Scope, b: Scope) -> (answer: Result<Scope, Error>)
    ensures answer == compose_scope(a, b),
{
    match (a, b) {
        (Scope::Zihaf, s) => Ok(s),
        (s, Scope::Zihaf) => Ok(s),
        (Scope::Terminal, Scope::Terminal) => Ok(Scope::Terminal),
        (Scope::Terminal, Scope::Licensed) => Ok(Scope::Terminal),
        (Scope::Licensed, Scope::Terminal) => Ok(Scope::Terminal),
        (Scope::Licensed, Scope::Licensed) => Ok(Scope::Licensed),
        (Scope::Initial, Scope::Initial) => Ok(Scope::Initial),
        _ => Err(Error::Scope),
    }
}

fn set_required_exec(
    mut p: Program,
    units: &[Unit],
    required: Option<Unit>,
) -> (answer: Result<Program, Error>)
    ensures match answer {
        Err(e) => set_required(units@, program_view(&p), required) == Err(e),
        Ok(q) => set_required(units@, program_view(&p), required) == Ok(program_view(&q))
            && q.actions.len() == p.actions.len() && q.expected.len() == p.expected.len(),
    },
{
    if let Some(u) = required {
        if units.is_empty() || units[units.len() - 1] != u {
            return Err(Error::Structure);
        }
        if p.required_final.is_some() && p.required_final != Some(u) {
            return Err(Error::Structure);
        }
        p.required_final = Some(u);
    }
    Ok(p)
}

fn illa_suffix_exec(
    p: Program,
    _units: &[Unit],
    _letters: &[m::Letter],
    i: Illa,
) -> (answer: Result<Program, Error>)
    requires p.actions.len() == _letters.len(), p.expected.len() == _letters.len(),
        matches!(i, Illa::Tarfil | Illa::Tadhyil | Illa::Tasbigh),
    ensures match answer {
        Err(e) => illa_edits(_units@, _letters@, program_view(&p), i) == Err(e),
        Ok(q) => illa_edits(_units@, _letters@, program_view(&p), i) == Ok(program_view(&q))
            && q.actions.len() == _letters.len() && q.expected.len() == _letters.len(),
    },
{
    let mut p = p;
    reveal(illa_edits);
    if !p.suffix.is_empty() {
        return Err(Error::Conflict);
    }
    let ghost before = program_view(&p);
    if i == Illa::Tarfil {
        p.suffix.extend_from_slice(&[1, 0]);
        assert(p.suffix@ == before.suffix + suffix_tarfil());
        assert(program_view(&p) == append_suffix(before, suffix_tarfil()));
    } else {
        p.suffix.push(0);
        assert(p.suffix@ == before.suffix + suffix_sakin());
        assert(program_view(&p) == append_suffix(before, suffix_sakin()));
    }
    Ok(p)
}

fn illa_qat_qasr_exec(
    p: Program,
    _units: &[Unit],
    letters: &[m::Letter],
    _i: Illa,
) -> (answer: Result<Program, Error>)
    requires p.actions.len() == letters.len(), p.expected.len() == letters.len(),
        matches!(_i, Illa::Qat | Illa::Qasr),
    ensures match answer {
        Err(e) => illa_edits(_units@, letters@, program_view(&p), _i) == Err(e),
        Ok(q) => illa_edits(_units@, letters@, program_view(&p), _i) == Ok(program_view(&q))
            && q.actions.len() == letters.len() && q.expected.len() == letters.len(),
    },
{
    reveal(illa_edits);
    let n = letters.len();
    if n < 2 {
        return Err(Error::OutOfRange);
    }
    let p = edit(p, letters, n - 1, 0, Action::Delete)?;
    edit(p, letters, n - 2, 1, Action::Silence)
}

fn illa_hadhadh_salm_exec(
    p: Program,
    _units: &[Unit],
    letters: &[m::Letter],
    _i: Illa,
) -> (answer: Result<Program, Error>)
    requires p.actions.len() == letters.len(), p.expected.len() == letters.len(),
        matches!(_i, Illa::Hadhadh | Illa::Salm),
    ensures match answer {
        Err(e) => illa_edits(_units@, letters@, program_view(&p), _i) == Err(e),
        Ok(q) => illa_edits(_units@, letters@, program_view(&p), _i) == Ok(program_view(&q))
            && q.actions.len() == letters.len() && q.expected.len() == letters.len(),
    },
{
    reveal(illa_edits);
    let n = letters.len();
    if n < 3 {
        return Err(Error::OutOfRange);
    }
    let p = edit(p, letters, n - 3, letters[n - 3].beat, Action::Delete)?;
    let p = edit(p, letters, n - 2, letters[n - 2].beat, Action::Delete)?;
    edit(p, letters, n - 1, letters[n - 1].beat, Action::Delete)
}

fn illa_hadhf_qatf_batr_exec(
    p: Program,
    units: &[Unit],
    letters: &[m::Letter],
    i: Illa,
) -> (answer: Result<Program, Error>)
    requires p.actions.len() == letters.len(), p.expected.len() == letters.len(),
        units.len() <= 3,
        matches!(i, Illa::Hadhf | Illa::Qatf | Illa::Batr),
    ensures match answer {
        Err(e) => illa_edits(units@, letters@, program_view(&p), i) == Err(e),
        Ok(q) => illa_edits(units@, letters@, program_view(&p), i) == Ok(program_view(&q))
            && q.actions.len() == letters.len() && q.expected.len() == letters.len(),
    },
{
    reveal(illa_edits);
    let n = letters.len();
    if n < 2 {
        return Err(Error::OutOfRange);
    }
    let p = edit(p, letters, n - 2, 1, Action::Delete)?;
    let p = edit(p, letters, n - 1, 0, Action::Delete)?;
    let p = if i == Illa::Qatf {
        edit(p, letters, 4, 1, Action::Silence)?
    } else {
        p
    };
    if i != Illa::Batr {
        return Ok(p);
    }
    if units.len() < 2 || units[units.len() - 2] != Unit::W {
        return Err(Error::Structure);
    }
    if n < 3 {
        return Err(Error::OutOfRange);
    }
    let p = edit(p, letters, n - 3, 0, Action::Delete)?;
    if n < 4 {
        return Err(Error::OutOfRange);
    }
    edit(p, letters, n - 4, 1, Action::Silence)
}

fn illa_waqf_kasf_exec(
    p: Program,
    _units: &[Unit],
    letters: &[m::Letter],
    i: Illa,
) -> (answer: Result<Program, Error>)
    requires p.actions.len() == letters.len(), p.expected.len() == letters.len(),
        matches!(i, Illa::Waqf | Illa::Kasf),
    ensures match answer {
        Err(e) => illa_edits(_units@, letters@, program_view(&p), i) == Err(e),
        Ok(q) => illa_edits(_units@, letters@, program_view(&p), i) == Ok(program_view(&q))
            && q.actions.len() == letters.len() && q.expected.len() == letters.len(),
    },
{
    reveal(illa_edits);
    let action = if i == Illa::Waqf {
        Action::Silence
    } else {
        Action::Delete
    };
    edit(p, letters, 6, 1, action)
}

fn illa_kharm_exec(
    p: Program,
    units: &[Unit],
    letters: &[m::Letter],
) -> (answer: Result<Program, Error>)
    requires p.actions.len() == letters.len(), p.expected.len() == letters.len(),
    ensures match answer {
        Err(e) => illa_edits(units@, letters@, program_view(&p), Illa::Kharm) == Err(e),
        Ok(q) => illa_edits(units@, letters@, program_view(&p), Illa::Kharm) == Ok(program_view(&q))
            && q.actions.len() == letters.len() && q.expected.len() == letters.len(),
    },
{
    reveal(illa_edits);
    if units.is_empty() || units[0] != Unit::W {
        return Err(Error::Structure);
    }
    edit(p, letters, 0, 1, Action::Delete)
}

fn apply_illa_exec(
    p: Program,
    units: &[Unit],
    letters: &[m::Letter],
    i: Illa,
    nonbinding: bool,
) -> (answer: Result<Program, Error>)
    requires p.actions.len() == letters.len(), p.expected.len() == letters.len(),
        units.len() <= 3,
    ensures match answer {
        Err(e) => apply_illa(units@, letters@, program_view(&p), i, nonbinding) == Err(e),
        Ok(q) => apply_illa(units@, letters@, program_view(&p), i, nonbinding) == Ok(program_view(&q))
            && q.actions.len() == letters.len() && q.expected.len() == letters.len(),
    },
{
    let desired = if nonbinding {
        Scope::Licensed
    } else if i == Illa::Kharm {
        Scope::Initial
    } else {
        Scope::Terminal
    };
    let mut p = p;
    p.scope = scope_step(p.scope, desired)?;
    let p = set_required_exec(p, units, illa_required_exec(i))?;
    match i {
        Illa::Tarfil | Illa::Tadhyil | Illa::Tasbigh => {
            illa_suffix_exec(p, units, letters, i)
        }
        Illa::Qat | Illa::Qasr => illa_qat_qasr_exec(p, units, letters, i),
        Illa::Hadhadh | Illa::Salm => illa_hadhadh_salm_exec(p, units, letters, i),
        Illa::Hadhf | Illa::Qatf | Illa::Batr => {
            illa_hadhf_qatf_batr_exec(p, units, letters, i)
        }
        Illa::Waqf | Illa::Kasf => illa_waqf_kasf_exec(p, units, letters, i),
        Illa::Kharm => illa_kharm_exec(p, units, letters),
    }
}

fn first_watad_exec(units: &[Unit], i: usize) -> (answer: Option<usize>)
    requires i <= units.len(),
    ensures match (answer, first_watad(units@, i as int)) {
        (None, None) => true,
        (Some(k), Some(j)) => k as int == j && k < units.len(),
        _ => false,
    },
    decreases units.len() - i,
{
    if i >= units.len() {
        None
    } else if units[i] == Unit::W {
        Some(i)
    } else {
        first_watad_exec(units, i + 1)
    }
}



fn apply_change_exec(
    mut p: Program,
    units: &[Unit],
    letters: &[m::Letter],
    change: Change,
) -> (answer: Result<Program, Error>)
    requires p.actions.len() == letters.len(), p.expected.len() == letters.len(),
        units.len() <= 3,
    ensures match answer {
        Err(e) => apply_change(units@, letters@, program_view(&p), change) == Err(e),
        Ok(q) => apply_change(units@, letters@, program_view(&p), change) == Ok(program_view(&q))
            && q.actions.len() == letters.len() && q.expected.len() == letters.len(),
    },
{
    reveal(apply_change);
    match change {
        Change::Unchanged => Ok(p),
        Change::Zihaf(z) => {
            let at = match z {
                Zihaf::Khabn | Zihaf::Idmar | Zihaf::Waqs => 1usize,
                Zihaf::Tayy => 3usize,
                Zihaf::Qabd | Zihaf::Asb | Zihaf::Aql => 4usize,
                Zihaf::Kaf => 6usize,
            };
            if at >= letters.len() {
                return Err(Error::OutOfRange);
            }
            let letter = letters[at];
            if !letter.sabab || letter.offset != 1 {
                return Err(Error::Structure);
            }
            let expected = match z {
                Zihaf::Khabn | Zihaf::Tayy | Zihaf::Qabd | Zihaf::Kaf => 0u8,
                Zihaf::Idmar | Zihaf::Waqs | Zihaf::Asb | Zihaf::Aql => 1u8,
            };
            if letter.beat != expected {
                return Err(Error::Precondition);
            }
            let action = match z {
                Zihaf::Idmar | Zihaf::Asb => Action::Silence,
                _ => Action::Delete,
            };
            p = edit(p, letters, at, expected, action)?;
            Ok(p)
        }
        Change::Illa(i) => apply_illa_exec(p, units, letters, i, false),
        Change::NonbindingIlla(i) => apply_illa_exec(p, units, letters, i, true),
        Change::Tashith => {
            p.scope = scope_step(p.scope, Scope::Terminal)?;
            let owner = match first_watad_exec(units, 0) {
                None => return Err(Error::Structure),
                Some(owner) => {
                    assert(owner < units.len());
                    owner
                }
            };
            assert(units.len() <= 3);
            assert(owner <= 2);
            let mut at: usize = 0;
            let mut k = 0;
            while k < owner
                invariant k <= owner, owner <= units.len(), owner <= 2,
                    units.len() <= 3, k <= 3, at <= 3 * k,
                    at == beats_before(units@, k as int),
                decreases owner - k,
            {
                let size = m::unit_size(units[k]);
                assert(k < owner);
                assert(owner <= 2);
                assert(units.len() <= 3);
                assert(k <= 1);
                assert(size <= 3);
                assert(at + size <= 3 * (k + 1));
                at += size;
                k += 1;
            }
            p = edit(p, letters, at, 1, Action::Delete)?;
            Ok(p)
        }
    }
}

fn is_duplicate(changes: &[Change], i: usize, change: Change) -> (yes: bool)
    requires i <= changes.len(),
    ensures yes == seen(changes@, i as int, change),
{
    let mut j = i;
    while j > 0
        invariant j <= i, i <= changes.len(),
            seen(changes@, i as int, change) == seen(changes@, j as int, change),
        decreases j,
    {
        if changes[j - 1] == change {
            assert(seen(changes@, j as int, change));
            assert(seen(changes@, i as int, change));
            return true;
        }
        assert(seen(changes@, j as int, change)
            == seen(changes@, j as int - 1, change));
        j -= 1;
    }
    assert(!seen(changes@, i as int, change));
    false
}

/// Normalize named recipes against the original base. Exact duplicate changes
/// are idempotent; incompatible scopes, edits, suffixes and preconditions reject.
#[cfg_attr(verus_keep_ghost, verifier::rlimit(105))]
pub fn compile(base: FootId, changes: &[Change]) -> (answer: Result<Program, Error>)
    ensures match answer {
        Err(e) => normalize_spec(base, changes@) == Err(e),
        Ok(p) => normalize_spec(base, changes@) == Ok(program_view(&p)),
    },
{
    let original = crate::adapter::encode_units(base);
    let units = original.as_slice();
    let letters = m::expand(units);
    let n = letters.len();
    let actions = {
        let mut actions = Vec::new();
        let mut k = 0;
        while k < n
            invariant k <= n, actions@.len() == k,
                forall|j: int| 0 <= j < k ==> actions@[j] == Action::Keep,
            decreases n - k,
        {
            actions.push(Action::Keep);
            k += 1;
        }
        actions
    };
    let expected = {
        let mut expected = Vec::new();
        let mut k = 0;
        while k < n
            invariant k <= n, n == letters.len(), expected@.len() == k,
                forall|j: int| 0 <= j < k ==> expected@[j] == letters@[j].beat,
            decreases n - k,
        {
            expected.push(letters[k].beat);
            k += 1;
        }
        expected
    };
    let p = Program {
        scope: Scope::Zihaf,
        required_final: None,
        actions,
        expected,
        suffix: Vec::new(),
    };
    assert(units@ == base.word_spec());
    assert(letters@ == m::expanded(units@, units@.len() as int));
    proof {
        reveal(letters_match);
        reveal(initial_match);
    }
    assert(letters_match(units@, letters@));
    assert(actions@ == Seq::new(n as nat, |j: int| Action::Keep));
    assert(expected@ == Seq::new(n as nat, |j: int| letters@[j].beat));
    assert(program_view(&p) == initial_norm(base));
    let ghost initial = program_view(&p);
    assert(initial_match(base, initial));
    let mut p = Some(p);
    let ghost mut current = initial;
    let mut i = 0;
    assert(normalize_spec(base, changes@) == normalize_steps(
        units@, letters@, initial, changes@, 0,
    ));
    while i < changes.len()
        invariant i <= changes.len(),
            units.len() <= 3,
            units@ == original@,
            original@ == base.word_spec(),
            letters_match(units@, letters@),
            initial_match(base, initial),
            match p {
                Some(q) => program_view(&q) == current
                    && q.actions.len() == letters.len()
                    && q.expected.len() == letters.len(),
                None => false,
            },
            normalize_steps(units@, letters@, initial, changes@, 0)
                == normalize_steps(units@, letters@, current, changes@, i as int),
        decreases changes.len() - i,
    {
        let change = changes[i];
        if change == Change::Unchanged {
            proof {
                normalize_steps_unchanged(units@, letters@, current, changes@, i as int);
            }
            i += 1;
            continue;
        }
        if is_duplicate(changes, i, change) {
            proof {
                normalize_steps_duplicate(units@, letters@, current, changes@, i as int);
            }
            i += 1;
        } else {
            let old = p.take().unwrap();
            match apply_change_exec(old, units, &letters, change) {
                Ok(q) => {
                    proof {
                        normalize_steps_step_ok(
                            units@, letters@, current, program_view(&q), changes@, i as int,
                        );
                    }
                    proof {
                        current = program_view(&q);
                    }
                    p = Some(q);
                    i += 1;
                }
                Err(e) => {
                    proof {
                        normalize_steps_step_err(
                            units@, letters@, current, changes@, i as int, e,
                        );
                    }
                    proof {
                        reveal(letters_match);
                        reveal(initial_match);
                    }
                    assert(letters@ == m::expanded(units@, units@.len() as int));
                    assert(initial == initial_norm(base));
                    assert(normalize_spec(base, changes@) == normalize_steps(
                        units@, letters@, initial_norm(base), changes@, 0,
                    ));
                    assert(normalize_steps(units@, letters@, initial, changes@, 0)
                        == normalize_steps(units@, letters@, initial_norm(base), changes@, 0));
                    assert(normalize_spec(base, changes@) == Err(e));
                    return Err(e);
                }
            }
        }
    }
    assert(i == changes.len());
    let out = p.unwrap();
    assert(program_view(&out) == current);
    assert(units@ == original@);
    assert(original@ == base.word_spec());
    assert(units@ == base.word_spec());
    proof {
        reveal(letters_match);
        reveal(initial_match);
    }
    assert(letters@ == m::expanded(units@, units@.len() as int));
    assert(initial == initial_norm(base));
    proof {
        normalize_steps_done(units@, letters@, current, changes@, i as int);
    }
    assert(normalize_steps(units@, letters@, current, changes@, i as int)
        == Ok(program_view(&out)));
    assert(normalize_spec(base, changes@) == normalize_steps(
        units@, letters@, initial_norm(base), changes@, 0,
    ));
    assert(normalize_steps(units@, letters@, initial, changes@, 0)
        == normalize_steps(units@, letters@, initial_norm(base), changes@, 0));
    assert(normalize_spec(base, changes@) == Ok(program_view(&out)));
    Ok(out)
}

} // verus!

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::OutOfRange => f.write_str("recipe target lies outside the original foot"),
            Error::Conflict => f.write_str("recipes make conflicting edits"),
            Error::Scope => f.write_str("recipes require incompatible structural scopes"),
            Error::Structure => f.write_str("recipe requires another unit decomposition"),
            Error::Precondition => f.write_str("recipe requires a different letter state"),
        }
    }
}

impl core::error::Error for Error {}
