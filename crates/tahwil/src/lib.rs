//! Typed زحاف and عِلَّ operations — the mechanical layer of `spec/transformations.md`.
//!
//! Domain (per `spec/transformations.md` §6 `[S3][S4][S5]` and `docs/TYPES.md`):
//!
//! * A simple زحاف is `(letter position, precondition, operation)`: one letter of
//!   the base foot's letter stream (1-based) must be ساكن or متحرك, and the
//!   operation is حذف (delete) or إسكان (silence). The struck positions are always
//!   *seconds of the sababs* — the classical domain statement «الزحاف ما لحق
//!   ثواني الأسباب».
//! * A compound زحاف is a fixed pair of simple ones, applied together against the
//!   original letter stream (no index shifting between the two ops).
//! * Ordinary illa requires an endpoint. Kharm requires the initial foot;
//!   explicitly sourced nonbinding exceptions and poem binding belong to profiles.
//!
//! The functions here are mechanical: they enforce structural preconditions only.
//! Two entry points exist per operation family. The raw ones (`apply_zihaf`,
//! `apply_compound`) take a beat word and check only letter state; the *checked*
//! ones (`apply_zihaf_checked`, `apply_compound_checked`) take a [`Foot`] and
//! additionally gate on sabab ownership — «الزحاف ما لحق ثواني الأسباب». Even the
//! checked ones are structural only: they do not check position, meter, form or
//! profile permissions (that remains the Phase 1 legality boundary).
//! *Attestation* — which `(foot, operation)` pairs occur in the poetry — is data in
//! `watad-lugha`'s source-tagged tables. In particular «العَصْب والعَقْل لا يكونان
//! إلا في مفاعلتن» is attestation data, not structure.
//!
//! No `unwrap`/`panic` on any input: every entry point returns `Result`.
#![deny(unsafe_code)]

use watad_types::{Beat, Foot, Role};
pub mod program;

pub use watad_checker::named::{Change, Illa, IllaKind, LetterPos, Operation, Precondition, Zihaf};

// ---------------------------------------------------------------------------
// Letter streams
// ---------------------------------------------------------------------------

/// Concatenate a foot's letters. One letter carries exactly one beat, so the
/// letter stream and the beat word are the same sequence ([`Beat`] is the letter's
/// prosodic state: `/` متحرك or `0` ساكن).
pub fn beats_of(foot: Foot) -> Box<[Beat]> {
    let mut out = Vec::with_capacity(8);
    for unit in foot.units() {
        out.extend_from_slice(unit.beats());
    }
    out.into_boxed_slice()
}

/// `/0`-notation of a beat word, e.g. `"/0//0"`.
pub fn pattern(beats: &[Beat]) -> String {
    beats.iter().map(|b| b.symbol()).collect()
}

// ---------------------------------------------------------------------------
// The typed target of an operation
// ---------------------------------------------------------------------------

/// الزحافات المزدوج — the four compound زحافات (`spec/transformations.md` §3 `[S3][S4]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CompoundZihaf {
    /// الخَبْل = الطي + الخبن.
    Khabl,
    /// الخَزْل = الطي + الإضمار («إسكان (تا) وحذف (ألف) مفاعلتن»).
    Khazl,
    /// الشَّكْل = الكف + الخبن.
    Shakl,
    /// النَّقْص = الكف + العصب (مفاعلتن only).
    Naqs,
}

impl CompoundZihaf {
    /// Every compound زحاف, in the classical order.
    pub const ALL: [CompoundZihaf; 4] = [
        CompoundZihaf::Khabl,
        CompoundZihaf::Khazl,
        CompoundZihaf::Shakl,
        CompoundZihaf::Naqs,
    ];

    /// The fixed pair of simple components.
    pub const fn components(self) -> &'static [Zihaf; 2] {
        match self {
            CompoundZihaf::Khabl => &[Zihaf::Tayy, Zihaf::Khabn],
            CompoundZihaf::Khazl => &[Zihaf::Tayy, Zihaf::Idmar],
            CompoundZihaf::Shakl => &[Zihaf::Kaf, Zihaf::Khabn],
            CompoundZihaf::Naqs => &[Zihaf::Kaf, Zihaf::Asb],
        }
    }

    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            CompoundZihaf::Khabl => "الخَبْل",
            CompoundZihaf::Khazl => "الخَزْل",
            CompoundZihaf::Shakl => "الشَّكْل",
            CompoundZihaf::Naqs => "النَّقْص",
        }
    }
}

