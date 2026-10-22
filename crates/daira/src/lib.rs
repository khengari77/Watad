//! Circles (دوائر), rotations, meters (بحور) and retained layouts.
//!
//! Named meters are generated from five circle words, declared once in
//! [`watad_types::CIRCLE_WORDS`]. [`Circle::rotate`] constructs [`Meter`]
//! values; foot sequences are derived from the structural foot dictionary.
//!
//! Compile-time assertions check circle primitivity, distinct rotations,
//! name coverage, preserved counts and segmentation consistency. Data and
//! source conventions are documented in `spec/circles.md`.
#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]

#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

use core::fmt;
use watad_types::{CIRCLE_WORDS as WORDS, FootId, Role, Unit, foot_window_at};

// ---------------------------------------------------------------------------
// Circles
// ---------------------------------------------------------------------------

/// Identity of a circle (دائرة).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CircleId {
    /// دائرة المختلف — word فَهُلْ عَلَيْ (W S W S S), 5 meters.
    Mukhtalif,
    /// دائرة المؤتلف — word (W T S), 3 meters.
    Muatalaf,
    /// دائرة المجتلب — word (W S S), 3 meters.
    Mujtalab,
    /// دائرة المشتبه — word (S S W S S W S S M), 9 meters.
    Mushtabih,
    /// دائرة المتفق — word (W S), 2 meters.
    Muttafiq,
}

// The circle words are the single seed [`watad_types::CIRCLE_WORDS`], imported
// as `WORDS` at the top of this file. Nothing here stores a second copy.

/// Repetitions of the rotation word per hemistich (§4).
const REPS: [usize; 5] = [2, 3, 3, 1, 4];

/// Meter names landing on each rotation, in circle order (§4, `[S3]`).
const NAMES: [&[MeterName]; 5] = [
    &[
        MeterName::Tawil,
        MeterName::Madid,
        MeterName::Mustatil,
        MeterName::Basit,
        MeterName::Mumtadd,
    ],
    &[MeterName::Wafir, MeterName::Kamil, MeterName::Mutawafir],
    &[MeterName::Hazaj, MeterName::Rajaz, MeterName::Ramal],
    &[
        MeterName::Sari,
        MeterName::Mutaaddi,
        MeterName::Munsarid,
        MeterName::Munsarih,
        MeterName::Khafif,
        MeterName::Mudari,
        MeterName::Muqtadab,
        MeterName::Mujtathth,
        MeterName::Mutarrid,
    ],
    &[MeterName::Mutakarib, MeterName::Mutadarak],
];

impl CircleId {
    /// Small stable id (0..5) for const bookkeeping.
    pub const fn id(self) -> usize {
        match self {
            CircleId::Mukhtalif => 0,
            CircleId::Muatalaf => 1,
            CircleId::Mujtalab => 2,
            CircleId::Mushtabih => 3,
            CircleId::Muttafiq => 4,
        }
    }

    /// Const inverse of [`CircleId::id`].
    ///
    /// # Panics
    /// Panics when `id >= 5`. Prefer [`CircleId::try_from_id`] at external
    /// boundaries; this is for bounded internal indices.
    pub const fn from_id(id: usize) -> CircleId {
        match id {
            0 => CircleId::Mukhtalif,
            1 => CircleId::Muatalaf,
            2 => CircleId::Mujtalab,
            3 => CircleId::Mushtabih,
            4 => CircleId::Muttafiq,
            _ => panic!("CircleId::from_id out of range"),
        }
    }

    /// The circle with this small id, or `None` when out of range — the checked
    /// boundary counterpart of [`CircleId::from_id`].
    pub const fn try_from_id(id: usize) -> Option<CircleId> {
        if id < 5 {
            Some(CircleId::from_id(id))
        } else {
            None
        }
    }

    /// Const equality.
    pub const fn same(self, other: CircleId) -> bool {
        self.id() == other.id()
    }

    /// Classical Arabic name (الدائرة).
    pub const fn name_ar(self) -> &'static str {
        match self {
            CircleId::Mukhtalif => "مختلف",
            CircleId::Muatalaf => "مؤتلف",
            CircleId::Mujtalab => "مجتلب",
            CircleId::Mushtabih => "مشتبه",
            CircleId::Muttafiq => "متفق",
        }
    }

    /// The primitive circle word (`spec/circles.md` §3).
    pub const fn word(self) -> &'static [Unit] {
        WORDS[self.id()]
    }

    /// Word length = number of rotations = number of meters on the circle.
    pub const fn period(self) -> usize {
        self.word().len()
    }

    /// Repetitions of one rotation per hemistich (§4).
    pub const fn reps_per_hemistich(self) -> usize {
        REPS[self.id()]
    }

    /// Units per hemistich = period × reps.
    pub const fn hemistich_len(self) -> usize {
        self.period() * self.reps_per_hemistich()
    }

    /// The meter names in circle order (rotation index = position).
    pub const fn meters(self) -> &'static [MeterName] {
        NAMES[self.id()]
    }
}

/// A circle (دائرة): identity only; the word is derived table data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Circle {
    id: CircleId,
}

impl Circle {
    /// Wrap a circle identity.
    pub const fn of(id: CircleId) -> Circle {
        Circle { id }
    }

    /// The circle identity.
    pub const fn id(self) -> CircleId {
        self.id
    }

    /// The primitive circle word.
    pub const fn word(self) -> &'static [Unit] {
        self.id.word()
    }

    /// Word length = number of meters on this circle.
    pub const fn period(self) -> usize {
        self.id.period()
    }

    /// Repetitions of one rotation per hemistich (`spec/circles.md` §4).
    pub const fn reps_per_hemistich(self) -> usize {
        self.id.reps_per_hemistich()
    }

    /// Units per hemistich = period × reps.
    pub const fn hemistich_len(self) -> usize {
        self.id.hemistich_len()
    }

    /// Generate the meter that lands on rotation `rot` of this circle.
    ///
    /// The **only** constructor of [`Meter`] in the workspace: every meter is a
    /// rotation of a circle word by construction.
    pub const fn rotate(self, rot: usize) -> Meter {
        assert!(rot < self.period(), "rotation outside the circle period");
        Meter {
            circle: self.id,
            rot: rot as u8,
        }
    }
}

