//! Literal source fixtures; declared contexts are not independent historical evidence.
use watad_checker::poem::Input;
use watad_daira::MeterName;
use watad_lugha::{catalog, meter::Scope, program::Change};
use watad_tahwil::{Illa::Qatf, Zihaf::Asb};
use watad_types::{Beat, FootId};

fn beats(s: &str) -> Vec<Beat> {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '/' => Beat::Mutaharrik,
            '0' => Beat::Sakin,
            _ => panic!("bad fixture"),
        })
        .collect()
}
fn bits(s: &str) -> Vec<u8> {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '/' => 1,
            '0' => 0,
            _ => panic!("bad fixture"),
        })
        .collect()
}

#[test]
fn exact_terminal_recipes_classes_contexts_and_requirement_membership() {
    let p = catalog::hashimi(MeterName::Wafir).unwrap().unwrap();
    assert_eq!(p.definition().id.revision, 2);
    let n = vec![];
    let a = vec![Change::Zihaf(Asb)];
    let q = vec![Change::Illa(Qatf)];
    // Literal source-form order; identical tasri rows share one source coordinate.
    let rows = [
        (3, 0, false, 0, &q, &q),
        (2, 0, false, 1, &n, &n),
        (2, 0, false, 2, &n, &a),
        (2, 0, false, 1, &a, &n),
        (2, 0, false, 2, &a, &a),
        (3, 1, true, 0, &q, &q),
        (2, 1, true, 1, &n, &n),
        (2, 1, true, 2, &a, &a),
    ];
    assert_eq!(p.definition().forms.len(), rows.len());
    assert_eq!(p.structured().forms.len(), rows.len());
    assert_eq!(p.kernel().forms.len(), rows.len());
    assert_eq!(p.definition().requirements, vec![vec![0, 1, 2, 5, 6]]);
    for (i, (feet, context, initial, class, arud, darb)) in rows.iter().enumerate() {
        let f = &p.definition().forms[i];
        assert_eq!(
            (f.feet, f.context, f.initial_only, f.class.0),
            (*feet, *context, *initial, *class)
        );
        assert_eq!(f.sadr[*feet - 1], vec![(*arud).clone()]);
        assert_eq!(f.ajuz[*feet - 1], vec![(*darb).clone()]);
        for slot in p.structured().forms[i]
            .sadr
            .iter()
            .chain(&p.structured().forms[i].ajuz)
        {
            assert_eq!(slot.units, FootId::Mafalatun.units());
        }
        let k = &p.kernel().forms[i];
        assert_eq!(
            (k.context, k.initial_only, k.class.0),
            (*context, *initial, *class)
        );
        assert_eq!(k.sadr.len(), *feet);
        assert_eq!(k.ajuz.len(), *feet);
    }
}

#[test]
fn poem_witness_is_sound_arud_anywhere_not_sound_body_or_darb() {
    let p = catalog::hashimi(MeterName::Wafir).unwrap().unwrap();
    let sound = "//0///0";
    let asb = "//0/0/0"; // Same surface as Mafailun, still Wafir's Asb recipe.
    for (darb, class) in [(sound, 1), (asb, 2)] {
        let altered_s = bits(&format!("{sound} {asb}"));
        let witness_s = bits(&format!("{asb} {sound}"));
        let ajuz = bits(&format!("{sound} {darb}"));
        let input = |sadr| Input {
            context: 0,
            sadr,
            ajuz: &ajuz,
        };
        assert!(
            p.recognize_poem(&[input(&altered_s), input(&altered_s)])
                .unwrap()
                .is_none()
        );
        for position in 0..3 {
            let lines: Vec<_> = (0..3)
                .map(|i| {
                    input(if i == position {
                        &witness_s
                    } else {
                        &altered_s
                    })
                })
                .collect();
            let cs = p.recognize_poem(&lines).unwrap().unwrap();
            assert!(p.check_poem(&lines, &cs));
            for (i, c) in cs.iter().enumerate() {
                assert_eq!(c.profile, p.definition().id);
                assert_eq!(c.context, 0);
                let f = &p.definition().forms[c.form];
                assert_eq!(f.class.0, class);
                assert_eq!(f.feet, 2);
                assert_eq!(
                    f.sadr[1][c.sadr[1].choice],
                    if i == position {
                        vec![]
                    } else {
                        vec![Change::Zihaf(Asb)]
                    }
                );
            }
            let mut invalid = cs;
            invalid[position] = p
                .recognize(
                    Scope {
                        context: 0,
                        initial: position == 0,
                    },
                    &beats(&format!("{sound} {asb}")),
                    &beats(&format!("{sound} {darb}")),
                )
                .unwrap()
                .unwrap();
            let all_altered: Vec<_> = (0..3).map(|_| input(&altered_s)).collect();
            for (i, c) in invalid.iter().enumerate() {
                assert!(p.check(
                    Scope {
                        context: 0,
                        initial: i == 0
                    },
                    &beats(&format!("{sound} {asb}")),
                    &beats(&format!("{sound} {darb}")),
                    c
                ));
            }
            assert!(!p.check_poem(&all_altered, &invalid));
        }
    }
}

#[test]
fn canonical_short_example_and_context_binding_controls() {
    let p = catalog::hashimi(MeterName::Wafir).unwrap().unwrap();
    // Mizan printed/PDF50: أعاتبها وآمرها / فتغضبني وتعصيني.
    let sound = bits("//0///0 //0///0");
    let cut = bits("//0///0 //0/0/0");
    let line = |context, sadr, ajuz| Input {
        context,
        sadr,
        ajuz,
    };
    let canonical = [line(0, &sound, &cut)];
    let cs = p.recognize_poem(&canonical).unwrap().unwrap();
    assert!(p.check_poem(&canonical, &cs));
    assert_eq!(p.definition().forms[cs[0].form].class.0, 2);
    // Neighbor changes only the arud to Asb: no sound witness remains.
    assert!(p.recognize_poem(&[line(0, &cut, &cut)]).unwrap().is_none());
    // A witness in another binding class cannot rescue an assignment.
    assert!(
        p.recognize_poem(&[line(0, &sound, &sound), line(0, &cut, &cut)])
            .unwrap()
            .is_none()
    );
    // Declared tasri class 2 is itself altered, and cannot witness sound arud.
    assert!(
        p.recognize_poem(&[line(1, &cut, &cut), line(0, &cut, &cut)])
            .unwrap()
            .is_none()
    );
    assert!(
        p.recognize_poem(&[line(1, &cut, &cut), line(0, &sound, &cut)])
            .unwrap()
            .is_some()
    );
    assert!(
        p.recognize_poem(&[line(0, &sound, &cut), line(1, &cut, &cut)])
            .unwrap()
            .is_none()
    );
    let full = bits("//0///0 //0///0 //0/0");
    assert!(
        p.recognize_poem(&[line(0, &full, &full), line(0, &cut, &cut)])
            .unwrap()
            .is_none()
    );
    assert!(
        p.recognize_poem(&[line(0, &full, &full)])
            .unwrap()
            .is_some()
    );
}