/// A three-valued answer to "may this عِلَّة apply here?".
///
/// Unknown source scope never becomes a permissive default. Current named
/// mechanics are resolved; profile permissions still carry independent unknown
/// and unsupported states. This coarse role gate is not a complete meter profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IllaScope {
    /// The sources permit it in this position.
    Allowed,
    /// The sources forbid it in this position.
    Forbidden,
    /// The sources do not yet decide this combination.
    Unknown,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// A زحاف could not apply mechanically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ZihafError {
    /// The foot is shorter than the struck position.
    #[error("{zihaf} strikes letter {position}, but the foot has only {len} letters")]
    OutOfRange {
        /// Name of the operation.
        zihaf: &'static str,
        /// 1-based struck position.
        position: u8,
        /// Length of the letter stream.
        len: usize,
    },
    /// The struck letter is not in the required state.
    #[error("{zihaf} needs letter {position} to be {expected}, but it is {actual}")]
    Precondition {
        /// Name of the operation.
        zihaf: &'static str,
        /// 1-based struck position.
        position: u8,
        /// ساكن or متحرك, as required.
        expected: &'static str,
        /// ساكن or متحرك, as found.
        actual: &'static str,
    },
    /// The struck letter lies inside a watad, not a sabab, so the زحاف is
    /// structurally illegal («الزحاف ما لحق ثواني الأسباب»).
    ///
    /// Produced only by the checked entry points ([`apply_zihaf_checked`],
    /// [`apply_compound_checked`]); the raw mechanics never classify a target.
    #[error("{zihaf} strikes letter {position}, which lies in a watad, not a sabab")]
    TargetsWatad {
        /// Name of the operation.
        zihaf: &'static str,
        /// 1-based struck position.
        position: u8,
    },
}

/// A failure of the raw mechanical rules, before unit provenance is considered.
///
/// Kept private so the public [`ZihafError`] can grow structural variants that
/// the mechanical path never produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
enum RuleError {
    #[error("{zihaf} strikes letter {position}, but the foot has only {len} letters")]
    OutOfRange {
        zihaf: &'static str,
        position: u8,
        len: usize,
    },
    #[error("{zihaf} needs letter {position} to be {expected}, but it is {actual}")]
    Precondition {
        zihaf: &'static str,
        position: u8,
        expected: &'static str,
        actual: &'static str,
    },
}

impl From<RuleError> for ZihafError {
    fn from(e: RuleError) -> ZihafError {
        match e {
            RuleError::OutOfRange {
                zihaf,
                position,
                len,
            } => ZihafError::OutOfRange {
                zihaf,
                position,
                len,
            },
            RuleError::Precondition {
                zihaf,
                position,
                expected,
                actual,
            } => ZihafError::Precondition {
                zihaf,
                position,
                expected,
                actual,
            },
        }
    }
}