// ---------------------------------------------------------------------------
// Meters
// ---------------------------------------------------------------------------

/// Usage status of a meter (مستعمل / مهمل).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// مستعمل.
    Mustaamil,
    /// مهمل.
    Muhmal,
}

impl Status {
    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Status::Mustaamil => "مستعمل",
            Status::Muhmal => "مهمل",
        }
    }
}

/// Identity of a meter (بحر): the 22 names = 16 مستعمل + 6 مهمل.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeterName {
    /// الطويل — مختلف rot 0.
    Tawil,
    /// المديد — مختلف rot 1.
    Madid,
    /// المستطيل — مختلف rot 2 `[S2]` (مهمل).
    Mustatil,
    /// البسيط — مختلف rot 3.
    Basit,
    /// الممتد — مختلف rot 4 `[S2]` (مهمل).
    Mumtadd,
    /// الوافر — مؤتلف rot 0.
    Wafir,
    /// الكامل — مؤتلف rot 1.
    Kamil,
    /// المتوافر — مؤتلف rot 2 `[S2]` (مهمل).
    Mutawafir,
    /// الهزج — مجتلب rot 0.
    Hazaj,
    /// الرجز — مجتلب rot 1.
    Rajaz,
    /// الرمل — مجتلب rot 2.
    Ramal,
    /// السريع — مشتبه rot 0.
    Sari,
    /// المتئد — مشتبه rot 1 `[S2]` (مهمل).
    Mutaaddi,
    /// المنسرد — مشتبه rot 2 `[S2]` (مهمل).
    Munsarid,
    /// المنسرح — مشتبه rot 3.
    Munsarih,
    /// الخفيف — مشتبه rot 4.
    Khafif,
    /// المضارع — مشتبه rot 5.
    Mudari,
    /// المقتضب — مشتبه rot 6.
    Muqtadab,
    /// المجتث — مشتبه rot 7.
    Mujtathth,
    /// المطرد — مشتبه rot 8 `[S2]` (مهمل).
    Mutarrid,
    /// المتقارب — متفق rot 0.
    Mutakarib,
    /// المتدارك — متفق rot 1. مستعمل عند الأخفش، مهمل عند الخليل `[S3]`.
    Mutadarak,
}

// ---------------------------------------------------------------------------
// Generated feet — the foot sequences are derived, not stored
// ---------------------------------------------------------------------------

/// Attested cut bindings for the three reps of دائرة المختلف that admit two
/// tilings — `(circle, rotation, first-foot length)`: المديد `(0, 1, 3)`،
/// المستطيل `(0, 2, 3)`، الممتد `(0, 4, 2)`. The other 19 reps tile uniquely
/// and need no attestation (`docs/TYPES.md`).
const CUTS: [(usize, usize, usize); 3] = [(0, 1, 3), (0, 2, 3), (0, 4, 2)];

/// The attested first-foot length for rep `(c, rot)`, or `0` when the rep's
/// tiling is unique.
const fn cut_binding(c: usize, rot: usize) -> usize {
    let mut i = 0;
    while i < CUTS.len() {
        if CUTS[i].0 == c && CUTS[i].1 == rot {
            return CUTS[i].2;
        }
        i += 1;
    }
    0
}

/// The global rotation index (circle-major) of `(c, rot)`.
const fn rot_index(c: usize, rot: usize) -> usize {
    let mut i = 0;
    let mut base = 0;
    while i < c {
        base += WORDS[i].len();
        i += 1;
    }
    base + rot
}

/// Number of complete tilings of rep `(c, rot)` from unit offset `off` into
/// named feet — windows of 2–3 units matching the derived foot dictionary.
const fn tile_count(c: usize, rot: usize, off: usize) -> usize {
    let n = WORDS[c].len();
    if off == n {
        return 1;
    }
    let mut total = 0;
    let mut l = 2;
    while l <= 3 {
        if off + l <= n && foot_window_at(c, rot, off, l).is_some() {
            total += tile_count(c, rot, off + l);
        }
        l += 1;
    }
    total
}

/// Fill `out` with the first complete tiling of rep `(c, rot)` from unit
/// offset `off`, forcing the first foot to length `forced` when `forced != 0`.
/// Returns the number of feet written, or `0` when no such tiling exists.
const fn fill_tiling(
    c: usize,
    rot: usize,
    off: usize,
    forced: usize,
    out: &mut [FootId; 4],
    written: &mut usize,
) -> usize {
    let n = WORDS[c].len();
    if off == n {
        return *written;
    }
    let mut l = 2;
    while l <= 3 {
        if off + l <= n
            && (forced == 0 || off > 0 || l == forced)
            && let Some(f) = foot_window_at(c, rot, off, l)
        {
            out[*written] = f;
            *written += 1;
            let done = fill_tiling(c, rot, off + l, 0, out, written);
            if done > 0 {
                return done;
            }
            *written -= 1;
        }
        l += 1;
    }
    0
}

/// The rep tiling of the `p`-th rotation: its named feet for one repetition —
/// the unique tiling, or the one selected by `CUTS`.
const fn rep_tiling(p: usize) -> ([FootId; 4], usize) {
    let (c, rot) = pair_at(p);
    let mut out = [FootId::FaUlun; 4];
    let mut written = 0;
    let len = fill_tiling(c, rot, 0, cut_binding(c, rot), &mut out, &mut written);
    (out, len)
}

/// The generated feet of every meter: its rep tiling repeated `REPS` times.
const fn gen_feet() -> ([[FootId; 4]; 22], [usize; 22]) {
    let mut table = [[FootId::FaUlun; 4]; 22];
    let mut lens = [0usize; 22];
    let mut p = 0;
    while p < 22 {
        let (c, _) = pair_at(p);
        let (rep, rep_len) = rep_tiling(p);
        let reps = REPS[c];
        let mut total = 0;
        let mut r = 0;
        while r < reps {
            let mut i = 0;
            while i < rep_len {
                if total >= 4 {
                    panic!("a meter's foot sequence exceeds 4 feet");
                }
                table[p][total] = rep[i];
                total += 1;
                i += 1;
            }
            r += 1;
        }
        lens[p] = total;
        p += 1;
    }
    (table, lens)
}

