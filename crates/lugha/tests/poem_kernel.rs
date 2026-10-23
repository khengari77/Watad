use proptest::prelude::*;
use watad_checker::{language::Slot, poem, verse::*};

fn form(class: u16, a: &[u8], b: &[u8]) -> Form {
    Form {
        context: 0,
        initial_only: false,
        class: DarbClass(class),
        sadr: vec![Slot {
            words: vec![a.to_vec()],
        }],
        ajuz: vec![Slot {
            words: vec![b.to_vec()],
        }],
        sadr_rules: vec![],
        ajuz_rules: vec![],
    }
}

#[test]
fn complete_class_search_avoids_a_greedy_first_verse_decision() {
    let mut p = Profile {
        id: ProfileId {
            family: 10,
            revision: 1,
        },
        binding: BindingPolicy::FixedDarbClass,
        forms: vec![
            form(0, &[1], &[0]),
            form(1, &[1], &[0]),
            form(1, &[1, 1], &[0]),
        ],
    };
    let input = [
        poem::Input {
            context: 0,
            sadr: &[1],
            ajuz: &[0],
        },
        poem::Input {
            context: 0,
            sadr: &[1, 1],
            ajuz: &[0],
        },
    ];
    let mut certificates = poem::recognize(&p, &input).unwrap().unwrap();
    assert!(poem::check(&p, &input, &certificates));
    assert!(
        certificates
            .iter()
            .all(|c| p.forms[c.form].class == DarbClass(1))
    );
    certificates[0].form = 0;
    assert!(!poem::check(&p, &input, &certificates));
    p.binding = BindingPolicy::Unconstrained;
    assert!(poem::check(&p, &input, &certificates));
    assert_eq!(poem::recognize(&p, &[]).err(), Some(poem::Error::EmptyPoem));
}

#[test]
fn first_verse_only_forms_cannot_be_replayed_later_in_a_poem() {
    let mut opening = form(7, &[1, 0], &[1]);
    opening.context = 99;
    opening.initial_only = true;
    let p = Profile {
        id: ProfileId {
            family: 11,
            revision: 1,
        },
        binding: BindingPolicy::FixedDarbClass,
        forms: vec![opening, form(7, &[1], &[0])],
    };
    let first = poem::Input {
        context: 99,
        sadr: &[1, 0],
        ajuz: &[1],
    };
    let ordinary = poem::Input {
        context: 0,
        sadr: &[1],
        ajuz: &[0],
    };
    assert!(poem::recognize(&p, &[first, ordinary]).unwrap().is_some());
    let ordinary = poem::Input {
        context: 0,
        sadr: &[1],
        ajuz: &[0],
    };
    let misplaced = poem::Input {
        context: 99,
        sadr: &[1, 0],
        ajuz: &[1],
    };
    assert!(
        poem::recognize(&p, &[ordinary, misplaced])
            .unwrap()
            .is_none()
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/poem_kernel.proptest-regressions")
        ))),
        .. ProptestConfig::default()
    })]

    #[test]
    fn poem_inference_equals_independent_compatible_class_intersection(
        raw in prop::collection::vec((0u16..3, any::<bool>(), 0u16..2,
            prop::collection::vec(0u8..2, 1..4), prop::collection::vec(0u8..2, 1..4)), 1..7),
        selections in prop::collection::vec(any::<usize>(), 1..8),
        fixed in any::<bool>(),
    ) {
        let mut forms = Vec::new();
        for (class, initial, context, a, b) in &raw {
            let mut f = form(*class, a, b);
            f.initial_only = *initial;
            f.context = *context;
            forms.push(f);
        }
        let p = Profile { id: ProfileId { family: 12, revision: 1 }, forms,
            binding: if fixed { BindingPolicy::FixedDarbClass } else { BindingPolicy::Unconstrained } };
        let input: Vec<_> = selections.iter().map(|index| {
            let r = &raw[index % raw.len()];
            poem::Input { context: r.2, sadr: &r.3, ajuz: &r.4 }
        }).collect();
        let compatible = |line: usize, class: Option<u16>| raw.iter().any(|r|
            class.is_none_or(|c| c == r.0) && (!r.1 || line == 0)
            && r.2 == input[line].context && r.3 == input[line].sadr && r.4 == input[line].ajuz);
        let expected = if fixed {
            (0..3).any(|class| (0..input.len()).all(|line| compatible(line, Some(class))))
        } else {
            (0..input.len()).all(|line| compatible(line, None))
        };
        let result = poem::recognize(&p, &input).unwrap();
        prop_assert_eq!(result.is_some(), expected);
        if let Some(c) = result {
            prop_assert!(poem::check(&p, &input, &c));
            prop_assert!(!poem::check(&p, &input, &c[..c.len() - 1]));
        }
    }
}
