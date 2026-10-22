//! Core prosodic types: `Beat`, `Sabab`, `Watad`, `Unit`, `Foot`, `Role`.
//!
//! Structure over strings: a [`Foot`] carries its unit decomposition, because the
//! same surface `/0` pattern can have different structure — «مستفعلن» is `SSW`
//! (watad majmūʿ at the end) in الرجز but `SMS` (watad mafrūq in the middle) in
//! الخفيف, and the two take different زحافات.
//!
//! `Sabab` and `Watad` are distinct structural types. The checked meter kernel
//! expands their actual primitive letter words and preserves original ownership;
//! valid zihaf targets only second sabab letters. Raw compatibility mechanics
//! remain separately available in `watad-tahwil`.
//!
//! Invariants only, no meter logic: this crate defines the alphabet and the
//! attested feet; circles and meters live in `watad-daira`. Design of record:
//! `docs/TYPES.md`. Data and sources: `spec/circles.md`.
#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]

#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

use core::fmt;
pub mod circle_theorems;
pub mod rotation;

verus! {
/// A single prosodic letter: متحرك or ساكن.
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Beat {
    /// متحرك — written `/`.
    Mutaharrik,
    /// ساكن — written `0`.
    Sakin,
}
} // verus!

impl Beat {
    /// The `/0`-notation symbol: `/` or `0`.
    pub const fn symbol(self) -> char {
        match self {
            Beat::Mutaharrik => '/',
            Beat::Sakin => '0',
        }
    }

    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Beat::Mutaharrik => "متحرك",
            Beat::Sakin => "ساكن",
        }
    }

    /// Const equality (derived `PartialEq` is not callable in `const fn`).
    pub const fn same(self, other: Beat) -> bool {
        matches!(
            (self, other),
            (Beat::Mutaharrik, Beat::Mutaharrik) | (Beat::Sakin, Beat::Sakin)
        )
    }
}

verus! {
/// سبب: exactly the two attested forms. The only legal target of a زحاف.
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Sabab {
    /// سبب خفيف `/0` (لَمْ).
    Khafif,
    /// سبب ثقيل `//` (لَكَ).
    Thaqil,
}
} // verus!

impl Sabab {
    /// Small stable id (0..2) for const bookkeeping.
    pub const fn id(self) -> usize {
        match self {
            Sabab::Khafif => 0,
            Sabab::Thaqil => 1,
        }
    }

    /// Const equality.
    pub const fn same(self, other: Sabab) -> bool {
        self.id() == other.id()
    }

    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Sabab::Khafif => "سبب خفيف",
            Sabab::Thaqil => "سبب ثقيل",
        }
    }
}

verus! {
/// وتد: exactly the two attested forms. Never a زحاف target.
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Watad {
    /// وتد مجموع `//0` (نَعَمْ).
    Majmu,
    /// وتد مفروق `/0/` (قَالَ).
    Mafruq,
}
} // verus!

impl Watad {
    /// Small stable id (2..4) for const bookkeeping (shared with [`Unit::id`]).
    pub const fn id(self) -> usize {
        match self {
            Watad::Majmu => 2,
            Watad::Mafruq => 3,
        }
    }

    /// Const equality.
    pub const fn same(self, other: Watad) -> bool {
        self.id() == other.id()
    }

    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Watad::Majmu => "وتد مجموع",
            Watad::Mafruq => "وتد مفروق",
        }
    }
}

verus! {
/// A prosodic unit: a sabab or a watad.
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Unit {
    /// A sabab (زحاف-able).
    Sabab(Sabab),
    /// A watad (immutable by زحاف).
    Watad(Watad),
}
} // verus!