/// Generated foot sequences of the 22 rotations — `(rows, lengths)`.
static FEET_GEN: ([[FootId; 4]; 22], [usize; 22]) = gen_feet();

impl MeterName {
    /// Small stable id (0..22) — the rotation index, circle-major.
    pub const fn id(self) -> usize {
        // The declaration order of the enum is the rotation order.
        match self {
            MeterName::Tawil => 0,
            MeterName::Madid => 1,
            MeterName::Mustatil => 2,
            MeterName::Basit => 3,
            MeterName::Mumtadd => 4,
            MeterName::Wafir => 5,
            MeterName::Kamil => 6,
            MeterName::Mutawafir => 7,
            MeterName::Hazaj => 8,
            MeterName::Rajaz => 9,
            MeterName::Ramal => 10,
            MeterName::Sari => 11,
            MeterName::Mutaaddi => 12,
            MeterName::Munsarid => 13,
            MeterName::Munsarih => 14,
            MeterName::Khafif => 15,
            MeterName::Mudari => 16,
            MeterName::Muqtadab => 17,
            MeterName::Mujtathth => 18,
            MeterName::Mutarrid => 19,
            MeterName::Mutakarib => 20,
            MeterName::Mutadarak => 21,
        }
    }

    /// Const equality.
    pub const fn same(self, other: MeterName) -> bool {
        self.id() == other.id()
    }

    /// Classical Arabic name (البحر).
    pub const fn name_ar(self) -> &'static str {
        match self {
            MeterName::Tawil => "الطويل",
            MeterName::Madid => "المديد",
            MeterName::Mustatil => "المستطيل",
            MeterName::Basit => "البسيط",
            MeterName::Mumtadd => "الممتد",
            MeterName::Wafir => "الوافر",
            MeterName::Kamil => "الكامل",
            MeterName::Mutawafir => "المتوافر",
            MeterName::Hazaj => "الهزج",
            MeterName::Rajaz => "الرجز",
            MeterName::Ramal => "الرمل",
            MeterName::Sari => "السريع",
            MeterName::Mutaaddi => "المتئد",
            MeterName::Munsarid => "المنسرد",
            MeterName::Munsarih => "المنسرح",
            MeterName::Khafif => "الخفيف",
            MeterName::Mudari => "المضارع",
            MeterName::Muqtadab => "المقتضب",
            MeterName::Mujtathth => "المجتث",
            MeterName::Mutarrid => "المطرد",
            MeterName::Mutakarib => "المتقارب",
            MeterName::Mutadarak => "المتدارك",
        }
    }

    /// Usage status (16 مستعمل + 6 مهمل).
    ///
    /// المتدارك is مستعمل per al-Akhfash and مهمل per al-Khalīl; the project
    /// states both ([`MeterName::status_at_khalil`], `spec/circles.md` §3).
    pub const fn status(self) -> Status {
        match self {
            MeterName::Mustatil
            | MeterName::Mumtadd
            | MeterName::Mutawafir
            | MeterName::Mutaaddi
            | MeterName::Munsarid
            | MeterName::Mutarrid => Status::Muhmal,
            _ => Status::Mustaamil,
        }
    }

    /// Status as al-Khalīl had it: المتدارك is مهمل there `[S3]`.
    pub const fn status_at_khalil(self) -> Status {
        match self {
            MeterName::Mutadarak => Status::Muhmal,
            other => other.status(),
        }
    }

    /// The named feet (دائرة form), in order — generated by segmenting the
    /// rotation word with the derived foot dictionary.
    pub fn feet(self) -> &'static [FootId] {
        let (table, lens) = &FEET_GEN;
        &table[self.id()][..lens[self.id()]]
    }

    /// The circle this meter lands on.
    pub const fn circle(self) -> CircleId {
        let mut c = 0;
        while c < 5 {
            let names = NAMES[c];
            let mut r = 0;
            while r < names.len() {
                if names[r].same(self) {
                    return CircleId::from_id(c);
                }
                r += 1;
            }
            c += 1;
        }
        panic!("MeterName not present in the circle tables (theorem 3)")
    }

    /// The rotation offset on its circle.
    pub const fn rotation(self) -> usize {
        let mut c = 0;
        while c < 5 {
            let names = NAMES[c];
            let mut r = 0;
            while r < names.len() {
                if names[r].same(self) {
                    return r;
                }
                r += 1;
            }
            c += 1;
        }
        panic!("MeterName not present in the circle tables (theorem 3)")
    }
}

/// A meter (بحر): `(circle, rotation)`, private fields.
///
/// Only [`Circle::rotate`] constructs one — every meter is a rotation of a
/// circle word by construction (`docs/TYPES.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Meter {
    circle: CircleId,
    rot: u8,
}

/// A word over units, up to the longest in layer 1 (hemistich = 10 units max).
/// A computed view: display renders the `/0` pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeterWord {
    units: [Unit; 10],
    len: u8,
}

impl MeterWord {
    /// The units of the word.
    pub fn as_slice(&self) -> &[Unit] {
        &self.units[..self.len as usize]
    }

    /// Number of units.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// True for the empty word (never produced by [`Meter`]).
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl fmt::Display for MeterWord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        watad_types::Pattern(self.as_slice()).fmt(f)
    }
}

impl Meter {
    /// The meter name that lands on this rotation.
    pub const fn name(self) -> MeterName {
        self.circle.meters()[self.rot as usize]
    }

    /// The circle this meter is a rotation of.
    pub const fn circle_id(self) -> CircleId {
        self.circle
    }

    /// The rotation offset on the circle.
    pub const fn rotation(self) -> usize {
        self.rot as usize
    }

    /// The named feet (دائرة form) — generated from the circle words.
    pub fn feet(self) -> &'static [FootId] {
        self.name().feet()
    }

    /// The complete (تمّام) form of this meter.
    pub fn form(self) -> MeterForm {
        MeterForm::of(self)
    }

    /// The rotation word (one period of the circle word).
    pub fn rotation_word(self) -> MeterWord {
        let word = self.circle.word();
        let (units, n) = watad_types::rotation::rotate::<10>(word, self.rotation())
            .expect("private Meter bounds and circle capacity");
        MeterWord {
            units,
            len: n as u8,
        }
    }

    /// The hemistich word = the rotation word repeated `reps` times
    /// (`spec/circles.md` §4: صدر andعجز share these feet in the دائرة form).
    pub fn hemistich_word(self) -> MeterWord {
        let base = self.rotation_word();
        let reps = self.circle.reps_per_hemistich();
        let (units, len) = watad_types::rotation::repeat::<10>(base.as_slice(), reps)
            .expect("circle hemistich capacity");
        MeterWord {
            units,
            len: len as u8,
        }
    }
}

