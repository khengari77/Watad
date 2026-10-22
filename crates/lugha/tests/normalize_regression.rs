use watad_checker::morphology::{self as m, Position, Scope};
use watad_lugha::program::{self, Change, Error};
use watad_tahwil::{Illa, LetterPos, Zihaf};
use watad_types::{FootId, Unit};

fn position(scope: Scope) -> Position {
    match scope {
        Scope::Initial => Position {
            initial: true,
            terminal: false,
        },
        Scope::Terminal => Position {
            initial: false,
            terminal: true,
        },
        Scope::Zihaf | Scope::Licensed => Position {
            initial: false,
            terminal: false,
        },
    }
}

fn valid_at(units: &[Unit], p: &m::Program, position: Position) -> bool {
    let letters = m::expand(units);
    let eligible = match p.scope {
        Scope::Zihaf => p.suffix.is_empty(),
        Scope::Terminal => position.terminal,
        Scope::Initial => position.initial && p.suffix.is_empty(),
        Scope::Licensed => true,
    };
    let final_matches = match p.required_final {
        None => true,
        Some(unit) => units.last() == Some(&unit),
    };
    !units.is_empty()
        && eligible
        && final_matches
        && p.actions.len() == letters.len()
        && p.expected.len() == letters.len()
        && letters.iter().enumerate().all(|(i, letter)| {
            p.actions[i] == m::Action::Keep
                || (p.expected[i] == letter.beat
                    && (p.actions[i] != m::Action::Silence || p.expected[i] == 1)
                    && (p.scope != Scope::Zihaf || (letter.sabab && letter.offset == 1)))
        })
        && p.suffix.iter().all(|byte| *byte <= 1)
}

fn declared_output(units: &[Unit], p: &m::Program) -> Vec<u8> {
    let mut word = Vec::new();
    for (letter, action) in m::expand(units).iter().zip(&p.actions) {
        match action {
            m::Action::Keep => word.push(letter.beat),
            m::Action::Delete => {}
            m::Action::Silence => word.push(0),
        }
    }
    word.extend_from_slice(&p.suffix);
    word
}

fn missing_scope_position(scope: Scope) -> Option<Position> {
    match scope {
        Scope::Terminal => Some(Position {
            initial: true,
            terminal: false,
        }),
        Scope::Initial => Some(Position {
            initial: false,
            terminal: true,
        }),
        Scope::Zihaf | Scope::Licensed => None,
    }
}

fn all_changes() -> Vec<Change> {
    let mut changes = vec![Change::Unchanged];
    changes.extend(Zihaf::ALL.map(Change::Zihaf));
    changes.extend(Illa::ALL.map(Change::Illa));
    changes.extend(Illa::ALL.map(Change::NonbindingIlla));
    changes.push(Change::Tashith);
    changes
}

#[test]
fn original_letter_precondition_is_checked_during_normalization() {
    assert_eq!(
        program::compile(FootId::Mafailun, &[Change::Illa(Illa::Qatf)]).unwrap_err(),
        Error::Precondition
    );
}

#[test]
fn letter_positions_are_restricted_to_named_zihaf_positions() {
    for value in 0..=u8::MAX {
        assert_eq!(
            LetterPos::new(value).is_some(),
            matches!(value, 2 | 4 | 5 | 7)
        );
        if let Some(pos) = LetterPos::new(value) {
            assert_eq!(pos.one_based(), value);
        }
    }
}

#[test]
fn same_letter_edits_with_different_operations_conflict() {
    assert_eq!(
        program::compile(
            FootId::Mafalatun,
            &[Change::Zihaf(Zihaf::Aql), Change::Illa(Illa::Qatf)],
        )
        .unwrap_err(),
        Error::Conflict
    );
    assert_eq!(
        program::compile(
            FootId::Mafalatun,
            &[Change::Illa(Illa::Qatf), Change::Zihaf(Zihaf::Aql)],
        )
        .unwrap_err(),
        Error::Conflict
    );
}

#[test]
fn long_unchanged_recipe_is_iterative() {
    let changes = vec![Change::Unchanged; 100_000];
    let p = program::compile(FootId::Mafailun, &changes).unwrap();
    let width = m::expand(FootId::Mafailun.units()).len();
    assert_eq!(p.actions.len(), width);
    assert_eq!(p.expected.len(), width);
    assert!(p.actions.iter().all(|a| *a == m::Action::Keep));
    assert!(p.suffix.is_empty());
}

