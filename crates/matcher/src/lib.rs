//! Exact recognition relative to explicitly supplied, validated profiles.
//! Empty results exhaust those profiles; they make no claim about omitted usage.
#![deny(unsafe_code)]
use core::ops::Range;
use watad_lugha::meter::{Certificate, Error, MeterProfile, Scope};
use watad_lugha::program::Change;
use watad_types::{Beat, FootId};

pub struct Match {
    pub profile: usize,
    pub certificate: Certificate,
}

#[derive(Debug, Clone)]
pub struct FootReading {
    pub base: FootId,
    pub changes: Vec<Change>,
    /// Half-open positions in the externally supplied hemistich.
    pub span: Range<usize>,
}

pub struct Reading {
    pub sadr: Vec<FootReading>,
    pub ajuz: Vec<FootReading>,
}

pub fn recognize(
    profiles: &[MeterProfile],
    scope: Scope,
    sadr: &[Beat],
    ajuz: &[Beat],
) -> Result<Vec<Match>, Error> {
    let mut found = Vec::new();
    for (profile, p) in profiles.iter().enumerate() {
        if let Some(certificate) = p.recognize(scope, sadr, ajuz)? {
            found.push(Match {
                profile,
                certificate,
            });
        }
    }
    Ok(found)
}

/// Replay before resolving any indices or displaying named transformations.
pub fn explain(
    p: &MeterProfile,
    scope: Scope,
    sadr: &[Beat],
    ajuz: &[Beat],
    c: &Certificate,
) -> Option<Reading> {
    if !p.check(scope, sadr, ajuz, c) {
        return None;
    }
    let f = &p.definition().forms[c.form];
    let feet = p.definition().meter.feet();
    let read = |recipes: &[Vec<Vec<Change>>], steps: &[watad_checker::language::Step]| {
        steps
            .iter()
            .enumerate()
            .map(|(i, s)| FootReading {
                base: feet[i],
                changes: recipes[i][s.choice].clone(),
                span: s.start..s.end,
            })
            .collect()
    };
    Some(Reading {
        sadr: read(&f.sadr, &c.sadr),
        ajuz: read(&f.ajuz, &c.ajuz),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub byte: usize,
    pub character: char,
}

/// Parse only explicit prosodic beats. Arabic orthography conversion is Layer 2.
pub fn parse(input: &str) -> Result<Vec<Beat>, ParseError> {
    input
        .char_indices()
        .filter(|(_, c)| !c.is_whitespace())
        .map(|(byte, character)| match character {
            '/' | '1' => Ok(Beat::Mutaharrik),
            '0' => Ok(Beat::Sakin),
            _ => Err(ParseError { byte, character }),
        })
        .collect()
}
