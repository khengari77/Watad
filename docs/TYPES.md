# Types and structural identity

## 1. Beats and units — `watad-types`

`Beat::{Mutaharrik, Sakin}` represents moving and still letters. `Sabab` has
`Khafif` (`/0`) and `Thaqil` (`//`); `Watad` has `Majmu` (`//0`) and
`Mafruq` (`/0/`). `Unit` contains either a sabab or a watad.

`Unit::beats()` provides the beat expansion. `Pattern` renders a computed
surface; it is not a structural identity. The authoritative words and meter
tables are in [circles](../spec/circles.md).

## 2. Feet — `watad-types`

`FootId` identifies eleven original feet: the ten canonical structures and
`Failatuka` for Mutawafir. `Foot` exposes the corresponding original unit word.
Each named original foot contains exactly one watad.

`MustafIlun` and `MustafIlunMafruq` have the same surface but different unit
ownership. The same distinction holds for `Failatun` and `FailatunMafruq`.
Transformations and circle-bound profiles preserve the original identity.

## 3. Meters — `watad-daira`

`CircleId` selects one of the five primitive words. `MeterName` identifies a
rotation, Arabic name and usage status. `MeterName::feet()` gives the generated
circle-form feet. Usage profiles may retain a shorter prefix and apply explicit
transformations; a display label never performs that edit implicitly.

`MeterForm` composes bounded circle segments with a terminal-shape label.
Its accessors expose actual units and feet. `TerminalRole::{Arud, Darb}`
identifies a hemistich end. `PositionContext` preserves initial and terminal
facts independently, including for a one-foot form.

## 4. Mechanical changes — `watad-tahwil`

`Zihaf` and `Illa` name mechanical changes. Ownership-checked operations reject
zihaf targets within watads and preserve original coordinates for compounds.
Position-aware structural derivation additionally checks scope.

The profile path uses named `Change` recipes compiled into original-letter
programs. See [transformations](../spec/transformations.md) and
[one-foot structural replay](../spec/derivation.md). Mechanical applicability
alone does not establish a meter's historical permission.

## 5. Profiles and certificates — `watad-lugha`, `watad-checker`

`Definition` is caller-supplied authority: identity, meter, source, binding,
forms and existence requirements. Validated `MeterProfile` owns immutable
source, structural and lowered representations.

`ProfileId` contains a family and revision. `DarbClass` expresses declared
poem binding; `Scope` carries external context and initial-position claims.
`Certificate` contains untrusted form/alternative/span claims and must be
replayed against expected authority and input. See the [API guide](API.md).

## 6. Guarantee boundaries

Verus checks the annotated production relations and algorithms listed in
[verification](../formal/README.md). Compile-time assertions and independent
tests additionally check source tables and integration. Historical names,
permissions and interpretation remain source data. APIs for Arabic spelling
conversion and rhyme-letter inference are not provided.
