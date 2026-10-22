use proptest::prelude::*;
use std::collections::BTreeSet;
use watad_checker::catalog::{Error, compile_groups, expand_form_choices};
use watad_checker::named::Change;
use watad_lugha::meter::{DarbClass, Form};

fn empty_form(context: u16) -> Form {
    Form {
        constraints: vec![],
        feet: 1,
        context,
        initial_only: context == 1,
        class: DarbClass(context + 2),
        sadr: vec![vec![vec![]]],
        ajuz: vec![],
    }
}

fn oracle(lens: &[usize], requirements: &[Vec<usize>]) -> Option<Vec<Vec<usize>>> {
    let mut starts = Vec::with_capacity(lens.len());
    let mut at = 0;
    for len in lens {
        starts.push(at);
        at += len;
    }
    let mut out = Vec::new();
    for group in requirements {
        if group.is_empty() {
            return None;
        }
        let mut indices = BTreeSet::new();
        for source in group {
            let start = *starts.get(*source)?;
            indices.extend(start..start + lens[*source]);
        }
        if indices.is_empty() {
            return None;
        }
        out.push(indices.into_iter().collect());
    }
    Some(out)
}

#[test]
fn ranges_and_requirement_unions_are_exact_for_empty_and_nonempty_groups() {
    let groups = vec![
        vec![empty_form(0), empty_form(1)],
        vec![],
        vec![empty_form(2)],
    ];
    let requirements = vec![vec![0], vec![2], vec![0, 1, 2], vec![0, 0]];
    let (out, ranges, remapped) = compile_groups(&groups, &requirements).unwrap();
    assert_eq!(ranges, vec![(0, 2), (2, 0), (2, 1)]);
    assert_eq!(
        remapped,
        vec![vec![0, 1], vec![2], vec![0, 1, 2], vec![0, 1]]
    );
    assert_eq!(out.len(), 3);
    for (source, branch) in groups.iter().flatten().zip(&out) {
        assert_eq!(source.context, branch.context);
        assert_eq!(source.initial_only, branch.initial_only);
        assert_eq!(source.class, branch.class);
        assert_eq!(source.constraints, branch.constraints);
        assert_eq!(source.feet, branch.feet);
        assert_eq!(source.sadr, branch.sadr);
        assert_eq!(source.ajuz, branch.ajuz);
    }
}

#[test]
fn empty_or_out_of_range_requirement_groups_reject() {
    let groups = vec![vec![empty_form(0)], vec![]];
    assert!(matches!(
        compile_groups(&groups, &[vec![]]),
        Err(Error::Requirement)
    ));
    assert!(matches!(
        compile_groups(&groups, &[vec![2]]),
        Err(Error::Requirement)
    ));
    assert!(matches!(
        compile_groups(&groups, &[vec![1]]),
        Err(Error::Requirement)
    ));
}

#[test]
fn empty_source_lists_and_empty_unions_are_exact() {
    let (out, ranges, remapped) = compile_groups(&[], &[]).unwrap();
    assert!(out.is_empty());
    assert!(ranges.is_empty());
    assert!(remapped.is_empty());

    assert!(matches!(
        compile_groups(&[], &[vec![]]),
        Err(Error::Requirement)
    ));
    assert!(matches!(
        compile_groups(&[], &[vec![0]]),
        Err(Error::Requirement)
    ));

    let all_empty = vec![vec![], vec![]];
    assert!(matches!(
        compile_groups(&all_empty, &[vec![0]]),
        Err(Error::Requirement)
    ));
    assert!(matches!(
        compile_groups(&all_empty, &[vec![0, 1]]),
        Err(Error::Requirement)
    ));

    let groups = vec![vec![empty_form(0)], vec![], vec![empty_form(1)]];
    let (out, ranges, remapped) =
        compile_groups(&groups, &[vec![0, 1], vec![1, 2], vec![0, 0]]).unwrap();
    assert_eq!(ranges, vec![(0, 1), (1, 0), (1, 1)]);
    assert_eq!(remapped, vec![vec![0], vec![1], vec![0]]);
    assert_eq!(out.len(), 2);

    let (out, _, remapped) = compile_groups(&groups, &[]).unwrap();
    assert_eq!(out.len(), 2);
    assert!(remapped.is_empty());

    assert!(matches!(
        compile_groups(&groups, &[vec![1]]),
        Err(Error::Requirement)
    ));
    assert!(matches!(
        compile_groups(&groups, &[vec![1, 1]]),
        Err(Error::Requirement)
    ));
}