// ---------------------------------------------------------------------------
// Forms — repetition, terminal shape, composition (`docs/TYPES.md`)
// ---------------------------------------------------------------------------

/// The terminal shape of a form (التفريع): تمّام، مجزوء، مقصور، منهوك، منشور.
///
/// This is **attestation name data**. The structural effect of a non-تمّام
/// shape is a layer-2 عِلَّة applied at the [`MeterForm`] terminal hook through
/// `watad-tahwil`; which shape attests to which meter is open
/// (`spec/transformations.md` §7.1) and is not encoded here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalShape {
    /// تمّام — the complete form.
    Tamam,
    /// مجزوء.
    Majzu,
    /// مقصور.
    Maqsor,
    /// منهوك.
    Manhuk,
    /// منشور.
    Mansur,
}

impl TerminalShape {
    /// The Arabic name (attestation data).
    pub const fn name_ar(self) -> &'static str {
        match self {
            TerminalShape::Tamam => "تمام",
            TerminalShape::Majzu => "مجزوء",
            TerminalShape::Maqsor => "مقصور",
            TerminalShape::Manhuk => "منهوك",
            TerminalShape::Mansur => "منشور",
        }
    }
}

/// Composition axis of a form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Composition {
    /// مفرد — one rotation word.
    Simple,
    /// مركب — several rotation words in sequence.
    Compound,
}

/// One segment of a form: a rotation word repeated [`Segment::reps`] times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Segment {
    circle: CircleId,
    rot: u8,
    reps: u8,
}

impl Segment {
    /// The circle of this segment.
    pub const fn circle(self) -> CircleId {
        self.circle
    }

    /// The rotation on its circle.
    pub const fn rotation(self) -> usize {
        self.rot as usize
    }

    /// Repetitions of the rotation word in this segment.
    pub const fn reps(self) -> usize {
        self.reps as usize
    }
}

/// Rejected [`MeterForm`] shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormError {
    /// A form needs at least one segment.
    NoSegments,
    /// A form supports at most four segments.
    TooManySegments,
    /// Segment `segment` has a repetition count outside `1..=255`.
    BadReps {
        /// The offending segment index.
        segment: usize,
    },
    /// Segment `segment`: rotation `rotation` is out of range for circle `circle`.
    RotationOutOfRange {
        /// The offending segment index.
        segment: usize,
        /// The circle's id.
        circle: usize,
        /// The rejected rotation.
        rotation: usize,
    },
}

impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormError::NoSegments => write!(f, "a form needs at least one segment"),
            FormError::TooManySegments => write!(f, "a form supports at most four segments"),
            FormError::BadReps { segment } => {
                write!(f, "segment {segment}: repetition count must be 1..=255")
            }
            FormError::RotationOutOfRange {
                segment,
                circle,
                rotation,
            } => write!(
                f,
                "segment {segment}: rotation {rotation} out of range for circle {circle}"
            ),
        }
    }
}

impl core::error::Error for FormError {}

verus! {
/// The end position of a hemistich: العروض (end of the صدر) or الضرب (end of
/// the عجز).
///
/// Only these two of the four [`Role`]s are valid terminal roles; [`Role::Ibtida`]
/// and [`Role::Hashw`] are rejected at construction, so a caller cannot pass a
/// nonterminal role as the hemistich end.
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalRole {
    /// العروض — the last foot of the صدر.
    Arud,
    /// الضرب — the last foot of the عجز.
    Darb,
}
} // verus!

impl TerminalRole {
    /// The corresponding [`Role`].
    pub const fn role(self) -> Role {
        match self {
            TerminalRole::Arud => Role::Arud,
            TerminalRole::Darb => Role::Darb,
        }
    }

    /// Build from a [`Role`], rejecting the nonterminal roles
    /// ([`Role::Ibtida`], [`Role::Hashw`]).
    pub const fn try_from_role(role: Role) -> Option<TerminalRole> {
        match role {
            Role::Arud => Some(TerminalRole::Arud),
            Role::Darb => Some(TerminalRole::Darb),
            Role::Ibtida | Role::Hashw => None,
        }
    }
}

/// Positional facts about a foot within a hemistich form.
///
/// A structural view, not a legality decision: it records where the foot sits,
/// not what operations it permits. Verse context and poem position are supplied
/// through the profile recognition APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PositionContext {
    /// 0-based index of the foot in the hemistich.
    pub foot_index: usize,
    /// Number of feet in the hemistich.
    pub foot_count: usize,
    /// True when this is the first foot (الابتداء).
    pub is_initial: bool,
    /// The terminal role when this is the terminal foot, else `None`.
    ///
    /// A one-foot form is both initial and terminal: `is_initial` is true *and*
    /// `endpoint` is `Some(..)`.
    pub endpoint: Option<TerminalRole>,
}

/// A meter form: rotation words × repetitions, plus a terminal shape.
///
/// Generated structure — [`MeterForm::foot_at`] and [`MeterForm::unit_at`]
/// compute from the five circle words; only the repetition counts and the
/// terminal shape are attestation (`docs/TYPES.md`). One form models one
/// hemistich; صدر and عجز share its feet (`spec/circles.md` §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeterForm {
    segments: [Segment; 4],
    len: u8,
    terminal: TerminalShape,
}

impl MeterForm {
    /// The complete (تمّام) form of `meter`: one segment at the attested
    /// repetitions per hemistich.
    pub fn of(meter: Meter) -> MeterForm {
        MeterForm {
            segments: [Segment {
                circle: meter.circle_id(),
                rot: meter.rotation() as u8,
                reps: meter.circle_id().reps_per_hemistich() as u8,
            }; 4],
            len: 1,
            terminal: TerminalShape::Tamam,
        }
    }

