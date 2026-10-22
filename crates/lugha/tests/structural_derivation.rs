use watad_daira::{Circle, CircleId, MeterForm, TerminalRole, TerminalShape};
use watad_lugha::derivation::{
    DerivationError, FootTrace, check_structural_trace, derive_structural_foot,
};
use watad_lugha::{CompoundZihaf, Illa, IllaError, Op, Zihaf, ZihafError, beats_of, pattern};
use watad_types::{Beat, Foot, FootId};

fn form(circle: CircleId, rotation: usize, reps: usize) -> MeterForm {
    MeterForm::new(&[(circle, rotation, reps)], TerminalShape::Tamam).unwrap()
}

#[test]
fn derived_operations_keep_original_coordinates_and_position() {
    let rajaz = form(CircleId::Mujtalab, 1, 3);
    let derived =
        derive_structural_foot(&rajaz, TerminalRole::Darb, 1, Some(Op::Zihaf(Zihaf::Khabn)))
            .unwrap();
    assert_eq!(pattern(derived.beats()), "//0//0");
    assert_eq!(derived.trace().base, FootId::MustafIlun);
    assert_eq!(derived.position().foot_index, 1);
    assert_eq!(derived.position().foot_count, 3);
    assert!(!derived.position().is_initial);
    assert_eq!(derived.position().endpoint, None);
    let replayed = check_structural_trace(&rajaz, TerminalRole::Darb, derived.trace()).unwrap();
    assert_eq!(replayed.beats(), derived.beats());

    let kamil = form(CircleId::Muatalaf, 1, 1);
    let compound = derive_structural_foot(
        &kamil,
        TerminalRole::Arud,
        0,
        Some(Op::Compound(CompoundZihaf::Khazl)),
    )
    .unwrap();
    // Silence original letter 2 and delete original letter 4 simultaneously.
    assert_eq!(pattern(compound.beats()), "/0///0");
    check_structural_trace(&kamil, TerminalRole::Arud, compound.trace()).unwrap();
}

#[test]
fn unchanged_feet_round_trip_across_the_catalog() {
    for c in 0..5 {
        let circle = Circle::of(CircleId::from_id(c));
        for rotation in 0..circle.period() {
            let f = circle.rotate(rotation).form();
            for terminal in [TerminalRole::Arud, TerminalRole::Darb] {
                for i in 0..f.foot_count() {
                    let derived = derive_structural_foot(&f, terminal, i, None).unwrap();
                    assert_eq!(derived.beats(), beats_of(Foot::of(f.foot_at(i))).as_ref());
                    assert_eq!(derived.position().is_initial, i == 0);
                    assert_eq!(
                        derived.position().endpoint,
                        if i == f.terminal_hook() {
                            Some(terminal)
                        } else {
                            None
                        }
                    );
                    let trace = derived.into_trace();
                    check_structural_trace(&f, terminal, &trace).unwrap();
                }
            }
        }
    }
}

#[test]
fn raw_success_does_not_authorize_a_watad_change() {
    // This form ends in فاعِ لاتن (M S S). Raw khabn can delete a watad letter.
    let f = form(CircleId::Mushtabih, 2, 1);
    let index = f.terminal_hook();
    assert_eq!(f.foot_at(index), FootId::FailatunMafruq);
    for operation in [Op::Zihaf(Zihaf::Khabn), Op::Compound(CompoundZihaf::Shakl)] {
        let raw = operation.apply(FootId::FailatunMafruq).unwrap();
        let trace = FootTrace {
            form: f,
            terminal: TerminalRole::Darb,
            foot_index: index,
            base: FootId::FailatunMafruq,
            operation: Some(operation),
            output: raw,
        };
        assert!(matches!(
            check_structural_trace(&f, TerminalRole::Darb, &trace),
            Err(DerivationError::Zihaf(ZihafError::TargetsWatad {
                position: 2,
                ..
            }))
        ));
    }
}

#[test]
fn range_and_state_errors_survive_the_bound_api() {
    let short = form(CircleId::Muttafiq, 0, 1);
    assert!(matches!(
        derive_structural_foot(&short, TerminalRole::Darb, 0, Some(Op::Zihaf(Zihaf::Kaf))),
        Err(DerivationError::Zihaf(ZihafError::OutOfRange {
            position: 7,
            len: 5,
            ..
        }))
    ));
    let f = form(CircleId::Mujtalab, 1, 1);
    assert!(matches!(
        derive_structural_foot(&f, TerminalRole::Darb, 0, Some(Op::Illa(Illa::Waqf))),
        Err(DerivationError::Illa(IllaError::Precondition {
            position: 7,
            ..
        }))
    ));
}

#[test]
fn endpoint_selection_never_promotes_a_nonterminal_foot() {
    let f = form(CircleId::Mujtalab, 1, 3);
    for terminal in [TerminalRole::Arud, TerminalRole::Darb] {
        for index in [0, 1] {
            assert_eq!(
                derive_structural_foot(&f, terminal, index, Some(Op::Illa(Illa::Qat))).unwrap_err(),
                DerivationError::IllaPosition {
                    illa: Illa::Qat,
                    foot_index: index,
                }
            );
        }
        let derived = derive_structural_foot(&f, terminal, 2, Some(Op::Illa(Illa::Qat))).unwrap();
        assert_eq!(pattern(derived.beats()), "/0/0/0");
    }

    let single = form(CircleId::Mujtalab, 1, 1);
    let derived =
        derive_structural_foot(&single, TerminalRole::Arud, 0, Some(Op::Illa(Illa::Qat))).unwrap();
    assert!(derived.position().is_initial);
    assert_eq!(derived.position().endpoint, Some(TerminalRole::Arud));
}