// The four primitive words are entered once for execution and specification.
macro_rules! unit_atoms {
    ($kind:ident { $($variant:ident => [$($beat:expr),* $(,)?]),* $(,)? }) => {
        verus! {
            impl $kind {
                pub open spec fn beat_spec(self) -> Seq<Beat> {
                    match self { $(Self::$variant => seq![$($beat),*],)* }
                }
                /// The original prosodic letters of this primitive unit.
                pub const fn beats(self) -> (word: &'static [Beat])
                    ensures word@ == self.beat_spec(), 2 <= word.len() <= 3,
                { match self { $(Self::$variant => &[$($beat),*],)* } }
            }
        }
    };
}

unit_atoms! { Sabab {
    Khafif => [Beat::Mutaharrik,Beat::Sakin],
    Thaqil => [Beat::Mutaharrik,Beat::Mutaharrik],
} }
unit_atoms! { Watad {
    Majmu => [Beat::Mutaharrik,Beat::Mutaharrik,Beat::Sakin],
    Mafruq => [Beat::Mutaharrik,Beat::Sakin,Beat::Mutaharrik],
} }

verus! {
impl Unit {
    /// Shorthand: سبب خفيف `/0`.
    pub const S: Unit = Unit::Sabab(Sabab::Khafif);
    /// Shorthand: سبب ثقيل `//`.
    pub const T: Unit = Unit::Sabab(Sabab::Thaqil);
    /// Shorthand: وتد مجموع `//0`.
    pub const W: Unit = Unit::Watad(Watad::Majmu);
    /// Shorthand: وتد مفروق `/0/`.
    pub const M: Unit = Unit::Watad(Watad::Mafruq);

    pub open spec fn beat_spec(self) -> Seq<Beat> {
        match self { Unit::Sabab(s) => s.beat_spec(), Unit::Watad(w) => w.beat_spec() }
    }

    /// The primitive letter word, shared with the checked expansion engine.
    pub const fn beats(self) -> (word: &'static [Beat])
        ensures word@ == self.beat_spec(), 2 <= word.len() <= 3,
    {
        match self { Unit::Sabab(s) => s.beats(), Unit::Watad(w) => w.beats() }
    }

}
} // verus!

impl Unit {
    /// Small stable id (0..4) for const bookkeeping.
    pub const fn id(self) -> usize {
        match self {
            Unit::Sabab(s) => s.id(),
            Unit::Watad(w) => w.id(),
        }
    }

    /// Const equality.
    pub const fn same(self, other: Unit) -> bool {
        self.id() == other.id()
    }

    /// Number of letters (beats) in the unit: 2 or 3.
    pub const fn beat_count(self) -> usize {
        self.beats().len()
    }

    /// True for وتد مجموع and وتد مفروق.
    pub const fn is_watad(self) -> bool {
        matches!(self, Unit::Watad(_))
    }

    /// True for سبب خفيف and سبب ثقيل.
    pub const fn is_sabab(self) -> bool {
        matches!(self, Unit::Sabab(_))
    }

    /// The sabab inside the unit, if any (typed view for زحاف APIs).
    pub const fn as_sabab(self) -> Option<Sabab> {
        match self {
            Unit::Sabab(s) => Some(s),
            Unit::Watad(_) => None,
        }
    }

    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Unit::Sabab(s) => s.name_ar(),
            Unit::Watad(w) => w.name_ar(),
        }
    }
}

/// Const equality for unit slices of possibly different lengths.
pub const fn units_eq(a: &[Unit], b: &[Unit]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if !a[i].same(b[i]) {
            return false;
        }
        i += 1;
    }
    true
}

verus! {
pub open spec fn is_watad_spec(u: Unit) -> bool {
    match u { Unit::Watad(_) => true, Unit::Sabab(_) => false }
}

pub open spec fn watad_count(units: Seq<Unit>, n: int) -> nat
    decreases n,
{
    if n <= 0 || n > units.len() { 0 }
    else { watad_count(units,n-1) + if is_watad_spec(units[n-1]) { 1nat } else { 0nat } }
}

/// True when the unit word contains exactly one watad.
///
/// Every attested تفعيلة satisfies this; it is the structural invariant behind
/// "the watad is never altered by زحاف".
pub const fn has_exactly_one_watad(units: &[Unit]) -> (yes: bool)
    ensures yes == (watad_count(units@,units.len() as int) == 1),
{
    let mut watads = 0usize;
    let mut i = 0;
    while i < units.len()
        invariant i <= units.len(), watads <= i, watads == watad_count(units@,i as int),
        decreases units.len()-i,
    {
        if matches!(units[i],Unit::Watad(_)) {
            watads += 1;
        }
        i += 1;
    }
    watads == 1
}
} // verus!

