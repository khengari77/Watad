use proptest::prelude::*;
use watad_checker::{
    adapter::encode_beats,
    adjacency::{self, Rule},
    choice,
    language::Step,
    verse::{Certificate, Scope},
};
use watad_daira::MeterName;
use watad_lugha::{
    catalog,
    meter::{DarbClass, Definition, Form, MeterProfile, ProfileId},
};
use watad_types::Beat;

const CONSTRAINED: [(MeterName, usize); 4] = [
    (MeterName::Madid, 12),
    (MeterName::Ramal, 12),
    (MeterName::Khafif, 10),
    (MeterName::Mujtathth, 2),
];

const REVISION_TWO: [MeterName; 5] = [
    MeterName::Madid,
    MeterName::Ramal,
    MeterName::Wafir,
    MeterName::Khafif,
    MeterName::Mujtathth,
];

fn beats(word: &[u8]) -> Vec<Beat> {
    word.iter()
        .map(|b| {
            if *b == 1 {
                Beat::Mutaharrik
            } else {
                Beat::Sakin
            }
        })
        .collect()
}

fn half_words(profile: &MeterProfile, form: usize, sadr: bool) -> Vec<Vec<Beat>> {
    let slots = if sadr {
        &profile.kernel().forms[form].sadr
    } else {
        &profile.kernel().forms[form].ajuz
    };
    if slots.is_empty() {
        return vec![Vec::new()];
    }
    let mut out = vec![Vec::new()];
    for slot in slots {
        let mut next = Vec::new();
        for prefix in &out {
            for word in &slot.words {
                let mut candidate = prefix.clone();
                candidate.extend(beats(word));
                next.push(candidate);
            }
        }
        out = next;
    }
    out
}

fn reference_profile(profile: &MeterProfile) -> Option<MeterProfile> {
    let definition = profile.definition();
    let (forms, requirements) = catalog::reference_expand_forms(
        definition.meter,
        definition.forms.clone(),
        definition.requirements.clone(),
    )
    .unwrap();
    if forms.is_empty() {
        return None;
    }
    MeterProfile::new(Definition {
        id: ProfileId {
            family: definition.id.family + 1,
            revision: 1,
        },
        meter: definition.meter,
        source: "independent reference expansion",
        binding: definition.binding,
        forms,
        requirements,
    })
    .ok()
}

#[test]
fn constrained_profiles_are_source_scale_and_rule_scoped() {
    for (meter, rows) in CONSTRAINED {
        let profile = catalog::hashimi(meter).unwrap().unwrap();
        let definition = profile.definition();
        assert_eq!(definition.forms.len(), rows, "{meter:?}");
        assert_eq!(profile.kernel().forms.len(), rows, "{meter:?}");
        assert_eq!(
            definition.id.revision, 2,
            "{meter:?} changed certificate coordinates"
        );
        for (i, form) in definition.forms.iter().enumerate() {
            assert!(!form.constraints.is_empty(), "{meter:?} form {i}");
            for rule in &form.constraints {
                assert!(adjacency::check_rule_scope_ok(
                    &profile.structured().forms[i].sadr,
                    *rule
                ));
                assert!(adjacency::check_rule_scope_ok(
                    &profile.structured().forms[i].ajuz,
                    *rule
                ));
            }
        }
    }
}

#[test]
fn compact_and_reference_languages_agree_exhaustively() {
    for (meter, _) in CONSTRAINED {
        let profile = catalog::hashimi(meter).unwrap().unwrap();
        let reference = reference_profile(&profile).expect("reference profile");
        let definition = profile.definition();
        for (i, form) in definition.forms.iter().enumerate() {
            let scope = Scope {
                context: form.context,
                initial: true,
            };
            for sadr in half_words(&profile, i, true) {
                for ajuz in half_words(&profile, i, false) {
                    let compact = profile.recognize(scope, &sadr, &ajuz).unwrap();
                    let expanded = reference.recognize(scope, &sadr, &ajuz).unwrap();
                    assert_eq!(
                        compact.is_some(),
                        expanded.is_some(),
                        "{meter:?} form {i} sadr {sadr:?} ajuz {ajuz:?}"
                    );
                    if let Some(certificate) = compact {
                        assert!(profile.check(scope, &sadr, &ajuz, &certificate));
                    }
                    if let Some(certificate) = expanded {
                        assert!(reference.check(scope, &sadr, &ajuz, &certificate));
                    }
                }
            }
        }
    }
}

