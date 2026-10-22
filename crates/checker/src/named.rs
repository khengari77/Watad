//! Named transformation vocabulary shared by the checked normalizer and the
//! mechanical transformation crate.
//!
//! The names and recipe data live inside the verified boundary so that
//! normalization does not translate a trusted enum into a second representable
//! operation. Executable callers still use the same public types through
//! `watad_tahwil` re-exports.
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus! {

/// The classical letter position inside a foot (1-based).
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LetterPos(u8);

/// What the target letter must be for an operation to apply.
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Precondition {
    /// ساكن (`0`).
    Sakin,
    /// متحرك (`/`).
    Mutaharrik,
}

/// What the operation does to the target letter.
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    /// حذف — remove the letter.
    Delete,
    /// إسكان — make the letter ساكن.
    Silence,
}

/// الزحافات المفرد — the eight simple زحافات (`spec/transformations.md` §2).
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Zihaf {
    /// الخَبْن: حذف ثاني الجزء ساكنًا.
    Khabn,
    /// الإِضْمار: إسكان ثاني الجزء متحركًا.
    Idmar,
    /// الوَقْص: حذف ثاني الجزء متحركًا.
    Waqs,
    /// الطَّيّ: حذف رابعه ساكنًا.
    Tayy,
    /// القَبْض: حذف خامسه ساكنًا.
    Qabd,
    /// العَصْب: إسكان خامسه بعد تحريكه.
    Asb,
    /// العَقْل: حذف خامسه متحركًا.
    Aql,
    /// الكَفّ: حذف سابعه ساكنًا.
    Kaf,
}

/// The taxonomy of [`Illa`].
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IllaKind {
    /// علل الزيادة.
    Ziyada,
    /// علل النقصان.
    Nuqsan,
    /// الخَرْم, in a class of its own.
    Kharm,
}

/// عِلَّة — a terminal change at العروض/الضرب (`spec/transformations.md` §4).
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Illa {
    /// التَّرْفيل.
    Tarfil,
    /// التَّذْييل.
    Tadhyil,
    /// التَّسْبيغ.
    Tasbigh,
    /// الحَذْف.
    Hadhf,
    /// القَطْف.
    Qatf,
    /// القَطْع.
    Qat,
    /// البَتْر.
    Batr,
    /// القَصْر.
    Qasr,
    /// الحَذَذ.
    Hadhadh,
    /// الصَّلْم.
    Salm,
    /// الوَقْف.
    Waqf,
    /// الكَسْف.
    Kasf,
    /// الخَرْم.
    Kharm,
}

/// One requested named change, in original foot coordinates.
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Change {
    Unchanged,
    Zihaf(Zihaf),
    Illa(Illa),
    /// A source profile may treat an illa as nonbinding variation.
    NonbindingIlla(Illa),
    /// Delete the first moving letter of the foot's majmu watad.
    Tashith,
}

pub open spec fn zihaf_index(z: Zihaf) -> int {
    match z {
        Zihaf::Khabn | Zihaf::Idmar | Zihaf::Waqs => 1,
        Zihaf::Tayy => 3,
        Zihaf::Qabd | Zihaf::Asb | Zihaf::Aql => 4,
        Zihaf::Kaf => 6,
    }
}

pub open spec fn zihaf_expected(z: Zihaf) -> u8 {
    match z {
        Zihaf::Khabn | Zihaf::Tayy | Zihaf::Qabd | Zihaf::Kaf => 0,
        Zihaf::Idmar | Zihaf::Waqs | Zihaf::Asb | Zihaf::Aql => 1,
    }
}

pub open spec fn zihaf_action(z: Zihaf) -> Operation {
    match z {
        Zihaf::Idmar | Zihaf::Asb => Operation::Silence,
        _ => Operation::Delete,
    }
}

pub open spec fn illa_required(i: Illa) -> Option<watad_types::Unit> {
    match i {
        Illa::Tarfil | Illa::Tadhyil | Illa::Qat | Illa::Hadhadh => Some(watad_types::Unit::W),
        Illa::Tasbigh | Illa::Hadhf | Illa::Qatf | Illa::Qasr | Illa::Batr => {
            Some(watad_types::Unit::S)
        }
        Illa::Salm | Illa::Waqf | Illa::Kasf => Some(watad_types::Unit::M),
        Illa::Kharm => None,
    }
}

pub open spec fn adds_suffix(i: Illa) -> bool {
    matches!(i, Illa::Tarfil | Illa::Tadhyil | Illa::Tasbigh)
}

impl LetterPos {
    /// 1-based index into the letter stream.
    pub const fn one_based(self) -> u8 {
        self.0
    }
}

impl Zihaf {
    /// The struck letter position (1-based).
    pub const fn position(self) -> LetterPos {
        match self {
            Zihaf::Khabn | Zihaf::Idmar | Zihaf::Waqs => LetterPos(2),
            Zihaf::Tayy => LetterPos(4),
            Zihaf::Qabd | Zihaf::Asb | Zihaf::Aql => LetterPos(5),
            Zihaf::Kaf => LetterPos(7),
        }
    }

    /// The precondition on the struck letter.
    pub const fn precondition(self) -> Precondition {
        match self {
            Zihaf::Khabn | Zihaf::Tayy | Zihaf::Qabd | Zihaf::Kaf => Precondition::Sakin,
            Zihaf::Idmar | Zihaf::Waqs | Zihaf::Asb | Zihaf::Aql => Precondition::Mutaharrik,
        }
    }

