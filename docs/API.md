# Rust API

Watad accepts explicit beats and an externally selected meter profile. Generated
`cargo doc` pages provide the full symbol reference; this guide covers the
public recognition path. The workspace crates can be used as local path
dependencies. These examples do not assume publication on crates.io.

| Task | Entry points |
| --- | --- |
| Parse beats | `watad_matcher::parse` |
| Enumerate meters | `watad_lugha::catalog::meters` |
| Usage profile | `watad_lugha::catalog::hashimi` |
| Unchanged circle profile | `watad_lugha::catalog::circle` |
| Verse recognition | `watad_matcher::recognize` |
| Checked explanation | `watad_matcher::explain` |
| Whole-poem recognition | `watad_qasida::recognize` |
| Whole-poem replay | `watad_qasida::check` |
| Custom source authority | `watad_lugha::meter::MeterProfile::new` |

Buildable examples: [scan.rs](../crates/matcher/examples/scan.rs) and
[poem.rs](../crates/qasida/examples/poem.rs).

## Beat input

`watad_matcher::parse(&str)` returns `Result<Vec<Beat>, ParseError>`.
`/` and `1` become `Beat::Mutaharrik`; `0` becomes `Beat::Sakin`.
Whitespace is ignored. Other characters produce `ParseError { byte, character }`,
where `byte` is the UTF-8 byte offset in the original string. Empty input parses
to an empty vector, but does not match a nonempty sadr.

The parser does not infer Arabic pronunciation, vocalization, or orthography.

## Profile selection

`catalog::hashimi(meter)` returns `Result<Option<MeterProfile>, Error>`:
`Some` contains a usage profile, while `None` means the rotation has no built-in
usage profile. `catalog::circle(meter)` returns an unchanged circle profile
for any of the 22 rotations.

`MeterProfile` owns immutable authority. `definition()` exposes source recipes;
`structured()` exposes original-base/program data; `kernel()` exposes the
lowered finite language. `is_circle_bound()` reports circle-origin binding.

Circle profiles use family `1000 + meter.id()`; usage profiles use
`2000 + meter.id()`. Both have revision `1`. IDs label definitions; they are
not hashes, authentication, or substitutes for the actual profile data.
See [coverage and conventions](../spec/profiles/classical-v1.md).

## Verse recognition and explanation

`watad_matcher::recognize(profiles, scope, sadr, ajuz)` returns a `Vec<Match>`
inside a `Result`. Each match contains a supplied-profile index and one
certificate. An empty vector means none of those profiles matches. It does not
enumerate every possible derivation within a matching profile.

For a single profile, `profile.recognize(scope, sadr, ajuz)` returns
`Result<Option<Certificate>, Error>`. Replay with
`profile.check(scope, sadr, ajuz, certificate)`.

`Scope { context, initial }` is supplied separately from the certificate.
`initial` is a caller assertion for a single-verse call; beats do not establish
verse position. `context` selects declared form metadata.

`watad_matcher::explain` replays a certificate before using its indices. It
returns original `FootId` values, named `Change` recipes, and half-open spans,
or `None` for invalid claims. Spans index parsed beats, not bytes or Arabic
characters.

## Whole poems

`watad_qasida::Verse` contains `context: u16`, `sadr: Vec<Beat>` and
`ajuz: Vec<Beat>`. Pass an ordered slice to
`watad_qasida::recognize(profiles, verses)`.

Each result selects one supplied profile and provides one certificate per
verse. Initial position comes from the verse index. The solver enforces
binding classes and every existence requirement, revisiting ambiguous readings
when necessary. `watad_qasida::check(profile, verses, certificates)` replays
the entire assignment. Individually valid verses can still fail poem checking.

Lower-level `MeterProfile::recognize_poem` and `check_poem` accept
`watad_checker::poem::Input` with encoded `u8` beats. Use the qasida adapter
for `Beat` values.

## Contexts and single-line forms

The core treats context numbers as profile-defined data. Built-in usage profiles
define `0` ordinary, `1` initial تصريع, `2` supplied ردف evidence for Basit's
conditional cut darb, and `3` that evidence combined with initial تصريع.
Unsupported contexts yield no match. Custom profiles may define other meanings.

A single-line form has no ajuz slots and requires empty ajuz input and empty
ajuz certificate steps. The sadr is never copied to fabricate an ajuz.
Ordinary two-half forms do not match empty ajuz.

## Certificates

`Certificate` records profile ID, context, form index and sadr/ajuz span steps.
Its public fields are untrusted claims. Check against the expected profile,
scope and original input before indexing or displaying a derivation.

Certificates are in-memory values. No stable serialization or cross-version
persistence contract is supplied. Form order and alternative indices are
observable: a certificate belongs to the exact definition used for replay.
Matching IDs alone do not make different definitions interchangeable.

## Custom definitions

`Definition` contains `id`, `meter`, `source`, `binding`, `forms` and
`requirements`. Forms carry retained foot counts, contexts, initial-only flags,
darb classes, original-letter exclusions and sadr/ajuz recipe alternatives.

`MeterProfile::new` validates provenance, layout, recipes, requirements and
structured validity, then binds original feet to the selected circle sequence.
Equal-surface feet cannot substitute for different structural ownership.

Each requirement group names source-form indices and requires a selected form
from that group somewhere in the poem. `FixedDarbClass` requires a common
class; `Unconstrained` omits that restriction. Neither grants absent recipes.

See [catalog compilation](../spec/catalog-compilation.md) for ordering,
duplicates, range unions and constraints. The generic structured-profile API is
available through `watad_checker::profile` for lower-level supplied authority.

## Errors and negative results

| Result | Meaning |
| --- | --- |
| `Ok(None)` from one profile | No derivation in that profile/context |
| `Ok(vec![])` from matcher/qasida | No supplied profile matches |
| `Error::MissingSource` | Empty or whitespace-only source provenance |
| `Error::Layout` | Invalid retained-foot count or half layout |
| `Error::Recipe(...)` | Invalid named recipe |
| `Error::Profile` | Invalid structured profile or profile validation failure |
| `Error::Requirement` | Empty requirement group or nonexistent form reference |
| `Error::EmptyPoem` | Empty poem recognition input |
| `false` from a checker | Claims fail the selected authority/input relation |

A complete negative exhausts the declared finite search. There is no heuristic
cutoff reported as no-match. Exact search can be exponential; allocation failure
and process limits are environmental limits, not language results.

See [transformations](../spec/transformations.md) for named mechanics and
[structural replay](../spec/derivation.md) for the one-foot API.
