//! Independent whole-word enumeration as a reference for the proved recognizer.
use proptest::prelude::*;
use watad_checker::language::{Slot, check, decide, recognize, word_matches};

fn enumerate(slots: &[Vec<Vec<u8>>]) -> Vec<Vec<u8>> {
    let mut words = vec![vec![]];
    for alternatives in slots {
        words = words
            .iter()
            .flat_map(|prefix| {
                alternatives
                    .iter()
                    .map(move |suffix| prefix.iter().chain(suffix).copied().collect())
            })
            .collect();
    }
    words
}

#[test]
fn empty_languages_epsilon_and_backtracking_have_exact_meanings() {
    assert!(decide(&[], &[]));
    assert!(!decide(&[], &[1]));
    assert!(!decide(&[Slot { words: vec![] }], &[]));
    assert!(decide(
        &[Slot {
            words: vec![vec![]]
        }],
        &[]
    ));
    let slots = [
        Slot {
            words: vec![vec![1], vec![1, 1]],
        },
        Slot {
            words: vec![vec![1, 0]],
        },
    ];
    assert!(decide(&slots, &[1, 1, 1, 0]));
    assert!(!decide(&slots, &[1, 1, 1, 0, 0]));
    assert!(!word_matches(&[1, 0], usize::MAX, &[1]));
    assert!(!word_matches(&[1, 0], 3, &[]));
    assert!(word_matches(&[1, 0], 2, &[]));
}

#[test]
fn certificates_cover_external_input_exactly_and_preserve_untrusted_claims() {
    let slots = [
        Slot {
            words: vec![vec![1, 0]],
        },
        Slot {
            words: vec![vec![1]],
        },
    ];
    let input = [1, 0, 1];
    let certificate = recognize(&slots, &input).unwrap();
    assert!(check(&slots, &input, &certificate));
    for i in 0..certificate.len() {
        for field in 0..3 {
            let mut changed = certificate.clone();
            match field {
                0 => changed[i].start = usize::MAX,
                1 => changed[i].end = usize::MAX,
                _ => changed[i].choice = usize::MAX,
            }
            assert!(!check(&slots, &input, &changed));
        }
    }
    assert!(!check(&slots, &input, &certificate[..1]));
    let mut extra = certificate.clone();
    extra.push(certificate[0]);
    assert!(!check(&slots, &input, &extra));
    let mut gap = certificate.clone();
    gap[1].start += 1;
    assert!(!check(&slots, &input, &gap));
    let mut overlap = certificate.clone();
    overlap[1].start -= 1;
    assert!(!check(&slots, &input, &overlap));
    let mut reordered = certificate.clone();
    reordered.swap(0, 1);
    assert!(!check(&slots, &input, &reordered));
    assert!(!check(&slots, &[1, 0, 1, 0], &certificate));
    assert!(!check(&slots, &[1, 1, 1], &certificate));
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/language.proptest-regressions")
        ))),
        .. ProptestConfig::default()
    })]

    #[test]
    fn finite_recognizer_equals_independent_whole_word_enumeration(
        raw in prop::collection::vec(
            prop::collection::vec(prop::collection::vec(0u8..3, 0..4), 0..4), 0..5),
        input in prop::collection::vec(0u8..3, 0..15),
    ) {
        let expected = enumerate(&raw);
        let slots: Vec<_> = raw.into_iter().map(|words| Slot { words }).collect();
        prop_assert_eq!(decide(&slots, &input), expected.contains(&input));
        for word in &expected {
            let certificate = recognize(&slots, word);
            prop_assert!(certificate.is_some());
            prop_assert!(check(&slots, word, &certificate.unwrap()));
        }
        if let Some(certificate) = recognize(&slots, &input) {
            prop_assert!(check(&slots, &input, &certificate));
            prop_assert!(expected.contains(&input));
        }
    }
}
