# Catalog compilation specification

Production profiles for
Madid, Ramal, Khafif and Mujtathth retain source-scale rows and carry checked
selected-choice exclusions instead of eagerly expanded branches. The
Verus-checked `watad_checker::structured` lowering compiles every source rule
to per-alternative deletion flags and proves the source/lowered correspondence;
`watad_checker::choice` is the production recognition and replay engine. The
old `watad_checker::expand::assignments` and `watad_checker::catalog` expansion
functions remain as an independent reference oracle and test-data path, not as
the normal production constructor. The `lugha` catalog supplies meter-specific
data. Generic recipe lowering and reference expansion/remapping have checked
contracts. Source assembly and exact-duplicate elimination remain ordinary
Rust; duplicate elimination preserves requirement-group membership and is
covered by focused poem regressions.

## 1. Inputs and authority

The source authority is a `watad_lugha::meter::Definition`
(`crates/lugha/src/meter.rs:11`): `id: ProfileId` (`family`, `revision`),
`meter: MeterName`, `source: &'static str`, `binding: BindingPolicy`,
`forms: Vec<Form>`, and `requirements: Vec<Vec<usize>>` (poem-wide existence
groups over source form indices).

A `Form` (`crates/checker/src/catalog.rs`, re-exported as
`watad_lugha::meter::Form`) has `constraints: Vec<adjacency::Rule>` with
`Rule { left, left_letter, right, right_letter }`
(`crates/checker/src/adjacency.rs:14`); `feet: usize`, the retained prefix
length of the circle-derived foot sequence `F(M) = MeterName::feet()`;
`context: u16`; `initial_only: bool`; `class: DarbClass`; and
`sadr`/`ajuz: Vec<Vec<Vec<Change>>>`. The outer index names a retained foot, the
next list is a finite ordered list of named recipe alternatives, and each recipe
is a sequence of `Change` (`crates/checker/src/named.rs:109`).

These are ordinary data. IDs are version labels, not authentication.
Meter/circle sequences, contexts, classes, binding, recipe names, constraints
and requirement groups are inputs. The core proves facts relative to them, not
source history.

## 2. Recipe alternatives and compatibility

Let `n = form.feet`, and let `A^s_j` and `A^a_j` be the ordered alternative
lists for sadr and ajuz slot `j` (`0 <= j < n`). A completed half assigns exactly
one index `c_j` with `0 <= c_j < |A_j|` for every retained slot. A half with no
slots (absent ajuz) has exactly one empty assignment `()`.
For recipe `r`, `program::compile(F(M)[j], r)` yields a `Program` or a recipe
error (`meter.rs:44`). Its `actions` are over expanded original letters.
`Rule.left`/`Rule.right` name foot slots; `left_letter`/`right_letter` are
zero-based indices into the corresponding compiled `actions`, exactly as
`adjacency::compatible` indexes them (`adjacency.rs:16`).

A completed half `c` is compatible iff for every declared rule `q`: the slots
`q.left`, `q.right` are in range; the selected compiled programs `P_left`,
`P_right` exist; the letter indices are within their `actions`; and it is not
the case that `P_left.actions[q.left_letter] == Delete` and
`P_right.actions[q.right_letter] == Delete`. Only `Delete` is excluded;
`Keep` and `Silence` are not. Compilation failures are input errors, not
compatible results. Rules are declarations, not a symmetric relation. Each rule
constrains only its named direction; a converse, or a rule with swapped
slots/letters, is a distinct declaration that must be listed and checked
separately. The catalog declares the Kaf/Khabn direction for Madid, Ramal,
Khafif and Mujtathth, and additionally the Khabn/Kaf converse for Mujtathth
(`crates/lugha/src/catalog.rs:29`, `catalog.rs:52`). Adding one direction never
adds the other.

`adjacency::valid` is the all-pairs table check: every alternative in the left
slot must be compatible with every alternative in the right slot
(`adjacency.rs:28`). Expansion may start from a table with conflicting neighbor
alternatives; it emits complete assignments and filters incompatible
combinations. A resulting branch has singleton slot lists, so the later
all-pairs check agrees with the selected-pair check. Production
`structured::compile_profile` instead checks rule-reference bounds and compiles
selected-choice flags; it allows conflicting alternatives to coexist. The
recognizer and certificate checker enforce the exclusions on the chosen path.

## 3. Production compact storage and the reference expander

