//! Generated invariants and adversarial traces, with shrinkable persisted seeds.
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use watad_daira::{CircleId, MeterForm, TerminalRole, TerminalShape};
use watad_lugha::derivation::{DerivationError, check_structural_trace, derive_structural_foot};
use watad_lugha::{CompoundZihaf, Illa, Op, Zihaf};
use watad_types::{Beat, Foot, FootId};

fn operation() -> impl Strategy<Value = Option<Op>> {
    let mut all = vec![None];
    all.extend(Zihaf::ALL.map(|z| Some(Op::Zihaf(z))));
    all.extend(CompoundZihaf::ALL.map(|c| Some(Op::Compound(c))));
    all.extend(Illa::ALL.map(|i| Some(Op::Illa(i))));
    proptest::sample::select(all)
}

fn generated_form() -> impl Strategy<Value = MeterForm> {
    (
        prop::collection::vec(
            (0usize..5, any::<usize>(), prop_oneof![1usize..5, Just(255)]),
            1..=4,
        ),
        0usize..5,
    )
        .prop_map(|(segments, shape)| {
            let segments: Vec<_> = segments
                .into_iter()
                .map(|(c, r, reps)| {
                    let circle = CircleId::from_id(c);
                    (circle, r % circle.period(), reps)
                })
                .collect();
            let shape = [
                TerminalShape::Tamam,
                TerminalShape::Majzu,
                TerminalShape::Maqsor,
                TerminalShape::Manhuk,
                TerminalShape::Mansur,
            ][shape];
            MeterForm::new(&segments, shape).unwrap()
        })
}

fn terminal(darb: bool) -> TerminalRole {
    if darb {
        TerminalRole::Darb
    } else {
        TerminalRole::Arud
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/properties.proptest-regressions")
        ))),
        .. ProptestConfig::default()
    })]

    #[test]
    fn beat_projection_preserves_actual_workspace_values(
        letters in prop::collection::vec(any::<bool>(), 0..257),
    ) {
        use watad_checker::adapter;
        let beats: Vec<_> = letters.iter().map(|moving| if *moving { Beat::Mutaharrik } else { Beat::Sakin }).collect();
        let codes = adapter::encode_beats(&beats);
        prop_assert_eq!(codes.len(), beats.len());
        for (&beat, &code) in beats.iter().zip(&codes) {
            prop_assert_eq!(code, if beat.symbol() == '/' { 1 } else { 0 });
        }
        let mut changed = beats.clone();
        if let Some(first) = changed.first_mut() {
            *first = if *first == Beat::Sakin { Beat::Mutaharrik } else { Beat::Sakin };
        } else {
            changed.push(Beat::Sakin);
        }
        prop_assert_ne!(&codes, &adapter::encode_beats(&changed));
    }

    #[test]
    fn letter_origins_reconstruct_original_indices(id in 0usize..11, input in any::<usize>()) {
        let foot = Foot::of(FootId::from_id(id));
        let total: usize = foot.units().iter().map(|u| u.beat_count()).sum();
        let index = input % total;
        let origin = foot.unit_at_letter(index).unwrap();
        let prefix: usize = foot.units()[..origin.unit_index].iter().map(|u| u.beat_count()).sum();
        prop_assert_eq!(prefix + origin.offset, index);
        prop_assert_eq!(origin.unit, foot.units()[origin.unit_index]);
        prop_assert!(origin.offset < origin.unit.beat_count());
        prop_assert!(foot.unit_at_letter(total.saturating_add(input)).is_none());
    }

    #[test]
    fn generated_structural_successes_replay(f in generated_form(), index in any::<usize>(), darb in any::<bool>(), op in operation()) {
        let index = index % f.foot_count();
        let terminal = terminal(darb);
        if let Ok(derived) = derive_structural_foot(&f, terminal, index, op) {
            let replayed = check_structural_trace(&f, terminal, derived.trace()).unwrap();
            prop_assert_eq!(derived.beats(), replayed.beats());
            prop_assert_eq!(derived.position(), replayed.position());
            prop_assert_eq!(derived.trace().base, f.foot_at(index));
            prop_assert_eq!(derived.position().endpoint, if index == f.terminal_hook() { Some(terminal) } else { None });
            let simple: Vec<_> = match op {
                Some(Op::Zihaf(z)) => vec![z],
                Some(Op::Compound(c)) => c.components().to_vec(),
                _ => vec![],
            };
            for z in simple {
                let origin = Foot::of(derived.trace().base).unit_at_letter(z.position().one_based() as usize - 1).unwrap();
                prop_assert!(origin.unit.is_sabab());
                prop_assert_eq!(origin.offset, 1);
            }
        }
    }

    #[test]
    fn altered_output_never_replays(f in generated_form(), index in any::<usize>(), darb in any::<bool>(), mutation in 0u8..3) {
        let terminal = terminal(darb);
        let mut trace = derive_structural_foot(&f, terminal, index % f.foot_count(), None).unwrap().into_trace();
        match mutation {
            0 => {
                let at = index % trace.output.len();
                trace.output[at] = if trace.output[at] == Beat::Sakin { Beat::Mutaharrik } else { Beat::Sakin };
            }
            1 => trace.output = trace.output[..trace.output.len() - 1].into(),
            _ => trace.output = trace.output.iter().copied().chain([Beat::Sakin]).collect(),
        }
        prop_assert_eq!(check_structural_trace(&f, terminal, &trace).unwrap_err(), DerivationError::OutputMismatch);
    }

    #[test]
    fn forged_identity_or_context_never_replays(f in generated_form(), index in any::<usize>(), darb in any::<bool>(), mutation in 0u8..3) {
        let terminal = terminal(darb);
        let mut trace = derive_structural_foot(&f, terminal, index % f.foot_count(), None).unwrap().into_trace();
        match mutation {
            0 => trace.base = FootId::from_id((trace.base.id() + 1) % 11),
            1 => trace.terminal = if darb { TerminalRole::Arud } else { TerminalRole::Darb },
            _ => trace.foot_index = usize::MAX,
        }
        prop_assert!(check_structural_trace(&f, terminal, &trace).is_err());
    }

}
