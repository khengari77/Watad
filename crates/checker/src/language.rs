//! General finite concatenation languages. No meter-specific branches or limits.
//!
//! Alternatives are profile data. Empty alternatives and an empty slot sequence
//! have their mathematical meanings; profile validation may impose stricter rules.

use alloc::vec::Vec;
#[cfg(not(verus_keep_ghost))]
use verus_builtin_macros::verus;
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus! {

/// Finite alternatives for one position, supplied by the selected profile.
pub struct Slot {
    pub words: Vec<Vec<u8>>,
}

/// A word agrees with an input interval, including its complete length.
pub open spec fn prefix(input: Seq<u8>, at: int, word: Seq<u8>) -> bool {
    0 <= at <= input.len() && at + word.len() <= input.len()
        && forall|j: int| 0 <= j < word.len() ==> word[j] == input[at + j]
}

pub fn word_matches(input: &[u8], at: usize, word: &[u8]) -> (yes: bool)
    ensures yes == prefix(input@, at as int, word@),
{
    if at > input.len() || word.len() > input.len() - at {
        return false;
    }
    let mut j = 0;
    while j < word.len()
        invariant
            at <= input.len(), word.len() <= input.len() - at,
            j <= word.len(),
            forall|k: int| 0 <= k < j ==> word@[k] == input@[at + k],
        decreases word.len() - j,
    {
        if word[j] != input[at + j] { return false; }
        j += 1;
    }
    true
}

/// Independent derivation relation: choose an alternative at each position;
/// concatenate the whole words, ending exactly at the end of the input.
pub open spec fn derives(slots: Seq<Slot>, input: Seq<u8>, i: int, at: int) -> bool
    decreases slots.len() - i,
{
    if i < 0 || i > slots.len() || at < 0 || at > input.len() { false }
    else if i == slots.len() { at == input.len() }
    else {
        exists|c: int| 0 <= c < slots[i].words@.len()
            && prefix(input, at, #[trigger] slots[i].words@[c]@)
            && derives(slots, input, i + 1, at + slots[i].words@[c]@.len())
    }
}

/// Exact reference decision: no search budget, no truncation, no false negative.
/// Exponential in the worst case; optimization must preserve this contract.
pub fn decide(slots: &[Slot], input: &[u8]) -> (yes: bool)
    ensures yes == derives(slots@, input@, 0, 0),
{
    recognize(slots, input).is_some()
}

/// A selected alternative and its exact half-open interval in the external input.
#[cfg_attr(verus_keep_ghost, derive(Structural))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub choice: usize,
    pub start: usize,
    pub end: usize,
}

/// Declarative certificate relation. Coverage is consecutive from `at`; the last
/// endpoint is the full input length. Caller-supplied intervals are not repaired.
pub open spec fn trace(slots: Seq<Slot>, input: Seq<u8>, i: int, at: int,
                       steps: Seq<Step>) -> bool
    decreases slots.len() - i,
{
    if i < 0 || i > slots.len() || at < 0 || at > input.len()
        || steps.len() != slots.len() - i { false }
    else if i == slots.len() { at == input.len() }
    else {
        let step = steps[0];
        let c = step.choice as int;
        &&& 0 <= c < slots[i].words@.len()
        &&& step.start == at
        &&& prefix(input, at, slots[i].words@[c]@)
        &&& step.end == at + slots[i].words@[c]@.len()
        &&& trace(slots, input, i + 1, step.end as int,
                  steps.subrange(1, steps.len() as int))
    }
}

fn check_from(slots: &[Slot], input: &[u8], i: usize, at: usize,
              steps: &[Step]) -> (yes: bool)
    requires i <= slots.len(), at <= input.len(),
    ensures yes == trace(slots@, input@, i as int, at as int, steps@),
    decreases slots.len() - i,
{
    if steps.len() != slots.len() - i { return false; }
    if i == slots.len() { return at == input.len(); }
    let step = steps[0];
    if step.start != at || step.choice >= slots[i].words.len() { return false; }
    let word = &slots[i].words[step.choice];
    if !word_matches(input, at, word) { return false; }
    let end = at + word.len();
    if step.end != end { return false; }
    check_from(slots, input, i + 1, end, &steps[1..])
}

/// Total check of an untrusted certificate against the actual input.
pub fn check(slots: &[Slot], input: &[u8], steps: &[Step]) -> (yes: bool)
    ensures yes == trace(slots@, input@, 0, 0, steps@),
{
    check_from(slots, input, 0, 0, steps)
}

pub proof fn trace_is_derivation(slots: Seq<Slot>, input: Seq<u8>, i: int,
                                 at: int, steps: Seq<Step>)
    requires trace(slots, input, i, at, steps),
    ensures derives(slots, input, i, at),
    decreases slots.len() - i,
{
    if i < slots.len() {
        let step = steps[0];
        trace_is_derivation(slots, input, i + 1, step.end as int,
                            steps.subrange(1, steps.len() as int));
        assert(exists|c: int| 0 <= c < slots[i].words@.len()
            && prefix(input, at, #[trigger] slots[i].words@[c]@)
            && derives(slots, input, i + 1, at + slots[i].words@[c]@.len()));
    }
}

fn recognize_from(slots: &[Slot], input: &[u8], i: usize, at: usize)
    -> (answer: Option<Vec<Step>>)
    requires i <= slots.len(), at <= input.len(),
    ensures
        answer.is_some() == derives(slots@, input@, i as int, at as int),
        match answer {
            Some(steps) => trace(slots@, input@, i as int, at as int, steps@),
            None => true,
        },
    decreases slots.len() - i,
{
    if i == slots.len() {
        return if at == input.len() { Some(Vec::new()) } else { None };
    }
    let words = &slots[i].words;
    let mut c = 0;
    while c < words.len()
        invariant
            i < slots.len(), at <= input.len(),
            words@ == slots@[i as int].words@,
            c <= words.len(),
            forall|k: int| 0 <= k < c ==> !(
                prefix(input@, at as int, #[trigger] words@[k]@)
                && derives(slots@, input@, i + 1, at + words@[k]@.len())),
        decreases words.len() - c,
    {
        let word = &words[c];
        if word_matches(input, at, word) {
            let end = at + word.len();
            if let Some(mut rest) = recognize_from(slots, input, i + 1, end) {
                let step = Step { choice: c, start: at, end };
                let ghost previous = rest@;
                rest.insert(0, step);
                proof {
                    previous.insert_ensures(0, step);
                    assert(rest@.subrange(1, rest@.len() as int) =~= previous);
                    trace_is_derivation(slots@, input@, i as int, at as int, rest@);
                }
                return Some(rest);
            }
        }
        c += 1;
    }
    None
}

/// Complete recognition with a certificate, not merely a positive Boolean.
pub fn recognize(slots: &[Slot], input: &[u8]) -> (answer: Option<Vec<Step>>)
    ensures
        answer.is_some() == derives(slots@, input@, 0, 0),
        match answer {
            Some(steps) => trace(slots@, input@, 0, 0, steps@),
            None => true,
        },
{
    recognize_from(slots, input, 0, 0)
}

} // verus!
