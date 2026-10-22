//! Attestation layer of `spec/transformations.md` — the linguistic facts, kept as data.
//!
//! The split of trust (see `docs/TYPES.md`):
//!
//! * `watad-tahwil` is mechanical: it applies a `(position, precondition, operation)`
//!   rule and knows nothing about poetry.
//! * This crate holds the **attestation**: which named `(foot, operation)` examples
//!   the classical sources state, what each derived form is called, and the verbatim
//!   restrictions («لا يكونان إلا في مفاعلتن» …). Nothing here is derivable; it is
//!   transcribed from `spec/transformations.md` §2–§4 with its source tags.
//!
//! **Name derivation** (`spec/transformations.md` §5): when a source says a derived form
//! «ينقل إلى» or «يُبدل إلى» a named foot, the statement is about the *beat word*:
//! the derived foot's beat word equals the named canonical foot's beat word. So the
//! name of a derived form is the canonical foot whose word matches it — computed by
//! [`names_of`], never hand-mapped. Twin words (فاعلاتن / فاعِ لاتن share one beat
//! word) make the name a *set*, not a single foot.
//!
//! The derived-name conformance test is `tests`: for every attested
//! `(foot, operation)` the resulting beat word must reproduce the source example.
//! One discrepancy is recorded and pinned in the tests (الشَّكْل on فاعلاتن,
//! `spec/transformations.md` §7.3).
//!
//! [`meter`] and [`catalog`] expose circle-bound source profiles through the
//! generic Verus-checked recognition/certificate/poem kernel. Profile identity,
//! contexts, binding classes and terminal restrictions are explicit data.
//!
//! [`derivation`] binds structural transformations to generated forms and checks
//! replayable single-foot traces. It does not decide linguistic permissions.
#![deny(unsafe_code)]

pub mod catalog;
pub mod derivation;
pub mod meter;
pub mod program;

use watad_types::{Beat, Foot, FootId};

pub use watad_tahwil::{
    CompoundZihaf, Illa, IllaError, Zihaf, ZihafError, apply_compound, apply_illa, apply_zihaf,
    beats_of, pattern,
};

// ---------------------------------------------------------------------------
// Name derivation (spec/transformations.md §5)
// ---------------------------------------------------------------------------

/// The canonical feet whose beat word equals `beats` — the §5 name of a derived
/// form. Beat words are not unique (فاعلاتن `/0//0/0` and فاعِ لاتن `/0//0/0` are
/// surface twins), so all matches are returned, in [`FootId::id`] order.
pub fn names_of(beats: &[Beat]) -> Box<[FootId]> {
    let mut out = Vec::with_capacity(2);
    for i in 0..N_FEET {
        let id = FootId::from_id(i);
        if beats_of(Foot::of(id)).as_ref() == beats {
            out.push(id);
        }
    }
    out.into_boxed_slice()
}

/// The canonical foot count — [`FootId::id`] is a bijection onto `0..N_FEET`.
const N_FEET: usize = 11;

// ---------------------------------------------------------------------------
// Operations and errors
// ---------------------------------------------------------------------------

/// One named operation from `spec/transformations.md` §2–§4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    /// A simple زحاف (§2).
    Zihaf(Zihaf),
    /// A compound زحاف (§3).
    Compound(CompoundZihaf),
    /// An عِلَّة (§4).
    Illa(Illa),
}

impl Op {
    /// Apply the operation to the base foot's letter stream.
    ///
    /// **Raw, non-validating mechanics.** This bypasses both the structural
    /// restrictions and the attestation tables entirely (it delegates to the
    /// non-validating `watad-tahwil` primitives); it does not check that the
    /// operation is permitted for this foot, position, meter or form. It must
    /// not be used as a legality oracle. For form-bound structural checking use
    /// [`derivation::derive_structural_foot`]; linguistic permission still
    /// requires a selected profile and the `meter::MeterProfile` certificate path.
    pub fn apply(self, base: FootId) -> Result<Box<[Beat]>, OpError> {
        let foot = Foot::of(base);
        match self {
            Op::Zihaf(z) => apply_zihaf(&beats_of(foot), z).map_err(OpError::Zihaf),
            Op::Compound(c) => apply_compound(&beats_of(foot), c).map_err(OpError::Zihaf),
            Op::Illa(i) => apply_illa(foot, i).map_err(OpError::Illa),
        }
    }

    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Op::Zihaf(z) => z.name_ar(),
            Op::Compound(c) => c.name_ar(),
            Op::Illa(i) => i.name_ar(),
        }
    }
}

