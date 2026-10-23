use std::collections::HashSet;

use watad_checker::{
    adjacency::{self, Rule},
    choice,
    language::Step,
    structured::FootSlot,
};
use watad_daira::MeterName;
use watad_lugha::{
    catalog,
    meter::{MeterProfile, Scope},
    program::{self, Change},
};
use watad_tahwil::Zihaf;
use watad_types::{Beat, FootId};

use Change::Zihaf as Z;
use Zihaf::{Kaf, Khabn};

fn profile() -> MeterProfile {
    catalog::hashimi(MeterName::Mujtathth).unwrap().unwrap()
}

fn body_recipes() -> Vec<Vec<Change>> {
    vec![vec![], vec![Z(Khabn)], vec![Z(Kaf)], vec![Z(Khabn), Z(Kaf)]]
}

fn arud_recipes() -> Vec<Vec<Change>> {
    body_recipes()
}

fn darb_recipes() -> Vec<Vec<Change>> {
    vec![vec![], vec![Change::Tashith], vec![Z(Khabn)]]
}

fn expected_rules() -> Vec<Rule> {
    vec![
        Rule {
            left: 0,
            left_letter: 6,
            right: 1,
            right_letter: 1,
        },
        Rule {
            left: 0,
            left_letter: 1,
            right: 1,
            right_letter: 6,
        },
    ]
}

fn allowed(left: &[Change], right: &[Change]) -> bool {
    let left_kaf = left.contains(&Z(Kaf));
    let left_khabn = left.contains(&Z(Khabn));
    let right_kaf = right.contains(&Z(Kaf));
    let right_khabn = right.contains(&Z(Khabn));
    !(left_kaf && right_khabn || left_khabn && right_kaf)
}

fn expected_pairs(
    lefts: &[Vec<Change>],
    rights: &[Vec<Change>],
) -> HashSet<(Vec<Change>, Vec<Change>)> {
    let mut out = HashSet::new();
    for left in lefts {
        for right in rights {
            if allowed(left, right) {
                out.insert((left.clone(), right.clone()));
            }
        }
    }
    out
}

fn recipe_set(recipes: &[Vec<Change>]) -> HashSet<Vec<Change>> {
    recipes.iter().cloned().collect()
}

fn slot_of(base: FootId, recipe: &[Change]) -> FootSlot {
    FootSlot {
        units: base.units().to_vec(),
        programs: vec![program::compile(base, recipe).unwrap()],
    }
}

fn body_surface(recipe: &[Change]) -> &'static str {
    match recipe {
        [] => "/0/0//0",
        [Z(Khabn)] => "//0//0",
        [Z(Kaf)] => "/0/0//",
        [Z(Khabn), Z(Kaf)] => "//0//",
        _ => panic!("unexpected body recipe: {recipe:?}"),
    }
}

fn failatun_surface(recipe: &[Change]) -> &'static str {
    match recipe {
        [] => "/0//0/0",
        [Change::Tashith] => "/0/0/0",
        [Z(Khabn)] => "///0/0",
        [Z(Kaf)] => "/0//0/",
        [Z(Khabn), Z(Kaf)] => "///0/",
        _ => panic!("unexpected Failatun recipe: {recipe:?}"),
    }
}

fn word(s: &str) -> Vec<Beat> {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '/' => Beat::Mutaharrik,
            '0' => Beat::Sakin,
            _ => panic!("bad fixture"),
        })
        .collect()
}

fn accepts(p: &MeterProfile, sadr: &str, ajuz: &str) -> bool {
    p.recognize(
        Scope {
            context: 0,
            initial: false,
        },
        &word(sadr),
        &word(ajuz),
    )
    .unwrap()
    .is_some()
}

fn selected_pairs(
    p: &MeterProfile,
    index: usize,
    sadr: bool,
) -> HashSet<(Vec<Change>, Vec<Change>)> {
    let f = &p.definition().forms[index];
    let k = &p.kernel().forms[index];
    let (slots, rules) = if sadr {
        (&f.sadr, &k.sadr_rules)
    } else {
        (&f.ajuz, &k.ajuz_rules)
    };
    let mut pairs = HashSet::new();
    for (i, left) in slots[0].iter().enumerate() {
        for (j, right) in slots[1].iter().enumerate() {
            let steps = [
                Step {
                    choice: i,
                    start: 0,
                    end: 0,
                },
                Step {
                    choice: j,
                    start: 0,
                    end: 0,
                },
            ];
            if choice::check_rules(rules, &steps) {
                pairs.insert((left.clone(), right.clone()));
            }
        }
    }
    pairs
}