#[test]
fn resolved_terminal_and_initial_illa_have_distinct_position_gates() {
    let f = form(CircleId::Muttafiq, 0, 3);
    for illa in [Illa::Batr, Illa::Qasr] {
        for terminal in [TerminalRole::Arud, TerminalRole::Darb] {
            for i in 0..f.foot_count() - 1 {
                assert_eq!(
                    derive_structural_foot(&f, terminal, i, Some(Op::Illa(illa))).unwrap_err(),
                    DerivationError::IllaPosition {
                        illa,
                        foot_index: i
                    }
                );
            }
            assert!(
                derive_structural_foot(&f, terminal, f.foot_count() - 1, Some(Op::Illa(illa)))
                    .is_ok()
            );
        }
    }
    assert!(derive_structural_foot(&f, TerminalRole::Darb, 0, Some(Op::Illa(Illa::Kharm))).is_ok());
    assert!(matches!(
        derive_structural_foot(&f, TerminalRole::Darb, 1, Some(Op::Illa(Illa::Kharm))),
        Err(DerivationError::IllaPosition { .. })
    ));
    let single = form(CircleId::Muttafiq, 0, 1);
    assert!(
        derive_structural_foot(&single, TerminalRole::Darb, 0, Some(Op::Illa(Illa::Kharm))).is_ok()
    );
}

#[test]
fn both_derivation_and_replay_reject_hostile_indices() {
    let f = form(CircleId::Mujtalab, 1, 1);
    let mut trace = derive_structural_foot(&f, TerminalRole::Darb, 0, None)
        .unwrap()
        .into_trace();
    for index in [f.foot_count(), usize::MAX] {
        let expected = DerivationError::FootOutOfRange {
            index,
            foot_count: 1,
        };
        assert_eq!(
            derive_structural_foot(&f, TerminalRole::Darb, index, None).unwrap_err(),
            expected
        );
        trace.foot_index = index;
        assert_eq!(
            check_structural_trace(&f, TerminalRole::Darb, &trace).unwrap_err(),
            expected
        );
    }
}

#[test]
fn equal_surface_does_not_allow_a_forged_base_identity() {
    let f = form(CircleId::Mujtalab, 1, 1);
    let mut trace = derive_structural_foot(&f, TerminalRole::Darb, 0, None)
        .unwrap()
        .into_trace();
    trace.base = FootId::MustafIlunMafruq;
    assert_eq!(trace.output, beats_of(Foot::MUSTAF_ILUN_MAFRUQ));
    assert_eq!(
        check_structural_trace(&f, TerminalRole::Darb, &trace).unwrap_err(),
        DerivationError::BaseMismatch {
            expected: FootId::MustafIlun,
            claimed: FootId::MustafIlunMafruq,
        }
    );
}

#[test]
fn output_and_operation_claims_are_replayed() {
    let f = form(CircleId::Mujtalab, 1, 1);
    let trace = derive_structural_foot(&f, TerminalRole::Darb, 0, None)
        .unwrap()
        .into_trace();
    let mut changed = trace.clone();
    changed.output[0] = Beat::Sakin;
    assert_eq!(
        check_structural_trace(&f, TerminalRole::Darb, &changed).unwrap_err(),
        DerivationError::OutputMismatch
    );
    for output in [
        Box::<[Beat]>::default(),
        trace.output[..trace.output.len() - 1].into(),
        trace.output.iter().copied().chain([Beat::Sakin]).collect(),
    ] {
        changed = trace.clone();
        changed.output = output;
        assert_eq!(
            check_structural_trace(&f, TerminalRole::Darb, &changed).unwrap_err(),
            DerivationError::OutputMismatch
        );
    }
    changed = trace;
    changed.operation = Some(Op::Zihaf(Zihaf::Khabn));
    assert_eq!(
        check_structural_trace(&f, TerminalRole::Darb, &changed).unwrap_err(),
        DerivationError::OutputMismatch
    );
}

#[test]
fn replay_requires_the_expected_form_label_and_terminal_role() {
    let f = form(CircleId::Mujtalab, 1, 1);
    let trace = derive_structural_foot(&f, TerminalRole::Darb, 0, None)
        .unwrap()
        .into_trace();
    let longer = form(CircleId::Mujtalab, 1, 2);
    assert_eq!(
        check_structural_trace(&longer, TerminalRole::Darb, &trace).unwrap_err(),
        DerivationError::FormMismatch
    );
    let relabeled = MeterForm::new(&[(CircleId::Mujtalab, 1, 1)], TerminalShape::Majzu).unwrap();
    assert_eq!(
        check_structural_trace(&relabeled, TerminalRole::Darb, &trace).unwrap_err(),
        DerivationError::FormMismatch
    );
    assert_eq!(
        check_structural_trace(&f, TerminalRole::Arud, &trace).unwrap_err(),
        DerivationError::TerminalMismatch
    );
}

#[test]
fn form_comparison_ignores_unused_storage_padding() {
    let generated = Circle::of(CircleId::Mujtalab).rotate(1).form();
    let constructed = form(CircleId::Mujtalab, 1, 3);
    let trace = derive_structural_foot(&generated, TerminalRole::Darb, 0, None)
        .unwrap()
        .into_trace();
    // The two constructors currently pad their private segment arrays differently.
    check_structural_trace(&constructed, TerminalRole::Darb, &trace).unwrap();
}