Production construction no longer calls `expand_forms`. For Madid, Ramal,
Khafif and Mujtathth, `catalog::hashimi` keeps the source forms (12, 12, 10 and
2 rows respectively) and the source requirement groups. Wafir keeps 8 rows after
removing two exact duplicate tasri rows; its total usage catalog is 147 rows.
`MeterProfile::new`
lowers each retained slot to its ordered terminal alternatives and compiles each
declared original-letter rule into `choice::Rule` flags: one boolean per
alternative stating whether that alternative deletes the rule's original letter.
Conflicting alternatives may coexist in the same slot; the selected path, not
the whole table, is checked during recognition and replay.

The reference expander `catalog::reference_expand_forms` is retained for
independent tests and the beat-language fingerprint. It applies the
legacy half expansion only to the same four meters, filters compatible complete
assignments, and remaps requirement groups. It is not called by production
`hashimi`. The expansion rules in the following sections describe this oracle;
the production path is source rows plus compiled flags.

## 4. Ordering and duplicate behavior

For a half, order compatible assignments lexicographically by
`(c_0, ..., c_{n-1})`, comparing at the first differing slot. Source alternative
indices are the comparison keys and are not renumbered. If every assignment is
compatible, the position is the mixed-radix index
`sum_j c_j * product_{k>j} |A_k|`.

Output order is: source forms in original order; within a source form, sadr
assignments outer and ajuz assignments inner; within each half, the lexicographic
order above. Each listed alternative index is a distinct assignment. Repeated
recipes, even with equal compiled surfaces, remain distinct assignments and
produce distinct branches. Expansion never deduplicates recipes or equal
surfaces. (`optional_zihaf` may avoid adding duplicate recipes while constructing
catalog data; that is source-data construction, not expansion.)

## 5. Form identity, compact branches and the reference oracle

Production `Definition.forms` is the ordered source-form list. A certificate
identifies a source form `f` and one selected alternative per retained slot.
There is no expanded branch index and no product over sadr and ajuz assignments
in the production authority. The poem requirements remain groups of source-form
indices, exactly as declared; production does not remap them. Each selected path
is checked against every compiled rule in its half, including alternatives with
equal beat surfaces but different deletion flags.

For the reference oracle, let `C_f^s` and `C_f^a` be the ordered compatible half
assignments of source form `f` and `L_f = |C_f^s| * |C_f^a|`. Reference forms
receive consecutive ranges: `start_0 = 0`,
`start_{f+1} = start_f + L_f`, so form `f` occupies `[start_f, start_f + L_f)`.
Within it, offset `o = r_s * |C_f^a| + r_a` is the `r_s`-th sadr assignment and
`r_a`-th ajuz assignment. Reference branch identity is exactly `(f, r_s, r_a)`.
If `L_f = 0`, the reference range is empty and no branch is emitted for `f`.
Each reference form copies `constraints`, `feet`, `context`, `initial_only` and
`class` unchanged; each slot's alternative list becomes the singleton containing
the selected recipe; an absent ajuz stays empty.

For each reference requirement group `G` (a set of source form indices), the
reference output group is `G' = union over f in G of [start_f, start_f + L_f)`.
An empty `G'` is invalid (`Requirement`). This preserves the poem meaning for the
oracle: at least one selected reference branch must have a source form in `G`.
The verified `catalog::flatten_groups`, `catalog::remap_requirements` and
`catalog::compile_groups` implement this reference mapping. Production source
requirements need no such remapping because form identity is unchanged.

## 6. Profile identity and retained circle-foot prefixes

Given the finalized `Definition`, production lowering preserves `id`, `meter`,
`source`, `binding`, `forms` and `requirements` unchanged. Reference expansion preserves the first four fields
and changes only `forms` and remapped `requirements`. Exact duplicate source rows
are removed only when every semantic field and every requirement-group membership
agree; the first source coordinate survives and requirement indices are remapped.
Wafir's two duplicate tasri rows meet this condition. Madid, Ramal, Wafir, Khafif
and Mujtathth use usage revision 2 because production certificate form
coordinates changed; revision 1 certificates are not valid under the new
authority. The other usage profiles and all circle profiles remain revision 1.

Every production form and every reference output form retains the same `feet`
and base identities: for each retained slot `j`, the compiled units are exactly
`F(M)[j].word_spec()`, the same prefix of the selected circle-derived sequence
(`crates/checker/src/profile.rs:26`). Neither path changes units, substitutes an
equal-surface foot with different ownership, reorders feet, or changes the
circle/rotation/repetition origin. The checked `profile::new_bound` validates the
circle layout and `basis_bound` before sealing (`profile.rs:128`,
`profile.rs:31`).

