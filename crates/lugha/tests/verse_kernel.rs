use watad_checker::{language::Slot, verse::*};

fn form(context: u16, class: u16, a: &[u8], b: &[u8]) -> Form {
    Form {
        context,
        initial_only: false,
        class: DarbClass(class),
        sadr: vec![Slot {
            words: vec![a.to_vec()],
        }],
        ajuz: vec![Slot {
            words: vec![b.to_vec()],
        }],
    }
}

#[test]
fn generic_profile_selection_pairing_context_and_metadata_are_checked() {
    let p = Profile {
        id: ProfileId {
            family: 500,
            revision: 9,
        },
        binding: BindingPolicy::FixedDarbClass,
        forms: vec![
            form(0, 17, &[1, 0], &[1]),
            form(0, 21, &[1], &[1, 0]),
            form(7, 21, &[1, 1], &[0]),
        ],
    };
    assert!(validate(&p));
    let mut c = recognize(
        &p,
        Scope {
            context: 0,
            initial: true,
        },
        &[1, 0],
        &[1],
        None,
    )
    .unwrap()
    .unwrap();
    assert!(check(
        &p,
        Scope {
            context: 0,
            initial: true
        },
        &[1, 0],
        &[1],
        &c
    ));
    assert!(
        recognize(
            &p,
            Scope {
                context: 0,
                initial: true
            },
            &[1, 0],
            &[1, 0],
            None
        )
        .unwrap()
        .is_none()
    );
    assert!(
        recognize(
            &p,
            Scope {
                context: 0,
                initial: true
            },
            &[1, 0],
            &[1],
            Some(DarbClass(21))
        )
        .unwrap()
        .is_none()
    );
    assert!(
        recognize(
            &p,
            Scope {
                context: 7,
                initial: true
            },
            &[1, 1],
            &[0],
            Some(DarbClass(21))
        )
        .unwrap()
        .is_some()
    );
    assert!(
        recognize(
            &p,
            Scope {
                context: 0,
                initial: true
            },
            &[1, 1],
            &[0],
            None
        )
        .unwrap()
        .is_none()
    );
    c.profile.revision += 1;
    assert!(!check(
        &p,
        Scope {
            context: 0,
            initial: true
        },
        &[1, 0],
        &[1],
        &c
    ));
    c.profile = p.id;
    c.context = 7;
    assert!(!check(
        &p,
        Scope {
            context: 0,
            initial: true
        },
        &[1, 0],
        &[1],
        &c
    ));
    c.context = 0;
    c.form = usize::MAX;
    assert!(!check(
        &p,
        Scope {
            context: 0,
            initial: true
        },
        &[1, 0],
        &[1],
        &c
    ));
}

#[test]
fn malformed_profile_is_an_error_instead_of_a_complete_negative() {
    for words in [vec![], vec![vec![]], vec![vec![2]]] {
        let p = Profile {
            id: ProfileId {
                family: 1,
                revision: 1,
            },
            binding: BindingPolicy::Unconstrained,
            forms: vec![Form {
                context: 0,
                initial_only: false,
                class: DarbClass(0),
                sadr: vec![Slot { words }],
                ajuz: vec![Slot {
                    words: vec![vec![1]],
                }],
            }],
        };
        assert!(!validate(&p));
        assert!(
            recognize(
                &p,
                Scope {
                    context: 0,
                    initial: true
                },
                &[1],
                &[1],
                None
            )
            .is_err()
        );
    }
}
