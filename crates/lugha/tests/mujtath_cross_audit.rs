//! Characterizes the selected convention; acceptance is not historical evidence.
use std::collections::HashSet;
use watad_checker::{choice, language::Step};

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
    for (i, definition) in p.definition().forms.iter().enumerate() {
        if definition.context != 0 {
            continue;
        }
        let structured = &p.structured().forms[i];
        let kernel = &p.kernel().forms[i];
        let outer = definition.sadr[0].iter().position(Vec::is_empty).unwrap();
        let end = definition.ajuz[1].iter().position(Vec::is_empty).unwrap();
        assert_eq!(structured.sadr[1].units, FootId::Failatun.units());
        assert_eq!(structured.ajuz[0].units, FootId::MustafIlunMafruq.units());
        for (a, arud) in definition.sadr[1].iter().enumerate() {
            for (b, body) in definition.ajuz[0].iter().enumerate() {
                let steps = |x, y| {
                    [
                        Step {
                            choice: x,
                            start: 0,
                            end: 0,
                        },
                        Step {
                            choice: y,
                            start: 0,
                            end: 0,
                        },
                    ]
                };
                if !choice::check_rules(&kernel.sadr_rules, &steps(outer, a))
                    || !choice::check_rules(&kernel.ajuz_rules, &steps(b, end))
                {
                    continue;
                }
                for (slot, base, recipe, index) in [
                    (&structured.sadr[1], FootId::Failatun, arud, a),
                    (&structured.ajuz[0], FootId::MustafIlunMafruq, body, b),
                ] {
                    let expected = program::compile(base, recipe).unwrap();
                    let actual = &slot.programs[index];
                    assert_eq!(actual.scope, expected.scope);
                    assert_eq!(actual.required_final, expected.required_final);
                    assert_eq!(actual.actions, expected.actions);
                    assert_eq!(actual.expected, expected.expected);
                    assert_eq!(actual.suffix, expected.suffix);
                }
                assert!(
                    actual.insert((arud.clone(), body.clone())),
                    "duplicate boundary assignment"
                );
            }
        }
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
            assert_eq!(selected.sadr[0][certificate.sadr[0].choice], vec![]);
            assert_eq!(selected.sadr[1][certificate.sadr[1].choice], recipes[a]);
            assert_eq!(selected.ajuz[0][certificate.ajuz[0].choice], recipes[b]);
            assert_eq!(selected.ajuz[1][certificate.ajuz[1].choice], vec![]);
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