## 7. Context, class, binding and poem requirements

`context` is opaque at the kernel boundary. The catalog uses `0` ordinary, `1`
tasri, `2` Basit ridf evidence and `3` their combination (`catalog.rs:16`).
Tasri forms are initial-only and give the arud the darb's terminal recipe; Basit
marks conditional cut-darb forms as ridf. These are ordinary catalog constructions; production retains them and the
reference expander copies context and `initial_only` unchanged.

`class: DarbClass` is copied per form. With `BindingPolicy::FixedDarbClass`, the
first selected poem form fixes the class for every verse; `Unconstrained`
applies no filter (`crates/checker/src/obligations.rs:71`). Context matching,
initial-only eligibility and first-verse position remain kernel checks
(`crates/checker/src/verse.rs:147`, `obligations.rs:89`).
For an absent ajuz, the kernel permits an empty ajuz slot sequence but requires
the external ajuz word and certificate ajuz steps to be empty; a nonempty ajuz
input never matches. Production poem groups name source forms directly; the
reference oracle remaps them as in §5. In either authority every output group
must contain at least one form selected somewhere in the poem
(`obligations::satisfied`, `obligations.rs:53`). Recognition backtracks over
complete assignments and does not commit to the first equal-surface reading.

## 8. Invalid inputs and boundary errors

`MeterProfile::new` checks layout and requirement ranges, then calls the checked
constructor (`meter.rs:55`). At the public boundary:

| Condition | Result |
| --- | --- |
| empty/whitespace `source` | `MissingSource` |
| `feet == 0`, `feet > len(F(M))`, `sadr.len() != feet`, or nonempty `ajuz.len() != feet` | `Layout` |
| any named recipe fails `program::compile` | `Recipe(program::Error)` |
| empty slot alternative list; out-of-range adjacency slot/letter reference; zero id family/revision; nonbinary or empty compiled word; missing one-watad structure; circle/basis mismatch; other invalid structured profile | `Profile` |
| a source requirement group is empty or names an index outside `forms` | `Requirement` |
| reference expansion: a group's remapped output union is empty | `Requirement` |
| compact production: a requirement group has no compatible selected path | accepted at construction; `recognize_poem` returns `Ok(None)` |
| `recognize_poem(&[])` | `EmptyPoem` (malformed profile still yields `Profile`) |

Compact production validates requirement indices but deliberately does not
enumerate all paths at construction; that would reintroduce eager expansion. A
structurally valid external profile whose requirement group has no compatible
selected path is therefore accepted by `MeterProfile::new`, and poem recognition
yields no witness. The reference expander rejects the same profile with
`Requirement`. This is a documented compact-boundary tradeoff, not a
construction-time satisfiability claim; the built-in catalog is exhaustively
compared against the reference oracle.

Empty-hemistich behavior follows from the word schema: every compiled
alternative is a nonempty binary word. Empty sadr input never matches. Empty ajuz
input matches only an absent-ajuz form; for a non-single-line form an empty ajuz
is not a valid layout or input. An empty slot alternative list is not "no
change"; it is a `Profile` error.

## 9. Proof boundary and trust

The verified `structured::compile_profile` production path proves, relative to
the source slots and rules: every retained alternative is compiled with its
source index and exact output word; every source rule's referenced slot and
letter exists; every compiled flag sequence equals
`deletion_flags(programs, letter)`; and `projected` implies
`valid(source) == verse::valid(target)`. The `choice` engine then proves exact
selected-path recognition, replay and completeness with those flags. The
reference-only `expand::assignments` separately proves, relative to §§2–4 and
the half-assignment ordering in §5: **soundness** (every emitted tuple is a
total in-range assignment of declared alternatives and satisfies every declared
rule on the actual compiled actions), **completeness** (every compatible total
assignment over the same slots and rules appears exactly once), and **source
order** (lexicographic by source alternative indices, duplicates preserved).

The verified `checker::catalog` reference-expander layer proves the adapter
facts used by the oracle and by the pre-compiler fingerprint:

- `lower_slots` preserves recipe order and count and relates every compiled
  program to `normalize::normalize_spec` on the original foot and recipe.
- `select_recipes` maps every selected source index to exactly the declared
  recipe at that index, without dropping, inventing or reordering alternatives.