// ---------------------------------------------------------------------------
// The generative seed
// ---------------------------------------------------------------------------

/// The five circle words (دوائر) — the **single generative seed** of the system.
///
/// Every foot pattern is a window of one of these words, rotated; the meters
/// themselves are their rotations (`watad-daira`). `FOOT_AT` records only
/// *where* each named foot sits — its unit pattern is derived from there and
/// from this array, never entered by hand (`docs/TYPES.md`).
// The macro uses the same seed tokens for the native constant view and the
// checked accessor's declarative word; there is no second hand-entered table.
macro_rules! circle_seeds {
    ($($index:literal => [$($unit:expr),* $(,)?]),* $(,)?) => {
        verus! {
            pub open spec fn circle_word_spec(index: usize) -> Seq<Unit> {
                match index { $($index => seq![$($unit),*],)* _ => Seq::empty() }
            }

            pub const fn circle_word(index: usize) -> (word: &'static [Unit])
                requires index < 5,
                ensures word@ == circle_word_spec(index), 2 <= word.len() <= 9,
            {
                match index { $($index => &[$($unit),*],)* _ => &[] }
            }
        }
        pub const CIRCLE_WORDS: [&[Unit]; 5] = [$(circle_word($index)),*];
    };
}

circle_seeds! {
    // مختلف: فَهُلْ عَلَيْ = فعولن مفاعيلن (on the half).
    0 => [Unit::W, Unit::S, Unit::W, Unit::S, Unit::S],
    // مؤتلف: مفاعلتن.
    1 => [Unit::W, Unit::T, Unit::S],
    // مجتلب: مفاعيلن.
    2 => [Unit::W, Unit::S, Unit::S],
    // مشتبه: مستفعلن مستفعلن مفعولاتُ.
    3 => [
        Unit::S,
        Unit::S,
        Unit::W,
        Unit::S,
        Unit::S,
        Unit::W,
        Unit::S,
        Unit::S,
        Unit::M,
    ],
    // متفق: فعولن.
    4 => [Unit::W, Unit::S],
}

verus! {
/// Where each named foot sits in the rotation space, in [`FootId::id`] order:
/// `(circle index, rotation, unit offset within the rep, length)`.
///
/// This name↔position binding is **attestation data** (`[S1]/[S3]`) — the only
/// foot-level data in the system. Everything else about a foot's structure is
/// derived from it and [`CIRCLE_WORDS`].
const FOOT_AT: [(usize, usize, usize, usize); 11] = [
    (4, 0, 0, 2), // فعولن
    (4, 1, 0, 2), // فاعلن
    (2, 0, 0, 3), // مفاعيلن
    (2, 1, 0, 3), // مستفعلن
    (2, 2, 0, 3), // فاعلاتن
    (1, 0, 0, 3), // مفاعلتن
    (1, 1, 0, 3), // متفاعلن
    (3, 0, 6, 3), // مفعولاتُ
    (3, 2, 6, 3), // فاعِ لاتن
    (3, 1, 6, 3), // مستفعِ لن
    (1, 2, 0, 3), // فاعلاتُكَ
];

/// The `i`-th unit of rotation `rot` of circle `circle`:
/// `word[(i + rot) mod n]` — the same rotation used by `watad-daira`.
const fn rotated_unit(circle: usize, rot: usize, i: usize) -> (unit: Unit)
    requires circle < 5, rot < 9, i <= 10,
    ensures unit == circle_word_spec(circle)[(i as int + rot as int) % circle_word_spec(circle).len() as int],
{
    let word = circle_word(circle);
    word[(i + rot) % word.len()]
}

/// The unit window at attestation position `at`, padded to 3 units and paired
/// with its true length.
pub open spec fn window_word(at: (usize,usize,usize,usize)) -> Seq<Unit> {
    let word = circle_word_spec(at.0);
    if word.len() == 0 { Seq::empty() }
    else { Seq::new(at.3 as nat, |j: int| word[(at.2 as int + j + at.1 as int) % word.len() as int]) }
}

const fn foot_window(at: (usize, usize, usize, usize)) -> (result: ([Unit; 3], usize))
    requires at.0 < 5, at.1 < 9, at.2 <= 6, at.3 <= 3,
    ensures result.1 == at.3, result.0@.subrange(0,result.1 as int) == window_word(at),
{
    let (circle, rot, off, len) = at;
    let mut units = [Unit::S; 3];
    let mut i = 0;
    while i < len
        invariant circle < 5, rot < 9, off <= 6, len <= 3, i <= len,
            at == (circle,rot,off,len),
            forall|j: int| 0 <= j < i ==> units@[j] == window_word(at)[j],
        decreases len-i,
    {
        units[i] = rotated_unit(circle, rot, off + i);
        i += 1;
    }
    assert(units@.subrange(0,len as int) =~= window_word(at));
    (units, len)
}

/// The 11 foot patterns **derived** from [`CIRCLE_WORDS`] × `FOOT_AT`.
const fn derive_foot_units() -> (table: [([Unit; 3], usize); 11])
    ensures forall|k: int| 0 <= k < 11 ==> table@[k].1 == FOOT_AT@[k].3
        && table@[k].0@.subrange(0,table@[k].1 as int) == window_word(FOOT_AT@[k]),
{
    let mut out = [([Unit::S; 3], 0); 11];
    let mut k = 0;
    while k < 11
        invariant k <= 11,
            forall|j: int| 0 <= j < k ==> out@[j].1 == FOOT_AT@[j].3
                && out@[j].0@.subrange(0,out@[j].1 as int) == window_word(FOOT_AT@[j]),
        decreases 11-k,
    {
        out[k] = foot_window(FOOT_AT[k]);
        k += 1;
    }
    out
}
} // verus!

static FOOT_UNITS: [([Unit; 3], usize); 11] = derive_foot_units();

/// True when the first `len` units of `u` carry exactly one watad.
const fn arr_one_watad(u: [Unit; 3], len: usize) -> bool {
    let mut watads = 0;
    let mut i = 0;
    while i < len {
        if u[i].is_watad() {
            watads += 1;
        }
        i += 1;
    }
    watads == 1
}

/// True when the first `len` units of `u` and `v` are pairwise identical.
const fn arr_eq(u: [Unit; 3], v: [Unit; 3], len: usize) -> bool {
    let mut i = 0;
    while i < len {
        if !u[i].same(v[i]) {
            return false;
        }
        i += 1;
    }
    true
}

/// Sanity of the derivation: the 11 derived windows are pairwise distinct,
/// 2–3 units long, and each carries exactly one watad.
const fn foot_derivation_sane() -> bool {
    let table = derive_foot_units();
    let mut i = 0;
    while i < 11 {
        let (u, len) = table[i];
        if len < 2 || len > 3 || !arr_one_watad(u, len) {
            return false;
        }
        let mut k = i + 1;
        while k < 11 {
            let (v, m) = table[k];
            if len == m && arr_eq(u, v, len) {
                return false;
            }
            k += 1;
        }
        i += 1;
    }
    true
}

const _: () = assert!(
    foot_derivation_sane(),
    "derived foot windows: not distinct / wrong length / not one-watad"
);

/// The named foot whose derived pattern equals the window at
/// `(circle, rot, off, len)`, if any — the inverse of the derivation.
///
/// Returns `None` (rather than panicking) when `circle`, `len` or `rot` are
/// outside the declared bounds. `off` may exceed the word length: windows wrap.
pub const fn foot_window_at(circle: usize, rot: usize, off: usize, len: usize) -> Option<FootId> {
    if circle >= CIRCLE_WORDS.len() || len == 0 || len > 3 {
        return None;
    }
    if rot >= CIRCLE_WORDS[circle].len() {
        return None;
    }
    let off = off % CIRCLE_WORDS[circle].len();
    let mut k = 0;
    while k < 11 {
        let (u, l) = foot_window(FOOT_AT[k]);
        if l == len {
            let mut same = true;
            let mut i = 0;
            while i < len {
                if !u[i].same(rotated_unit(circle, rot, off + i)) {
                    same = false;
                }
                i += 1;
            }
            if same {
                return Some(FootId::from_id(k));
            }
        }
        k += 1;
    }
    None
}

verus! {
/// Closed named-base registry. Historical attestation is external source data.
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FootId {
    /// فعولن `//0/0`.
    FaUlun,
    /// فاعلن `/0//0`.
    FaIlun,
    /// مفاعيلن `//0/0/0`.
    Mafailun,
    /// مستفعلن `/0/0//0` — watad majmūʿ at the end.
    MustafIlun,
    /// فاعلاتن `/0//0/0`.
    Failatun,
    /// مفاعلتن `//0///0`.
    Mafalatun,
    /// متفاعلن `///0//0`.
    MutaFailun,
    /// مفعولاتُ `/0/0/0/`.
    MafUlatu,
    /// فاعِ لاتن `/0//0/0` — watad mafrūq in the middle (same surface as فاعلاتن).
    FailatunMafruq,
    /// مستفعِ لن `/0/0//0` — watad mafrūq in the middle (same surface as مستفعلن).
    MustafIlunMafruq,
    /// فاعلاتُكَ `/0//0//` — attested only in المتوافر.
    Failatuka,
}
} // verus!

macro_rules! foot_indices {
    ($($variant:ident => $index:literal),* $(,)?) => {
        verus! {
            impl FootId {
                pub open spec fn index(self) -> usize {
                    match self { $(Self::$variant => $index,)* }
                }

                /// Stable registry index. Names/index bindings are source data.
                pub const fn id(self) -> (i: usize)
                    ensures i == self.index(), i < 11,
                { match self { $(Self::$variant => $index,)* } }

                pub closed spec fn word_spec(self) -> Seq<Unit> {
                    window_word(FOOT_AT@[self.index() as int])
                }

                pub proof fn one_watad(self)
                    ensures watad_count(self.word_spec(),self.word_spec().len() as int) == 1,
                {
                    reveal_with_fuel(watad_count,10);
                    reveal(FootId::word_spec);
                    reveal(window_word);
                    reveal(circle_word_spec);
                    reveal(Unit::S); reveal(Unit::T); reveal(Unit::W); reveal(Unit::M);
                    match self {
                        $(Self::$variant => {
                            assert(self.index() == $index);
                            assert(self.word_spec() == window_word(FOOT_AT@[$index as int]));
                            assert(self.word_spec().len() == FOOT_AT@[$index as int].3);
                            assert(watad_count(window_word(FOOT_AT@[$index as int]),FOOT_AT@[$index as int].3 as int) == 1) by(compute);
                            assert(watad_count(self.word_spec(),self.word_spec().len() as int) == 1);
                        },)*
                    }
                }

                /// The actual circle-derived pattern, with its original identity.
                pub const fn pattern(self) -> (result: ([Unit;3],usize))
                    ensures 2 <= result.1 <= 3,
                        result.0@.subrange(0,result.1 as int) == self.word_spec(),
                        watad_count(result.0@.subrange(0,result.1 as int),result.1 as int) == 1,
                {
                    proof { self.one_watad(); }
                    foot_window(FOOT_AT[self.id()])
                }
            }
        }
    };
}

foot_indices! {
    FaUlun => 0, FaIlun => 1, Mafailun => 2, MustafIlun => 3,
    Failatun => 4, Mafalatun => 5, MutaFailun => 6, MafUlatu => 7,
    FailatunMafruq => 8, MustafIlunMafruq => 9, Failatuka => 10,
}

impl FootId {
    /// The foot with this small id (0..11) — [`FootId::id`] inverted.
    ///
    /// # Panics
    /// Panics when `id >= 11`. Prefer [`FootId::try_from_id`] at external
    /// boundaries; this is for bounded internal indices.
    pub const fn from_id(id: usize) -> FootId {
        match id {
            0 => FootId::FaUlun,
            1 => FootId::FaIlun,
            2 => FootId::Mafailun,
            3 => FootId::MustafIlun,
            4 => FootId::Failatun,
            5 => FootId::Mafalatun,
            6 => FootId::MutaFailun,
            7 => FootId::MafUlatu,
            8 => FootId::FailatunMafruq,
            9 => FootId::MustafIlunMafruq,
            10 => FootId::Failatuka,
            _ => panic!("FootId id out of range"),
        }
    }

    /// The foot with this small id, or `None` when out of range — the checked
    /// boundary counterpart of [`FootId::from_id`].
    pub const fn try_from_id(id: usize) -> Option<FootId> {
        if id < 11 {
            Some(FootId::from_id(id))
        } else {
            None
        }
    }

    /// Const equality.
    pub const fn same(self, other: FootId) -> bool {
        self.id() == other.id()
    }

    /// The unit decomposition of the foot — the window of [`CIRCLE_WORDS`] at
    /// this foot's attestation position. Derived, never entered, so a foot
    /// cannot lie about its structure.
    pub fn units(self) -> &'static [Unit] {
        let (units, len) = &FOOT_UNITS[self.id()];
        &units[..*len]
    }

    /// Classical Arabic name (تفعيلة).
    pub const fn name_ar(self) -> &'static str {
        match self {
            FootId::FaUlun => "فعولن",
            FootId::FaIlun => "فاعلن",
            FootId::Mafailun => "مفاعيلن",
            FootId::MustafIlun => "مستفعلن",
            FootId::Failatun => "فاعلاتن",
            FootId::Mafalatun => "مفاعلتن",
            FootId::MutaFailun => "متفاعلن",
            FootId::MafUlatu => "مفعولاتُ",
            FootId::FailatunMafruq => "فاعِ لاتن",
            FootId::MustafIlunMafruq => "مستفعِ لن",
            FootId::Failatuka => "فاعلاتُكَ",
        }
    }
}