    /// A form from `(circle, rotation, reps)` segments and a terminal shape.
    ///
    /// Segments may mix rotation words across circles (المركب); a single
    /// segment is the مفرد case. `reps` is the repetition count per hemistich
    /// (the attested one is [`CircleId::reps_per_hemistich`]).
    pub fn new(
        segments: &[(CircleId, usize, usize)],
        terminal: TerminalShape,
    ) -> Result<MeterForm, FormError> {
        if segments.is_empty() {
            return Err(FormError::NoSegments);
        }
        if segments.len() > 4 {
            return Err(FormError::TooManySegments);
        }
        let mut out = MeterForm {
            segments: [Segment {
                circle: CircleId::Mukhtalif,
                rot: 0,
                reps: 0,
            }; 4],
            len: segments.len() as u8,
            terminal,
        };
        let mut i = 0;
        while i < segments.len() {
            let (circle, rot, reps) = segments[i];
            if reps == 0 || reps > u8::MAX as usize {
                return Err(FormError::BadReps { segment: i });
            }
            if rot >= circle.period() {
                return Err(FormError::RotationOutOfRange {
                    segment: i,
                    circle: circle.id(),
                    rotation: rot,
                });
            }
            out.segments[i] = Segment {
                circle,
                rot: rot as u8,
                reps: reps as u8,
            };
            i += 1;
        }
        Ok(out)
    }

    /// The segments of the form.
    pub fn segments(&self) -> &[Segment] {
        &self.segments[..self.len as usize]
    }

    /// The terminal shape (attestation name).
    pub const fn terminal(&self) -> TerminalShape {
        self.terminal
    }

    /// مفرد or مركب, from the segment count.
    pub fn composition(&self) -> Composition {
        if self.len == 1 {
            Composition::Simple
        } else {
            Composition::Compound
        }
    }

    /// Number of named feet in the hemistich form (generated).
    pub fn foot_count(&self) -> usize {
        let mut total = 0;
        let mut s = 0;
        while s < self.len as usize {
            let seg = self.segments[s];
            let (_, rep_len) = rep_tiling(rot_index(seg.circle.id(), seg.rotation()));
            total += rep_len * seg.reps();
            s += 1;
        }
        total
    }

    /// The `i`-th named foot of the hemistich form, or `None` when `i` is out of
    /// range — the checked counterpart of [`Self::foot_at`].
    pub fn try_foot_at(&self, i: usize) -> Option<FootId> {
        if i >= self.foot_count() {
            return None;
        }
        let mut s = 0;
        let mut base = 0;
        while s < self.len as usize {
            let seg = self.segments[s];
            let (rep, rep_len) = rep_tiling(rot_index(seg.circle.id(), seg.rotation()));
            let count = rep_len * seg.reps();
            if i < base + count {
                return Some(rep[(i - base) % rep_len]);
            }
            base += count;
            s += 1;
        }
        None
    }

    /// The `i`-th named foot of the hemistich form (generated).
    ///
    /// # Panics
    /// Panics when `i >= self.foot_count()`. Prefer [`Self::try_foot_at`] at
    /// external boundaries; this is for bounded internal indices.
    pub fn foot_at(&self, i: usize) -> FootId {
        self.try_foot_at(i).expect("form foot index out of range")
    }

    /// Number of units in the hemistich form (generated).
    pub fn unit_count(&self) -> usize {
        let mut total = 0;
        let mut s = 0;
        while s < self.len as usize {
            let seg = self.segments[s];
            total += seg.circle.word().len() * seg.reps();
            s += 1;
        }
        total
    }

    /// The `i`-th unit of the hemistich form, or `None` when `i` is out of
    /// range — the checked counterpart of [`Self::unit_at`].
    pub fn try_unit_at(&self, i: usize) -> Option<Unit> {
        if i >= self.unit_count() {
            return None;
        }
        let mut s = 0;
        let mut base = 0;
        while s < self.len as usize {
            let seg = self.segments[s];
            let n = seg.circle.word().len();
            let count = n * seg.reps();
            if i < base + count {
                return Some(rot_unit(seg.circle.id(), seg.rotation(), (i - base) % n));
            }
            base += count;
            s += 1;
        }
        None
    }

    /// The `i`-th unit of the hemistich form (generated).
    ///
    /// # Panics
    /// Panics when `i >= self.unit_count()`. Prefer [`Self::try_unit_at`] at
    /// external boundaries; this is for bounded internal indices.
    pub fn unit_at(&self, i: usize) -> Unit {
        self.try_unit_at(i).expect("form unit index out of range")
    }

    /// The role of foot `i`: [`Role::Ibtida`] first, [`Role::Hashw`] in the
    /// middle, `terminal` last. Returns `None` when `i` is out of range.
    ///
    /// Terminal takes precedence over initial, so a one-foot hemistich reports
    /// its terminal role rather than losing it. Use [`Self::is_initial`] and
    /// [`Self::is_terminal`] when both facts are needed: a one-foot form is both
    /// initial and terminal, and a single [`Role`] cannot carry both.
    ///
    /// `terminal` is a [`TerminalRole`], so a nonterminal role cannot be passed
    /// as the hemistich end.
    pub fn role_at(&self, i: usize, terminal: TerminalRole) -> Option<Role> {
        if i >= self.foot_count() {
            return None;
        }
        let last = self.terminal_hook();
        Some(if i == last {
            terminal.role()
        } else if i == 0 {
            Role::Ibtida
        } else {
            Role::Hashw
        })
    }

    /// True when foot `i` is the first of the hemistich (الابتداء).
    pub fn is_initial(&self, i: usize) -> bool {
        i == 0
    }

    /// True when foot `i` is the last of the hemistich (العروض/الضرب hook).
    pub fn is_terminal(&self, i: usize) -> bool {
        i == self.terminal_hook()
    }

    /// The terminal foot index — the hook where a عِلَّة attaches
    /// (`watad-tahwil::apply_illa` at [`Role::Arud`] / [`Role::Darb`]).
    pub fn terminal_hook(&self) -> usize {
        self.foot_count() - 1
    }