#[test]
fn long_identical_tarfil_recipe_adds_one_suffix() {
    let changes = vec![Change::Illa(Illa::Tarfil); 100_000];
    let p = program::compile(FootId::MustafIlun, &changes).unwrap();
    assert_eq!(p.suffix, vec![1, 0]);
    assert_eq!(p.scope, Scope::Terminal);
}

#[test]
fn incompatible_suffix_producers_and_terminal_scope_steps_reject() {
    let pairs = [
        (Illa::Tarfil, Illa::Tadhyil, Error::Conflict),
        (Illa::Tadhyil, Illa::Tarfil, Error::Conflict),
        (Illa::Tarfil, Illa::Tasbigh, Error::Structure),
        (Illa::Tasbigh, Illa::Tarfil, Error::Structure),
    ];
    for (first, second, expected) in pairs {
        assert_eq!(
            program::compile(
                FootId::MustafIlun,
                &[Change::Illa(first), Change::Illa(second)],
            )
            .unwrap_err(),
            expected,
            "{first:?} {second:?}"
        );
    }
    for (first, second) in [
        (
            Change::Illa(Illa::Tarfil),
            Change::NonbindingIlla(Illa::Qat),
        ),
        (
            Change::NonbindingIlla(Illa::Qat),
            Change::Illa(Illa::Tarfil),
        ),
    ] {
        let p = program::compile(FootId::MustafIlun, &[first, second]).unwrap();
        assert_eq!(p.scope, Scope::Terminal, "{first:?} {second:?}");
        assert!(
            m::apply(
                FootId::MustafIlun.units(),
                &p,
                Position {
                    initial: false,
                    terminal: true
                }
            )
            .is_ok()
        );
    }
}

#[test]
fn successful_single_and_paired_changes_reach_morphology_on_every_base() {
    let changes = all_changes();
    assert_eq!(changes.len(), 36);
    let mut singles = 0;
    let mut pairs = 0;
    let mut accepted = 0;
    for id in 0..11 {
        let base = FootId::from_id(id);
        let width = m::expand(base.units()).len();
        for c in &changes {
            singles += 1;
            if let Ok(p) = program::compile(base, &[*c]) {
                accepted += 1;
                assert_eq!(p.actions.len(), width, "{base:?} {c:?}");
                assert_eq!(p.expected.len(), width, "{base:?} {c:?}");
                assert!(p.suffix.iter().all(|byte| *byte <= 1), "{base:?} {c:?}");
                let compatible = position(p.scope);
                assert!(valid_at(base.units(), &p, compatible), "{base:?} {c:?}");
                assert!(
                    m::apply(base.units(), &p, compatible).is_ok(),
                    "{base:?} {c:?}"
                );
                if let Some(incompatible) = missing_scope_position(p.scope) {
                    assert!(!valid_at(base.units(), &p, incompatible), "{base:?} {c:?}");
                }
            }
        }
        for a in &changes {
            for b in &changes {
                pairs += 1;
                if let Ok(p) = program::compile(base, &[*a, *b]) {
                    accepted += 1;
                    assert_eq!(p.actions.len(), width, "{base:?} {a:?} {b:?}");
                    assert_eq!(p.expected.len(), width, "{base:?} {a:?} {b:?}");
                    assert!(
                        p.suffix.iter().all(|byte| *byte <= 1),
                        "{base:?} {a:?} {b:?}"
                    );
                    let compatible = position(p.scope);
                    assert!(
                        valid_at(base.units(), &p, compatible),
                        "{base:?} {a:?} {b:?}"
                    );
                    assert!(
                        m::apply(base.units(), &p, compatible).is_ok(),
                        "{base:?} {a:?} {b:?}"
                    );
                    if let Some(incompatible) = missing_scope_position(p.scope) {
                        assert!(
                            !valid_at(base.units(), &p, incompatible),
                            "{base:?} {a:?} {b:?}"
                        );
                    }
                }
            }
        }
    }
    assert_eq!(singles, 36 * 11);
    assert_eq!(pairs, 14_256);
    assert!(accepted > 0);
}

