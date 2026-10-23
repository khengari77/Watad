use proptest::prelude::*;
use watad_checker::{
    choice,
    language::{Slot, Step},
};

fn rule() -> choice::Rule {
    choice::Rule {
        left: 0,
        right: 1,
        left_deleted: vec![true, false],
        right_deleted: vec![true],
    }
}

#[test]
fn equal_surfaces_do_not_hide_a_required_alternative() {
    let slots = [
        Slot {
            words: vec![vec![1], vec![1]],
        },
        Slot {
            words: vec![vec![0]],
        },
    ];
    let rules = [rule()];
    assert!(choice::validate(&slots, &rules));
    let steps = choice::recognize(&slots, &rules, &[1, 0]).unwrap();
    assert_eq!(steps[0].choice, 1);
    assert!(choice::check(&slots, &rules, &[1, 0], &steps));
    let mut forged = steps;
    forged[0].choice = 0;
    assert!(!choice::check(&slots, &rules, &[1, 0], &forged));
}

#[test]
fn empty_halves_and_invalid_rule_references_are_exact() {
    assert_eq!(choice::recognize(&[], &[], &[]), Some(vec![]));
    assert!(choice::recognize(&[], &[], &[1]).is_none());
    assert!(!choice::validate(&[], &[rule()]));
    assert!(choice::recognize(&[], &[rule()], &[]).is_none());
    assert!(!choice::check(&[], &[rule()], &[], &[]));
    let slots = [Slot {
        words: vec![vec![1]],
    }];
    let bad = choice::Rule {
        left: 0,
        right: 0,
        left_deleted: vec![],
        right_deleted: vec![false],
    };
    assert!(!choice::validate(&slots, &[bad]));
}

fn exhaustive(slots: &[Slot], rules: &[choice::Rule], input: &[u8]) -> bool {
    fn visit(slots: &[Slot], rules: &[choice::Rule], input: &[u8], path: &mut Vec<usize>) -> bool {
        if path.len() == slots.len() {
            let word: Vec<_> = path
                .iter()
                .enumerate()
                .flat_map(|(i, c)| slots[i].words[*c].iter().copied())
                .collect();
            return word == input
                && rules.iter().all(|r| {
                    let Some(&l) = path.get(r.left) else {
                        return false;
                    };
                    let Some(&a) = path.get(r.right) else {
                        return false;
                    };
                    match (r.left_deleted.get(l), r.right_deleted.get(a)) {
                        (Some(l), Some(a)) => !(*l && *a),
                        _ => false,
                    }
                });
        }
        for c in 0..slots[path.len()].words.len() {
            path.push(c);
            let yes = visit(slots, rules, input, path);
            path.pop();
            if yes {
                return true;
            }
        }
        false
    }
    visit(slots, rules, input, &mut vec![])
}

proptest! {
    #[test]
    fn complete_search_matches_independent_word_and_rule_oracle(
        raw in prop::collection::vec(prop::collection::vec(prop::collection::vec(0u8..2,0..4),0..4),0..4),
        edges in prop::collection::vec((0usize..4,0usize..4,any::<u8>(),any::<u8>()),0..4),
        input in prop::collection::vec(0u8..2,0..10),
    ) {
        let slots:Vec<_> = raw.into_iter().map(|words|Slot{words}).collect();
        let rules:Vec<_> = edges.into_iter().map(|(left,right,l,r)|choice::Rule {
            left,right,
            left_deleted:(0..slots.get(left).map_or(0,|s|s.words.len())).map(|i| l&(1<<i)!=0).collect(),
            right_deleted:(0..slots.get(right).map_or(0,|s|s.words.len())).map(|i| r&(1<<i)!=0).collect(),
        }).collect();
        let result = choice::recognize(&slots,&rules,&input);
        prop_assert_eq!(result.is_some(),exhaustive(&slots,&rules,&input));
        if let Some(steps) = result {
            prop_assert!(choice::check(&slots,&rules,&input,&steps));
            let mut forged = steps;
            if let Some(step) = forged.first_mut() {step.start = usize::MAX;}
            else {forged.push(Step{choice:0,start:0,end:0});}
            prop_assert!(!choice::check(&slots,&rules,&input,&forged));
        }
    }
}