    /// The positional context of foot `i`, or `None` when `i` is out of range.
    ///
    /// `endpoint` supplies the terminal role to report when `i` is the terminal
    /// foot; it is ignored for non-terminal feet. Unlike [`Self::role_at`], this
    /// keeps `is_initial` and `endpoint` as separate facts, so a one-foot form
    /// reports both.
    pub fn position_context(
        &self,
        i: usize,
        endpoint: Option<TerminalRole>,
    ) -> Option<PositionContext> {
        if i >= self.foot_count() {
            return None;
        }
        let is_terminal = self.is_terminal(i);
        Some(PositionContext {
            foot_index: i,
            foot_count: self.foot_count(),
            is_initial: self.is_initial(i),
            endpoint: if is_terminal { endpoint } else { None },
        })
    }
}

// ---------------------------------------------------------------------------
// Theorems 1–5 (`spec/circles.md` §6) — const checks over the tables
// ---------------------------------------------------------------------------

/// The `i`-th unit of rotation `rot` of circle `c`, without materialising it.
const fn rot_unit(c: usize, rot: usize, i: usize) -> Unit {
    let word = WORDS[c];
    word[(rot + i) % word.len()]
}

/// `(circle, rotation)` of the `p`-th of the 22 rotations.
const fn pair_at(p: usize) -> (usize, usize) {
    let mut c = 0;
    let mut rem = p;
    while c < 5 {
        let n = WORDS[c].len();
        if rem < n {
            return (c, rem);
        }
        rem -= n;
        c += 1;
    }
    panic!("rotation index out of range")
}

/// Theorem 1 — primitivity: no circle word equals a non-trivial rotation of
/// itself, so each circle has exactly `period` distinct rotations.
const fn theorem1_primitivity() -> bool {
    let mut c = 0;
    while c < 5 {
        let word = WORDS[c];
        let n = word.len();
        let mut k = 1;
        while k < n {
            let mut same = true;
            let mut i = 0;
            while i < n {
                if !word[i].same(word[(i + k) % n]) {
                    same = false;
                }
                i += 1;
            }
            if same {
                return false;
            }
            k += 1;
        }
        c += 1;
    }
    true
}

/// Theorem 2 — disjointness: the 22 rotation words are pairwise distinct.
const fn theorem2_disjointness() -> bool {
    let mut p = 0;
    while p < 22 {
        let (c1, r1) = pair_at(p);
        let n1 = WORDS[c1].len();
        let mut q = p + 1;
        while q < 22 {
            let (c2, r2) = pair_at(q);
            let n2 = WORDS[c2].len();
            if n1 == n2 {
                let mut same = true;
                let mut i = 0;
                while i < n1 {
                    if !rot_unit(c1, r1, i).same(rot_unit(c2, r2, i)) {
                        same = false;
                    }
                    i += 1;
                }
                if same {
                    return false;
                }
            }
            q += 1;
        }
        p += 1;
    }
    true
}

/// Theorem 3 — totality + injectivity: each circle lists exactly `period`
/// names, all 22 names are pairwise distinct, and the status counts are
/// 22 = 16 مستعمل + 6 مهمل.
const fn theorem3_names() -> bool {
    let mut c = 0;
    while c < 5 {
        if NAMES[c].len() != WORDS[c].len() {
            return false;
        }
        c += 1;
    }
    let mut p = 0;
    while p < 22 {
        let (c1, r1) = pair_at(p);
        let mut q = p + 1;
        while q < 22 {
            let (c2, r2) = pair_at(q);
            if NAMES[c1][r1].same(NAMES[c2][r2]) {
                return false;
            }
            q += 1;
        }
        p += 1;
    }
    let mut muhmal = 0;
    p = 0;
    while p < 22 {
        let (c, r) = pair_at(p);
        if matches!(NAMES[c][r].status(), Status::Muhmal) {
            muhmal += 1;
        }
        p += 1;
    }
    muhmal == 6
}

/// Theorem 4 — counts preserved: a rotation preserves the multiset of units
/// and the total number of beats of its circle word.
const fn theorem4_counts() -> bool {
    let mut c = 0;
    while c < 5 {
        let word = WORDS[c];
        let n = word.len();
        let mut rot = 0;
        while rot < n {
            let mut base = [0usize; 4];
            let mut shifted = [0usize; 4];
            let mut base_beats = 0usize;
            let mut shifted_beats = 0usize;
            let mut i = 0;
            while i < n {
                base[word[i].id()] += 1;
                base_beats += word[i].beat_count();
                let u = rot_unit(c, rot, i);
                shifted[u.id()] += 1;
                shifted_beats += u.beat_count();
                i += 1;
            }
            let mut m = 0;
            while m < 4 {
                if base[m] != shifted[m] {
                    return false;
                }
                m += 1;
            }
            if base_beats != shifted_beats {
                return false;
            }
            rot += 1;
        }
        c += 1;
    }
    true
}

/// Theorem 5 — segmentation: every rep tiles completely into named feet; the
/// reps with two tilings are exactly the attested `CUTS` ones (and never
/// more than two); and each meter's generated feet are its rep tiling
/// repeated `reps` times. Completeness of the tiling is by construction:
/// [`fill_tiling`] accepts only dictionary windows and stops exactly at the
/// rep length.
const fn theorem5_segmentation() -> bool {
    let (table, lens) = gen_feet();
    let mut p = 0;
    while p < 22 {
        let (c, rot) = pair_at(p);
        let reps = REPS[c];
        let (rep, rep_len) = rep_tiling(p);
        if rep_len == 0 {
            return false;
        }
        let sols = tile_count(c, rot, 0);
        if sols == 0 || sols > 2 {
            return false;
        }
        if (sols == 2) != (cut_binding(c, rot) != 0) {
            return false;
        }
        if lens[p] != rep_len * reps {
            return false;
        }
        let mut i = 0;
        while i < lens[p] {
            if !table[p][i].same(rep[i % rep_len]) {
                return false;
            }
            i += 1;
        }
        p += 1;
    }
    true
}

const _: () = assert!(theorem1_primitivity(), "T1: a circle word is not primitive");
const _: () = assert!(
    theorem2_disjointness(),
    "T2: two rotation words are identical"
);
const _: () = assert!(
    theorem3_names(),
    "T3: names not total/injective or not 16+6"
);
const _: () = assert!(theorem4_counts(), "T4: a rotation changed unit/beat counts");
const _: () = assert!(
    theorem5_segmentation(),
    "T5: rep tiling / attestation mismatch"
);