#[test]
fn invalid_source_rule_references_reject_whole_profile() {
    let rules = [
        Rule {
            left: 2,
            left_letter: 0,
            right: 1,
            right_letter: 0,
        },
        Rule {
            left: 0,
            left_letter: 0,
            right: 2,
            right_letter: 0,
        },
        Rule {
            left: 0,
            left_letter: usize::MAX,
            right: 1,
            right_letter: 0,
        },
        Rule {
            left: 0,
            left_letter: 0,
            right: 1,
            right_letter: usize::MAX,
        },
    ];
    for rule in rules {
        let form = Form {
            constraints: vec![rule],
            feet: 2,
            context: 0,
            initial_only: false,
            class: DarbClass(0),
            sadr: vec![vec![vec![]], vec![vec![]]],
            ajuz: vec![vec![vec![]], vec![vec![]]],
        };
        assert!(
            MeterProfile::new(Definition {
                id: ProfileId {
                    family: 9001,
                    revision: 1,
                },
                meter: MeterName::Mujtathth,
                source: "invalid rule fixture",
                binding: watad_checker::verse::BindingPolicy::FixedDarbClass,
                forms: vec![form],
                requirements: vec![],
            })
            .is_err()
        );
    }
}

#[test]
fn all_revision_two_profiles_reject_the_legacy_revision() {
    for meter in REVISION_TWO {
        let profile = catalog::hashimi(meter).unwrap().unwrap();
        let scope = Scope {
            context: 0,
            initial: true,
        };
        let certificate = (0..profile.definition().forms.len())
            .find_map(|i| {
                for sadr in half_words(&profile, i, true) {
                    for ajuz in half_words(&profile, i, false) {
                        if let Some(certificate) = profile.recognize(scope, &sadr, &ajuz).unwrap() {
                            return Some((sadr, ajuz, certificate));
                        }
                    }
                }
                None
            })
            .expect("a valid fixture");
        let (sadr, ajuz, mut certificate) = certificate;
        assert!(profile.check(scope, &sadr, &ajuz, &certificate));
        certificate.profile.revision = 1;
        assert!(!profile.check(scope, &sadr, &ajuz, &certificate));
    }
}

#[test]
fn production_replay_rejects_a_forbidden_sadr_correlation() {
    let (profile, form, scope, sadr, sadr_steps, ajuz, ajuz_steps) = find_forbidden_case(true);
    let kernel = &profile.kernel().forms[form];
    let sadr_bits = encode_beats(&sadr);
    let ajuz_bits = encode_beats(&ajuz);
    assert!(choice::check(&kernel.sadr, &[], &sadr_bits, &sadr_steps));
    assert!(choice::check(&kernel.ajuz, &[], &ajuz_bits, &ajuz_steps));
    assert!(choice::check(
        &kernel.ajuz,
        &kernel.ajuz_rules,
        &ajuz_bits,
        &ajuz_steps
    ));
    assert!(!choice::check(
        &kernel.sadr,
        &kernel.sadr_rules,
        &sadr_bits,
        &sadr_steps
    ));
    let certificate = Certificate {
        profile: profile.definition().id,
        context: scope.context,
        form,
        sadr: sadr_steps,
        ajuz: ajuz_steps,
    };
    assert!(!profile.check(scope, &sadr, &ajuz, &certificate));
}

#[test]
fn production_replay_rejects_a_forbidden_ajuz_correlation() {
    let (profile, form, scope, sadr, sadr_steps, ajuz, ajuz_steps) = find_forbidden_case(false);
    let kernel = &profile.kernel().forms[form];
    let sadr_bits = encode_beats(&sadr);
    let ajuz_bits = encode_beats(&ajuz);
    assert!(choice::check(&kernel.sadr, &[], &sadr_bits, &sadr_steps));
    assert!(choice::check(&kernel.ajuz, &[], &ajuz_bits, &ajuz_steps));
    assert!(choice::check(
        &kernel.sadr,
        &kernel.sadr_rules,
        &sadr_bits,
        &sadr_steps
    ));
    assert!(!choice::check(
        &kernel.ajuz,
        &kernel.ajuz_rules,
        &ajuz_bits,
        &ajuz_steps
    ));
    let certificate = Certificate {
        profile: profile.definition().id,
        context: scope.context,
        form,
        sadr: sadr_steps,
        ajuz: ajuz_steps,
    };
    assert!(!profile.check(scope, &sadr, &ajuz, &certificate));
}

#[test]
fn choice_schema_still_rejects_out_of_range_compiled_rules() {
    let slots = [watad_checker::language::Slot {
        words: vec![vec![1]],
    }];
    let bad = choice::Rule {
        left: 1,
        right: 0,
        left_deleted: vec![],
        right_deleted: vec![false],
    };
    assert!(!choice::validate(&slots, &[bad]));
}

