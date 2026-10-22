use watad_checker::{
    morphology::{Position, Scope},
    structured::{self, FootSlot},
    verse,
};
use watad_lugha::program::{self, Change};
use watad_tahwil::{Illa, Zihaf};
use watad_types::{FootId, Unit};

#[test]
fn projection_preserves_program_choices_and_rejects_every_bad_alternative() {
    let base = FootId::Mafailun;
    let mut slot = FootSlot {
        units: base.units().to_vec(),
        programs: vec![
            program::compile(base, &[]).unwrap(),
            program::compile(base, &[Change::Zihaf(Zihaf::Qabd)]).unwrap(),
        ],
    };
    let position = Position {
        initial: true,
        terminal: true,
    };
    let lowered = structured::compile(&slot, position).unwrap();
    assert_eq!(
        lowered.words,
        vec![vec![1, 1, 0, 1, 0, 1, 0], vec![1, 1, 0, 1, 1, 0]]
    );
    slot.programs[1].expected[4] = 1;
    assert!(structured::compile(&slot, position).is_err());
    slot.programs.pop();
    for units in [vec![], vec![Unit::S], vec![Unit::W, Unit::M]] {
        slot.units = units;
        assert!(structured::compile(&slot, position).is_err());
    }
}

#[test]
fn composition_cannot_hide_a_zihaf_inside_a_terminal_scope() {
    assert!(
        program::compile(
            FootId::FailatunMafruq,
            &[Change::Zihaf(Zihaf::Khabn), Change::Illa(Illa::Hadhf)]
        )
        .is_err()
    );
    assert!(
        program::compile(
            FootId::FailatunMafruq,
            &[Change::Illa(Illa::Hadhf), Change::Zihaf(Zihaf::Khabn)]
        )
        .is_err()
    );
    assert!(
        program::compile(
            FootId::Mafalatun,
            &[Change::Zihaf(Zihaf::Asb), Change::Zihaf(Zihaf::Aql)]
        )
        .is_err()
    );
}

#[test]
fn named_terminal_combinations_match_independent_patterns() {
    let rows: &[(FootId, &[Change], &str)] = &[
        (FootId::FaUlun, &[Change::Illa(Illa::Qasr)], "//00"),
        (FootId::FaUlun, &[Change::Illa(Illa::Batr)], "/0"),
        (FootId::Failatun, &[Change::Illa(Illa::Batr)], "/0/0"),
        (FootId::Mafailun, &[Change::Illa(Illa::Kharm)], "/0/0/0"),
        (
            FootId::MafUlatu,
            &[Change::Zihaf(Zihaf::Tayy), Change::Illa(Illa::Kasf)],
            "/0//0",
        ),
        (
            FootId::MafUlatu,
            &[Change::Zihaf(Zihaf::Tayy), Change::Illa(Illa::Waqf)],
            "/0//00",
        ),
        (FootId::Failatun, &[Change::Tashith], "/0/0/0"),
        (
            FootId::MutaFailun,
            &[Change::Zihaf(Zihaf::Idmar), Change::Illa(Illa::Hadhadh)],
            "/0/0",
        ),
    ];
    for (base, changes, expected) in rows {
        let p = program::compile(*base, changes).unwrap();
        let actual = watad_checker::morphology::apply(
            base.units(),
            &p,
            Position {
                initial: true,
                terminal: true,
            },
        )
        .unwrap();
        let wanted: Vec<u8> = expected.bytes().map(|b| u8::from(b == b'/')).collect();
        assert_eq!(actual, wanted, "{base:?} {changes:?}");
    }
}

#[test]
fn every_named_illa_matches_the_independent_legacy_mechanics_on_every_base() {
    for id in 0..11 {
        let base = FootId::from_id(id);
        for illa in Illa::ALL {
            let actual = program::compile(base, &[Change::Illa(illa)])
                .ok()
                .and_then(|p| {
                    watad_checker::morphology::apply(
                        base.units(),
                        &p,
                        Position {
                            initial: true,
                            terminal: true,
                        },
                    )
                    .ok()
                });
            let expected = watad_tahwil::apply_illa(watad_types::Foot::of(base), illa)
                .ok()
                .map(|w| {
                    w.iter()
                        .map(|b| u8::from(*b == watad_types::Beat::Mutaharrik))
                        .collect::<Vec<_>>()
                });
            assert_eq!(actual, expected, "{base:?} {illa:?}");
        }
    }
}

#[test]
fn whole_profile_projection_checks_terminal_positions_and_never_repairs_data() {
    let base = FootId::Mafailun;
    let foot = || FootSlot {
        units: base.units().to_vec(),
        programs: vec![program::compile(base, &[]).unwrap()],
    };
    let mut p = structured::Profile {
        id: verse::ProfileId {
            family: 500,
            revision: 1,
        },
        binding: verse::BindingPolicy::FixedDarbClass,
        forms: vec![structured::Form {
            sadr_constraints: vec![],
            ajuz_constraints: vec![],
            context: 7,
            initial_only: true,
            class: verse::DarbClass(9),
            sadr: vec![foot(), foot()],
            ajuz: vec![foot()],
        }],
    };
    let lowered = structured::compile_profile(&p).unwrap();
    assert_eq!(lowered.id, p.id);
    assert_eq!(lowered.forms[0].class, verse::DarbClass(9));
    assert_eq!(lowered.forms[0].context, 7);
    assert!(lowered.forms[0].initial_only);
    assert_eq!(lowered.forms[0].sadr.len(), 2);
    p.forms[0].sadr[0].programs[0].scope = Scope::Terminal;
    assert!(structured::compile_profile(&p).is_err());
    p.forms[0].sadr.remove(1); // A single foot is both initial and terminal.
    assert!(structured::compile_profile(&p).is_ok());
    p.forms[0].ajuz.clear();
    assert!(structured::compile_profile(&p).is_ok()); // Explicit single-line layout.
    p.forms[0].sadr.clear();
    assert!(structured::compile_profile(&p).is_err());
}