#[cfg(test)]
mod tests {
    use super::*;

    const fn letter(u: Unit) -> char {
        match u {
            Unit::S => 'S',
            Unit::T => 'T',
            Unit::W => 'W',
            Unit::M => 'M',
        }
    }

    fn word_letters(w: &MeterWord) -> String {
        w.as_slice().iter().map(|u| letter(*u)).collect()
    }

    #[test]
    fn all_22_rotations_generate_all_names() {
        let mut seen = [false; 22];
        let mut count = 0;
        for c in 0..5 {
            let circle = Circle::of(CircleId::from_id(c));
            for rot in 0..circle.period() {
                let meter = circle.rotate(rot);
                seen[meter.name().id()] = true;
                count += 1;
            }
        }
        assert_eq!(count, 22);
        assert!(
            seen.iter().all(|s| *s),
            "some meter name was never generated"
        );
    }

    #[test]
    fn name_lookups_are_consistent() {
        for name in [
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
        ] {
            let circle = Circle::of(name.circle());
            let meter = circle.rotate(name.rotation());
            assert!(
                meter.name().same(name),
                "round trip failed for {}",
                name.name_ar()
            );
        }
    }

    #[test]
    fn status_counts() {
        let mut used = 0;
        let mut muhmal = 0;
        for c in 0..5 {
            for name in CircleId::from_id(c).meters() {
                match name.status() {
                    Status::Mustaamil => used += 1,
                    Status::Muhmal => muhmal += 1,
                }
            }
        }
        assert_eq!((used, muhmal), (16, 6));
        // The Akhfash convention states المتدارك explicitly (spec §3).
        assert_eq!(MeterName::Mutadarak.status(), Status::Mustaamil);
        assert_eq!(MeterName::Mutadarak.status_at_khalil(), Status::Muhmal);
    }

    #[test]
    fn hemistich_is_rotation_repeated() {
        for c in 0..5 {
            let circle = Circle::of(CircleId::from_id(c));
            for rot in 0..circle.period() {
                let meter = circle.rotate(rot);
                let hemi = meter.hemistich_word();
                assert_eq!(hemi.len(), circle.hemistich_len());
                let base = meter.rotation_word();
                for i in 0..hemi.len() {
                    assert!(hemi.as_slice()[i].same(base.as_slice()[i % base.len()]));
                }
            }
        }
    }

    #[test]
    fn tawil_example() {
        // الطويل: rotation WSWSS, hemistich ×2 = فعولن مفاعيلن فعولن مفاعيلن.
        let meter = Circle::of(CircleId::Mukhtalif).rotate(0);
        assert!(meter.name().same(MeterName::Tawil));
        assert_eq!(word_letters(&meter.rotation_word()), "WSWSS");
        assert_eq!(meter.rotation_word().to_string(), "//0/0//0/0/0");
        assert_eq!(word_letters(&meter.hemistich_word()), "WSWSSWSWSS");
        assert_eq!(
            meter.hemistich_word().to_string(),
            "//0/0//0/0/0//0/0//0/0/0"
        );
    }

    #[test]
    fn attested_cuts_select_the_classical_feet() {
        // The three ambiguous reps of دائرة المختلف (`CUTS`): the attested
        // choice is the classical foot sequence (`spec/circles.md` §4).
        assert_eq!(
            MeterName::Madid.feet(),
            &[
                FootId::Failatun,
                FootId::FaIlun,
                FootId::Failatun,
                FootId::FaIlun
            ][..]
        );
        assert_eq!(
            MeterName::Mustatil.feet(),
            &[
                FootId::Mafailun,
                FootId::FaUlun,
                FootId::Mafailun,
                FootId::FaUlun
            ][..]
        );
        assert_eq!(
            MeterName::Mumtadd.feet(),
            &[
                FootId::FaIlun,
                FootId::Failatun,
                FootId::FaIlun,
                FootId::Failatun
            ][..]
        );
    }

    #[test]
    fn form_of_meter_matches_meter() {
        for c in 0..5 {
            let circle = Circle::of(CircleId::from_id(c));
            for rot in 0..circle.period() {
                let meter = circle.rotate(rot);
                let form = meter.form();
                assert_eq!(form.composition(), Composition::Simple);
                assert_eq!(form.terminal(), TerminalShape::Tamam);
                assert_eq!(form.foot_count(), meter.feet().len());
                for i in 0..form.foot_count() {
                    assert!(form.foot_at(i).same(meter.feet()[i]));
                }
                let hemi = meter.hemistich_word();
                assert_eq!(form.unit_count(), hemi.len());
                for i in 0..form.unit_count() {
                    assert!(form.unit_at(i).same(hemi.as_slice()[i]));
                }
            }
        }
    }

    #[test]
    fn compound_form_mixes_rotation_words() {
        // مركب: one rep of الطويل's rotation word + one rep of الرمل's.
        let form = MeterForm::new(
            &[(CircleId::Mukhtalif, 0, 1), (CircleId::Mujtalab, 2, 1)],
            TerminalShape::Tamam,
        )
        .unwrap();
        assert_eq!(form.composition(), Composition::Compound);
        assert_eq!(form.segments().len(), 2);
        assert_eq!(form.unit_count(), 5 + 3);
        assert_eq!(form.foot_count(), 2 + 1);
        assert_eq!(form.role_at(0, TerminalRole::Darb), Some(Role::Ibtida));
        assert_eq!(form.role_at(1, TerminalRole::Darb), Some(Role::Hashw));
        assert_eq!(form.role_at(2, TerminalRole::Darb), Some(Role::Darb));
        assert_eq!(form.terminal_hook(), 2);
    }

