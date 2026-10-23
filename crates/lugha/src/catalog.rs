//! Source data kept outside the mathematical kernel. Circle profiles describe
//! generated rotations; usage profiles describe the explicitly documented subset.
use crate::{
    meter::{self, DarbClass, Definition, Form, MeterProfile, ProfileId},
    program::Change,
};
use Change::{Illa as I, Zihaf as Z};
use watad_checker::verse::BindingPolicy;
use watad_daira::{CircleId, MeterName};
use watad_tahwil::{Illa, Zihaf};
use watad_types::FootId;

pub const HASHIMI: &str = "Hashimi terminal matrix with explicit Kafi/Qistas licences; sources and conventions: spec/profiles/classical-v1.md";

/// Caller-supplied context facts; beat strings cannot establish Arabic rhyme.
pub const TASRI: u16 = 1;
pub const RIDF: u16 = 2;

pub fn meters() -> impl Iterator<Item = MeterName> {
    (0..5).flat_map(|c| CircleId::from_id(c).meters().iter().copied())
}

fn branch(meter: MeterName, n: usize, arud: &[Change], darb: &[Change], class: u16) -> Form {
    let mut sadr = vec![vec![vec![]]; n];
    let mut ajuz = sadr.clone();
    sadr[n - 1] = vec![arud.to_vec()];
    ajuz[n - 1] = vec![darb.to_vec()];
    debug_assert!(n <= meter.feet().len());
    let mut constraints: Vec<watad_checker::adjacency::Rule> = if matches!(
        meter,
        MeterName::Madid | MeterName::Ramal | MeterName::Khafif | MeterName::Mujtathth
    ) {
        (0..n - 1)
            .filter(|i| {
                meter.feet()[*i]
                    .units()
                    .iter()
                    .map(|u| u.beat_count())
                    .sum::<usize>()
                    >= 7
            })
            .map(|i| watad_checker::adjacency::Rule {
                left: i,
                left_letter: 6,
                right: i + 1,
                right_letter: 1,
            })
            .collect()
    } else {
        vec![]
    };
    if meter == MeterName::Mujtathth {
        constraints.extend((0..n - 1).map(|i| watad_checker::adjacency::Rule {
            left: i,
            left_letter: 1,
            right: i + 1,
            right_letter: 6,
        }));
    }
    Form {
        constraints,
        feet: n,
        context: 0,
        initial_only: false,
        class: DarbClass(class),
        sadr,
        ajuz,
    }
}

/// All 22 generated circles, including the six unused rotations. This never
/// claims that an unchanged circular form is an attested usage form.
pub fn circle(meter: MeterName) -> Result<MeterProfile, meter::Error> {
    MeterProfile::new(Definition {
        id: ProfileId {
            family: 1000 + meter.id() as u16,
            revision: 1,
        },
        meter,
        source: "spec/circles.md §§3–4, [S1–S3]",
        binding: BindingPolicy::FixedDarbClass,
        forms: vec![branch(meter, meter.feet().len(), &[], &[], 0)],
        requirements: vec![],
    })
}