/// [`Op::apply`] can fail on a زحاف rule or an عِلَّة precondition.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OpError {
    /// The زحاف rule did not apply.
    #[error(transparent)]
    Zihaf(#[from] ZihafError),
    /// The عِلَّة precondition did not hold.
    #[error(transparent)]
    Illa(#[from] IllaError),
}

// ---------------------------------------------------------------------------
// Attestation tables (spec/transformations.md §2–§4) — linguistic facts as data
// ---------------------------------------------------------------------------

/// One source-stated example: applying `op` to `base` reproduces `expected_pattern`,
/// the beat word of the source's named result `result_word`.
///
/// `name_transfer` is `Some(x)` when the source names a *canonical* foot
/// («ينقل إلى فعولن», «إلى مفاعيلن»): the result's beat word must then match
/// [`FootId`] `x` — the §5 check, done in the tests via [`names_of`]. When the
/// source names a bare derived word (متفعلن، مفاعل، فعول …), the word is recorded
/// in `result_word` and only its beat word is checked.
#[derive(Debug, Clone, Copy)]
pub struct Attestation {
    /// The base foot (تفعيلة) named by the source.
    pub base: FootId,
    /// The named operation.
    pub op: Op,
    /// The result word as the source states it (voweled where the source does).
    pub result_word: &'static str,
    /// The result word's beat word, `/` متحرك or `0` ساكن.
    pub expected_pattern: &'static str,
    /// The canonical foot the source transfers the name to, if it names one.
    pub name_transfer: Option<FootId>,
    /// Source tag from `spec/transformations.md` §2–§4.
    pub source: &'static str,
}

/// Every named example of `spec/transformations.md` §2–§4, verbatim results included.
///
/// Beat words of the named results (letter by letter, from the voweled source
/// words): مُتَفْعِلُن `//0//0` · مُتْفَاعِلُن `/0/0//0` · مُفَاعِلُن `//0//0` ·
/// مُسْتَعِلُنْ `/0///0` · مَفَاعِلُن `//0//0` · فَعُول `//0/` · مَفَاعِيلُن
/// `//0/0/0` · فَاعِلاتُ `/0//0/` · مُفْتَعِلُن `/0///0` · فَعِلَاتُ `///0/` ·
/// مَفَاعِل `//0//` · مَفَاعِيل `//0/0/` · مُتَفَاعِلاتُن `///0//0/0` ·
/// مُتَفَاعِلَانْ `///0//00` · مُسْتَفْعِلَانْ `/0/0//00` · فَاعِلَانْ `/0//00` ·
/// فَاعِلاتَانْ `/0//0/00` · فَاعِلْ `/0/0` · مُتَفَاعِلْ `///0/0` · مُسْتَفْعِلْ
/// `/0/0/0` · مَفَاعِل (من القطف) `//0/0` · مُتْفَا `///0` · مَفْعُو `/0/0`.
pub const ATTESTED: &[Attestation] = &[
    // -- §2 simple زحافات [S3][S4] ------------------------------------------
    Attestation {
        base: FootId::MustafIlun,
        op: Op::Zihaf(Zihaf::Khabn),
        result_word: "مُتَفْعِلُن (متفعلن)",
        expected_pattern: "//0//0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::MutaFailun,
        op: Op::Zihaf(Zihaf::Idmar),
        result_word: "مُتْفَاعِلُن — pattern of مستفعلن",
        expected_pattern: "/0/0//0",
        name_transfer: Some(FootId::MustafIlun),
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::MutaFailun,
        op: Op::Zihaf(Zihaf::Waqs),
        result_word: "مُفَاعِلُن — pattern of مفاعلن",
        expected_pattern: "//0//0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::MustafIlun,
        op: Op::Zihaf(Zihaf::Tayy),
        result_word: "مُسْتَعِلُنْ (مستعلن)",
        expected_pattern: "/0///0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::Mafailun,
        op: Op::Zihaf(Zihaf::Qabd),
        result_word: "مَفَاعِلُن (مفاعلن)",
        expected_pattern: "//0//0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::FaUlun,
        op: Op::Zihaf(Zihaf::Qabd),
        result_word: "فَعُول",
        expected_pattern: "//0/",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::Mafalatun,
        op: Op::Zihaf(Zihaf::Asb),
        result_word: "«فيبدل … مفاعيلن»",
        expected_pattern: "//0/0/0",
        name_transfer: Some(FootId::Mafailun),
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::Mafalatun,
        op: Op::Zihaf(Zihaf::Aql),
        result_word: "«فيبدل … مفاعلن» — pattern of مفاعلن",
        expected_pattern: "//0//0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::Failatun,
        op: Op::Zihaf(Zihaf::Kaf),
        result_word: "فَاعِلاتُ",
        expected_pattern: "/0//0/",
        name_transfer: None,
        source: "[S3][S4]",
    },
    // -- §3 compound زحافات [S3][S4] ----------------------------------------
    Attestation {
        base: FootId::MutaFailun,
        op: Op::Compound(CompoundZihaf::Khazl),
        result_word: "مُفْتَعِلُن",
        expected_pattern: "/0///0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        // §7.3 discrepancy: the source claims «ينقلان إلى فاعلن» but the
        // mechanical result is فَعِلَاتُ. Pinned in the tests; see
        // `spec/transformations.md` §7.3. The source wording stands verbatim there.
        base: FootId::Failatun,
        op: Op::Compound(CompoundZihaf::Shakl),
        result_word: "فَعِلَاتُ — [S3][S4] claim «فاعلن», see §7.3",
        expected_pattern: "///0/",
        name_transfer: None,
        source: "[S3][S4] / §7.3",
    },
    Attestation {
        base: FootId::MustafIlunMafruq,
        op: Op::Compound(CompoundZihaf::Shakl),
        result_word: "مَفَاعِل",
        expected_pattern: "//0//",
        name_transfer: None,
        source: "[S3][S4] / §7.3",
    },
    Attestation {
        base: FootId::Mafalatun,
        op: Op::Compound(CompoundZihaf::Naqs),
        result_word: "مَفَاعِيل",
        expected_pattern: "//0/0/",
        name_transfer: None,
        source: "[S3][S4]",
    },
    // -- §4.1 علل الزيادة [S3][S4] ------------------------------------------
    Attestation {
        base: FootId::FaIlun,
        op: Op::Illa(Illa::Tarfil),
        result_word: "فَاعِلاتُن (فاعلاتن)",
        expected_pattern: "/0//0/0",
        name_transfer: Some(FootId::Failatun),
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::MutaFailun,
        op: Op::Illa(Illa::Tarfil),
        result_word: "مُتَفَاعِلاتُن",
        expected_pattern: "///0//0/0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::MutaFailun,
        op: Op::Illa(Illa::Tadhyil),
        result_word: "مُتَفَاعِلَانْ",
        expected_pattern: "///0//00",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::MustafIlun,
        op: Op::Illa(Illa::Tadhyil),
        result_word: "مُسْتَفْعِلَانْ",
        expected_pattern: "/0/0//00",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::FaIlun,
        op: Op::Illa(Illa::Tadhyil),
        result_word: "فَاعِلَانْ",
        expected_pattern: "/0//00",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::Failatun,
        op: Op::Illa(Illa::Tasbigh),
        result_word: "فَاعِلاتَانْ",
        expected_pattern: "/0//0/00",
        name_transfer: None,
        source: "[S3][S4]",
    },
    // -- §4.2 علل النقصان [S3][S4] ------------------------------------------
    Attestation {
        base: FootId::Mafalatun,
        op: Op::Illa(Illa::Qatf),
        result_word: "مَفَاعِل — «وينقل إلى فعولن»",
        expected_pattern: "//0/0",
        name_transfer: Some(FootId::FaUlun),
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::FaIlun,
        op: Op::Illa(Illa::Qat),
        result_word: "فَاعِلْ",
        expected_pattern: "/0/0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::MutaFailun,
        op: Op::Illa(Illa::Qat),
        result_word: "مُتَفَاعِلْ",
        expected_pattern: "///0/0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::MustafIlun,
        op: Op::Illa(Illa::Qat),
        result_word: "مُسْتَفْعِلْ («بإسكان اللام»)",
        expected_pattern: "/0/0/0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::MutaFailun,
        op: Op::Illa(Illa::Hadhadh),
        result_word: "«وينقل الباقي إلى فَعِلن»",
        expected_pattern: "///0",
        name_transfer: None,
        source: "[S3][S4]",
    },
    Attestation {
        base: FootId::MafUlatu,
        op: Op::Illa(Illa::Salm),
        result_word: "«وينقل الباقي إلى فَعْلن»",
        expected_pattern: "/0/0",
        name_transfer: None,
        source: "[S3][S4]",
    },
];

/// A verbatim domain restriction: `op` occurs only on `bases` in the sources.
#[derive(Debug, Clone, Copy)]
pub struct Restriction {
    /// The restricted operation.
    pub op: Op,
    /// The only base foot(s) the source allows.
    pub bases: &'static [FootId],
    /// The verbatim wording from the source.
    pub quote: &'static str,
    /// Source tag from `spec/transformations.md`.
    pub source: &'static str,
}

/// Verbatim restrictions (`spec/transformations.md` §2, §3 `[S3][S4]`) — attestation data.
pub const RESTRICTIONS: &[Restriction] = &[
    Restriction {
        op: Op::Zihaf(Zihaf::Asb),
        bases: &[FootId::Mafalatun],
        quote: "«والعصْب: إسكانه بعد تَحرُّكه … وهذان لا يكونان إلا في مفاعلتن»",
        source: "[S3][S4]",
    },
    Restriction {
        op: Op::Zihaf(Zihaf::Aql),
        bases: &[FootId::Mafalatun],
        quote: "«والعَقْل: حذفه متحركًا وهذان لا يكونان إلا في مفاعلتن»",
        source: "[S3][S4]",
    },
    Restriction {
        op: Op::Compound(CompoundZihaf::Khazl),
        bases: &[FootId::MutaFailun],
        quote: "«ويُحصَر في إسكان (تا) وحذف (ألف) متفاعلن»",
        source: "[S3][S4]",
    },
    Restriction {
        op: Op::Compound(CompoundZihaf::Shakl),
        bases: &[FootId::Failatun, FootId::MustafIlunMafruq],
        quote: "«وانحصر في حذف الألف الأولى والنون من (فاعلاتن) … والسين والنون من (مستفعِ لن)»",
        source: "[S3][S4]",
    },
    Restriction {
        op: Op::Compound(CompoundZihaf::Naqs),
        bases: &[FootId::Mafalatun],
        quote: "«ومع العصب نقصٌ، ويدخل مفاعلتن فقط»",
        source: "[S3][S4]",
    },
];

// ---------------------------------------------------------------------------
// Derived-name conformance tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// §6: «for every attested (foot, operation, position) the classical name
    /// and the resulting beat word must match the source examples above.»
    #[test]
    fn attested_examples_match_the_source() {
        for row in ATTESTED {
            let result = row
                .op
                .apply(row.base)
                .unwrap_or_else(|e| panic!("{} on {:?} failed: {e}", row.op.name_ar(), row.base));
            assert_eq!(
                pattern(&result),
                row.expected_pattern,
                "{} on {:?} → {}, expected {} ({})",
                row.op.name_ar(),
                row.base,
                row.result_word,
                row.expected_pattern,
                row.source,
            );
            if let Some(target) = row.name_transfer {
                assert!(
                    names_of(&result).contains(&target),
                    "{} on {:?}: «ينقل إلى {:?}» but the result word is {}",
                    row.op.name_ar(),
                    row.base,
                    target,
                    pattern(&result),
                );
            }
        }
    }

    /// §5 twins: one beat word can carry two canonical names (فاعلاتن /
    /// فاعِ لاتن), so name derivation returns a set.
    #[test]
    fn twin_words_share_a_name_set() {
        let word = beats_of(Foot::of(FootId::Failatun));
        let names = names_of(&word);
        assert!(names.contains(&FootId::Failatun));
        assert!(names.contains(&FootId::FailatunMafruq));
    }

    /// `spec/transformations.md` §7.3 — pinned discrepancy: الشَّكْل on فاعلاتن gives
    /// فَعِلَاتُ `///0/` mechanically, while `[S3][S4]` say «ينقلان إلى فاعلن»
    /// (`/0//0`). This test asserts the discrepancy *is* reproducible; when the
    /// printed authority is checked (§7.3), flip this test and the row.
    #[test]
    fn shakl_on_failatun_remains_a_recorded_discrepancy() {
        let result = Op::Compound(CompoundZihaf::Shakl)
            .apply(FootId::Failatun)
            .unwrap();
        assert_eq!(pattern(&result), "///0/");
        assert!(!names_of(&result).contains(&FootId::FaIlun));
    }

    /// The same الشَّكْل on مستفعِ لن matches its named result (§7.3 ✓).
    #[test]
    fn shakl_on_mustaf_ilun_mafruq_matches_mafil() {
        let result = Op::Compound(CompoundZihaf::Shakl)
            .apply(FootId::MustafIlunMafruq)
            .unwrap();
        assert_eq!(pattern(&result), "//0//");
    }

    /// Attestation consistency: every ATTESTED row of a restricted op uses one
    /// of the restricted bases — the table cannot drift from RESTRICTIONS.
    #[test]
    fn restricted_ops_appear_only_on_their_attested_bases() {
        for row in ATTESTED {
            if let Some(r) = RESTRICTIONS.iter().find(|r| r.op == row.op) {
                assert!(
                    r.bases.contains(&row.base),
                    "{} on {:?} contradicts the restriction {}",
                    row.op.name_ar(),
                    row.base,
                    r.quote,
                );
            }
        }
    }

    /// The restriction data states exactly the ops the sources restrict, once
    /// each, and unresolved عِلَل carry no restriction row.
    #[test]
    fn restrictions_cover_exactly_the_stated_ops() {
        // `spec/transformations.md` §2, §3 [S3][S4]: العصْب، العَقْل، الخَزْل، الشَّكْل، النَّقْص.
        let expected = [
            Op::Zihaf(Zihaf::Asb),
            Op::Zihaf(Zihaf::Aql),
            Op::Compound(CompoundZihaf::Khazl),
            Op::Compound(CompoundZihaf::Shakl),
            Op::Compound(CompoundZihaf::Naqs),
        ];
        let restricted: Vec<Op> = RESTRICTIONS.iter().map(|r| r.op).collect();
        assert_eq!(restricted.len(), expected.len());
        for op in expected {
            assert_eq!(
                restricted.iter().filter(|r| **r == op).count(),
                1,
                "{} must be restricted exactly once",
                op.name_ar()
            );
        }
        // Unresolved عِلَل must not be given a (permissive) restriction row.
        for op in [
            Op::Illa(Illa::Batr),
            Op::Illa(Illa::Qasr),
            Op::Illa(Illa::Kharm),
        ] {
            assert!(
                !restricted.contains(&op),
                "{} must have no restriction",
                op.name_ar()
            );
        }
        // Every restriction carries its evidence.
        for r in RESTRICTIONS {
            assert!(!r.bases.is_empty());
            assert!(!r.quote.is_empty());
            assert!(!r.source.is_empty());
        }
    }
}