    #[test]
    fn one_foot_form_is_both_initial_and_terminal() {
        // A valid single-foot form must not lose its terminal fact: `role_at`
        // reports the terminal role, and the positional predicates expose both.
        let mut single = None;
        'outer: for c in 0..5 {
            let circle = Circle::of(CircleId::from_id(c));
            for rot in 0..circle.period() {
                if let Ok(form) = MeterForm::new(&[(circle.id(), rot, 1)], TerminalShape::Tamam)
                    && form.foot_count() == 1
                {
                    single = Some(form);
                    break 'outer;
                }
            }
        }
        let form = single.expect("some rotation yields a single-foot form");
        assert!(form.is_initial(0));
        assert!(form.is_terminal(0));
        assert_eq!(form.role_at(0, TerminalRole::Darb), Some(Role::Darb));
        assert_eq!(form.role_at(0, TerminalRole::Arud), Some(Role::Arud));
        assert_eq!(form.terminal_hook(), 0);
        assert!(!form.is_terminal(1));
        assert!(!form.is_terminal(usize::MAX));
    }

    #[test]
    fn role_at_and_terminal_role_are_checked() {
        let form = MeterForm::new(&[(CircleId::Mukhtalif, 0, 1)], TerminalShape::Tamam).unwrap();
        let last = form.terminal_hook();
        // Out-of-range indices are rejected rather than silently reported.
        assert_eq!(form.role_at(form.foot_count(), TerminalRole::Darb), None);
        assert_eq!(form.role_at(usize::MAX, TerminalRole::Darb), None);
        // A valid index resolves.
        assert!(form.role_at(last, TerminalRole::Darb).is_some());
        // Nonterminal roles cannot be used as the hemistich end.
        assert_eq!(TerminalRole::try_from_role(Role::Ibtida), None);
        assert_eq!(TerminalRole::try_from_role(Role::Hashw), None);
        assert_eq!(
            TerminalRole::try_from_role(Role::Arud),
            Some(TerminalRole::Arud)
        );
        assert_eq!(
            TerminalRole::try_from_role(Role::Darb),
            Some(TerminalRole::Darb)
        );
    }

    #[test]
    fn position_context_separates_initial_and_endpoint() {
        let form = MeterForm::new(
            &[(CircleId::Mukhtalif, 0, 1), (CircleId::Mujtalab, 2, 1)],
            TerminalShape::Tamam,
        )
        .unwrap();
        assert_eq!(
            form.position_context(0, Some(TerminalRole::Darb)),
            Some(PositionContext {
                foot_index: 0,
                foot_count: 3,
                is_initial: true,
                endpoint: None,
            })
        );
        assert_eq!(
            form.position_context(1, Some(TerminalRole::Darb)),
            Some(PositionContext {
                foot_index: 1,
                foot_count: 3,
                is_initial: false,
                endpoint: None,
            })
        );
        assert_eq!(
            form.position_context(2, Some(TerminalRole::Darb)),
            Some(PositionContext {
                foot_index: 2,
                foot_count: 3,
                is_initial: false,
                endpoint: Some(TerminalRole::Darb),
            })
        );
        // Out-of-range indices are rejected.
        assert_eq!(form.position_context(3, Some(TerminalRole::Darb)), None);
        assert_eq!(form.position_context(usize::MAX, None), None);
    }

    #[test]
    fn one_foot_position_context_carries_both_facts() {
        let mut single = None;
        'outer: for c in 0..5 {
            let circle = Circle::of(CircleId::from_id(c));
            for rot in 0..circle.period() {
                if let Ok(form) = MeterForm::new(&[(circle.id(), rot, 1)], TerminalShape::Tamam)
                    && form.foot_count() == 1
                {
                    single = Some(form);
                    break 'outer;
                }
            }
        }
        let form = single.expect("some rotation yields a single-foot form");
        let ctx = form
            .position_context(0, Some(TerminalRole::Arud))
            .expect("in-range");
        assert!(ctx.is_initial);
        assert_eq!(ctx.endpoint, Some(TerminalRole::Arud));
        // No endpoint supplied leaves the terminal fact unset, not invented.
        assert_eq!(form.position_context(0, None).unwrap().endpoint, None);
    }

    #[test]
    fn form_accessors_are_checked() {
        let form = MeterForm::new(&[(CircleId::Mukhtalif, 0, 1)], TerminalShape::Tamam).unwrap();
        // Out-of-range indices return None instead of panicking.
        assert_eq!(form.try_foot_at(form.foot_count()), None);
        assert_eq!(form.try_foot_at(usize::MAX), None);
        assert_eq!(form.try_unit_at(form.unit_count()), None);
        assert_eq!(form.try_unit_at(usize::MAX), None);
        // In-range indices agree with the panicking accessors.
        for i in 0..form.foot_count() {
            assert_eq!(form.try_foot_at(i), Some(form.foot_at(i)));
        }
        for i in 0..form.unit_count() {
            assert_eq!(form.try_unit_at(i), Some(form.unit_at(i)));
        }
    }

    #[test]
    fn terminal_shape_is_metadata_only() {
        // Switching the terminal shape changes no generated units or feet — the
        // shape is metadata; actual truncation or transformation must be explicit.
        let tamam = MeterForm::new(&[(CircleId::Mukhtalif, 0, 1)], TerminalShape::Tamam).unwrap();
        for shape in [
            TerminalShape::Majzu,
            TerminalShape::Maqsor,
            TerminalShape::Manhuk,
            TerminalShape::Mansur,
        ] {
            let other = MeterForm::new(&[(CircleId::Mukhtalif, 0, 1)], shape).unwrap();
            assert_eq!(tamam.unit_count(), other.unit_count());
            assert_eq!(tamam.foot_count(), other.foot_count());
            assert!((0..tamam.unit_count()).all(|i| tamam.unit_at(i).same(other.unit_at(i))));
        }
    }

    #[test]
    fn form_validation_rejects_bad_shapes() {
        assert_eq!(
            MeterForm::new(&[], TerminalShape::Tamam),
            Err(FormError::NoSegments)
        );
        assert_eq!(
            MeterForm::new(&[(CircleId::Mukhtalif, 5, 1)], TerminalShape::Tamam),
            Err(FormError::RotationOutOfRange {
                segment: 0,
                circle: 0,
                rotation: 5
            })
        );
        assert_eq!(
            MeterForm::new(&[(CircleId::Mukhtalif, 0, 0)], TerminalShape::Majzu),
            Err(FormError::BadReps { segment: 0 })
        );
    }

    #[test]
    fn terminal_shape_names_are_data() {
        assert_eq!(TerminalShape::Tamam.name_ar(), "تمام");
        assert_eq!(TerminalShape::Majzu.name_ar(), "مجزوء");
        assert_eq!(TerminalShape::Mansur.name_ar(), "منشور");
    }
}