- `expand_form_choices` emits the exact Cartesian product of the sadr and ajuz
  assignments at position `s * |ajuz choices| + a`; `expanded_form` guarantees
  that every emitted form copies `constraints`, `feet`, `context`,
  `initial_only` and `class`, and that an absent ajuz stays absent.
- `flatten_groups` assigns each source form its exact contiguous output range,
  proves `out.len() == total_forms(groups)` (including zero for an empty source
  list), preserves branch order, and makes later ranges start where earlier ones
  end. Empty inner groups get zero-length ranges.
- `remap_requirements` makes each output requirement group exactly the increasing
  filter of all output indices covered by its source ranges. A proved validity
  predicate requires nonempty groups, in-range source indices, and at least one
  covered output index; valid groups succeed and invalid groups return
  `Error::Requirement`. Successful groups are proved nonempty, strictly sorted,
  duplicate-free and exactly the filtered union.
- `compile_groups` composes range construction and requirement remapping, proves
  the exact range/union relation and both Result branches, and succeeds with
  empty forms, ranges and requirements on `([], [])`.
- `Form` cloning used by `flatten_groups` preserves all metadata, constraints and
  recipe structure.

Theorem inputs are the finite ordered alternative tables and declared rules; the
compiled `Program` for each `(slot, alternative)` is supplied by
`normalize::compile` before expansion. Existing checked pieces reused:
`adjacency::check <=> adjacency::valid` (`adjacency.rs`) for the reference
oracle; `normalize::compile` against `normalize_spec` and
`normalize_spec_valid` (`crates/checker/src/normalize.rs`);
`structured::compile_profile` validity characterization and `projected`
(`crates/checker/src/structured.rs`); `structured::projection_equivalent` and
the compiled-rule/flag helpers; `choice::trace`/`derives`/`recognize`; and the
sealed `profile::new_bound` invariant (`profile.rs`).

What remains ordinary Rust: catalog tables and licence decisions
(`catalog::hashimi`, `body`, `usage_licences`, `branch`, `optional_zihaf`);
recipe vocabulary and choice of base; grouping source forms; choosing revision
labels; calling `MeterProfile::new` and the reference-expander test path;
error mapping; `Definition`/`MeterProfile` construction and public APIs; CLI and
display; and meter/circle name tables. The adapter chooses which declared data
to pass to the checked functions, but rule lowering, selected-path enforcement,
reference range/union calculations and metadata preservation are proved.
Historical permission and source completeness remain outside the proof.

The pinned boundary: workspace Rust is 1.92.0, while Verus uses its separate
pinned Rust 1.98.1 toolchain and checks the same crate roots; source outside
`verus!` remains ordinary Rust even when its crate is passed to Verus
(`formal/README.md`). Only the expansion and catalog-compilation theorems, not
their callers or tests, belong to the proved boundary.

```text
PROVED RELATIVE TO INPUTS (Verus, pinned toolchain)
  inputs: ordered source slots/programs; source adjacency rules; source forms;
          source requirement groups
  COMPACT_LOWERING_THEOREM: exact source program/word preservation; exact
          deletion-flag compilation; source/lowered validity equivalence;
          source form/requirement identity preserved
  SELECTED_CHOICE_THEOREM: exact recognition, replay, completeness and
          equal-surface program identity under selected flags
  REFERENCE_EXPANSION_THEOREM: soundness + completeness of compatible total
          assignments; lexicographic source order; duplicate indices preserved
  REFERENCE_CATALOG_THEOREM: source-index recipe selection; exact sadr x ajuz
          product; contiguous ranges; exact requirement union/filter membership
  reused: adjacency::check/valid (reference); normalize::compile ==
          normalize_spec; normalize_spec_valid; structured::compile_profile/
          projected; projection_equivalent; choice trace/derives/recognize;
          profile::new_bound invariant; obligations derives_from/trace/satisfied
  NOT proved: historical permission, source interpretation, or catalog
          completeness against Arabic usage

ORDINARY RUST ADAPTERS (outside Verus)
  catalog tables/licences: hashimi, body, usage_licences, branch,
      optional_zihaf
  grouping source forms and calling lower_slots / compile_profile;
      reference-expander calls to assignments / expand_form_choices /
      compile_groups
  Definition/MeterProfile construction, error mapping, public API, CLI
  meter/circle name tables, metadata display
```

Mechanical compatibility is not historical permission. The relation says only
that a combination is allowed by the declared recipes and exclusions. Catalog
correctness relative to those declarations does not prove historical
completeness; unknown readings remain unknown
(`spec/profiles/classical-v1.md:93`). Independent tests support integration but
are not proofs.