fn all_paths(slots: &[watad_checker::structured::FootSlot]) -> Vec<Vec<usize>> {
    if slots.is_empty() {
        return vec![Vec::new()];
    }
    let mut out = vec![Vec::new()];
    for slot in slots {
        let mut next = Vec::new();
        for prefix in &out {
            for choice in 0..slot.programs.len() {
                let mut candidate = prefix.clone();
                candidate.push(choice);
                next.push(candidate);
            }
        }
        out = next;
    }
    out
}

fn source_path_words(profile: &MeterProfile, form: usize, path: &[usize], sadr: bool) -> Vec<Beat> {
    let slots = if sadr {
        &profile.kernel().forms[form].sadr
    } else {
        &profile.kernel().forms[form].ajuz
    };
    path.iter()
        .enumerate()
        .flat_map(|(slot, choice)| beats(&slots[slot].words[*choice]))
        .collect()
}

fn source_path_ok(form: &watad_checker::structured::Form, path: &[usize], sadr: bool) -> bool {
    use watad_checker::morphology::Action;
    let (slots, rules) = if sadr {
        (&form.sadr, &form.sadr_constraints)
    } else {
        (&form.ajuz, &form.ajuz_constraints)
    };
    rules.iter().all(|rule| {
        let Some(&left_choice) = path.get(rule.left) else {
            return false;
        };
        let Some(&right_choice) = path.get(rule.right) else {
            return false;
        };
        let left = &slots[rule.left].programs[left_choice];
        let right = &slots[rule.right].programs[right_choice];
        !(rule.left_letter < left.actions.len()
            && rule.right_letter < right.actions.len()
            && left.actions[rule.left_letter] == Action::Delete
            && right.actions[rule.right_letter] == Action::Delete)
    })
}

fn source_oracle(
    profile: &MeterProfile,
    form: usize,
    scope: Scope,
    sadr: &[Beat],
    ajuz: &[Beat],
) -> bool {
    let source = &profile.structured().forms[form];
    let definition = &profile.definition().forms[form];
    if definition.context != scope.context || (definition.initial_only && !scope.initial) {
        return false;
    }
    all_paths(&source.sadr).into_iter().any(|sadr_path| {
        source_path_ok(source, &sadr_path, true)
            && source_path_words(profile, form, &sadr_path, true) == sadr
            && all_paths(&source.ajuz).into_iter().any(|ajuz_path| {
                source_path_ok(source, &ajuz_path, false)
                    && source_path_words(profile, form, &ajuz_path, false) == ajuz
            })
    })
}

fn steps_for_path(profile: &MeterProfile, form: usize, path: &[usize], sadr: bool) -> Vec<Step> {
    let slots = if sadr {
        &profile.kernel().forms[form].sadr
    } else {
        &profile.kernel().forms[form].ajuz
    };
    let mut at = 0;
    path.iter()
        .enumerate()
        .map(|(slot, choice)| {
            let end = at + slots[slot].words[*choice].len();
            let step = Step {
                choice: *choice,
                start: at,
                end,
            };
            at = end;
            step
        })
        .collect()
}

fn violated_rules(profile: &MeterProfile, form: usize, path: &[usize], sadr: bool) -> Vec<usize> {
    let rules = if sadr {
        &profile.kernel().forms[form].sadr_rules
    } else {
        &profile.kernel().forms[form].ajuz_rules
    };
    rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| {
            rule.left_deleted[path[rule.left]] && rule.right_deleted[path[rule.right]]
        })
        .map(|(index, _)| index)
        .collect()
}

fn valid_other_path(
    profile: &MeterProfile,
    form: usize,
    sadr: bool,
) -> Option<(Vec<Beat>, Vec<Step>)> {
    let source = &profile.structured().forms[form];
    let slots = if sadr { &source.sadr } else { &source.ajuz };
    let kernel = &profile.kernel().forms[form];
    let (kernel_slots, rules) = if sadr {
        (&kernel.sadr, &kernel.sadr_rules)
    } else {
        (&kernel.ajuz, &kernel.ajuz_rules)
    };
    all_paths(slots).into_iter().find_map(|path| {
        let words = source_path_words(profile, form, &path, sadr);
        let bits = encode_beats(&words);
        let steps = steps_for_path(profile, form, &path, sadr);
        (choice::check(kernel_slots, rules, &bits, &steps)
            && choice::check(kernel_slots, &[], &bits, &steps))
        .then_some((words, steps))
    })
}

type ForbiddenCase = (
    MeterProfile,
    usize,
    Scope,
    Vec<Beat>,
    Vec<Step>,
    Vec<Beat>,
    Vec<Step>,
);

