use std::fmt::Write as _;
use watad_lugha::catalog;

fn hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

const BASELINE: &[(usize, u64)] = &[
    (0, 14392082618841318173),
    (1, 13687479835588942898),
    (3, 4700440537733232076),
    (5, 11926285994852746849),
    (6, 11631999485730656010),
    (8, 11334243385780839483),
    (9, 8682980753362656279),
    (10, 11431288835845911496),
    (11, 2136104517324997127),
    (14, 14870756833396224155),
    (15, 15185450832149317457),
    (16, 4275819842523201680),
    (17, 13689194395925248849),
    (18, 11627960058083806691),
    (20, 10189063652370708246),
    (21, 685073681851426056),
];

#[test]
fn catalog_structure_matches_the_reviewed_baseline() {
    let mut usage = 0;
    for meter in catalog::meters() {
        let Some(p) = catalog::hashimi(meter).unwrap() else {
            continue;
        };
        let d = p.definition();
        let (forms, requirements) =
            catalog::reference_expand_forms(meter, d.forms.clone(), d.requirements.clone())
                .unwrap();
        let mut s = String::new();
        let _ = write!(
            s,
            "{} {} {:?} {}",
            forms.len(),
            requirements.len(),
            d.binding,
            forms.iter().map(|f| f.feet).sum::<usize>()
        );
        for f in &forms {
            let _ = write!(
                s,
                "|{},{},{},{},{}",
                f.feet,
                f.context,
                f.initial_only,
                f.class.0,
                f.constraints.len()
            );
            for r in &f.constraints {
                let _ = write!(
                    s,
                    "c{},{},{},{}",
                    r.left, r.left_letter, r.right, r.right_letter
                );
            }
            for slot in &f.sadr {
                let _ = write!(s, "s[");
                for recipe in slot {
                    let _ = write!(s, "{recipe:?},");
                }
                let _ = write!(s, "]");
            }
            for slot in &f.ajuz {
                let _ = write!(s, "a[");
                for recipe in slot {
                    let _ = write!(s, "{recipe:?},");
                }
                let _ = write!(s, "]");
            }
        }
        for g in &requirements {
            let _ = write!(s, "g{g:?}");
        }
        let _ = write!(s, "reqs{:?}", requirements);
        let actual = hash(s.as_bytes());
        let expected = BASELINE
            .iter()
            .find_map(|(id, hash)| (*id == meter.id()).then_some(*hash))
            .unwrap_or_else(|| panic!("unexpected usage meter {}", meter.name_ar()));
        assert_eq!(actual, expected, "{}", meter.name_ar());
        usage += 1;
    }
    assert_eq!(usage, BASELINE.len());
}