/// The unit that owns a letter of a foot, and the letter's offset within it.
///
/// Produced by [`Foot::unit_at_letter`]. A structural fact only: it reports
/// ownership, not whether an operation is legal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LetterOrigin {
    /// Index of the owning unit within [`Foot::units`].
    pub unit_index: usize,
    /// The owning unit.
    pub unit: Unit,
    /// 0-based offset of the letter within the unit.
    pub offset: usize,
}

/// A foot: an identity plus its derived unit word.
///
/// The only inhabitants are the 11 attested feet ([`FootId`] is exhaustive), and
/// the unit word is derived from the id. Zihāf/ʿilla results are *not* `Foot`s.
/// Transformed results are beat sequences or original-base programs, depending
/// on the selected transformation API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Foot(FootId);

impl Foot {
    /// Wrap a foot identity.
    pub const fn of(id: FootId) -> Foot {
        Foot(id)
    }

    /// The foot identity.
    pub const fn id(self) -> FootId {
        self.0
    }

    /// The unit decomposition of the foot.
    pub fn units(self) -> &'static [Unit] {
        self.0.units()
    }

    /// Resolve a 0-based `letter` of the foot's letter stream to the unit that
    /// owns it, and the 0-based offset within that unit.
    ///
    /// Returns `None` when `letter` is past the end of the foot. This is the
    /// structural ownership fact a legality boundary needs to tell a sabab
    /// target from a watad target; it does not itself authorize an operation.
    pub fn unit_at_letter(self, letter: usize) -> Option<LetterOrigin> {
        let mut offset = letter;
        for (unit_index, unit) in self.units().iter().enumerate() {
            let count = unit.beat_count();
            if offset < count {
                return Some(LetterOrigin {
                    unit_index,
                    unit: *unit,
                    offset,
                });
            }
            offset -= count;
        }
        None
    }

    /// Classical Arabic name (تفعيلة).
    pub const fn name_ar(self) -> &'static str {
        self.0.name_ar()
    }

    /// فعولن `//0/0`.
    pub const FA_ULUN: Foot = Foot::of(FootId::FaUlun);
    /// فاعلن `/0//0`.
    pub const FA_ILUN: Foot = Foot::of(FootId::FaIlun);
    /// مفاعيلن `//0/0/0`.
    pub const MAF_AILUN: Foot = Foot::of(FootId::Mafailun);
    /// مستفعلن `/0/0//0` (watad majmūʿ at the end).
    pub const MUSTAF_ILUN: Foot = Foot::of(FootId::MustafIlun);
    /// فاعلاتن `/0//0/0`.
    pub const FA_ILATUN: Foot = Foot::of(FootId::Failatun);
    /// مفاعلتن `//0///0`.
    pub const MAF_ALATUN: Foot = Foot::of(FootId::Mafalatun);
    /// متفاعلن `///0//0`.
    pub const MUTAFA_ILUN: Foot = Foot::of(FootId::MutaFailun);
    /// مفعولاتُ `/0/0/0/`.
    pub const MAF_ULATU: Foot = Foot::of(FootId::MafUlatu);
    /// فاعِ لاتن `/0//0/0` (watad mafrūq in the middle).
    pub const FA_ILATUN_MAFRUQ: Foot = Foot::of(FootId::FailatunMafruq);
    /// مستفعِ لن `/0/0//0` (watad mafrūq in the middle).
    pub const MUSTAF_ILUN_MAFRUQ: Foot = Foot::of(FootId::MustafIlunMafruq);
    /// فاعلاتُكَ `/0//0//` (`spec/circles.md` §2, `[S2]`).
    pub const FA_ILATUKA: Foot = Foot::of(FootId::Failatuka);

    /// Every attested foot, in id order.
    pub const ATTESTED: [Foot; 11] = [
        Foot::FA_ULUN,
        Foot::FA_ILUN,
        Foot::MAF_AILUN,
        Foot::MUSTAF_ILUN,
        Foot::FA_ILATUN,
        Foot::MAF_ALATUN,
        Foot::MUTAFA_ILUN,
        Foot::MAF_ULATU,
        Foot::FA_ILATUN_MAFRUQ,
        Foot::MUSTAF_ILUN_MAFRUQ,
        Foot::FA_ILATUKA,
    ];
}