/// An عِلَّة could not apply mechanically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum IllaError {
    /// Kharm requires an initial majmu watad.
    #[error("{illa} needs an initial watad majmu")]
    WrongInitialUnit { illa: &'static str },
    /// Batr requires a final khafif sabab immediately after a majmu watad.
    #[error("{illa} needs a final watad majmu followed by a sabab khafif")]
    WrongFinalStructure { illa: &'static str },
    /// The foot does not end in the unit the عِلَّة needs.
    #[error("{illa} needs a foot ending in {expected}, but this one ends in {actual}")]
    WrongFinalUnit {
        /// Name of the operation.
        illa: &'static str,
        /// Required final unit.
        expected: &'static str,
        /// Actual final unit.
        actual: &'static str,
    },
    /// The foot is too short for the operation.
    #[error("{illa} needs at least {min_len} letters, but the foot has {len}")]
    TooShort {
        /// Name of the operation.
        illa: &'static str,
        /// Minimum length.
        min_len: usize,
        /// Actual length.
        len: usize,
    },
    /// The struck letter is not in the required state (وقف/كسف need السابع المتحرك).
    #[error("{illa} needs letter {position} to be {expected}, but it is {actual}")]
    Precondition {
        /// Name of the operation.
        illa: &'static str,
        /// 1-based struck position.
        position: u8,
        /// ساكن or متحرك, as required.
        expected: &'static str,
        /// ساكن or متحرك, as found.
        actual: &'static str,
    },
    /// No mechanical rule yet (see `spec/transformations.md` §7).
    #[error("{illa} has no mechanical rule yet (see spec/transformations.md §7)")]
    NoMechanicalRule {
        /// Name of the operation.
        illa: &'static str,
    },
}

// ---------------------------------------------------------------------------
// Application
// ---------------------------------------------------------------------------

/// An `(position, precondition, operation)` triple — the machine mapping of
/// `spec/transformations.md` §6. Built from the typed sets; positions are original-stream
/// positions when several rules apply together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rule {
    pos: usize,
    precondition: Precondition,
    operation: Operation,
    name: &'static str,
}

fn rule_of(z: Zihaf) -> Rule {
    Rule {
        pos: z.position().one_based() as usize,
        precondition: z.precondition(),
        operation: z.operation(),
        name: z.name_ar(),
    }
}

fn apply_rules(beats: &[Beat], rules: &[Rule]) -> Result<Box<[Beat]>, RuleError> {
    // Validate every rule against the ORIGINAL letter stream.
    for r in rules {
        if r.pos > beats.len() {
            return Err(RuleError::OutOfRange {
                zihaf: r.name,
                position: r.pos as u8,
                len: beats.len(),
            });
        }
        let found = beats[r.pos - 1];
        let ok = match r.precondition {
            Precondition::Sakin => found.same(Beat::Sakin),
            Precondition::Mutaharrik => found.same(Beat::Mutaharrik),
        };
        if !ok {
            return Err(RuleError::Precondition {
                zihaf: r.name,
                position: r.pos as u8,
                expected: r.precondition.name_ar(),
                actual: found.name_ar(),
            });
        }
    }

    // Apply all rules against the ORIGINAL positions in one walk.
    let mut out = Vec::with_capacity(beats.len());
    for (i, b) in beats.iter().enumerate() {
        let pos = i + 1;
        let mut deleted = false;
        let mut beat = *b;
        for r in rules {
            if r.pos == pos {
                match r.operation {
                    Operation::Delete => deleted = true,
                    Operation::Silence => beat = Beat::Sakin,
                }
            }
        }
        if !deleted {
            out.push(beat);
        }
    }
    Ok(out.into_boxed_slice())
}

/// Apply a simple زحاف to a beat word.
///
/// **Raw, non-validating mechanics.** This checks only the letter-state
/// precondition at the struck position; it has no unit provenance, so it cannot
/// tell whether that position lies inside a sabab of *this* foot. Use
/// [`apply_zihaf_checked`] for the structural sabab-ownership gate.
pub fn apply_zihaf(beats: &[Beat], z: Zihaf) -> Result<Box<[Beat]>, ZihafError> {
    apply_rules(beats, &[rule_of(z)]).map_err(ZihafError::from)
}

/// Apply a compound زحاف (both components against the original stream).
///
/// **Raw, non-validating mechanics** — see [`apply_zihaf`].
pub fn apply_compound(beats: &[Beat], c: CompoundZihaf) -> Result<Box<[Beat]>, ZihafError> {
    let rules = [rule_of(c.components()[0]), rule_of(c.components()[1])];
    apply_rules(beats, &rules).map_err(ZihafError::from)
}

/// Where a simple زحاف strikes, structurally.
///
/// The classical domain of زحاف is «ما لحق ثواني الأسباب» — the struck letter
/// must lie inside a sabab. This classifies the target of `z` in `foot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZihafTarget {
    /// The struck letter lies inside a sabab of the foot.
    Sabab,
    /// The struck letter lies inside a watad of the foot — structurally illegal.
    Watad,
    /// The foot has no letter at the struck position.
    TooShort,
}

