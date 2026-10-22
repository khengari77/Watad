use proptest::prelude::*;
use watad_checker::{
    adjacency::Rule,
    expand,
    morphology::{Action, Program, Scope},
    structured::FootSlot,
};

fn program(deletes: u8) -> Program {
    Program {
        scope: Scope::Zihaf,
        required_final: None,
        actions: (0..8)
            .map(|i| {
                if deletes & (1 << i) == 0 {
                    Action::Keep
                } else {
                    Action::Delete
                }
            })
            .collect(),
        expected: vec![],
        suffix: vec![],
    }
}

fn oracle(slots: &[FootSlot], rules: &[Rule]) -> Vec<Vec<usize>> {
    fn rule_holds(slots: &[FootSlot], rule: Rule, choice: &[usize]) -> bool {
        if rule.left >= slots.len() || rule.right >= slots.len() {
            return false;
        }
        let left = &slots[rule.left].programs[choice[rule.left]].actions;
        let right = &slots[rule.right].programs[choice[rule.right]].actions;
        rule.left_letter < left.len()
            && rule.right_letter < right.len()
            && !(left[rule.left_letter] == Action::Delete
                && right[rule.right_letter] == Action::Delete)
    }

    fn visit(
        slots: &[FootSlot],
        rules: &[Rule],
        at: usize,
        choice: &mut Vec<usize>,
        out: &mut Vec<Vec<usize>>,
    ) {
        if at == slots.len() {
            if rules.iter().all(|rule| rule_holds(slots, *rule, choice)) {
                out.push(choice.clone());
            }
            return;
        }
        for alternative in 0..slots[at].programs.len() {
            choice.push(alternative);
            visit(slots, rules, at + 1, choice, out);
            choice.pop();
        }
    }

    let mut out = Vec::new();
    visit(slots, rules, 0, &mut Vec::new(), &mut out);
    out
}

fn slots(programs: Vec<Vec<u8>>) -> Vec<FootSlot> {
    programs
        .into_iter()
        .map(|programs| FootSlot {
            units: vec![],
            programs: programs.into_iter().map(program).collect(),
        })
        .collect()
}

fn rule(left: usize, left_letter: usize, right: usize, right_letter: usize) -> Rule {
    Rule {
        left,
        left_letter,
        right,
        right_letter,
    }
}

#[test]
fn no_rules_is_the_full_cartesian_product_in_source_order() {
    let slots = slots(vec![vec![0, 1], vec![2], vec![3, 4]]);
    assert_eq!(
        expand::assignments(&slots, &[]),
        vec![vec![0, 0, 0], vec![0, 0, 1], vec![1, 0, 0], vec![1, 0, 1]]
    );
}

#[test]
fn absent_half_has_one_empty_assignment() {
    assert_eq!(expand::assignments(&[], &[]), vec![Vec::<usize>::new()]);
}

#[test]
fn repeated_program_values_remain_distinct_assignments_in_source_order() {
    let slots = slots(vec![vec![0, 0], vec![1, 1]]);
    assert_eq!(
        expand::assignments(&slots, &[]),
        vec![vec![0, 0], vec![0, 1], vec![1, 0], vec![1, 1]]
    );
}

#[test]
fn only_paired_deletes_are_excluded() {
    let slots = slots(vec![vec![0b01, 0b10], vec![0b01, 0b10]]);
    let rules = [rule(0, 0, 1, 0)];
    assert_eq!(
        expand::assignments(&slots, &rules),
        vec![vec![0, 1], vec![1, 0], vec![1, 1]]
    );
}

#[test]
fn out_of_range_rules_emit_nothing() {
    let slots = slots(vec![vec![0], vec![0]]);
    assert!(expand::assignments(&slots, &[rule(0, 0, 2, 0)]).is_empty());
    assert!(expand::assignments(&slots, &[rule(0, 8, 1, 0)]).is_empty());
}

proptest! {
    #[test]
    fn assignments_match_the_independent_exhaustive_oracle(
        tables in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 0..4),
            0..4,
        ),
        raw_rules in prop::collection::vec((0usize..4, 0usize..8, 0usize..4, 0usize..8), 0..5),
    ) {
        let slots = slots(tables);
        let rules: Vec<_> = raw_rules
            .into_iter()
            .map(|(left, left_letter, right, right_letter)| {
                rule(left, left_letter, right, right_letter)
            })
            .collect();
        prop_assert_eq!(expand::assignments(&slots, &rules), oracle(&slots, &rules));
    }
}
