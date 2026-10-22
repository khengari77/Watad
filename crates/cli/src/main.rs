#![deny(unsafe_code)]
use std::process::ExitCode;
use watad_lugha::{
    catalog,
    meter::{MeterProfile, Scope},
};

const HELP: &str = "Watad — explicit beat-level recognition\n\
  watad meters\n\
  watad example METER [FORM] [--circle]\n\
  watad scan SADR AJUZ [--meter METER] [--circle] [--initial] [--context N]\n\
  watad poem FILE [--meter METER] [--circle]\n\
METER is a 0–21 circle-order ID or an Arabic meter name. Beats: / or 1 = moving,\n\
0 = still. Quote arguments containing spaces. A poem has one `[CONTEXT:] SADR |\n\
AJUZ` per line; an empty AJUZ represents a declared single-line form. Context 0\n\
is ordinary, 1 tasri (first verse only), 2 supplied ridf evidence for Basit, 3 both.\n\
--circle selects unchanged generated forms, including unused rotations.\n\
The usage catalog is a documented source subset, not a complete classical oracle.\n\
Standard Arabic spelling conversion is Layer 2 and is not performed here.";

fn load(circle: bool, selected: Option<&str>) -> Result<Vec<MeterProfile>, String> {
    let meters: Vec<_> = catalog::meters()
        .filter(|m| {
            selected.is_none_or(|s| s == m.name_ar() || s.parse::<usize>().ok() == Some(m.id()))
        })
        .collect();
    if meters.is_empty() {
        return Err("unknown meter".into());
    }
    let mut profiles = Vec::new();
    for meter in meters {
        if circle {
            profiles.push(catalog::circle(meter).map_err(|e| e.to_string())?);
        } else if let Some(p) = catalog::hashimi(meter).map_err(|e| e.to_string())? {
            profiles.push(p);
        }
    }
    if profiles.is_empty() {
        return Err("no usage profile for this unused rotation; select --circle".into());
    }
    Ok(profiles)
}

fn parse(s: &str) -> Result<Vec<watad_types::Beat>, String> {
    watad_matcher::parse(s)
        .map_err(|e| format!("invalid beat {:?} at byte {}", e.character, e.byte))
}

fn show(p: &MeterProfile, c: &watad_lugha::meter::Certificate) {
    let id = p.definition().id;
    println!(
        "{} · profile {}:{} · branch {} · class {}",
        p.definition().meter.name_ar(),
        id.family,
        id.revision,
        c.form,
        p.kernel().forms[c.form].class.0
    );
}

fn run() -> Result<bool, String> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() || args == ["--help"] || args == ["help"] {
        println!("{HELP}");
        return Ok(true);
    }
    let command = args.remove(0);
    let mut circle = false;
    let mut initial = false;
    let mut context = 0;
    let mut meter = None;
    let mut positional = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--circle" => circle = true,
            "--initial" => initial = true,
            "--meter" | "--context" => {
                let option = &args[i];
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| format!("{option} requires a value"))?;
                if option == "--meter" {
                    meter = Some(value.clone());
                } else {
                    context = value.parse::<u16>().map_err(|_| "context must be a u16")?;
                }
            }
            a if a.starts_with("--") => return Err(format!("unknown option {a}")),
            _ => positional.push(args[i].clone()),
        }
        i += 1;
    }
    match command.as_str() {
        "meters" if positional.is_empty() => {
            for m in catalog::meters() {
                let count = catalog::hashimi(m)
                    .map_err(|e| e.to_string())?
                    .map(|p| p.definition().forms.len())
                    .unwrap_or(0);
                println!(
                    "{}\t{}\t{}\t{} usage branches",
                    m.id(),
                    m.name_ar(),
                    m.status().name_ar(),
                    count
                );
            }
            Ok(true)
        }
        "example" if (1..=2).contains(&positional.len()) => {
            let profiles = load(circle, Some(&positional[0]))?;
            let p = &profiles[0];
            let f = positional
                .get(1)
                .map(|s| s.parse::<usize>())
                .transpose()
                .map_err(|_| "invalid branch")?
                .unwrap_or(0);
            let f = p.kernel().forms.get(f).ok_or("branch is out of range")?;
            if f.context != 0 {
                eprintln!(
                    "This branch requires context {}{}",
                    f.context,
                    if f.initial_only {
                        " and an initial verse"
                    } else {
                        ""
                    }
                );
            }
            let s: String = f
                .sadr
                .iter()
                .flat_map(|s| s.words[0].iter())
                .map(|b| if *b == 1 { '/' } else { '0' })
                .collect();
            let a: String = f
                .ajuz
                .iter()
                .flat_map(|s| s.words[0].iter())
                .map(|b| if *b == 1 { '/' } else { '0' })
                .collect();
            println!("{s} | {a}");
            Ok(true)
        }
        "scan" if positional.len() == 2 => {
            let profiles = load(circle, meter.as_deref())?;
            let (s, a) = (parse(&positional[0])?, parse(&positional[1])?);
            let scope = Scope { context, initial };
            let found =
                watad_matcher::recognize(&profiles, scope, &s, &a).map_err(|e| e.to_string())?;
            for m in &found {
                let p = &profiles[m.profile];
                show(p, &m.certificate);
                let reading = watad_matcher::explain(p, scope, &s, &a, &m.certificate)
                    .ok_or("certificate replay failed")?;
                for (end, feet) in [("صدر", reading.sadr), ("عجز", reading.ajuz)] {
                    for foot in feet {
                        let changes = if foot.changes.is_empty() {
                            "سالم".into()
                        } else {
                            foot.changes
                                .iter()
                                .map(|c| c.name_ar())
                                .collect::<Vec<_>>()
                                .join(" + ")
                        };
                        println!(
                            "  {end}: {} · {changes} · {}..{}",
                            foot.base.name_ar(),
                            foot.span.start,
                            foot.span.end
                        );
                    }
                }
            }
            Ok(!found.is_empty())
        }
        "poem" if positional.len() == 1 => {
            let profiles = load(circle, meter.as_deref())?;
            let text = std::fs::read_to_string(&positional[0]).map_err(|e| e.to_string())?;
            let mut verses = Vec::new();
            for (line, content) in text
                .lines()
                .enumerate()
                .filter(|(_, s)| !s.trim().is_empty())
            {
                let (line_context, content) = if let Some((prefix, tail)) = content.split_once(':')
                {
                    (
                        prefix
                            .trim()
                            .parse::<u16>()
                            .map_err(|_| format!("line {} has an invalid context", line + 1))?,
                        tail,
                    )
                } else {
                    (context, content)
                };
                let (s, a) = content
                    .split_once('|')
                    .ok_or_else(|| format!("line {} requires SADR | AJUZ", line + 1))?;
                verses.push(watad_qasida::Verse {
                    context: line_context,
                    sadr: parse(s)?,
                    ajuz: parse(a)?,
                });
            }
            let found = watad_qasida::recognize(&profiles, &verses).map_err(|e| e.to_string())?;
            for m in &found {
                let p = &profiles[m.profile];
                if !watad_qasida::check(p, &verses, &m.certificates) {
                    return Err("poem certificate replay failed".into());
                }
                for c in &m.certificates {
                    show(p, c);
                }
            }
            Ok(!found.is_empty())
        }
        _ => Err(HELP.into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            println!("No match in the selected profiles.");
            ExitCode::from(1)
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}