/// Classify the target of a simple زحاف in a foot, using unit ownership.
pub fn zihaf_target(foot: Foot, z: Zihaf) -> ZihafTarget {
    let letter = z.position().one_based() as usize - 1;
    match foot.unit_at_letter(letter) {
        None => ZihafTarget::TooShort,
        Some(origin) if origin.unit.is_sabab() => ZihafTarget::Sabab,
        Some(_) => ZihafTarget::Watad,
    }
}

/// Apply a simple زحاف with the structural sabab-ownership gate.
///
/// Unlike [`apply_zihaf`], this rejects a strike that lands inside a watad
/// ([`ZihafError::TargetsWatad`]). It is a *structural* gate only: it does not
/// check position, meter, form or profile permissions (Phase 1).
pub fn apply_zihaf_checked(foot: Foot, z: Zihaf) -> Result<Box<[Beat]>, ZihafError> {
    match zihaf_target(foot, z) {
        ZihafTarget::Sabab => apply_zihaf(&beats_of(foot), z),
        ZihafTarget::Watad => Err(ZihafError::TargetsWatad {
            zihaf: z.name_ar(),
            position: z.position().one_based(),
        }),
        ZihafTarget::TooShort => Err(ZihafError::OutOfRange {
            zihaf: z.name_ar(),
            position: z.position().one_based(),
            len: beats_of(foot).len(),
        }),
    }
}

/// Apply a compound زحاف with the structural sabab-ownership gate.
///
/// Both components must strike inside a sabab; see [`apply_zihaf_checked`].
pub fn apply_compound_checked(foot: Foot, c: CompoundZihaf) -> Result<Box<[Beat]>, ZihafError> {
    for component in c.components() {
        match zihaf_target(foot, *component) {
            ZihafTarget::Sabab => {}
            ZihafTarget::Watad => {
                return Err(ZihafError::TargetsWatad {
                    zihaf: component.name_ar(),
                    position: component.position().one_based(),
                });
            }
            ZihafTarget::TooShort => {
                return Err(ZihafError::OutOfRange {
                    zihaf: component.name_ar(),
                    position: component.position().one_based(),
                    len: beats_of(foot).len(),
                });
            }
        }
    }
    apply_compound(&beats_of(foot), c)
}

fn final_unit(foot: Foot) -> watad_types::Unit {
    let units = foot.units();
    units[units.len() - 1]
}

/// Translate a failure of the mechanical rules into an [`IllaError`], keeping
/// the distinction between "foot too short" and "letter in the wrong state".
///
/// Only an out-of-range strike means the foot is too short; a failed
/// precondition is preserved as a [`IllaError::Precondition`].
fn illa_rule_error(illa: &'static str, min_len: usize, len: usize, err: RuleError) -> IllaError {
    match err {
        RuleError::OutOfRange { .. } => IllaError::TooShort { illa, min_len, len },
        RuleError::Precondition {
            position,
            expected,
            actual,
            ..
        } => IllaError::Precondition {
            illa,
            position,
            expected,
            actual,
        },
    }
}