## 10. Observables and enforcement

| Observable | Enforcement |
| --- | --- |
| All 22 circle and 16 usage profiles construct circle-bound and replay checked witnesses | independent test `crates/lugha/tests/catalog.rs:20`; construction relies on verified `profile::new_bound` |
| Correlated production profiles stay at source scale (Madid 12, Ramal 12, Wafir 8, Khafif 10, Mujtath 2; 147 total rows) and carry compiled rule flags | independent tests `crates/lugha/tests/compact_profiles.rs`; witness-membership dedup tests `crates/lugha/src/catalog.rs`; proof `structured::compiled_rules`/`projected` |
| Compact and reference-expanded languages agree exhaustively, including invalid source combinations | independent exhaustive catalog comparison and generated source-rule proptest `crates/lugha/tests/compact_profiles.rs` |
| Revision-2 certificate coordinates reject revision-1 certificates for the five changed usage profiles | independent test `crates/lugha/tests/compact_profiles.rs`; certificate authority guard `verse::check` |
| Declared catalog rule references are rechecked; conflicting unselected alternatives are allowed; out-of-range rules reject; Mujtathth preserves both Kaf/Khabn directions and arud endpoint scope | proof `adjacency::rule_scope_ok` and `structured::compile_rule`; reference all-pairs proof `adjacency::check <=> adjacency::valid`; independent tests `crates/lugha/tests/adjacency.rs`, `crates/lugha/tests/compact_profiles.rs` |
| Independent canonical words for all 16 meters; obligatory changes and single-line layouts are not guessed | independent tests `crates/lugha/tests/source_examples.rs:19`, `:74`; not proof of source completeness |
| Restricted Hazaj accepts 36 of 625 tuples; forged identity/context/form/steps reject | independent test `crates/lugha/tests/verus_conformance.rs:83`, `:128`; kernel legality is proved by `verse::legal` |
| Poem requirements backtrack across equal surfaces; missing witness, empty group and invalid index fail; Wafir needs an unaltered shortened arud somewhere; identical rows in different witness groups cannot merge | proof `obligations::derives_from`, `trace`, `satisfied`; independent tests `crates/lugha/tests/obligations.rs:24`, `:102`, `catalog.rs:71`; dedup regressions `crates/lugha/src/catalog.rs` |
| Original recipes normalize exactly as `normalize_spec` and are mechanically valid when eligible | proof `normalize::compile` and `normalize_spec_valid` |
| Reference half expansion soundness, completeness, uniqueness and source order | proof `expand::assignments`/`enumerate` (`crates/checker/src/expand.rs`); negative controls `expand-completeness`, `expand-adjacency`, `expand-skip-first`, `expand-order-reversed` |
| Independent expansion oracle, repeated equal alternatives, empty half, rule direction and out-of-range behavior | independent proptest and deterministic tests `crates/lugha/tests/expand.rs` |
| Recipe selection, exact sadr x ajuz product, metadata/class/context/feet preservation, absent ajuz and duplicate source indices | proof `catalog::select_recipes`, `catalog::expand_form_choices`, `catalog::expanded_form`; negative controls `catalog-sadr-alternative`, `catalog-ajuz-alternative`, `catalog-invented-recipe`, `catalog-source-index`, `catalog-context`, `catalog-class`; independent test `crates/lugha/tests/catalog_compilation.rs` |
| Reference-expanded output order, duplicates, metadata/identity/prefix preservation against the recorded reference baseline | independent fingerprint regression `crates/lugha/tests/baseline_fingerprint.rs`, using the hidden reference expander |
| Exact source-form ranges, empty source-list coverage, and exact requirement union/filter membership | proof `catalog::flatten_groups`, `catalog::remap_requirements`, `catalog::compile_groups`; negative controls `catalog-requirement-index`, `catalog-empty-invented-form`; independent deterministic/property tests `catalog_compilation.rs` |
| A requirement group that is empty, names an out-of-range source index, or has an empty total covered union rejects; a zero-length selected range is allowed when another selected range contributes output | proof `catalog::remap_requirements`, `catalog::compile_groups`; negative control `catalog-requirement-empty-output`; independent tests `catalog_compilation.rs`; public profile error tests `verus_conformance.rs` |
| Layout/MissingSource and EmptyPoem errors | independent tests `verus_conformance.rs:232`, `:227` |
