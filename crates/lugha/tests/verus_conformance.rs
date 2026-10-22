//! Runtime conformance of an explicit restricted Hazaj profile through the
//! generic Verus-checked engine. There is no Hazaj-specific acceptance path.
use watad_checker::poem;
use watad_checker::verse::{BindingPolicy, ProfileId, Scope};
use watad_daira::MeterName;
use watad_lugha::catalog;
use watad_lugha::meter::{DarbClass, Definition, Form, MeterProfile};
use watad_lugha::program::Change::{self, Illa as I, Zihaf as Z};
use watad_tahwil::{Illa, Zihaf};
use watad_types::Beat;

const ORDINARY: Scope = Scope {
    context: 0,
    initial: false,
};

fn body() -> Vec<Vec<Change>> {
    vec![vec![], vec![Z(Zihaf::Qabd)], vec![Z(Zihaf::Kaf)]]
}

fn arud() -> Vec<Vec<Change>> {
    vec![vec![], vec![Z(Zihaf::Kaf)]]
}

fn form(darb: Vec<Change>, class: u16) -> Form {
    Form {
        constraints: vec![],
        feet: 2,
        context: 0,
        initial_only: false,
        class: DarbClass(class),
        sadr: vec![body(), arud()],
        ajuz: vec![body(), vec![darb]],
    }
}

/// The old `HASHIMI_HAZAJ` language, now only explicit data for the general
/// engine: 3 body x 2 arud x 3 body x 2 darb = 36 accepted combinations.
fn restricted_hazaj() -> MeterProfile {
    MeterProfile::new(Definition {
        id: ProfileId {
            family: 1,
            revision: 1,
        },
        meter: MeterName::Hazaj,
        source: "spec/profiles/hashimi-hazaj-v1.md",
        binding: BindingPolicy::FixedDarbClass,
        forms: vec![form(vec![], 0), form(vec![I(Illa::Hadhf)], 1)],
        requirements: vec![],
    })
    .unwrap()
}

fn word(code: usize) -> Vec<Beat> {
    ["//0/0/0", "//0//0", "//0/0/", "//0/0", ""][code]
        .bytes()
        .map(|b| {
            if b == b'/' {
                Beat::Mutaharrik
            } else {
                Beat::Sakin
            }
        })
        .collect()
}

fn hemistich(codes: [usize; 4]) -> (Vec<Beat>, Vec<Beat>) {
    let mut sadr = word(codes[0]);
    sadr.extend(word(codes[1]));
    let mut ajuz = word(codes[2]);
    ajuz.extend(word(codes[3]));
    (sadr, ajuz)
}

fn encoded(beats: &[Beat]) -> Vec<u8> {
    beats
        .iter()
        .map(|b| u8::from(*b == Beat::Mutaharrik))
        .collect()
}

#[test]
fn general_engine_matches_declared_hazaj_language_for_all_625_tuples() {
    let profile = restricted_hazaj();
    let mut accepted = 0;
    for i in 0..5 {
        for j in 0..5 {
            for k in 0..5 {
                for l in 0..5 {
                    let codes = [i, j, k, l];
                    let (sadr, ajuz) = hemistich(codes);
                    let expected = i < 3 && (j == 0 || j == 2) && k < 3 && (l == 0 || l == 3);
                    let certificate = profile.recognize(ORDINARY, &sadr, &ajuz).unwrap();
                    assert_eq!(certificate.is_some(), expected, "{codes:?}");
                    if let Some(certificate) = certificate {
                        assert!(profile.check(ORDINARY, &sadr, &ajuz, &certificate));
                        assert_eq!(
                            profile.kernel().forms[certificate.form].class,
                            DarbClass(u16::from(l == 3))
                        );
                    }
                    accepted += usize::from(expected);
                }
            }
        }
    }
    assert_eq!(accepted, 36);
}

#[test]
fn restricted_endpoints_keep_their_old_permissions() {
    let profile = restricted_hazaj();
    for (s0, kaf, a0, hadhf) in [
        (0, false, 0, false),
        (1, true, 2, false),
        (2, false, 1, true),
    ] {
        let (sadr, ajuz) = hemistich([s0, if kaf { 2 } else { 0 }, a0, if hadhf { 3 } else { 0 }]);
        assert!(profile.recognize(ORDINARY, &sadr, &ajuz).unwrap().is_some());
    }
    for (sadr_code, darb_code) in [(1, 0), (0, 2), (3, 3)] {
        let (sadr, ajuz) = hemistich([0, sadr_code, 0, darb_code]);
        assert!(profile.recognize(ORDINARY, &sadr, &ajuz).unwrap().is_none());
    }
}