    /// The operation on the struck letter.
    pub const fn operation(self) -> Operation {
        match self {
            Zihaf::Khabn | Zihaf::Waqs | Zihaf::Tayy | Zihaf::Qabd | Zihaf::Aql | Zihaf::Kaf => {
                Operation::Delete
            }
            Zihaf::Idmar | Zihaf::Asb => Operation::Silence,
        }
    }
}

} // verus!

impl LetterPos {
    /// letter 2 — «ثاني الجزء».
    pub const SECOND: LetterPos = LetterPos(2);
    /// letter 4 — «رابعه».
    pub const FOURTH: LetterPos = LetterPos(4);
    /// letter 5 — «خامسه».
    pub const FIFTH: LetterPos = LetterPos(5);
    /// letter 7 — «سابعه».
    pub const SEVENTH: LetterPos = LetterPos(7);

    /// The positions a named simple زحاف may strike.
    pub const ALL: [LetterPos; 4] = [
        LetterPos::SECOND,
        LetterPos::FOURTH,
        LetterPos::FIFTH,
        LetterPos::SEVENTH,
    ];

    /// Smart constructor: a named simple زحاف strikes only 2, 4, 5 or 7.
    pub const fn new(one_based: u8) -> Option<LetterPos> {
        match one_based {
            2 => Some(LetterPos::SECOND),
            4 => Some(LetterPos::FOURTH),
            5 => Some(LetterPos::FIFTH),
            7 => Some(LetterPos::SEVENTH),
            _ => None,
        }
    }
}

impl Precondition {
    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Precondition::Sakin => "ساكن",
            Precondition::Mutaharrik => "متحرك",
        }
    }
}

impl Operation {
    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Operation::Delete => "حذف",
            Operation::Silence => "إسكان",
        }
    }
}

impl Zihaf {
    /// Every simple زحاف, in the classical order.
    pub const ALL: [Zihaf; 8] = [
        Zihaf::Khabn,
        Zihaf::Idmar,
        Zihaf::Waqs,
        Zihaf::Tayy,
        Zihaf::Qabd,
        Zihaf::Asb,
        Zihaf::Aql,
        Zihaf::Kaf,
    ];

    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Zihaf::Khabn => "الخَبْن",
            Zihaf::Idmar => "الإِضْمار",
            Zihaf::Waqs => "الوَقْص",
            Zihaf::Tayy => "الطَّيّ",
            Zihaf::Qabd => "القَبْض",
            Zihaf::Asb => "العَصْب",
            Zihaf::Aql => "العَقْل",
            Zihaf::Kaf => "الكَفّ",
        }
    }

    /// Source attestation restriction: العَصْب والعَقْل are Mafalatun-only.
    pub const fn restricted_to_mafalatun(self) -> bool {
        matches!(self, Zihaf::Asb | Zihaf::Aql)
    }
}

impl Illa {
    /// Every عِلَّة, in the order of the source listings.
    pub const ALL: [Illa; 13] = [
        Illa::Tarfil,
        Illa::Tadhyil,
        Illa::Tasbigh,
        Illa::Hadhf,
        Illa::Qatf,
        Illa::Qat,
        Illa::Batr,
        Illa::Qasr,
        Illa::Hadhadh,
        Illa::Salm,
        Illa::Waqf,
        Illa::Kasf,
        Illa::Kharm,
    ];

    /// زيادة / نقصان / الخرم.
    pub const fn kind(self) -> IllaKind {
        match self {
            Illa::Tarfil | Illa::Tadhyil | Illa::Tasbigh => IllaKind::Ziyada,
            Illa::Kharm => IllaKind::Kharm,
            _ => IllaKind::Nuqsan,
        }
    }

    /// Classical Arabic name.
    pub const fn name_ar(self) -> &'static str {
        match self {
            Illa::Tarfil => "التَّرْفيل",
            Illa::Tadhyil => "التَّذْييل",
            Illa::Tasbigh => "التَّسْبيغ",
            Illa::Hadhf => "الحَذْف",
            Illa::Qatf => "القَطْف",
            Illa::Qat => "القَطْع",
            Illa::Batr => "البَتْر",
            Illa::Qasr => "القَصْر",
            Illa::Hadhadh => "الحَذَذ",
            Illa::Salm => "الصَّلْم",
            Illa::Waqf => "الوَقْف",
            Illa::Kasf => "الكَسْف",
            Illa::Kharm => "الخَرْم",
        }
    }

    /// «ولا تكون في الحشو بل في الضرب والعروض، ما عدا الخَرْم».
    pub const fn may_occur_in_hashw(self) -> bool {
        matches!(self, Illa::Kharm)
    }

    /// Every named mechanical recipe is implemented.
    pub const fn has_mechanical_rule(self) -> bool {
        true
    }
}

impl Change {
    pub const fn name_ar(self) -> &'static str {
        match self {
            Self::Unchanged => "سالم",
            Self::Zihaf(z) => z.name_ar(),
            Self::Illa(i) | Self::NonbindingIlla(i) => i.name_ar(),
            Self::Tashith => "التشعيث",
        }
    }
}