#[test]
fn representative_successful_recipes_preserve_shape_and_scope() {
    let cases: &[(FootId, &[Change], Scope, &[u8])] = &[
        (
            FootId::Mafailun,
            &[Change::Zihaf(Zihaf::Qabd)],
            Scope::Zihaf,
            &[],
        ),
        (
            FootId::MustafIlun,
            &[Change::Illa(Illa::Tarfil)],
            Scope::Terminal,
            &[1, 0],
        ),
        (
            FootId::MustafIlun,
            &[Change::NonbindingIlla(Illa::Tarfil)],
            Scope::Licensed,
            &[1, 0],
        ),
        (FootId::Mafailun, &[Change::Tashith], Scope::Terminal, &[]),
        (
            FootId::Mafailun,
            &[Change::Illa(Illa::Tasbigh)],
            Scope::Terminal,
            &[0],
        ),
    ];
    for &(base, changes, scope, suffix) in cases {
        let width = m::expand(base.units()).len();
        let p = program::compile(base, changes)
            .unwrap_or_else(|error| panic!("{base:?} {changes:?}: {error:?}"));
        assert_eq!(p.actions.len(), width, "{base:?} {changes:?}");
        assert_eq!(p.expected.len(), width, "{base:?} {changes:?}");
        assert_eq!(p.scope, scope, "{base:?} {changes:?}");
        assert_eq!(p.suffix.as_slice(), suffix, "{base:?} {changes:?}");
        assert!(
            p.suffix.iter().all(|byte| *byte <= 1),
            "{base:?} {changes:?}"
        );
        let compatible = position(p.scope);
        assert!(
            valid_at(base.units(), &p, compatible),
            "{base:?} {changes:?}"
        );
        assert!(
            m::apply(base.units(), &p, compatible).is_ok(),
            "{base:?} {changes:?}"
        );
        match p.scope {
            Scope::Terminal | Scope::Initial => {
                let incompatible = Position {
                    initial: false,
                    terminal: false,
                };
                assert!(
                    !valid_at(base.units(), &p, incompatible),
                    "{base:?} {changes:?}"
                );
                assert!(
                    m::apply(base.units(), &p, incompatible).is_err(),
                    "{base:?} {changes:?}"
                );
            }
            Scope::Zihaf | Scope::Licensed => {}
        }
    }
}

type ValidRecipeCase = (
    FootId,
    &'static [Change],
    Scope,
    &'static [u8],
    Option<(usize, m::Action, u8)>,
);

#[test]
fn representative_valid_recipes_check_actions_beats_and_suffixes() {
    let cases: &[ValidRecipeCase] = &[
        (
            FootId::Failatun,
            &[Change::Tashith],
            Scope::Terminal,
            &[],
            Some((2, m::Action::Delete, 1)),
        ),
        (
            FootId::Mafailun,
            &[Change::Illa(Illa::Kharm)],
            Scope::Initial,
            &[],
            Some((0, m::Action::Delete, 1)),
        ),
        (
            FootId::MustafIlun,
            &[Change::Illa(Illa::Tarfil)],
            Scope::Terminal,
            &[1, 0],
            None,
        ),
        (
            FootId::Mafailun,
            &[Change::Illa(Illa::Tasbigh)],
            Scope::Terminal,
            &[0],
            None,
        ),
        (
            FootId::MustafIlun,
            &[Change::NonbindingIlla(Illa::Tarfil)],
            Scope::Licensed,
            &[1, 0],
            None,
        ),
        (
            FootId::Mafailun,
            &[Change::Zihaf(Zihaf::Qabd)],
            Scope::Zihaf,
            &[],
            Some((4, m::Action::Delete, 0)),
        ),
    ];
    for &(base, changes, scope, suffix, edit) in cases {
        let letters = m::expand(base.units());
        let p = program::compile(base, changes)
            .unwrap_or_else(|error| panic!("{base:?} {changes:?}: {error:?}"));
        assert_eq!(p.scope, scope, "{base:?} {changes:?}");
        assert_eq!(p.suffix.as_slice(), suffix, "{base:?} {changes:?}");
        assert_eq!(p.actions.len(), letters.len(), "{base:?} {changes:?}");
        assert_eq!(p.expected.len(), letters.len(), "{base:?} {changes:?}");

        let compatible = position(p.scope);
        assert!(
            valid_at(base.units(), &p, compatible),
            "{base:?} {changes:?}"
        );
        let word = m::apply(base.units(), &p, compatible).unwrap();
        assert_eq!(
            word,
            declared_output(base.units(), &p),
            "{base:?} {changes:?}"
        );

        match edit {
            Some((index, action, beat)) => {
                assert_eq!(p.actions[index], action, "{base:?} {changes:?}");
                assert_eq!(p.expected[index], beat, "{base:?} {changes:?}");
                assert_eq!(letters[index].beat, beat, "{base:?} {changes:?}");
                for (i, letter) in letters.iter().enumerate() {
                    if i != index {
                        assert_eq!(p.actions[i], m::Action::Keep, "{base:?} {changes:?}");
                        assert_eq!(p.expected[i], letter.beat, "{base:?} {changes:?}");
                    }
                }
            }
            None => {
                assert!(
                    p.actions.iter().all(|action| *action == m::Action::Keep),
                    "{base:?} {changes:?}"
                );
                assert_eq!(
                    p.expected,
                    letters.iter().map(|letter| letter.beat).collect::<Vec<_>>(),
                    "{base:?} {changes:?}"
                );
            }
        }

        if let Some(incompatible) = missing_scope_position(p.scope) {
            assert!(
                !valid_at(base.units(), &p, incompatible),
                "{base:?} {changes:?}"
            );
            assert!(
                !valid_at(
                    base.units(),
                    &p,
                    Position {
                        initial: false,
                        terminal: false,
                    },
                ),
                "{base:?} {changes:?}"
            );
            assert!(
                m::apply(base.units(), &p, incompatible).is_err(),
                "{base:?} {changes:?}"
            );
        }
    }

    let initial = program::compile(FootId::Mafailun, &[Change::Illa(Illa::Kharm)]).unwrap();
    assert!(
        m::apply(
            FootId::Mafailun.units(),
            &initial,
            Position {
                initial: false,
                terminal: true,
            },
        )
        .is_err()
    );

    let terminal = program::compile(FootId::Failatun, &[Change::Tashith]).unwrap();
    assert!(
        m::apply(
            FootId::Failatun.units(),
            &terminal,
            Position {
                initial: true,
                terminal: false,
            },
        )
        .is_err()
    );
}