#[test]
fn certificate_replay_rejects_forged_identity_context_and_steps() {
    let profile = restricted_hazaj();
    let (sadr, ajuz) = hemistich([1, 2, 1, 0]);
    let certificate = || profile.recognize(ORDINARY, &sadr, &ajuz).unwrap().unwrap();
    assert!(profile.check(ORDINARY, &sadr, &ajuz, &certificate()));

    let mut forged = certificate();
    forged.profile.revision += 1;
    assert!(!profile.check(ORDINARY, &sadr, &ajuz, &forged));

    let mut forged = certificate();
    forged.context = catalog::TASRI;
    assert!(!profile.check(ORDINARY, &sadr, &ajuz, &forged));

    let mut forged = certificate();
    forged.form = 1;
    assert!(!profile.check(ORDINARY, &sadr, &ajuz, &forged));

    let mut forged = certificate();
    forged.sadr.pop();
    assert!(!profile.check(ORDINARY, &sadr, &ajuz, &forged));

    let mut forged = certificate();
    forged.ajuz[0].choice += 1;
    assert!(!profile.check(ORDINARY, &sadr, &ajuz, &forged));
}

#[test]
fn same_profile_id_with_extended_rules_uses_the_generic_engine() {
    let restricted = restricted_hazaj();
    let extended = MeterProfile::new(Definition {
        id: restricted.definition().id,
        meter: MeterName::Hazaj,
        source: "extended fixture with an additional arud licence",
        binding: restricted.definition().binding,
        forms: vec![
            form(vec![], 0),
            form(vec![I(Illa::Hadhf)], 1),
            Form {
                constraints: vec![],
                feet: 2,
                context: 0,
                initial_only: false,
                class: DarbClass(0),
                sadr: vec![body(), vec![vec![], vec![Z(Zihaf::Qabd)]]],
                ajuz: vec![body(), vec![vec![]]],
            },
        ],
        requirements: vec![],
    })
    .unwrap();
    let (sadr, ajuz) = hemistich([0, 1, 0, 0]);
    assert!(
        restricted
            .recognize(ORDINARY, &sadr, &ajuz)
            .unwrap()
            .is_none()
    );
    let certificate = extended.recognize(ORDINARY, &sadr, &ajuz).unwrap().unwrap();
    assert!(extended.check(ORDINARY, &sadr, &ajuz, &certificate));
}

#[test]
fn poem_binding_uses_one_darb_class_for_every_verse() {
    let profile = restricted_hazaj();
    let (sadr, ajuz) = hemistich([0, 0, 0, 0]);
    let (other_sadr, other_ajuz) = hemistich([1, 2, 1, 0]);
    let sadr_codes = encoded(&sadr);
    let ajuz_codes = encoded(&ajuz);
    let other_sadr_codes = encoded(&other_sadr);
    let other_ajuz_codes = encoded(&other_ajuz);
    let input = [
        poem::Input {
            context: 0,
            sadr: &sadr_codes,
            ajuz: &ajuz_codes,
        },
        poem::Input {
            context: 0,
            sadr: &other_sadr_codes,
            ajuz: &other_ajuz_codes,
        },
    ];
    let certificates = profile.recognize_poem(&input).unwrap().unwrap();
    assert!(profile.check_poem(&input, &certificates));
    assert_eq!(
        profile.kernel().forms[certificates[0].form].class,
        DarbClass(0)
    );

    let mut mixed_certificates = profile.recognize_poem(&input).unwrap().unwrap();
    mixed_certificates[1].form = 1;
    assert!(!profile.check_poem(&input, &mixed_certificates));

    let mut forged = profile.recognize_poem(&input).unwrap().unwrap();
    forged[0].ajuz[1].end += 1;
    assert!(!profile.check_poem(&input, &forged));
    assert_eq!(
        profile.recognize_poem(&[]).err(),
        Some(watad_lugha::meter::Error::EmptyPoem)
    );
}

#[test]
fn invalid_profile_definitions_are_rejected() {
    let valid = Definition {
        id: ProfileId {
            family: 1,
            revision: 1,
        },
        meter: MeterName::Hazaj,
        source: "fixture",
        binding: BindingPolicy::FixedDarbClass,
        forms: vec![form(vec![], 0)],
        requirements: vec![],
    };
    assert!(matches!(
        MeterProfile::new(Definition {
            source: " ",
            ..valid
        }),
        Err(watad_lugha::meter::Error::MissingSource)
    ));
    for requirements in [vec![vec![]], vec![vec![1]]] {
        assert!(matches!(
            MeterProfile::new(Definition {
                id: ProfileId {
                    family: 1,
                    revision: 1,
                },
                meter: MeterName::Hazaj,
                source: "fixture",
                binding: BindingPolicy::FixedDarbClass,
                forms: vec![form(vec![], 0)],
                requirements,
            }),
            Err(watad_lugha::meter::Error::Requirement)
        ));
    }
    let extra_foot = Form {
        feet: 2,
        sadr: vec![body()],
        ..form(vec![], 0)
    };
    assert!(matches!(
        MeterProfile::new(Definition {
            id: ProfileId {
                family: 1,
                revision: 1,
            },
            meter: MeterName::Hazaj,
            source: "fixture",
            binding: BindingPolicy::FixedDarbClass,
            forms: vec![extra_foot],
            requirements: vec![],
        }),
        Err(watad_lugha::meter::Error::Layout)
    ));
}
