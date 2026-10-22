use watad_lugha::{catalog, meter::Scope};

fn main() -> Result<(), watad_lugha::meter::Error> {
    let meter = catalog::meters()
        .find(|m| m.name_ar() == "الهزج")
        .expect("known meter");
    let profile = catalog::hashimi(meter)?.expect("used meter has a usage profile");
    let profiles = [profile];
    let sadr = watad_matcher::parse("//0/0/0 //0/0/0").expect("literal beats");
    let ajuz = sadr.clone();
    let scope = Scope {
        context: 0,
        initial: false,
    };

    for matched in watad_matcher::recognize(&profiles, scope, &sadr, &ajuz)? {
        let profile = &profiles[matched.profile];
        let reading = watad_matcher::explain(profile, scope, &sadr, &ajuz, &matched.certificate)
            .expect("recognized certificate replays");
        println!("{}", profile.definition().meter.name_ar());
        for foot in reading.sadr {
            println!("{} {:?} {:?}", foot.base.name_ar(), foot.changes, foot.span);
        }
    }
    Ok(())
}