/// Position of a foot in the hemistich.
///
/// Runtime role metadata for tables and structural diagnostics. Profile
/// programs separately check initial and terminal eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// الابتداء — the first foot.
    Ibtida,
    /// الحشو — the middle feet (only زحاف applies, never عيلة).
    Hashw,
    /// العروض — the last foot of the صدر.
    Arud,
    /// الضرب — the last foot of theعجز.
    Darb,
}

impl Role {
    /// Small stable id (0..4) for const bookkeeping.
    pub const fn id(self) -> usize {
        match self {
            Role::Ibtida => 0,
            Role::Hashw => 1,
            Role::Arud => 2,
            Role::Darb => 3,
        }
    }

    /// Const equality.
    pub const fn same(self, other: Role) -> bool {
        self.id() == other.id()
    }

    /// True for العروض and الضرب — the positions where a عِلَّة may apply.
    pub const fn may_carry_illa(self) -> bool {
        matches!(self, Role::Arud | Role::Darb)
    }

    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Role::Ibtida => "ابتداء",
            Role::Hashw => "حشو",
            Role::Arud => "عروض",
            Role::Darb => "ضرب",
        }
    }
}

/// Renders a unit word as a `/0` pattern: «فعولن» → `//0/0`.
///
/// A computed view only — never an identity (`docs/TYPES.md`).
#[derive(Debug, Clone, Copy)]
pub struct Pattern<'a>(pub &'a [Unit]);

