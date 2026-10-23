use std::collections::BTreeSet;
use watad_checker::{adapter::encode_beats, poem::Input, verse::Scope};
use watad_daira::MeterName;
use watad_lugha::catalog;
use watad_types::Beat;

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

#[test]
fn all_circles_and_all_sixteen_usage_entries_compile_and_produce_checked_witnesses() {
    let mut circles = 0;
    let mut usage = 0;
    for meter in catalog::meters() {
        let circle = catalog::circle(meter).unwrap();
        assert!(circle.is_circle_bound());
        assert_eq!(circle.kernel().forms[0].sadr.len(), meter.feet().len());
        circles += 1;
        if let Some(p) = catalog::hashimi(meter).unwrap_or_else(|e| panic!("{meter:?}: {e}")) {
            usage += 1;
            assert!(p.is_circle_bound());
            let mut boundaries = BTreeSet::new();
            for (f, form) in p.kernel().forms.iter().enumerate() {
                let source = &p.structured().forms[f];
                let sadr_choices =
                    watad_checker::expand::assignments(&source.sadr, &source.sadr_constraints);
                let ajuz_choices = if source.ajuz.is_empty() {
                    vec![Vec::new()]
                } else {
                    watad_checker::expand::assignments(&source.ajuz, &source.ajuz_constraints)
                };
                let Some(sadr_choice) = sadr_choices.first() else {
                    continue;
                };
                let Some(ajuz_choice) = ajuz_choices.first() else {
                    continue;
                };
                let s: Vec<_> = sadr_choice
                    .iter()
                    .enumerate()
                    .flat_map(|(slot, choice)| beats(&form.sadr[slot].words[*choice]))
                    .collect();
                let a: Vec<_> = ajuz_choice
                    .iter()
                    .enumerate()
                    .flat_map(|(slot, choice)| beats(&form.ajuz[slot].words[*choice]))
                    .collect();
                // Every branch gets local witness replay. Exercise the public
                // profile boundary per distinct policy/layout combination,
                // avoiding quadratic repeated whole-profile validation.
                if !boundaries.insert((
                    form.context,
                    form.class.0,
                    form.sadr.len(),
                    form.ajuz.len(),
                )) {
                    continue;
                }
                let scope = Scope {
                    context: form.context,
                    initial: true,
                };
                let c = p
                    .recognize(scope, &s, &a)
                    .unwrap()
                    .unwrap_or_else(|| panic!("{meter:?} form {f}"));
                assert!(p.check(scope, &s, &a, &c));
                let mut extra = s.clone();
                extra.push(Beat::Sakin);
                assert!(!p.check(scope, &extra, &a, &c));
                assert_eq!(encode_beats(&s).len(), s.len());
            }
        }
    }
    assert_eq!((circles, usage), (22, 16));
}

#[test]
fn wafir_requires_a_witness_of_the_unaltered_shortened_arud() {
    let p = catalog::hashimi(MeterName::Wafir).unwrap().unwrap();
    let f = &p.kernel().forms[3]; // Altered arud, original shortened class.
    let s: Vec<_> = f.sadr.iter().flat_map(|s| s.words[0].clone()).collect();
    let a: Vec<_> = f.ajuz.iter().flat_map(|s| s.words[0].clone()).collect();
    let input = Input {
        context: 0,
        sadr: &s,
        ajuz: &a,
    };
    assert!(p.recognize_poem(&[input]).unwrap().is_none());
    let input = Input {
        context: 0,
        sadr: &s,
        ajuz: &a,
    };
    assert!(
        p.recognize_poem(&[
            input,
            Input {
                context: 0,
                sadr: &s,
                ajuz: &a,
            }
        ])
        .unwrap()
        .is_none()
    );
    let f = &p.kernel().forms[2];
    let s_alt_darb: Vec<_> = f.sadr.iter().flat_map(|s| s.words[0].clone()).collect();
    let a_alt_darb: Vec<_> = f.ajuz.iter().flat_map(|s| s.words[0].clone()).collect();
    assert!(
        p.recognize_poem(&[Input {
            context: 0,
            sadr: &s_alt_darb,
            ajuz: &a_alt_darb,
        }])
        .unwrap()
        .is_some()
    );
    let f = &p.kernel().forms[1];
    let good_s: Vec<_> = f.sadr.iter().flat_map(|s| s.words[0].clone()).collect();
    let input = [
        Input {
            context: 0,
            sadr: &s,
            ajuz: &a,
        },
        Input {
            context: 0,
            sadr: &good_s,
            ajuz: &a,
        },
    ];
    let c = p.recognize_poem(&input).unwrap().unwrap();
    assert!(p.check_poem(&input, &c));
}