/// Apply an عِلَّة to a foot's letter stream.
///
/// **Raw, non-validating mechanics.** It checks the structural precondition of
/// the particular عِلَّة against the foot's final unit, but it receives no
/// position, meter, form or profile: it cannot decide whether an عِلَّة is legal
/// in this context (that is [`may_carry_illa`] plus a profile decision). Callers
/// that need legal application must go through a legality boundary (Phase 1).
///
/// Mechanical rules (from `spec/transformations.md` §4):
/// `Tarfil`/`Tadhyil`/`Tasbigh` append `/0`, `0`, `0` to the stated final unit;
/// `Qat`/`Hadhadh`/`Salm`/`Qatf`/`Hadhf` act on the final unit;
/// `Waqf`/`Kasf` silence/delete letter 7 when it is متحرك («وليس لنا سابع
/// متحركٍ إلا (تاء) مفعولات»). Batr combines final sabab deletion with Qat;
/// Qasr shortens the final khafif sabab; Kharm removes an initial majmu letter.
pub fn apply_illa(foot: Foot, illa: Illa) -> Result<Box<[Beat]>, IllaError> {
    let beats = beats_of(foot);
    let n = beats.len();
    let finalu = final_unit(foot);
    let mut out: Vec<Beat> = Vec::with_capacity(n + 2);

    let needs_final = |expected: &'static str| -> Result<(), IllaError> {
        let actual = finalu.name_ar();
        let ok = match illa {
            Illa::Tarfil | Illa::Tadhyil | Illa::Qat | Illa::Hadhadh => {
                finalu.same(watad_types::Unit::W)
            }
            Illa::Tasbigh | Illa::Hadhf | Illa::Qatf => finalu.same(watad_types::Unit::S),
            Illa::Salm => finalu.same(watad_types::Unit::M),
            _ => true,
        };
        if ok {
            Ok(())
        } else {
            Err(IllaError::WrongFinalUnit {
                illa: illa.name_ar(),
                expected,
                actual,
            })
        }
    };

    match illa {
        Illa::Tarfil => {
            needs_final("وتد مجموع")?;
            out.extend_from_slice(&beats);
            out.extend_from_slice(SABAB_KHAFIF_BEATS);
        }
        Illa::Tadhyil => {
            needs_final("وتد مجموع")?;
            out.extend_from_slice(&beats);
            out.push(Beat::Sakin);
        }
        Illa::Tasbigh => {
            needs_final("سبب خفيف")?;
            out.extend_from_slice(&beats);
            out.push(Beat::Sakin);
        }
        Illa::Hadhf => {
            needs_final("سبب خفيف")?;
            out.extend_from_slice(&beats[..n - 2]);
        }
        Illa::Qatf => {
            needs_final("سبب خفيف")?;
            // «الحذف مع العصب»: drop the final sabab khafīf and silence letter 5.
            let rules = [
                Rule {
                    pos: 5,
                    precondition: Precondition::Mutaharrik,
                    operation: Operation::Silence,
                    name: Illa::Qatf.name_ar(),
                },
                Rule {
                    pos: n - 1,
                    precondition: Precondition::Mutaharrik,
                    operation: Operation::Delete,
                    name: Illa::Qatf.name_ar(),
                },
                Rule {
                    pos: n,
                    precondition: Precondition::Sakin,
                    operation: Operation::Delete,
                    name: Illa::Qatf.name_ar(),
                },
            ];
            return apply_rules(&beats, &rules)
                .map_err(|e| illa_rule_error(Illa::Qatf.name_ar(), 7, n, e));
        }
        Illa::Qat => {
            // «حذف ساكن الوتد المجموع وإسكان ما قبله»: //0 → /0.
            needs_final("وتد مجموع")?;
            out.extend_from_slice(&beats[..n - 3]);
            out.push(Beat::Mutaharrik);
            out.push(Beat::Sakin);
        }
        Illa::Hadhadh => {
            // «حذف وتد مجموع».
            needs_final("وتد مجموع")?;
            out.extend_from_slice(&beats[..n - 3]);
        }
        Illa::Salm => {
            // «ومفروقٍ صَلْمٌ»: حذف وتد مفروق.
            needs_final("وتد مفروق")?;
            out.extend_from_slice(&beats[..n - 3]);
        }
        Illa::Waqf => {
            // «إسكان السابع المتحرك».
            let rules = [Rule {
                pos: 7,
                precondition: Precondition::Mutaharrik,
                operation: Operation::Silence,
                name: Illa::Waqf.name_ar(),
            }];
            let result = apply_rules(&beats, &rules)
                .map_err(|e| illa_rule_error(Illa::Waqf.name_ar(), 7, n, e))?;
            if finalu != watad_types::Unit::M {
                return Err(IllaError::WrongFinalUnit {
                    illa: illa.name_ar(),
                    expected: "وتد مفروق",
                    actual: finalu.name_ar(),
                });
            }
            return Ok(result);
        }
        Illa::Kasf => {
            // «وحذفه» — delete the السابع المتحرك.
            let rules = [Rule {
                pos: 7,
                precondition: Precondition::Mutaharrik,
                operation: Operation::Delete,
                name: Illa::Kasf.name_ar(),
            }];
            let result = apply_rules(&beats, &rules)
                .map_err(|e| illa_rule_error(Illa::Kasf.name_ar(), 7, n, e))?;
            if finalu != watad_types::Unit::M {
                return Err(IllaError::WrongFinalUnit {
                    illa: illa.name_ar(),
                    expected: "وتد مفروق",
                    actual: finalu.name_ar(),
                });
            }
            return Ok(result);
        }
        Illa::Qasr => {
            if finalu != watad_types::Unit::S {
                return Err(IllaError::WrongFinalUnit {
                    illa: illa.name_ar(),
                    expected: "سبب خفيف",
                    actual: finalu.name_ar(),
                });
            }
            out.extend_from_slice(&beats[..n - 2]);
            out.push(Beat::Sakin);
        }
        Illa::Batr => {
            let units = foot.units();
            if units.len() < 2
                || units[units.len() - 2..] != [watad_types::Unit::W, watad_types::Unit::S]
            {
                return Err(IllaError::WrongFinalStructure {
                    illa: illa.name_ar(),
                });
            }
            out.extend_from_slice(&beats[..n - 5]);
            out.extend_from_slice(&[Beat::Mutaharrik, Beat::Sakin]);
        }
        Illa::Kharm => {
            if foot.units().first() != Some(&watad_types::Unit::W) {
                return Err(IllaError::WrongInitialUnit {
                    illa: illa.name_ar(),
                });
            }
            out.extend_from_slice(&beats[1..]);
        }
    }
    Ok(out.into_boxed_slice())
}

