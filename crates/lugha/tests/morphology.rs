use proptest::prelude::*;
use watad_checker::morphology::{self as m, Action, Position, Program, Scope};
use watad_tahwil::{Operation, Precondition, Zihaf, apply_compound_checked, apply_zihaf_checked};
use watad_types::{Beat, Foot, FootId, Unit};

const BODY: Position = Position {
    initial: false,
    terminal: false,
};

fn program(units: &[Unit]) -> Program {
    let letters = m::expand(units);
    Program {
        scope: Scope::Zihaf,
        required_final: None,
        actions: vec![Action::Keep; letters.len()],
        expected: letters.iter().map(|l| l.beat).collect(),
        suffix: vec![],
    }
}

fn strike(p: &mut Program, z: Zihaf) -> bool {
    let i = usize::from(z.position().one_based()) - 1;
    if i >= p.actions.len() {
        return false;
    }
    p.expected[i] = u8::from(z.precondition() == Precondition::Mutaharrik);
    p.actions[i] = match z.operation() {
        Operation::Delete => Action::Delete,
        Operation::Silence => Action::Silence,
    };
    true
}

fn bytes(word: &[Beat]) -> Vec<u8> {
    word.iter()
        .map(|b| u8::from(*b == Beat::Mutaharrik))
        .collect()
}

#[test]
fn all_named_simple_and_compound_edits_agree_with_structural_reference() {
    for id in 0..11 {
        let foot = Foot::of(FootId::from_id(id));
        let units = foot.units();
        assert_eq!(
            m::expand(units).iter().map(|l| l.beat).collect::<Vec<_>>(),
            bytes(&watad_tahwil::beats_of(foot))
        );
        for z in Zihaf::ALL {
            let mut p = program(units);
            let fits = strike(&mut p, z);
            let reference = apply_zihaf_checked(foot, z).map(|w| bytes(&w));
            if fits {
                assert_eq!(
                    m::apply(units, &p, BODY).ok(),
                    reference.ok(),
                    "{foot:?} {z:?}"
                );
            } else {
                assert!(reference.is_err());
            }
        }
        for c in watad_tahwil::CompoundZihaf::ALL {
            let mut p = program(units);
            let fits = c.components().iter().all(|z| strike(&mut p, *z));
            let reference = apply_compound_checked(foot, c).map(|w| bytes(&w));
            if fits {
                assert_eq!(
                    m::apply(units, &p, BODY).ok(),
                    reference.ok(),
                    "{foot:?} {c:?}"
                );
            } else {
                assert!(reference.is_err());
            }
        }
    }
}

#[test]
fn terminal_and_initial_guards_and_original_preconditions_are_enforced() {
    let units = FootId::Mafailun.units();
    let mut p = program(units);
    p.scope = Scope::Terminal;
    p.required_final = Some(Unit::S);
    p.actions[5] = Action::Delete;
    p.actions[6] = Action::Delete;
    let end = Position {
        initial: false,
        terminal: true,
    };
    assert_eq!(m::apply(units, &p, end).unwrap(), vec![1, 1, 0, 1, 0]);
    assert!(m::apply(units, &p, BODY).is_err());
    p.required_final = Some(Unit::W);
    assert!(m::apply(units, &p, end).is_err());
    p.required_final = None;
    p.expected[6] = 1;
    assert!(m::apply(units, &p, end).is_err());
    let mut p = program(units);
    p.scope = Scope::Initial;
    p.actions[0] = Action::Delete;
    assert!(m::apply(units, &p, BODY).is_err());
    let first = Position {
        initial: true,
        terminal: false,
    };
    assert!(m::apply(units, &p, first).is_ok());
    p.suffix.push(0);
    assert!(m::apply(units, &p, first).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/morphology.proptest-regressions")
        ))),
        .. ProptestConfig::default()
    })]
    #[test]
    fn arbitrary_edits_match_an_independent_original_coordinate_oracle(
        unit_ids in prop::collection::vec(0usize..4, 0..9),
        actions in prop::collection::vec(0u8..3, 0..30),
        expected in prop::collection::vec(0u8..3, 0..30),
        suffix in prop::collection::vec(0u8..3, 0..4),
        scope in 0u8..4, initial in any::<bool>(), terminal in any::<bool>(),
    ) {
        let units: Vec<_> = unit_ids.iter().map(|i| [Unit::S, Unit::T, Unit::W, Unit::M][*i]).collect();
        let scope = [Scope::Zihaf, Scope::Terminal, Scope::Initial, Scope::Licensed][usize::from(scope)];
        let p = Program { scope, required_final: None, actions: actions.iter().map(|a| [Action::Keep, Action::Delete, Action::Silence][usize::from(*a)]).collect(), expected, suffix };
        let original: Vec<_> = units.iter().flat_map(|u| u.beats().iter().enumerate().map(move |(offset,b)| (*u, offset, u8::from(*b == Beat::Mutaharrik)))).collect();
        let eligible = match scope { Scope::Zihaf => p.suffix.is_empty(), Scope::Terminal => terminal, Scope::Initial => initial && p.suffix.is_empty(), Scope::Licensed => true };
        let valid = !units.is_empty() && eligible && p.actions.len() == original.len() && p.expected.len() == original.len()
            && p.suffix.iter().all(|b| *b <= 1)
            && original.iter().enumerate().all(|(i,(u,offset,b))| p.actions[i] == Action::Keep || (p.expected[i] == *b && (p.actions[i] != Action::Silence || *b == 1) && (scope != Scope::Zihaf || (u.is_sabab() && *offset == 1))));
        let result = m::apply(&units, &p, Position { initial, terminal });
        prop_assert_eq!(result.is_ok(), valid);
        if valid {
            let mut wanted = Vec::new();
            for (i, (_,_,b)) in original.iter().enumerate() {
                match p.actions[i] { Action::Keep => wanted.push(*b), Action::Delete => {}, Action::Silence => wanted.push(0) }
            }
            wanted.extend_from_slice(&p.suffix);
            prop_assert_eq!(result.unwrap(), wanted);
        }
    }
}
