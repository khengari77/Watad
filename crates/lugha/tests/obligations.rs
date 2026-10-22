use proptest::prelude::*;
use watad_checker::{
    language::Slot,
    obligations::{self as o, Requirements},
    poem::Input,
    verse::{BindingPolicy, DarbClass, Form, Profile, ProfileId},
};

fn form(s: u8, a: u8, class: u16) -> Form {
    Form {
        context: 0,
        initial_only: false,
        class: DarbClass(class),
        sadr: vec![Slot {
            words: vec![vec![s]],
        }],
        ajuz: vec![Slot {
            words: vec![vec![a]],
        }],
    }
}

#[test]
fn obligations_force_backtracking_across_equal_surfaces() {
    let p = Profile {
        id: ProfileId {
            family: 90,
            revision: 1,
        },
        binding: BindingPolicy::FixedDarbClass,
        forms: vec![form(1, 0, 0), form(1, 0, 0), form(0, 1, 0)],
    };
    let input = [
        Input {
            context: 0,
            sadr: &[1],
            ajuz: &[0],
        },
        Input {
            context: 0,
            sadr: &[0],
            ajuz: &[1],
        },
    ];
    let required = Requirements {
        groups: vec![vec![1], vec![2]],
    };
    let mut c = o::recognize(&p, &input, &required).unwrap().unwrap();
    assert_eq!(c[0].form, 1);
    assert!(o::check(&p, &input, &required, &c));
    c[0].form = 0; // Same word, but the required derivation is now absent.
    assert!(!o::check(&p, &input, &required, &c));
    assert!(o::recognize(&p, &input[..1], &required).unwrap().is_none());
    for groups in [vec![vec![]], vec![vec![usize::MAX]]] {
        assert!(
            o::recognize(&p, &input, &Requirements { groups })
                .unwrap()
                .is_none()
        );
    }
}

fn oracle(
    rows: &[(u8, u8, u16)],
    input: &[(u8, u8)],
    groups: &[Vec<usize>],
    fixed: bool,
    at: usize,
    selected: &mut Vec<usize>,
) -> bool {
    if at == input.len() {
        return groups
            .iter()
            .all(|g| g.iter().any(|f| selected.contains(f)));
    }
    for (f, (s, a, class)) in rows.iter().enumerate() {
        if (*s, *a) != input[at]
            || (fixed
                && selected
                    .first()
                    .is_some_and(|first| rows[*first].2 != *class))
        {
            continue;
        }
        selected.push(f);
        if oracle(rows, input, groups, fixed, at + 1, selected) {
            return true;
        }
        selected.pop();
    }
    false
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/obligations.proptest-regressions")
        ))),
        .. ProptestConfig::default()
    })]
    #[test]
    fn recognition_matches_exhaustive_assignment_oracle(
        rows in prop::collection::vec((0u8..2,0u8..2,0u16..3),1..6),
        words in prop::collection::vec((0u8..2,0u8..2),1..5),
        groups in prop::collection::vec(prop::collection::vec(0usize..7,0..4),0..4),
        fixed in any::<bool>(),
    ) {
        let p = Profile { id: ProfileId { family: 91, revision: 1 }, binding: if fixed { BindingPolicy::FixedDarbClass } else { BindingPolicy::Unconstrained }, forms: rows.iter().map(|(s,a,c)| form(*s,*a,*c)).collect() };
        let owned: Vec<_> = words.iter().map(|(s,a)| (vec![*s],vec![*a])).collect();
        let input: Vec<_> = owned.iter().map(|(s,a)| Input { context: 0, sadr:s, ajuz:a }).collect();
        let expected = oracle(&rows,&words,&groups,fixed,0,&mut vec![]);
        let r = Requirements { groups };
        let result = o::recognize(&p,&input,&r).unwrap();
        prop_assert_eq!(result.is_some(), expected);
        if let Some(c) = result {
            prop_assert!(o::check(&p,&input,&r,&c));
            prop_assert!(!o::check(&p,&input,&r,&c[..c.len()-1]));
        }
    }
}