fn find_forbidden_case(sadr: bool) -> ForbiddenCase {
    for meter in CONSTRAINED.map(|(meter, _)| meter) {
        let profile = catalog::hashimi(meter).unwrap().unwrap();
        for form in 0..profile.structured().forms.len() {
            let source = &profile.structured().forms[form];
            let definition = &profile.definition().forms[form];
            if (sadr && source.sadr.is_empty()) || (!sadr && source.ajuz.is_empty()) {
                continue;
            }
            let scope = Scope {
                context: definition.context,
                initial: true,
            };
            let paths = if sadr {
                all_paths(&source.sadr)
            } else {
                all_paths(&source.ajuz)
            };
            for path in paths {
                if violated_rules(&profile, form, &path, sadr).len() != 1 {
                    continue;
                }
                let words = source_path_words(&profile, form, &path, sadr);
                let bits = encode_beats(&words);
                let steps = steps_for_path(&profile, form, &path, sadr);
                let kernel = &profile.kernel().forms[form];
                let (kernel_slots, rules) = if sadr {
                    (&kernel.sadr, &kernel.sadr_rules)
                } else {
                    (&kernel.ajuz, &kernel.ajuz_rules)
                };
                if !choice::check(kernel_slots, &[], &bits, &steps)
                    || choice::check(kernel_slots, rules, &bits, &steps)
                {
                    continue;
                }
                if let Some((other_words, other_steps)) = valid_other_path(&profile, form, !sadr) {
                    return if sadr {
                        (profile, form, scope, words, steps, other_words, other_steps)
                    } else {
                        (profile, form, scope, other_words, other_steps, words, steps)
                    };
                }
            }
        }
    }
    panic!("no single-rule forbidden source path");
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, .. ProptestConfig::default() })]

    #[test]
    fn arbitrary_source_rules_lower_to_the_same_selected_language(
        recipes in prop::collection::vec(
            prop::collection::vec(
                prop_oneof![
                    Just(Vec::new()),
                    Just(vec![watad_checker::named::Change::Zihaf(watad_checker::named::Zihaf::Khabn)]),
                    Just(vec![watad_checker::named::Change::Zihaf(watad_checker::named::Zihaf::Kaf)]),
                ],
                1..3,
            ),
            1..=2,
        ),
        edges in prop::collection::vec((0usize..2, 0usize..2, 0usize..2, 0usize..2), 0..3),
        initial_only in any::<bool>(),
        initial in any::<bool>(),
        sadr_bits in prop::collection::vec(0u8..2, 0..8),
        ajuz_bits in prop::collection::vec(0u8..2, 0..8),
    ) {
        let feet = recipes.len();
        let sadr = recipes.clone();
        let ajuz = recipes;
        let form = Form {
            constraints: edges.into_iter().map(|(left, left_letter, right, right_letter)| Rule {
                left, left_letter, right, right_letter,
            }).collect(),
            feet,
            context: 0,
            initial_only,
            class: DarbClass(0),
            sadr,
            ajuz,
        };
        let profile = MeterProfile::new(Definition {
            id: ProfileId { family: 9100, revision: 1 },
            meter: MeterName::Mujtathth,
            source: "generated source fixture",
            binding: watad_checker::verse::BindingPolicy::FixedDarbClass,
            forms: vec![form],
            requirements: vec![],
        });
        let profile = match profile { Ok(profile) => profile, Err(_) => return Ok(()) };
        let scope = Scope { context: 0, initial };
        let mut inputs = Vec::new();
        for sadr_path in all_paths(&profile.structured().forms[0].sadr) {
            for ajuz_path in all_paths(&profile.structured().forms[0].ajuz) {
                inputs.push((
                    source_path_words(&profile, 0, &sadr_path, true),
                    source_path_words(&profile, 0, &ajuz_path, false),
                ));
            }
        }
        inputs.push((
            sadr_bits.into_iter().map(|b| if b == 1 { Beat::Mutaharrik } else { Beat::Sakin }).collect(),
            ajuz_bits.into_iter().map(|b| if b == 1 { Beat::Mutaharrik } else { Beat::Sakin }).collect(),
        ));
        for (sadr, ajuz) in inputs {
            let expected = source_oracle(&profile, 0, scope, &sadr, &ajuz);
            let actual = profile.recognize(scope, &sadr, &ajuz).unwrap();
            prop_assert_eq!(actual.is_some(), expected);
            if let Some(certificate) = actual {
                prop_assert!(profile.check(scope, &sadr, &ajuz, &certificate));
            }
        }
    }
}
