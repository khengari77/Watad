# Restricted Hazaj profile fixture

This finite language is specified independently for
`crates/lugha/tests/verus_conformance.rs`, using
`ProfileId { family: 1, revision: 1 }` with the generic `MeterProfile` API.
It is a test fixture, distinct from `catalog::hashimi(MeterName::Hazaj)`.

## Authority and declared coverage

**[H1]** Ahmad al-Hashimi, *Mizan al-Dhahab fi Sinaʿat Shiʿr al-ʿArab*,
«دروس علم العروض البحور», section «البحر السادس: الهزج», especially its
examples, «ملخص الهزج», and concluding verse on obligatory shortening.
[Publisher's text](https://www.safahat.org/books/50259706/1.2/).

The chapter gives two feet per hemistich, a standard arud, and standard/deleted
forms of darb. Its summary permits qabd and kaf in the body, mentions kaf at the
arud, and excludes simultaneous qabd/kaf in one foot. This source check is not an
independent scholarly audit of the edition.

This implementation supports ordinary verses with one operation per original
foot. Tasri, sequential edits, and unresolved endpoint readings are excluded or
unknown. It specifies the supported sublanguage completely; it does **not** claim
to cover every historical usage of hazaj.

The binding policy also uses the existing definition of binding illa in
[layer2.md](../transformations.md): the chosen darb class is fixed across ordinary verses.
Optional body/arud zihaf are not bound by that policy.

## Generated layout

Both hemistichs use these original feet:

```text
Mafailun | Mafailun
```

Layout data is `[(Mujtalab, rotation 0, repetitions 2)]`, with label `Majzu`.
Retaining two repetitions is explicit; the label alone performs no edit. The
three-repetition circle form is not this usage form.

## Permission matrix

`None` means no operation; it requires explicit permission as well.

| Position | None | Qabd | Kaf | Hadhf |
| --- | --- | --- | --- | --- |
| Body (the first foot of each hemistich) | Allowed | Allowed | Allowed | Forbidden |
| Arud | Allowed | Unknown | Allowed | Forbidden in this ordinary subprofile |
| Darb | Allowed | Unknown | Unknown | Allowed |

Only the explicitly listed recipes and ordinary context belong to this fixture.
Excluded or unknown readings are not claims of universal historical prohibition.

The summary's endpoint wording merits further edition/source review before
extending qabd to the arud or kaf/qabd to the darb. These statuses are conservative
coverage decisions, not newly invented classical prohibitions. No op in the
current single-operation model represents qabd combined with kaf.

## Pairing and binding

| Arud operation | Darb operation | Darb class |
| --- | --- | --- |
| None | None | 0: standard |
| Kaf | None | 0: standard |
| None | Hadhf | 1: deleted |
| Kaf | Hadhf | 1: deleted |

Within this profile a poem must retain one class. Arud kaf and body variations may
change between verses. Empty poems have no class witness and are rejected.

The four pairs are the Cartesian pairing of the explicitly supported arud and
darb alternatives. Their scope is ordinary verses in this subprofile, not a
universal pairing theorem for other meters or exceptional contexts.

## Independent finite acceptance description

Let `B = {None, Qabd, Kaf}`, `A = {None, Kaf}`, and `D = {None, Hadhf}`. The allowed
operation tuples, ordered as sadr first/last then ajuz first/last, are exactly:

```text
B × A × B × D
```

This gives 3 × 2 × 3 × 2 = **36 operation derivations**, 18 per darb class.
All are covered by an exhaustive regression with these independent expected words:

| Original Mafailun operation | Resulting beat word |
| --- | --- |
| None | `//0/0/0` |
| Qabd | `//0//0` |
| Kaf | `//0/0/` |
| Hadhf | `//0/0` |

These counts concern one verse and this declared operation model. Poem length is
not globally bounded; this is not a count of all possible poems or Arabic readings.

## Scope

Unresolved endpoint readings and exceptional contexts are outside this fixture.
The generic catalog's broader Hazaj profile has separate identity and data.
The fixture's 36 combinations are tested over an independent finite surface
product; its certificate checker is the general profile engine.