fn body(meter: MeterName, base: FootId) -> Vec<Vec<Change>> {
    use MeterName::*;
    use Zihaf::*;
    let mut choices = vec![vec![]];
    let variants: Vec<Vec<Change>> = match meter {
        Tawil | Hazaj => {
            if base == FootId::Mafailun {
                vec![vec![Z(Qabd)], vec![Z(Kaf)]]
            } else {
                vec![vec![Z(Qabd)]]
            }
        }
        Madid => {
            if base == FootId::Failatun {
                vec![vec![Z(Khabn)], vec![Z(Kaf)]]
            } else {
                vec![vec![Z(Khabn)]]
            }
        }
        Basit => {
            if base == FootId::MustafIlun {
                vec![vec![Z(Khabn)], vec![Z(Tayy)], vec![Z(Khabn), Z(Tayy)]]
            } else {
                vec![vec![Z(Khabn)]]
            }
        }
        Wafir => vec![vec![Z(Asb)], vec![Z(Aql)], vec![Z(Asb), Z(Kaf)]],
        Kamil => vec![vec![Z(Idmar)], vec![Z(Waqs)], vec![Z(Idmar), Z(Tayy)]],
        Rajaz | Sari => vec![vec![Z(Khabn)], vec![Z(Tayy)], vec![Z(Khabn), Z(Tayy)]],
        Ramal => vec![vec![Z(Khabn)], vec![Z(Kaf)], vec![Z(Khabn), Z(Kaf)]],
        Khafif => vec![vec![Z(Khabn)], vec![Z(Kaf)]],
        Munsarih => vec![vec![Z(Khabn)], vec![Z(Tayy)]],
        Mudari => return vec![vec![Z(Qabd)], vec![Z(Kaf)]],
        Muqtadab => return vec![vec![Z(Tayy)], vec![Z(Khabn)]],
        Mujtathth => vec![vec![Z(Khabn)], vec![Z(Kaf)], vec![Z(Khabn), Z(Kaf)]],
        Mutakarib => vec![vec![Z(Qabd)]],
        Mutadarak => vec![vec![Z(Khabn)], vec![Change::NonbindingIlla(Illa::Qat)]],
        _ => vec![],
    };
    choices.extend(variants);
    choices
}

fn optional_zihaf(choices: &mut Vec<Vec<Change>>, z: Zihaf) {
    for recipe in choices.clone() {
        if recipe.contains(&Z(z)) || recipe.contains(&Change::Tashith) {
            continue;
        }
        let mut changed = recipe;
        changed.push(Z(z));
        if !choices.contains(&changed) {
            choices.push(changed);
        }
    }
}

/// Compile one source form into every completed half that satisfies the
/// verified adjacency expansion. Source order and duplicate alternatives are
/// preserved by index.
fn expand_form(meter: MeterName, form: &Form) -> Result<Vec<Form>, meter::Error> {
    let sadr_slots =
        watad_checker::catalog::lower_slots(meter.feet(), &form.sadr).map_err(map_catalog_error)?;
    let sadr_choices = watad_checker::expand::assignments(&sadr_slots, &form.constraints);
    let ajuz_choices = if form.ajuz.is_empty() {
        vec![Vec::new()]
    } else {
        let ajuz_slots = watad_checker::catalog::lower_slots(meter.feet(), &form.ajuz)
            .map_err(map_catalog_error)?;
        watad_checker::expand::assignments(&ajuz_slots, &form.constraints)
    };
    watad_checker::catalog::expand_form_choices(form, &sadr_choices, &ajuz_choices)
        .map_err(map_catalog_error)
}

fn map_catalog_error(error: watad_checker::catalog::Error) -> meter::Error {
    match error {
        watad_checker::catalog::Error::Recipe(error) => meter::Error::Recipe(error),
        watad_checker::catalog::Error::Layout => meter::Error::Layout,
        watad_checker::catalog::Error::Requirement => meter::Error::Requirement,
    }
}

/// Remove only rows identical in every semantic field and in requirement-group
/// membership, keeping the first source coordinate. A duplicate that belongs to
/// a different witness group is not interchangeable even when its recipes match.
fn remove_exact_duplicates(
    forms: Vec<Form>,
    requirements: Vec<Vec<usize>>,
) -> (Vec<Form>, Vec<Vec<usize>>) {
    let membership: Vec<Vec<bool>> = (0..forms.len())
        .map(|index| {
            requirements
                .iter()
                .map(|group| group.contains(&index))
                .collect()
        })
        .collect();
    let mut unique: Vec<(Form, Vec<bool>)> = Vec::new();
    let mut old_to_new = Vec::with_capacity(forms.len());
    for (index, form) in forms.into_iter().enumerate() {
        match unique.iter().position(|(kept, kept_membership)| {
            kept == &form && kept_membership == &membership[index]
        }) {
            Some(index) => old_to_new.push(index),
            None => {
                old_to_new.push(unique.len());
                unique.push((form, membership[index].clone()));
            }
        }
    }
    let unique = unique.into_iter().map(|(form, _)| form).collect();
    let requirements = requirements
        .into_iter()
        .map(|group| {
            let mut mapped: Vec<_> = group.into_iter().map(|index| old_to_new[index]).collect();
            mapped.sort_unstable();
            mapped.dedup();
            mapped
        })
        .collect();
    (unique, requirements)
}

