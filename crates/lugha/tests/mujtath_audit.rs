use std::collections::HashSet;

use watad_checker::{
    adjacency::{self, Rule},
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

#[test]
fn mujtath_declared_matrix_is_exact_endpoint_and_reciprocity_rules() {
    let p = profile();
    let rules = expected_rules();
    let mut ordinary = 0;
    let mut bodies = HashSet::new();
    let mut aruds = HashSet::new();
    let mut darbs = HashSet::new();
    let mut sadr_pairs = HashSet::new();
    let mut ajuz_pairs = HashSet::new();
    let mut combinations = HashSet::new();

    for f in p.definition().forms.iter().filter(|f| f.context == 0) {
        ordinary += 1;
        assert_eq!(f.feet, 2);
        assert_eq!(f.constraints, rules);
        assert_eq!(f.sadr.len(), 2);
        assert_eq!(f.ajuz.len(), 2);

        for recipe in &f.sadr[0] {
            bodies.insert(recipe.clone());
        }
        for recipe in &f.sadr[1] {
            aruds.insert(recipe.clone());
        }
        for recipe in &f.ajuz[0] {
            bodies.insert(recipe.clone());
        }
        for recipe in &f.ajuz[1] {
            darbs.insert(recipe.clone());
        }
        for left in &f.sadr[0] {
            for right in &f.sadr[1] {
                sadr_pairs.insert((left.clone(), right.clone()));
            }
        }
        for left in &f.ajuz[0] {
            for right in &f.ajuz[1] {
                ajuz_pairs.insert((left.clone(), right.clone()));
            }
        }
        assert_eq!(f.sadr[0].len(), 1);
        assert_eq!(f.sadr[1].len(), 1);
        assert_eq!(f.ajuz[0].len(), 1);
        assert_eq!(f.ajuz[1].len(), 1);
        combinations.insert((
            f.sadr[0][0].clone(),
            f.sadr[1][0].clone(),
            f.ajuz[0][0].clone(),
            f.ajuz[1][0].clone(),
        ));
    }

    assert_eq!(ordinary, 90);
    assert_eq!(bodies, recipe_set(&body_recipes()));
    assert_eq!(aruds, recipe_set(&arud_recipes()));
    assert_eq!(darbs, recipe_set(&darb_recipes()));
    assert_eq!(sadr_pairs, expected_pairs(&body_recipes(), &arud_recipes()));
    assert_eq!(ajuz_pairs, expected_pairs(&body_recipes(), &darb_recipes()));
    assert_eq!(sadr_pairs.len(), 9);
    assert_eq!(ajuz_pairs.len(), 10);

    let mut expected_forms = HashSet::new();
    for (left_body, left_arud) in &sadr_pairs {
        for (right_body, right_darb) in &ajuz_pairs {
            expected_forms.insert((
                left_body.clone(),
                left_arud.clone(),
                right_body.clone(),
                right_darb.clone(),
            ));
        }
    }
    assert_eq!(combinations, expected_forms);

    let mut tasri = 0;
    let mut tasri_arud = HashSet::new();
    let mut tasri_darb = HashSet::new();
    for f in p.definition().forms.iter().filter(|f| f.context == 1) {
        tasri += 1;
        for recipe in &f.sadr[1] {
            tasri_arud.insert(recipe.clone());
        }
        for recipe in &f.ajuz[1] {
            tasri_darb.insert(recipe.clone());
        }
    }
    assert_eq!(tasri, 100);
    assert_eq!(tasri_arud, tasri_darb);
    assert_eq!(tasri_arud, recipe_set(&darb_recipes()));
    assert_eq!(tasri_darb, recipe_set(&darb_recipes()));

    for f in &p.structured().forms {
        assert_eq!(f.sadr_constraints, rules);
        assert_eq!(f.ajuz_constraints, rules);
        assert!(adjacency::check(&f.sadr, &f.sadr_constraints));
        assert!(adjacency::check(&f.ajuz, &f.ajuz_constraints));
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
