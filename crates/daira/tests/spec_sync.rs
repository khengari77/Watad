//! Checks that every cell of the `spec/circles.md` §4 tables matches the code.
//!
//! The code regenerates the meters; this test pins the *spec document* to the
//! code, so a hand-typed typo in a word or `/0` pattern cell fails the build.
//! ("Rows are the authoritative table; `watad-daira` regenerates and checks
//! every cell" — `spec/circles.md` §4.)

use watad_daira::{Circle, CircleId, MeterName, Status};
use watad_types::Unit;

fn letter(u: Unit) -> char {
    match u {
        Unit::Sabab(watad_types::Sabab::Khafif) => 'S',
        Unit::Sabab(watad_types::Sabab::Thaqil) => 'T',
        Unit::Watad(watad_types::Watad::Majmu) => 'W',
        Unit::Watad(watad_types::Watad::Mafruq) => 'M',
    }
}

fn circle_by_name_ar(name: &str) -> Option<CircleId> {
    [
        CircleId::Mukhtalif,
        CircleId::Muatalaf,
        CircleId::Mujtalab,
        CircleId::Mushtabih,
        CircleId::Muttafiq,
    ]
    .into_iter()
    .find(|c| c.name_ar() == name)
}

fn meter_by_name_ar(name: &str) -> Option<MeterName> {
    const ALL: [MeterName; 22] = [
        MeterName::Tawil,
        MeterName::Madid,
        MeterName::Mustatil,
        MeterName::Basit,
        MeterName::Mumtadd,
        MeterName::Wafir,
        MeterName::Kamil,
        MeterName::Mutawafir,
        MeterName::Hazaj,
        MeterName::Rajaz,
        MeterName::Ramal,
        MeterName::Sari,
        MeterName::Mutaaddi,
        MeterName::Munsarid,
        MeterName::Munsarih,
        MeterName::Khafif,
        MeterName::Mudari,
        MeterName::Muqtadab,
        MeterName::Mujtathth,
        MeterName::Mutarrid,
        MeterName::Mutakarib,
        MeterName::Mutadarak,
    ];
    ALL.into_iter().find(|m| m.name_ar() == name)
}

#[test]
fn spec_section4_cells_match_code() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../spec/circles.md");
    let text = std::fs::read_to_string(path).expect("spec/circles.md must be readable");

    let mut in_section4 = false;
    let mut current_circle: Option<CircleId> = None;
    let mut rows = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for line in text.lines() {
        if line.starts_with("## ") {
            in_section4 = line.starts_with("## 4.");
            continue;
        }
        if !in_section4 {
            continue;
        }
        if let Some(rest) = line.strip_prefix("### ") {
            let name = rest.split(' ').next().unwrap_or("");
            current_circle = circle_by_name_ar(name);
            if current_circle.is_none() {
                errors.push(format!("unknown circle heading: {name}"));
            }
            continue;
        }
        if !line.starts_with("| ") || line.starts_with("| -") || line.starts_with("| rot") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // | <rot> | <meter> | <status> | <word> | <pattern> | <feet> |
        if cells.len() < 8 {
            continue;
        }
        let (Some(circle), Ok(rot)) = (current_circle, cells[1].parse::<usize>()) else {
            errors.push(format!("unparsable row: {line}"));
            continue;
        };
        rows += 1;

        let Some(name) = meter_by_name_ar(cells[2]) else {
            errors.push(format!("rot {rot}: unknown meter name {}", cells[2]));
            continue;
        };

        // Position: name lands exactly here on this circle.
        if !name.circle().same(circle) || name.rotation() != rot {
            errors.push(format!(
                "{}: name lives at {:?} rot {}, not {:?} rot {rot}",
                cells[2],
                name.circle(),
                name.rotation(),
                circle
            ));
        }

        let meter = Circle::of(circle).rotate(rot);
        if !meter.name().same(name) {
            errors.push(format!(
                "rot {rot} of {:?} generates {:?}, not {}",
                circle,
                meter.name().name_ar(),
                cells[2]
            ));
        }

        // Status cell.
        let want_status = if cells[3].starts_with("مستعمل") {
            Status::Mustaamil
        } else {
            Status::Muhmal
        };
        if name.status() != want_status {
            errors.push(format!(
                "{}: status cell {} disagrees with code",
                cells[2], cells[3]
            ));
        }

        // Word cell (unit letters).
        let want_word: String = cells[4].chars().filter(|c| *c != ' ').collect();
        let got_word: String = meter
            .rotation_word()
            .as_slice()
            .iter()
            .map(|u| letter(*u))
            .collect();
        if got_word != want_word {
            errors.push(format!(
                "{}: word cell '{}' but code says '{}'",
                cells[2], want_word, got_word
            ));
        }

        // Pattern cell (`/0` string of the rotation word).
        let want_pattern = cells[5].trim_matches('`');
        let got_pattern = meter.rotation_word().to_string();
        if got_pattern != want_pattern {
            errors.push(format!(
                "{}: pattern cell '{}' but code says '{}'",
                cells[2], want_pattern, got_pattern
            ));
        }

        // Feet cell (Arabic names, optional [S2] source tag).
        let want_feet = cells[6].split(" [").next().unwrap_or("").trim();
        let got_feet: Vec<&str> = meter.feet().iter().map(|f| f.name_ar()).collect();
        let got_feet = got_feet.join(" ");
        if got_feet != want_feet {
            errors.push(format!(
                "{}: feet cell '{}' but code says '{}'",
                cells[2], want_feet, got_feet
            ));
        }
    }

    assert_eq!(rows, 22, "expected 22 meter rows in spec §4");
    assert!(
        errors.is_empty(),
        "spec ⇄ code mismatches:\n{}",
        errors.join("\n")
    );
}