#[test]
fn mujtath_declared_matrix_is_exact_endpoint_and_reciprocity_rules() {
    let p = profile();
    let rules = expected_rules();
    assert_eq!(p.definition().forms.len(), 2);
    assert_eq!(p.definition().id.revision, 2);
    let ordinary: Vec<_> = p
        .definition()
        .forms
        .iter()
        .enumerate()
        .filter(|(_, f)| f.context == 0)
        .collect();
    assert_eq!(ordinary.len(), 1);
    let (index, f) = ordinary[0];
    assert_eq!(f.feet, 2);
    assert_eq!(f.constraints, rules);
    assert_eq!(f.sadr, vec![body_recipes(), arud_recipes()]);
    assert_eq!(f.ajuz, vec![body_recipes(), darb_recipes()]);
    let sadr_pairs = selected_pairs(&p, index, true);
    let ajuz_pairs = selected_pairs(&p, index, false);
    assert_eq!(sadr_pairs, expected_pairs(&body_recipes(), &arud_recipes()));
    assert_eq!(ajuz_pairs, expected_pairs(&body_recipes(), &darb_recipes()));
    assert_eq!(sadr_pairs.len(), 9);
    assert_eq!(ajuz_pairs.len(), 10);
    let combinations: HashSet<_> = sadr_pairs
        .iter()
        .flat_map(|(a, b)| {
            ajuz_pairs
                .iter()
                .map(move |(c, d)| (a.clone(), b.clone(), c.clone(), d.clone()))
        })
        .collect();
    assert_eq!(combinations.len(), 90);
    let tasri: Vec<_> = p
        .definition()
        .forms
        .iter()
        .enumerate()
        .filter(|(_, f)| f.context == 1)
        .collect();
    assert_eq!(tasri.len(), 1);
    let (index, f) = tasri[0];
    assert!(f.initial_only);
    assert_eq!(recipe_set(&f.sadr[1]), recipe_set(&darb_recipes()));
    assert_eq!(recipe_set(&f.ajuz[1]), recipe_set(&darb_recipes()));
    assert_eq!(
        selected_pairs(&p, index, true),
        expected_pairs(&body_recipes(), &darb_recipes())
    );
    assert_eq!(
        selected_pairs(&p, index, false),
        expected_pairs(&body_recipes(), &darb_recipes())
    );
    assert_eq!(
        selected_pairs(&p, index, true).len() * selected_pairs(&p, index, false).len(),
        100
    );
    for f in &p.structured().forms {
        assert_eq!(f.sadr_constraints, rules);
        assert_eq!(f.ajuz_constraints, rules);
        for r in &rules {
            assert!(adjacency::check_rule_scope_ok(&f.sadr, *r));
            assert!(adjacency::check_rule_scope_ok(&f.ajuz, *r));
        }
    }
}

#[test]
fn literal_mujtath_recipes_satisfy_both_muaqabah_rules() {
    let rules = expected_rules();

    for left in body_recipes() {
        for right in arud_recipes() {
            let slots = [
                slot_of(FootId::MustafIlunMafruq, &left),
                slot_of(FootId::Failatun, &right),
            ];
            assert_eq!(
                adjacency::check(&slots, &rules),
                allowed(&left, &right),
                "arud pair {left:?} / {right:?}"
            );
        }
    }

    for left in body_recipes() {
        for right in darb_recipes() {
            let slots = [
                slot_of(FootId::MustafIlunMafruq, &left),
                slot_of(FootId::Failatun, &right),
            ];
            assert_eq!(
                adjacency::check(&slots, &rules),
                allowed(&left, &right),
                "darb pair {left:?} / {right:?}"
            );
        }
    }

    assert!(!adjacency::check(
        &[
            slot_of(FootId::MustafIlunMafruq, &[Z(Kaf)]),
            slot_of(FootId::Failatun, &[Z(Khabn)]),
        ],
        &rules
    ));
    assert!(!adjacency::check(
        &[
            slot_of(FootId::MustafIlunMafruq, &[Z(Khabn)]),
            slot_of(FootId::Failatun, &[Z(Kaf)]),
        ],
        &rules
    ));
    assert!(adjacency::check(
        &[
            slot_of(FootId::MustafIlunMafruq, &[Z(Kaf)]),
            slot_of(FootId::Failatun, &[Z(Kaf)]),
        ],
        &rules
    ));
}

#[test]
fn mujtath_recognition_matches_literal_surface_matrix() {
    let p = profile();
    let bodies = body_recipes();
    let aruds = arud_recipes();
    let darbs = darb_recipes();
    let default_sadr = format!(
        "{}{}",
        body_surface(&bodies[0]),
        failatun_surface(&aruds[0])
    );
    let default_ajuz = format!(
        "{}{}",
        body_surface(&bodies[0]),
        failatun_surface(&darbs[0])
    );

    let mut expected_sadr = HashSet::new();
    for left in &bodies {
        for right in &aruds {
            if allowed(left, right) {
                expected_sadr.insert(format!("{}{}", body_surface(left), failatun_surface(right)));
            }
        }
    }
    let mut expected_ajuz = HashSet::new();
    for left in &bodies {
        for right in &darbs {
            if allowed(left, right) {
                expected_ajuz.insert(format!("{}{}", body_surface(left), failatun_surface(right)));
            }
        }
    }

    for left in &bodies {
        for right in &aruds {
            let sadr = format!("{}{}", body_surface(left), failatun_surface(right));
            assert_eq!(
                accepts(&p, &sadr, &default_ajuz),
                expected_sadr.contains(&sadr),
                "arud surface {left:?} / {right:?}"
            );
        }
    }
    for left in &bodies {
        for right in &darbs {
            let ajuz = format!("{}{}", body_surface(left), failatun_surface(right));
            assert_eq!(
                accepts(&p, &default_sadr, &ajuz),
                expected_ajuz.contains(&ajuz),
                "darb surface {left:?} / {right:?}"
            );
        }
    }

    let unsupported_arud = vec![Change::Tashith];
    let sadr = format!(
        "{}{}",
        body_surface(&bodies[0]),
        failatun_surface(&unsupported_arud)
    );
    assert!(!expected_sadr.contains(&sadr));
    assert!(!accepts(&p, &sadr, &default_ajuz));
    for unsupported_darb in [vec![Z(Kaf)], vec![Z(Khabn), Z(Kaf)]] {
        let ajuz = format!(
            "{}{}",
            body_surface(&bodies[0]),
            failatun_surface(&unsupported_darb)
        );
        assert!(!expected_ajuz.contains(&ajuz));
        assert!(!accepts(&p, &default_sadr, &ajuz));
    }
}
