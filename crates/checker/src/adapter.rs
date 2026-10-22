//! Proven projections of actual workspace values into the certificate alphabet.
//!
//! These functions are shared by Cargo and Verus. They do not validate a meter
//! form or map the linguistic operation enum; those obligations remain outside.

use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;
use watad_types::{Beat, FootId, Unit};

verus! {

/// Use the proved circle window of the actual named identity, without a
/// separately stored beat or unit table in the adapter.
pub fn encode_units(base: FootId) -> (units: Vec<Unit>)
    ensures units@ == base.word_spec(), 2 <= units@.len() <= 3,
        watad_types::watad_count(units@,units@.len() as int) == 1,
{
    let (pattern,len) = base.pattern();
    let mut units = Vec::new();
    let mut i = 0;
    while i < len
        invariant 2 <= len <= 3, i <= len, units@ == pattern@.subrange(0,i as int),
            pattern@.subrange(0,len as int) == base.word_spec(),
            watad_types::watad_count(pattern@.subrange(0,len as int),len as int) == 1,
        decreases len-i,
    {
        units.push(pattern[i]);
        i += 1;
    }
    units
}

/// Specification of the byte alphabet, independent of the executable match.
pub open spec fn beat_code(beat: Beat) -> u8 {
    if beat == Beat::Mutaharrik { 1 } else { 0 }
}

/// Encode one beat without changing its meaning.
pub fn encode_beat(beat: Beat) -> (code: u8)
    ensures code == beat_code(beat),
{
    match beat {
        Beat::Mutaharrik => 1,
        Beat::Sakin => 0,
    }
}

/// Preserve both the length and every original position of an arbitrary word.
pub fn encode_beats(beats: &[Beat]) -> (codes: Vec<u8>)
    ensures
        codes@.len() == beats@.len(),
        forall|i: int| 0 <= i < beats@.len() ==> #[trigger] codes@[i] == beat_code(beats@[i]),
{
    let mut codes = Vec::new();
    let mut i = 0;
    while i < beats.len()
        invariant
            i <= beats.len(),
            codes.len() == i,
            forall|j: int| 0 <= j < i ==> #[trigger] codes@[j] == beat_code(beats@[j]),
        decreases beats.len() - i,
    {
        codes.push(encode_beat(beats[i]));
        i += 1;
    }
    codes
}

/// Different beat words cannot become equal after the alphabet projection.
pub proof fn encoding_is_injective(a: Seq<Beat>, b: Seq<Beat>, codes: Seq<u8>)
    requires
        a.len() == codes.len(), b.len() == codes.len(),
        forall|i: int| 0 <= i < a.len() ==> #[trigger] codes[i] == beat_code(a[i]),
        forall|i: int| 0 <= i < b.len() ==> #[trigger] codes[i] == beat_code(b[i]),
    ensures a == b,
{
    assert forall|i: int| 0 <= i < a.len() implies a[i] == b[i] by {
        assert(codes[i] == beat_code(a[i]));
        assert(codes[i] == beat_code(b[i]));
        match (a[i], b[i]) {
            (Beat::Mutaharrik, Beat::Mutaharrik) => {},
            (Beat::Sakin, Beat::Sakin) => {},
            _ => {},
        }
    }
    assert(a =~= b);
}

} // verus!