#[test]
fn equal_recipes_at_distinct_indices_remain_distinct_branches() {
    let source = Form {
        constraints: vec![],
        feet: 2,
        context: 1,
        initial_only: true,
        class: DarbClass(2),
        sadr: vec![
            vec![vec![Change::Unchanged], vec![Change::Unchanged]],
            vec![vec![Change::Zihaf(watad_checker::named::Zihaf::Qabd)]],
        ],
        ajuz: vec![
            vec![
                vec![Change::Zihaf(watad_checker::named::Zihaf::Kaf)],
                vec![Change::Zihaf(watad_checker::named::Zihaf::Kaf)],
            ],
            vec![vec![Change::Illa(watad_checker::named::Illa::Hadhf)]],
        ],
    };
    let sadr_choices = vec![vec![0, 0], vec![1, 0]];
    let ajuz_choices = vec![vec![0, 0], vec![1, 0]];
    let out = expand_form_choices(&source, &sadr_choices, &ajuz_choices).unwrap();
    assert_eq!(out.len(), 4);
    assert_eq!(
        out[0].sadr,
        vec![
            vec![vec![Change::Unchanged]],
            vec![vec![Change::Zihaf(watad_checker::named::Zihaf::Qabd)]],
        ]
    );
    assert_eq!(out[0].sadr, out[1].sadr);
    assert_eq!(out[0].sadr, out[2].sadr);
    assert_eq!(out[0].sadr, out[3].sadr);
    assert_eq!(
        out[0].ajuz,
        vec![
            vec![vec![Change::Zihaf(watad_checker::named::Zihaf::Kaf)]],
            vec![vec![Change::Illa(watad_checker::named::Illa::Hadhf)]],
        ]
    );
    assert_eq!(out[0].ajuz, out[1].ajuz);
    assert_eq!(out[1].ajuz, out[2].ajuz);
    assert_eq!(out[2].ajuz, out[3].ajuz);
    for branch in &out {
        assert_eq!(branch.context, source.context);
        assert_eq!(branch.initial_only, source.initial_only);
        assert_eq!(branch.class, source.class);
        assert_eq!(branch.feet, source.feet);
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/catalog_compilation.proptest-regressions")
        ))),
        .. ProptestConfig::default()
    })]

    #[test]
    fn compile_groups_matches_an_independent_range_union_oracle(
        lens in prop::collection::vec(0usize..4, 0..5),
        requirements in prop::collection::vec(prop::collection::vec(0usize..7, 0..4), 0..4),
    ) {
        let groups: Vec<Vec<Form>> = lens
            .iter()
            .map(|len| (0..*len).map(|i| empty_form((i % 3) as u16)).collect())
            .collect();
        let actual = compile_groups(&groups, &requirements);
        let expected = oracle(&lens, &requirements);
        let actual_remapped = actual.as_ref().ok().map(|(_, _, r)| r);
        prop_assert_eq!(actual_remapped, expected.as_ref());
        if let Ok((out, ranges, _)) = actual {
            prop_assert_eq!(out.len(), lens.iter().sum::<usize>());
            prop_assert_eq!(ranges.len(), lens.len());
            for (at, (group, len)) in groups.iter().zip(&lens).enumerate() {
                prop_assert_eq!(ranges[at].1, *len);
                for (i, branch) in group.iter().enumerate() {
                    prop_assert_eq!(out[ranges[at].0 + i].context, branch.context);
                }
            }
        }
    }
}
