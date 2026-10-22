//! Characterizes the selected convention; acceptance is not historical evidence.
use std::collections::HashSet;

use watad_daira::MeterName;
use watad_lugha::{
    catalog,
    meter::Scope,
    program::{self, Change},
};
use watad_tahwil::Zihaf::{Kaf, Khabn};
use watad_types::{Beat, FootId};

use Change::Zihaf as Z;

fn word(s: &str) -> Vec<Beat> {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '/' => Beat::Mutaharrik,
            '0' => Beat::Sakin,
            _ => panic!("invalid literal fixture"),
        })
        .collect()
}

#[test]
fn ordinary_cross_boundary_retains_exact_unconstrained_recipe_product() {
    let p = catalog::hashimi(MeterName::Mujtathth).unwrap().unwrap();
    // Independent literal recipes and surfaces, ordered N, Khabn, Kaf, Shakl.
    let recipes = [vec![], vec![Z(Khabn)], vec![Z(Kaf)], vec![Z(Khabn), Z(Kaf)]];
    let aruds = ["/0//0/0", "///0/0", "/0//0/", "///0/"];
    let bodies = ["/0/0//0", "//0//0", "/0/0//", "//0//"];
    assert_eq!(p.definition().forms.len(), p.structured().forms.len());
    let mut actual = HashSet::new();
    for (definition, expanded) in p.definition().forms.iter().zip(&p.structured().forms) {
        if definition.context != 0
            || definition.sadr[0] != vec![vec![]]
            || definition.ajuz[1] != vec![vec![]]
        {
            continue;
        }
        assert_eq!(definition.sadr[1].len(), 1);
        assert_eq!(definition.ajuz[0].len(), 1);
        let arud = &definition.sadr[1][0];
        let body = &definition.ajuz[0][0];
        // Explicit identities matter: these reverse the within-half foot order.
        assert_eq!(expanded.sadr[1].units, FootId::Failatun.units());
        assert_eq!(expanded.ajuz[0].units, FootId::MustafIlunMafruq.units());
        for (slot, base, recipe) in [
            (&expanded.sadr[1], FootId::Failatun, arud),
            (&expanded.ajuz[0], FootId::MustafIlunMafruq, body),
        ] {
            assert_eq!(slot.programs.len(), 1);
            let expected = program::compile(base, recipe).unwrap();
            let actual = &slot.programs[0];
            assert_eq!(actual.scope, expected.scope);
            assert_eq!(actual.required_final, expected.required_final);
            assert_eq!(actual.actions, expected.actions);
            assert_eq!(actual.expected, expected.expected);
            assert_eq!(actual.suffix, expected.suffix);
        }
        assert!(
            actual.insert((arud.clone(), body.clone())),
            "duplicate boundary form"
        );
    }
    let expected: HashSet<_> = recipes
        .iter()
        .flat_map(|a| recipes.iter().map(move |b| (a.clone(), b.clone())))
        .collect();
    assert_eq!(actual, expected);
    for (a, arud) in aruds.iter().enumerate() {
        for (b, body) in bodies.iter().enumerate() {
            let sadr = word(&format!("/0/0//0 {arud}"));
            let ajuz = word(&format!("{body} /0//0/0"));
            let scope = Scope {
                context: 0,
                initial: false,
            };
            let certificate = p
                .recognize(scope, &sadr, &ajuz)
                .unwrap()
                .unwrap_or_else(|| panic!("boundary {:?} -> {:?}", recipes[a], recipes[b]));
            assert!(p.check(scope, &sadr, &ajuz, &certificate));
            assert_eq!(certificate.profile, p.definition().id);
            assert_eq!(certificate.context, 0);
            let selected = &p.definition().forms[certificate.form];
            assert_eq!(selected.sadr, vec![vec![vec![]], vec![recipes[a].clone()]]);
            assert_eq!(selected.ajuz, vec![vec![recipes[b].clone()], vec![vec![]]]);
        }
    }
    // Includes arud Kaf -> initial Khabn (both adjacent sakin letters lost),
    // and arud Khabn -> initial Kaf (different, nonadjacent letters lost).
}

#[test]
fn moving_deletions_inside_either_half_still_rejects_forbidden_neighbors() {
    let p = catalog::hashimi(MeterName::Mujtathth).unwrap().unwrap();
    let scope = Scope {
        context: 0,
        initial: false,
    };
    for (sadr, ajuz) in [
        ("/0/0// ///0/0", "/0/0//0 /0//0/0"), // sadr Kaf -> Khabn
        ("//0//0 /0//0/", "/0/0//0 /0//0/0"), // sadr Khabn -> Kaf
        ("/0/0//0 /0//0/0", "/0/0// ///0/0"), // ajuz Kaf -> Khabn
    ] {
        assert!(
            p.recognize(scope, &word(sadr), &word(ajuz))
                .unwrap()
                .is_none(),
            "{sadr} | {ajuz}"
        );
    }
}
