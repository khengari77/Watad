use watad_lugha::catalog;
use watad_qasida::Verse;
use watad_types::Beat;

fn main() -> Result<(), watad_lugha::meter::Error> {
    let meter = catalog::meters()
        .find(|m| m.name_ar() == "الهزج")
        .expect("known meter");
    let profile = catalog::hashimi(meter)?.expect("used meter has a usage profile");
    let profiles = [profile];
    let half: Vec<_> = "//0/0/0//0/0/0"
        .chars()
        .map(|c| match c {
            '/' => Beat::Mutaharrik,
            '0' => Beat::Sakin,
            _ => unreachable!("literal beats"),
        })
        .collect();
    let verses: Vec<_> = (0..2)
        .map(|_| Verse {
            context: 0,
            sadr: half.clone(),
            ajuz: half.clone(),
        })
        .collect();

    for matched in watad_qasida::recognize(&profiles, &verses)? {
        let profile = &profiles[matched.profile];
        assert!(watad_qasida::check(profile, &verses, &matched.certificates));
        println!(
            "{}: {} checked verses",
            profile.definition().meter.name_ar(),
            matched.certificates.len()
        );
    }
    Ok(())
}