#[test]
fn required_final_and_suffix_stay_consistent_with_scopes() {
    for id in 0..11 {
        let base = FootId::from_id(id);
        for illa in Illa::ALL {
            if let Ok(p) = program::compile(base, &[Change::Illa(illa)]) {
                let compatible = position(p.scope);
                assert!(
                    m::apply(base.units(), &p, compatible).is_ok(),
                    "{base:?} {illa:?}"
                );
                assert!(valid_at(base.units(), &p, compatible), "{base:?} {illa:?}");
                if p.scope != Scope::Zihaf {
                    let incompatible = match p.scope {
                        Scope::Initial => Position {
                            initial: false,
                            terminal: false,
                        },
                        Scope::Terminal => Position {
                            initial: false,
                            terminal: false,
                        },
                        Scope::Zihaf | Scope::Licensed => unreachable!(),
                    };
                    assert!(
                        m::apply(base.units(), &p, incompatible).is_err(),
                        "{base:?} {illa:?}"
                    );
                    assert!(
                        !valid_at(base.units(), &p, incompatible),
                        "{base:?} {illa:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn a_nonbinding_tarfil_keeps_the_licensed_scope_for_later_zihaf() {
    let p = program::compile(
        FootId::MustafIlun,
        &[
            Change::NonbindingIlla(Illa::Tarfil),
            Change::Zihaf(Zihaf::Khabn),
        ],
    )
    .unwrap();
    assert_eq!(p.scope, Scope::Licensed);
    assert_eq!(p.suffix, vec![1, 0]);
    assert!(matches!(p.actions[1], m::Action::Delete));
    assert!(
        m::apply(
            FootId::MustafIlun.units(),
            &p,
            Position {
                initial: false,
                terminal: false
            }
        )
        .is_ok()
    );
}

#[test]
fn out_of_range_structure_and_scope_failures_are_distinct() {
    assert_eq!(
        program::compile(FootId::FaIlun, &[Change::Zihaf(Zihaf::Kaf)]).unwrap_err(),
        Error::OutOfRange
    );
    assert_eq!(
        program::compile(
            FootId::Mafailun,
            &[Change::Illa(Illa::Kharm), Change::Illa(Illa::Tarfil)],
        )
        .unwrap_err(),
        Error::Scope
    );
    assert_eq!(
        program::compile(FootId::Mafailun, &[Change::Illa(Illa::Batr)]).unwrap_err(),
        Error::Structure
    );
}