/// سبب خفيف `/0` — the unit الترفيل appends.
const SABAB_KHAFIF_BEATS: &[Beat] = watad_types::Unit::S.beats();

/// All positions reachable by a simple زحاف, for exhaustive tests.
pub fn simple_target_positions() -> &'static [LetterPos] {
    &LetterPos::ALL
}

/// The scope of an عِلَّة in a given foot position, per the sources.
///
/// Kharm requires the initial position. Other named illa require an endpoint.
/// This coarse role API cannot express simultaneous initial/terminal flags;
/// form-bound derivation handles that case using PositionContext.
pub const fn illa_scope(role: Role, illa: Illa) -> IllaScope {
    if matches!(illa, Illa::Kharm) {
        if matches!(role, Role::Ibtida) {
            IllaScope::Allowed
        } else {
            IllaScope::Forbidden
        }
    } else if role.may_carry_illa() || illa.may_occur_in_hashw() {
        IllaScope::Allowed
    } else {
        IllaScope::Forbidden
    }
}

/// [`Role`] gate for the عِلَل: true only when [`illa_scope`] is
/// [`IllaScope::Allowed`].
///
/// Unresolved عِلَل return `false` here; use [`illa_scope`] when the distinction
/// between "forbidden" and "not yet decided" matters.
pub const fn may_carry_illa(role: Role, illa: Illa) -> bool {
    matches!(illa_scope(role, illa), IllaScope::Allowed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use watad_types::FootId;

    fn beats_of_id(id: FootId) -> Box<[Beat]> {
        beats_of(Foot::of(id))
    }

    #[test]
    fn foot_beats_match_the_layer1_tables() {
        assert_eq!(pattern(&beats_of_id(FootId::FaIlun)), "/0//0");
        assert_eq!(pattern(&beats_of_id(FootId::MustafIlun)), "/0/0//0");
        assert_eq!(pattern(&beats_of_id(FootId::Mafalatun)), "//0///0");
    }

    #[test]
    fn khabn_on_mustaf_ilun_is_mutaflun_pattern() {
        // مستفعلن → متفعلن (spec/transformations.md §2).
        let out = apply_zihaf(&beats_of_id(FootId::MustafIlun), Zihaf::Khabn).unwrap();
        assert_eq!(pattern(&out), "//0//0");
    }

    #[test]
    fn preconditions_are_enforced() {
        // Khabn needs a ساكن at 2; مفاعلتن letter 2 is متحرك (/).
        let err = apply_zihaf(&beats_of_id(FootId::Mafalatun), Zihaf::Khabn).unwrap_err();
        assert!(matches!(err, ZihafError::Precondition { position: 2, .. }));
    }

    #[test]
    fn short_feet_are_rejected() {
        // فعولن has 5 letters; الكف strikes letter 7.
        let err = apply_zihaf(&beats_of_id(FootId::FaUlun), Zihaf::Kaf).unwrap_err();
        assert!(matches!(
            err,
            ZihafError::OutOfRange {
                position: 7,
                len: 5,
                ..
            }
        ));
    }

    #[test]
    fn compound_applies_both_rules_against_the_original_stream() {
        // الخَزْل on متفاعلن: إسكان التاء (2) وحذف الألف (4).
        let out = apply_compound(&beats_of_id(FootId::MutaFailun), CompoundZihaf::Khazl).unwrap();
        assert_eq!(pattern(&out), "/0///0");
    }

    #[test]
    fn qat_turns_final_watad_majmu_into_sabab_khafif() {
        // فاعلن → فاعلْ (spec/transformations.md §4.2).
        let out = apply_illa(Foot::of(FootId::FaIlun), Illa::Qat).unwrap();
        assert_eq!(pattern(&out), "/0/0");
    }

    #[test]
    fn kharm_is_the_only_hashw_illa() {
        for illa in Illa::ALL {
            assert_eq!(illa.may_occur_in_hashw(), illa == Illa::Kharm);
        }
    }

    #[test]
    fn the_eight_simple_zihaf_target_the_seconds_of_the_sababs() {
        // The named simple زحاف strike exactly letters 2, 4, 5 and 7.
        let mut positions: Vec<u8> = Zihaf::ALL
            .iter()
            .map(|z| z.position().one_based())
            .collect();
        positions.sort_unstable();
        positions.dedup();
        assert_eq!(positions, vec![2, 4, 5, 7]);
    }

    #[test]
    fn waqf_and_kasf_preserve_the_precondition_error() {
        // مستفعلن `/0/0//0` letter 7 is ساكن; وقف/كسف need السابع المتحرك, so the
        // failure is a precondition, not "too short" (len == min_len == 7).
        for illa in [Illa::Waqf, Illa::Kasf] {
            let err = apply_illa(Foot::MUSTAF_ILUN, illa).unwrap_err();
            assert!(
                matches!(err, IllaError::Precondition { position: 7, .. }),
                "{illa:?} on مستفعلن should fail its precondition, got {err:?}"
            );
        }
    }

    #[test]
    fn qatf_preserves_the_precondition_error() {
        // مفاعيلن `//0/0/0` letter 5 is ساكن; قطف needs it متحرك.
        let err = apply_illa(Foot::MAF_AILUN, Illa::Qatf).unwrap_err();
        assert!(
            matches!(err, IllaError::Precondition { position: 5, .. }),
            "قطف on مفاعيلن should fail its precondition, got {err:?}"
        );
    }

    #[test]
    fn genuinely_short_feet_still_report_too_short() {
        // فعولن has 5 letters; وقف strikes letter 7 — an out-of-range strike.
        let err = apply_illa(Foot::FA_ULUN, Illa::Waqf).unwrap_err();
        assert!(matches!(
            err,
            IllaError::TooShort {
                min_len: 7,
                len: 5,
                ..
            }
        ));
    }

    #[test]
    fn khabn_strikes_inside_a_mafruq_watad_in_raw_mechanics() {
        // Structural ownership matters: فاعِ لاتن begins with a
        // وتد مفروق whose second letter is global position 2, so الخبن deletes
        // a watad letter. The raw mechanical API carries no unit provenance and
        // must accept it; a legality boundary (Phase 1) is what rejects it.
        let foot = Foot::FA_ILATUN_MAFRUQ;
        let beats = beats_of(foot);

        // Which unit owns beat index 1 (letter 2)?
        let owner = foot.unit_at_letter(1).map(|o| o.unit);
        assert_eq!(
            owner,
            Some(watad_types::Unit::M),
            "letter 2 is in a mafrūq watad"
        );

        let out = apply_zihaf(&beats, Zihaf::Khabn).unwrap();
        assert_eq!(pattern(&out), "///0/0");
    }

    #[test]
    fn zihaf_target_classifies_sabab_watad_and_short() {
        // فاعِ لاتن begins with a mafrūq watad; letter 2 is inside it.
        assert_eq!(
            zihaf_target(Foot::FA_ILATUN_MAFRUQ, Zihaf::Khabn),
            ZihafTarget::Watad
        );
        // مستفعلن `/0/0//0`: letter 2 is the ثاني of the initial sabab khafīf.
        assert_eq!(
            zihaf_target(Foot::MUSTAF_ILUN, Zihaf::Khabn),
            ZihafTarget::Sabab
        );
        // فعولن has 5 letters; الكف strikes letter 7.
        assert_eq!(
            zihaf_target(Foot::FA_ULUN, Zihaf::Kaf),
            ZihafTarget::TooShort
        );
    }

    #[test]
    fn checked_zihaf_rejects_a_watad_target() {
        // The §2.2 counterexample is rejected at the checked boundary while the
        // raw mechanics still accept it.
        let err = apply_zihaf_checked(Foot::FA_ILATUN_MAFRUQ, Zihaf::Khabn).unwrap_err();
        assert!(
            matches!(err, ZihafError::TargetsWatad { position: 2, .. }),
            "expected TargetsWatad at 2, got {err:?}"
        );
        assert!(apply_zihaf(&beats_of(Foot::FA_ILATUN_MAFRUQ), Zihaf::Khabn).is_ok());
    }

    #[test]
    fn checked_zihaf_accepts_a_sabab_target() {
        let out = apply_zihaf_checked(Foot::MUSTAF_ILUN, Zihaf::Khabn).unwrap();
        assert_eq!(pattern(&out), "//0//0");
    }

    #[test]
    fn checked_compound_rejects_a_watad_component() {
        // الخَبْل = الطيّ(4) + الخبن(2); on فاعِ لاتن the الخبن component targets
        // the mafrūq watad, so the structural gate rejects the whole compound.
        let err = apply_compound_checked(Foot::FA_ILATUN_MAFRUQ, CompoundZihaf::Khabl).unwrap_err();
        assert!(matches!(err, ZihafError::TargetsWatad { .. }));
        // Raw mechanics never classify a target; they fail here on letter state.
        assert!(matches!(
            apply_compound(&beats_of(Foot::FA_ILATUN_MAFRUQ), CompoundZihaf::Khabl),
            Err(ZihafError::Precondition { .. })
        ));
    }

    #[test]
    fn illa_scope_is_tri_state_and_not_permissive_by_default() {
        // A decided عِلَّة: allowed at العروض/الضرب, forbidden in الحشو.
        assert_eq!(illa_scope(Role::Arud, Illa::Qat), IllaScope::Allowed);
        assert_eq!(illa_scope(Role::Darb, Illa::Qat), IllaScope::Allowed);
        assert_eq!(illa_scope(Role::Ibtida, Illa::Qat), IllaScope::Forbidden);
        assert_eq!(illa_scope(Role::Hashw, Illa::Qat), IllaScope::Forbidden);
        assert!(may_carry_illa(Role::Darb, Illa::Qat));
        assert!(!may_carry_illa(Role::Hashw, Illa::Qat));

        // Newly resolved terminal recipes remain confined to endpoints.
        for illa in [Illa::Batr, Illa::Qasr] {
            assert!(illa.has_mechanical_rule());
            for role in [Role::Ibtida, Role::Hashw, Role::Arud, Role::Darb] {
                assert_eq!(
                    may_carry_illa(role, illa),
                    matches!(role, Role::Arud | Role::Darb)
                );
            }
        }
        assert!(may_carry_illa(Role::Ibtida, Illa::Kharm));
        assert!(!may_carry_illa(Role::Hashw, Illa::Kharm));
        assert!(!may_carry_illa(Role::Darb, Illa::Kharm));
    }
}
