use watad_checker::{
    adapter::encode_units,
    layout,
    obligations::Requirements,
    profile::Profile,
    structured::{self, FootSlot},
    verse,
};
use watad_daira::MeterName;
use watad_lugha::{catalog, program};
use watad_types::FootId;

fn definition(meter: MeterName, corrupt: bool) -> structured::Profile {
    let half = || {
        meter
            .feet()
            .iter()
            .map(|base| FootSlot {
                units: encode_units(if corrupt && *base == FootId::MustafIlunMafruq {
                    FootId::MustafIlun
                } else {
                    *base
                }),
                programs: vec![program::compile(*base, &[]).unwrap()],
            })
            .collect()
    };
    structured::Profile {
        id: verse::ProfileId {
            family: 9000,
            revision: 1,
        },
        binding: verse::BindingPolicy::FixedDarbClass,
        forms: vec![structured::Form {
            context: 0,
            initial_only: false,
            class: verse::DarbClass(0),
            sadr: half(),
            ajuz: half(),
            sadr_constraints: vec![],
            ajuz_constraints: vec![],
        }],
    }
}

#[test]
fn every_generated_meter_is_independently_checked_against_its_circle() {
    for meter in catalog::meters() {
        let (c, r, n) = (
            meter.circle().id(),
            meter.rotation(),
            meter.circle().reps_per_hemistich(),
        );
        let feet = meter.feet();
        assert!(layout::check(c, r, n, feet), "{meter:?}");
        assert!(!layout::check(c, r, n, &feet[..feet.len() - 1]));
        let mut extra = feet.to_vec();
        extra.push(feet[0]);
        assert!(!layout::check(c, r, n, &extra));
        assert!(!layout::check(c, usize::MAX, n, feet));
        assert!(!layout::check(usize::MAX, r, n, feet));
        assert!(!layout::check(c, r, 0, feet));
        assert!(!layout::check(c, r, usize::MAX, feet));
        let p = Profile::new_bound(
            definition(meter, false),
            Requirements { groups: vec![] },
            c,
            r,
            n,
            feet.to_vec(),
        )
        .unwrap();
        assert!(p.is_circle_bound());
        let origin = p.origin().unwrap();
        assert_eq!((origin.0, origin.1, origin.2), (c, r, n));
        assert_eq!(origin.3, feet);
    }
}

#[test]
fn equal_surface_feet_cannot_forge_circle_or_profile_ownership() {
    let m = MeterName::Khafif;
    let (c, r, n) = (
        m.circle().id(),
        m.rotation(),
        m.circle().reps_per_hemistich(),
    );
    let mut wrong = m.feet().to_vec();
    wrong[1] = FootId::MustafIlun;
    assert!(!layout::check(c, r, n, &wrong));
    // The corrupted words remain well-formed generic feet and have exactly
    // the same beats. Circle-bound construction must still reject them.
    assert!(Profile::new(definition(m, true), Requirements { groups: vec![] }).is_ok());
    assert!(
        Profile::new_bound(
            definition(m, true),
            Requirements { groups: vec![] },
            c,
            r,
            n,
            m.feet().to_vec()
        )
        .is_err()
    );
}