impl fmt::Display for Pattern<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for unit in self.0 {
            for beat in unit.beats() {
                write!(f, "{}", beat.symbol())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_beats_match_the_spec() {
        assert_eq!(Pattern(&[Unit::S]).to_string(), "/0");
        assert_eq!(Pattern(&[Unit::T]).to_string(), "//");
        assert_eq!(Pattern(&[Unit::W]).to_string(), "//0");
        assert_eq!(Pattern(&[Unit::M]).to_string(), "/0/");
    }

    #[test]
    fn every_attested_foot_has_exactly_one_watad() {
        for foot in Foot::ATTESTED {
            assert!(
                has_exactly_one_watad(foot.units()),
                "{} must have exactly one watad",
                foot.name_ar()
            );
        }
    }

    #[test]
    fn attested_foot_patterns() {
        let expect = [
            (Foot::FA_ULUN, "//0/0"),
            (Foot::FA_ILUN, "/0//0"),
            (Foot::MAF_AILUN, "//0/0/0"),
            (Foot::MUSTAF_ILUN, "/0/0//0"),
            (Foot::FA_ILATUN, "/0//0/0"),
            (Foot::MAF_ALATUN, "//0///0"),
            (Foot::MUTAFA_ILUN, "///0//0"),
            (Foot::MAF_ULATU, "/0/0/0/"),
            (Foot::FA_ILATUN_MAFRUQ, "/0//0/0"),
            (Foot::MUSTAF_ILUN_MAFRUQ, "/0/0//0"),
            (Foot::FA_ILATUKA, "/0//0//"),
        ];
        for (foot, pattern) in expect {
            assert_eq!(
                Pattern(foot.units()).to_string(),
                pattern,
                "{}",
                foot.name_ar()
            );
        }
    }

    #[test]
    fn structure_over_surface() {
        // Same surface, different structure — the reason feet are not strings.
        assert_eq!(
            Pattern(Foot::MUSTAF_ILUN.units()).to_string(),
            Pattern(Foot::MUSTAF_ILUN_MAFRUQ.units()).to_string()
        );
        assert!(!units_eq(
            Foot::MUSTAF_ILUN.units(),
            Foot::MUSTAF_ILUN_MAFRUQ.units()
        ));
        assert_eq!(
            Pattern(Foot::FA_ILATUN.units()).to_string(),
            Pattern(Foot::FA_ILATUN_MAFRUQ.units()).to_string()
        );
        assert!(!units_eq(
            Foot::FA_ILATUN.units(),
            Foot::FA_ILATUN_MAFRUQ.units()
        ));
    }

    #[test]
    fn sabab_watad_split() {
        // A watad is not a sabab at the type level; the view helpers agree.
        assert!(Unit::W.is_watad() && Unit::M.is_watad());
        assert!(Unit::S.is_sabab() && Unit::T.is_sabab());
        assert!(Unit::S.as_sabab().is_some());
        assert!(Unit::W.as_sabab().is_none());
    }

    #[test]
    fn checked_boundaries_reject_out_of_range() {
        assert_eq!(FootId::try_from_id(0), Some(FootId::FaUlun));
        assert_eq!(FootId::try_from_id(11), None);
        // Out-of-range circle, bad len, and len == 0 are rejected, not panicked.
        assert_eq!(foot_window_at(5, 0, 0, 3), None);
        assert_eq!(foot_window_at(0, 0, 0, 4), None);
        assert_eq!(foot_window_at(0, 0, 0, 0), None);
        // A valid window still resolves (circle 2, rot 2, off 0, len 3 = فاعلاتن).
        assert_eq!(foot_window_at(2, 2, 0, 3), Some(FootId::Failatun));
    }

    #[test]
    fn unit_at_letter_resolves_ownership() {
        // فاعلاتن `/0//0/0` = S W S → letters 0,1 | 2,3,4 | 5,6.
        let foot = Foot::FA_ILATUN;
        assert_eq!(
            foot.unit_at_letter(0),
            Some(LetterOrigin {
                unit_index: 0,
                unit: Unit::S,
                offset: 0
            })
        );
        assert_eq!(
            foot.unit_at_letter(3),
            Some(LetterOrigin {
                unit_index: 1,
                unit: Unit::W,
                offset: 1
            })
        );
        assert_eq!(
            foot.unit_at_letter(5),
            Some(LetterOrigin {
                unit_index: 2,
                unit: Unit::S,
                offset: 0
            })
        );
    }

    #[test]
    fn unit_at_letter_covers_every_letter_of_every_foot() {
        for foot in Foot::ATTESTED {
            let total: usize = foot.units().iter().map(|u| u.beat_count()).sum();
            for letter in 0..total {
                let origin = foot
                    .unit_at_letter(letter)
                    .expect("in-range letter must resolve");
                assert_eq!(origin.unit, foot.units()[origin.unit_index]);
                assert!(origin.offset < origin.unit.beat_count());
            }
            // The end and beyond resolve to nothing.
            assert_eq!(foot.unit_at_letter(total), None);
            assert_eq!(foot.unit_at_letter(usize::MAX), None);
        }
    }

    #[test]
    fn extreme_window_offsets_wrap_without_overflow() {
        for (circle, word) in CIRCLE_WORDS.iter().enumerate() {
            for rot in 0..word.len() {
                for off in [usize::MAX, usize::MAX - 1, usize::MAX - 2] {
                    for len in 1..=3 {
                        assert_eq!(
                            foot_window_at(circle, rot, off, len),
                            foot_window_at(circle, rot, off % word.len(), len)
                        );
                    }
                }
            }
        }
    }
}
