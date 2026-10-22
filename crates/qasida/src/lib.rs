//! Whole-poem inference. Every result uses one externally selected profile,
//! derives initial position from the verse index, and carries all verse witnesses.
#![deny(unsafe_code)]
use watad_checker::{adapter::encode_beats, poem::Input};
use watad_lugha::meter::{Certificate, Error, MeterProfile};
use watad_types::Beat;

pub struct Verse {
    pub context: u16,
    pub sadr: Vec<Beat>,
    pub ajuz: Vec<Beat>,
}
pub struct Match {
    pub profile: usize,
    pub certificates: Vec<Certificate>,
}

fn encode(verses: &[Verse]) -> Vec<(Vec<u8>, Vec<u8>)> {
    verses
        .iter()
        .map(|v| (encode_beats(&v.sadr), encode_beats(&v.ajuz)))
        .collect()
}

pub fn recognize(profiles: &[MeterProfile], verses: &[Verse]) -> Result<Vec<Match>, Error> {
    if verses.is_empty() {
        return Err(Error::EmptyPoem);
    }
    let encoded = encode(verses);
    let input: Vec<_> = verses
        .iter()
        .zip(&encoded)
        .map(|(v, (s, a))| Input {
            context: v.context,
            sadr: s,
            ajuz: a,
        })
        .collect();
    let mut found = Vec::new();
    for (profile, p) in profiles.iter().enumerate() {
        if let Some(certificates) = p.recognize_poem(&input)? {
            found.push(Match {
                profile,
                certificates,
            });
        }
    }
    Ok(found)
}

pub fn check(p: &MeterProfile, verses: &[Verse], certificates: &[Certificate]) -> bool {
    let encoded = encode(verses);
    let input: Vec<_> = verses
        .iter()
        .zip(&encoded)
        .map(|(v, (s, a))| Input {
            context: v.context,
            sadr: s,
            ajuz: a,
        })
        .collect();
    p.check_poem(&input, certificates)
}