/// Apply the verified half expansion to every source form and remap poem
/// requirement groups through the resulting contiguous source-form ranges.
/// Reference expander retained as an independent test oracle. Production
/// construction stores the source rows and reports them directly.
#[doc(hidden)]
pub fn reference_expand_forms(
    meter: MeterName,
    forms: Vec<Form>,
    requirements: Vec<Vec<usize>>,
) -> Result<(Vec<Form>, Vec<Vec<usize>>), meter::Error> {
    if !matches!(
        meter,
        MeterName::Madid | MeterName::Ramal | MeterName::Khafif | MeterName::Mujtathth
    ) {
        return Ok((forms, requirements));
    }
    let mut groups = Vec::new();
    for form in &forms {
        groups.push(expand_form(meter, form)?);
    }
    let (expanded, _, remapped) = watad_checker::catalog::compile_groups(&groups, &requirements)
        .map_err(map_catalog_error)?;
    Ok((expanded, remapped))
}

fn usage_licences(meter: MeterName, forms: &mut Vec<Form>) {
    use Illa::*;
    use MeterName::*;
    use Zihaf::*;
    // Explicit single-line forms: the absent ajuz is empty, never a duplicate
    // of the line or a fabricated second hemistich.
    let singles: Vec<(usize, Vec<Change>)> = match meter {
        Rajaz => vec![(3, vec![]), (3, vec![I(Qat)]), (2, vec![])],
        Sari => vec![(3, vec![I(Waqf)]), (3, vec![I(Kasf)])],
        Munsarih => vec![(2, vec![I(Waqf)]), (2, vec![I(Kasf)])],
        _ => vec![],
    };
    for (i, (n, end)) in singles.iter().enumerate() {
        let mut f = branch(meter, *n, end, &[], 100 + i as u16);
        f.ajuz.clear();
        forms.push(f);
    }
    for f in forms.iter_mut() {
        for half in [&mut f.sadr, &mut f.ajuz] {
            for i in 0..half.len().saturating_sub(1) {
                half[i] = body(meter, meter.feet()[i]);
            }
        }
        if meter == Kamil {
            optional_zihaf(&mut f.sadr[f.feet - 1], Idmar);
            if !f.ajuz.is_empty() {
                optional_zihaf(&mut f.ajuz[f.feet - 1], Idmar);
            }
        }
        if meter == Hazaj {
            optional_zihaf(&mut f.sadr[f.feet - 1], Kaf);
        }
        if meter == Mudari {
            optional_zihaf(&mut f.sadr[f.feet - 1], Kaf);
        }
        if meter == Rajaz {
            for half in [&mut f.sadr, &mut f.ajuz] {
                if let Some(end) = half.last_mut() {
                    optional_zihaf(end, Khabn);
                    if end[0].is_empty() {
                        optional_zihaf(end, Tayy);
                    }
                }
            }
        }
        if meter == Basit && f.feet == 3 {
            optional_zihaf(&mut f.sadr[f.feet - 1], Khabn);
            optional_zihaf(&mut f.ajuz[f.feet - 1], Khabn);
        }
        if matches!(meter, Ramal | Khafif | Mujtathth | Mutadarak) {
            optional_zihaf(&mut f.sadr[f.feet - 1], Khabn);
            if !f.ajuz.is_empty() {
                optional_zihaf(&mut f.ajuz[f.feet - 1], Khabn);
            }
        }
        if meter == Mujtathth {
            // Selected strict reading: Kaf/Shakl belong to the arud only;
            // the preceding sin must survive (al-Qistas).
            f.sadr[f.feet - 1].extend([vec![Z(Kaf)], vec![Z(Khabn), Z(Kaf)]]);
        }
        if meter == Mutakarib && f.feet == 4 {
            f.sadr[f.feet - 1].extend([vec![Z(Qabd)], vec![Change::NonbindingIlla(Hadhf)]]);
        }
        if meter == Mutadarak {
            if f.sadr[f.feet - 1][0].is_empty() {
                f.sadr[f.feet - 1].push(vec![Change::NonbindingIlla(Qat)]);
            }
            if !f.ajuz.is_empty() && f.ajuz[f.feet - 1][0].is_empty() {
                f.ajuz[f.feet - 1].push(vec![Change::NonbindingIlla(Qat)]);
            }
        }
        if matches!(meter, Tawil | Wafir | Hazaj | Mudari | Mutakarib) {
            for mut recipe in f.sadr[0].clone() {
                recipe.push(I(Kharm));
                f.sadr[0].push(recipe);
            }
        }
    }
    // Tasri is a separate externally selected context and a first-verse-only
    // branch. Its binding class remains the ordinary branch's class.
    if meter == Basit {
        for f in forms.iter_mut() {
            if f.feet == 4 && f.ajuz[f.feet - 1].iter().any(|r| r.contains(&I(Qat))) {
                f.context = RIDF;
            }
        }
        for mut f in forms.clone().into_iter().filter(|f| f.context == 0) {
            f.context = RIDF;
            forms.push(f);
        }
    }
    for mut f in forms.clone().into_iter().filter(|f| !f.ajuz.is_empty()) {
        f.context |= TASRI;
        f.initial_only = true;
        f.sadr[f.feet - 1] = f.ajuz[f.feet - 1].clone();
        forms.push(f);
    }
}

