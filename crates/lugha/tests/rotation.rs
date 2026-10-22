use proptest::prelude::*;
use watad_types::{Unit, rotation};

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/rotation.proptest-regressions")
        ))),
        .. ProptestConfig::default()
    })]
    #[test]
    fn bounded_circle_operations_match_independent_sequence_oracles(
        ids in prop::collection::vec(0usize..4,0..24),
        start in prop_oneof![0usize..30,Just(usize::MAX)],
        reps in prop_oneof![0usize..30,Just(usize::MAX)],
    ) {
        let word: Vec<_> = ids.iter().map(|i| [Unit::S,Unit::T,Unit::W,Unit::M][*i]).collect();
        let rotated = rotation::rotate::<32>(&word,start);
        prop_assert_eq!(rotated.is_some(),!word.is_empty() && start < word.len());
        if let Some((a,n)) = rotated {
            let wanted: Vec<_> = word.iter().skip(start).chain(word.iter().take(start)).copied().collect();
            prop_assert_eq!(&a[..n],wanted.as_slice());
        }
        let repeated = rotation::repeat::<32>(&word,reps);
        prop_assert_eq!(repeated.is_some(),(word.len() as u128)*(reps as u128) <= 32);
        if let Some((a,n)) = repeated {
            let wanted: Vec<_> = if word.is_empty() { vec![] } else {
                std::iter::repeat_n(word.as_slice(),reps).flatten().copied().collect()
            };
            prop_assert_eq!(&a[..n],wanted.as_slice());
        }
    }
}

#[test]
fn capacity_errors_and_overflow_are_errors_instead_of_truncation() {
    assert!(rotation::rotate::<1>(&[Unit::W, Unit::S], 0).is_none());
    assert!(rotation::repeat::<1>(&[Unit::W, Unit::S], 1).is_none());
    assert!(rotation::repeat::<32>(&[Unit::W, Unit::S], usize::MAX).is_none());
    assert_eq!(rotation::repeat::<0>(&[], usize::MAX).unwrap().1, 0);
}
