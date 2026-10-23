use watad_checker::{
    adjacency::{self, Rule},
    structured::FootSlot,
};
use watad_lugha::program::{self, Change};
use watad_tahwil::Zihaf;
use watad_types::FootId;

fn slot(base: FootId, choices: &[Option<Zihaf>]) -> FootSlot {
    FootSlot {
        units: base.units().to_vec(),
        programs: choices
            .iter()
            .map(|z| {
                program::compile(base, &z.map(Change::Zihaf).into_iter().collect::<Vec<_>>())
                    .unwrap()
            })
            .collect(),
    }
}

#[test]
fn every_pair_of_independent_alternatives_must_satisfy_the_exclusion() {
    let rule = Rule {
        left: 0,
        left_letter: 6,
        right: 1,
        right_letter: 1,
    };
    let bad = [
        slot(FootId::Failatun, &[None, Some(Zihaf::Kaf)]),
        slot(FootId::Failatun, &[None, Some(Zihaf::Khabn)]),
    ];
    assert!(!adjacency::check(&bad, &[rule]));
    let good = [
        slot(FootId::Failatun, &[None, Some(Zihaf::Kaf)]),
        slot(FootId::Failatun, &[None]),
    ];
    assert!(adjacency::check(&good, &[rule]));
    let good = [
        slot(FootId::Failatun, &[None]),
        slot(FootId::Failatun, &[None, Some(Zihaf::Khabn)]),
    ];
    assert!(adjacency::check(&good, &[rule]));
    for bad in [
        Rule {
            left: usize::MAX,
            ..rule
        },
        Rule {
            right_letter: usize::MAX,
            ..rule
        },
    ] {
        assert!(!adjacency::check(&good, &[bad]));
    }
}

#[test]
fn every_declared_catalog_constraint_is_scoped_to_its_selected_path() {
    for meter in watad_lugha::catalog::meters() {
        if let Some(p) = watad_lugha::catalog::hashimi(meter).unwrap() {
            for f in &p.structured().forms {
                for rule in &f.sadr_constraints {
                    assert!(adjacency::check_rule_scope_ok(&f.sadr, *rule));
                }
                for rule in &f.ajuz_constraints {
                    assert!(adjacency::check_rule_scope_ok(&f.ajuz, *rule));
                }
            }
        }
    }
}

#[test]
fn mujtath_strict_profile_preserves_both_correlations_and_endpoint_scope() {
    use Change::Zihaf as Z;
    use Zihaf::{Kaf, Khabn};
    let p = watad_lugha::catalog::hashimi(watad_daira::MeterName::Mujtathth)
        .unwrap()
        .unwrap();
    let d = p.definition();
    let (forms, _) = watad_lugha::catalog::reference_expand_forms(
        watad_daira::MeterName::Mujtathth,
        d.forms.clone(),
        d.requirements.clone(),
    )
    .unwrap();
    let mut arud_shakl = false;
    for f in &forms {
        assert_eq!(f.constraints.len(), 2);
        for half in [&f.sadr, &f.ajuz] {
            for left in &half[0] {
                for right in &half[1] {
                    assert!(!(left.contains(&Z(Kaf)) && right.contains(&Z(Khabn))));
                    assert!(!(left.contains(&Z(Khabn)) && right.contains(&Z(Kaf))));
                }
            }
        }
        assert!(f.ajuz[1].iter().all(|r| !r.contains(&Z(Kaf))));
        if f.context == 0
            && f.sadr[1]
                .iter()
                .any(|r| r.contains(&Z(Kaf)) && r.contains(&Z(Khabn)))
        {
            assert!(f.sadr[0].iter().all(Vec::is_empty));
            arud_shakl = true;
        }
    }
    assert!(arud_shakl);
}