/// Terminal-pair matrix of al-Hashimi's numbered meter chapters. Body licences
/// and exceptional single-line forms are tracked separately in the coverage
/// inventory; this profile does not silently infer them from structural fit.
pub fn hashimi(meter: MeterName) -> Result<Option<MeterProfile>, meter::Error> {
    use Illa::*;
    use MeterName::*;
    use Zihaf::*;
    let pairs: Vec<(usize, Vec<Change>, Vec<Change>)> = match meter {
        Tawil => vec![
            (4, vec![Z(Qabd)], vec![]),
            (4, vec![Z(Qabd)], vec![Z(Qabd)]),
            (4, vec![Z(Qabd)], vec![I(Hadhf)]),
        ],
        Madid => vec![
            (3, vec![], vec![]),
            (3, vec![I(Hadhf)], vec![I(Qasr)]),
            (3, vec![I(Hadhf)], vec![I(Hadhf)]),
            (3, vec![I(Hadhf)], vec![I(Batr)]),
            (3, vec![Z(Khabn), I(Hadhf)], vec![Z(Khabn), I(Hadhf)]),
            (3, vec![Z(Khabn), I(Hadhf)], vec![I(Batr)]),
        ],
        Basit => vec![
            (4, vec![Z(Khabn)], vec![Z(Khabn)]),
            (4, vec![Z(Khabn)], vec![I(Qat)]),
            (3, vec![], vec![I(Tadhyil)]),
            (3, vec![], vec![]),
            (3, vec![], vec![I(Qat)]),
            (3, vec![I(Qat)], vec![I(Qat)]),
        ],
        Wafir => vec![
            (3, vec![I(Qatf)], vec![I(Qatf)]),
            (2, vec![], vec![]),
            (2, vec![], vec![Z(Asb)]),
        ],
        Kamil => vec![
            (3, vec![], vec![]),
            (3, vec![], vec![I(Qat)]),
            (3, vec![], vec![Z(Idmar), I(Hadhadh)]),
            (3, vec![I(Hadhadh)], vec![I(Hadhadh)]),
            (3, vec![I(Hadhadh)], vec![Z(Idmar), I(Hadhadh)]),
            (2, vec![], vec![I(Tarfil)]),
            (2, vec![], vec![I(Tadhyil)]),
            (2, vec![], vec![]),
            (2, vec![], vec![I(Qat)]),
        ],
        Hazaj => vec![(2, vec![], vec![]), (2, vec![], vec![I(Hadhf)])],
        Rajaz => vec![
            (3, vec![], vec![]),
            (3, vec![], vec![I(Qat)]),
            (2, vec![], vec![]),
        ],
        Ramal => vec![
            (3, vec![I(Hadhf)], vec![]),
            (3, vec![I(Hadhf)], vec![I(Qasr)]),
            (3, vec![I(Hadhf)], vec![I(Hadhf)]),
            (2, vec![], vec![I(Tasbigh)]),
            (2, vec![], vec![]),
            (2, vec![], vec![I(Hadhf)]),
        ],
        Sari => vec![
            (3, vec![Z(Tayy), I(Kasf)], vec![Z(Tayy), I(Waqf)]),
            (3, vec![Z(Tayy), I(Kasf)], vec![Z(Tayy), I(Kasf)]),
            (3, vec![Z(Tayy), I(Kasf)], vec![I(Salm)]),
            (
                3,
                vec![Z(Khabn), Z(Tayy), I(Kasf)],
                vec![Z(Khabn), Z(Tayy), I(Kasf)],
            ),
            (3, vec![Z(Khabn), Z(Tayy), I(Kasf)], vec![I(Salm)]),
        ],
        Munsarih => vec![
            (3, vec![], vec![Z(Tayy)]),
            (3, vec![], vec![I(Qat)]),
            (3, vec![Z(Tayy)], vec![Z(Tayy)]),
        ],
        Khafif => vec![
            (3, vec![], vec![]),
            (3, vec![], vec![I(Hadhf)]),
            (3, vec![I(Hadhf)], vec![I(Hadhf)]),
            (2, vec![], vec![]),
            (2, vec![], vec![Z(Khabn), I(Qasr)]),
        ],
        Mudari => vec![(2, vec![], vec![])],
        Muqtadab => vec![(2, vec![Z(Tayy)], vec![Z(Tayy)])],
        Mujtathth => vec![(2, vec![], vec![])],
        Mutakarib => vec![
            (4, vec![], vec![]),
            (4, vec![], vec![I(Qasr)]),
            (4, vec![], vec![I(Hadhf)]),
            (4, vec![], vec![I(Batr)]),
            (3, vec![I(Hadhf)], vec![I(Hadhf)]),
            (3, vec![I(Hadhf)], vec![I(Batr)]),
        ],
        Mutadarak => vec![
            (4, vec![], vec![]),
            (3, vec![], vec![]),
            (3, vec![], vec![Z(Khabn), I(Tarfil)]),
            (3, vec![], vec![I(Tadhyil)]),
        ],
        Mustatil | Mumtadd | Mutawafir | Mutaaddi | Munsarid | Mutarrid => return Ok(None),
    };
    let mut forms: Vec<_> = pairs
        .iter()
        .enumerate()
        .map(|(i, (n, a, d))| branch(meter, *n, a, d, i as u16))
        .collect();
    // Obligatory change: the first foot of Mudari is never سالم. These are
    // alternatives, so qabd and kaff cannot strike that foot together.
    if meter == Mudari {
        for f in &mut forms {
            f.sadr[0] = vec![vec![Z(Qabd)], vec![Z(Kaf)]];
            f.ajuz[0] = f.sadr[0].clone();
        }
    }
    // Tashith is optional and does not establish a different binding class.
    if meter == Mujtathth || meter == Khafif {
        for f in &mut forms {
            if (meter == Mujtathth || f.feet == meter.feet().len())
                && f.ajuz[f.feet - 1] == vec![vec![]]
            {
                f.ajuz[f.feet - 1].push(vec![Change::Tashith]);
            }
        }
    }
    let mut requirements = vec![];
    if meter == Wafir {
        // The unaltered shortened arud must occur at least once. Full Wafir
        // trivially discharges this condition; altered shortened branches share
        // the original class and cannot satisfy it by themselves.
        requirements.push((0..forms.len()).collect());
        for f in forms.clone().into_iter().filter(|f| f.feet == 2) {
            let mut altered = f;
            altered.sadr[1] = vec![vec![Z(Asb)]];
            forms.push(altered);
        }
    }
    usage_licences(meter, &mut forms);
    let (forms, mut requirements) = remove_exact_duplicates(forms, requirements);
    if meter == Wafir {
        requirements = vec![
            forms
                .iter()
                .enumerate()
                .filter(|(_, f)| {
                    f.feet == 3 || f.sadr[f.feet - 1].iter().all(|r| !r.contains(&Z(Asb)))
                })
                .map(|(i, _)| i)
                .collect(),
        ];
    }
    MeterProfile::new(Definition {
        id: ProfileId {
            family: 2000 + meter.id() as u16,
            revision: if matches!(
                meter,
                MeterName::Madid
                    | MeterName::Ramal
                    | MeterName::Wafir
                    | MeterName::Khafif
                    | MeterName::Mujtathth
            ) {
                2
            } else {
                1
            },
        },
        meter,
        source: HASHIMI,
        binding: BindingPolicy::FixedDarbClass,
        forms,
        requirements,
    })
    .map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use watad_checker::poem;

    fn form() -> Form {
        Form {
            constraints: vec![],
            feet: 2,
            context: 0,
            initial_only: false,
            class: DarbClass(0),
            sadr: vec![vec![vec![]]; 2],
            ajuz: vec![vec![vec![]]; 2],
        }
    }

    fn profile(forms: Vec<Form>, requirements: Vec<Vec<usize>>) -> MeterProfile {
        MeterProfile::new(Definition {
            id: ProfileId {
                family: 9900,
                revision: 1,
            },
            meter: MeterName::Mujtathth,
            source: "dedup regression",
            binding: BindingPolicy::FixedDarbClass,
            forms,
            requirements,
        })
        .unwrap()
    }

    fn words(profile: &MeterProfile, form: usize) -> (Vec<u8>, Vec<u8>) {
        let k = &profile.kernel().forms[form];
        let sadr = k
            .sadr
            .iter()
            .flat_map(|slot| slot.words[0].iter().copied())
            .collect();
        let ajuz = k
            .ajuz
            .iter()
            .flat_map(|slot| slot.words[0].iter().copied())
            .collect();
        (sadr, ajuz)
    }

    #[test]
    fn exact_duplicates_with_same_witness_membership_merge() {
        let f = form();
        let (forms, requirements) = remove_exact_duplicates(vec![f.clone(), f], vec![vec![0, 1]]);
        assert_eq!(forms.len(), 1);
        assert_eq!(requirements, vec![vec![0]]);
    }

    #[test]
    fn duplicates_in_different_witness_groups_remain_distinct() {
        let f = form();
        let (forms, requirements) =
            remove_exact_duplicates(vec![f.clone(), f], vec![vec![0], vec![1]]);
        assert_eq!(forms.len(), 2);
        assert_eq!(requirements, vec![vec![0], vec![1]]);

        let p = profile(forms, requirements);
        let (s0, a0) = words(&p, 0);
        let (s1, a1) = words(&p, 1);
        let first = [poem::Input {
            context: 0,
            sadr: &s0,
            ajuz: &a0,
        }];
        assert!(p.recognize_poem(&first).unwrap().is_none());
        let both = [
            poem::Input {
                context: 0,
                sadr: &s0,
                ajuz: &a0,
            },
            poem::Input {
                context: 0,
                sadr: &s1,
                ajuz: &a1,
            },
        ];
        let certificates = p.recognize_poem(&both).unwrap().unwrap();
        assert!(p.check_poem(&both, &certificates));
    }
}
