# Catalog compilation specification

The contracts in §§2–5 are enforced by Verus-checked
`watad_checker::expand::assignments` and `watad_checker::catalog`
(`crates/checker/src/expand.rs`, `crates/checker/src/catalog.rs`). The `lugha`
catalog supplies meter-specific data and calls those checked functions; it does
not compute recipe selection, form ranges or requirement unions itself.

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
all-pairs check agrees with the selected-pair check. Expansion does not weaken
or repair a table supplied directly to `structured::compile_profile`.

## 3. Scope of the catalog expansion pass

The current catalog applies half expansion only to forms of Madid, Ramal, Khafif
and Mujtathth (`catalog::expand_forms`). Mujtathth also supplies the reciprocal
direction. This is the declared scope of that pass, not a claim that no other
meter could use the generic relation. Expansion is per half and independent: the
sadr assignment is drawn from the sadr slot lists, the ajuz assignment from the
ajuz slot lists, and the full branch is the pair `(sadr assignment, ajuz
assignment)`. If ajuz is absent (`form.ajuz == []`, as for the explicit
single-line branches), there are no ajuz slots and exactly one empty ajuz
assignment. An absent ajuz is never replaced by a duplicate of sadr, a
fabricated hemistich, or a product over a missing half.

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

## 5. Form expansion, branch identity and requirements

For source form `f`, let `C_f^s` and `C_f^a` be its ordered compatible half
assignments and `L_f = |C_f^s| * |C_f^a|`. Source forms receive consecutive
ranges: `start_0 = 0`, `start_{f+1} = start_f + L_f`, so form `f` occupies
`[start_f, start_f + L_f)`. Within it, offset
`o = r_s * |C_f^a| + r_a` is the `r_s`-th sadr assignment and `r_a`-th ajuz
assignment. Output branch identity is exactly `(f, r_s, r_a)`; the mapping is
`(f, r_s, r_a) -> start_f + r_s * |C_f^a| + r_a`, inverted from the containing
range and division by `|C_f^a|`. If `L_f = 0`, the range is empty and no branch
is emitted for `f`.

Each emitted form copies `constraints`, `feet`, `context`, `initial_only` and
`class` unchanged. Its sadr/ajuz slots keep the same length and retained bases;
each slot's alternative list becomes the singleton containing the selected
recipe. An absent ajuz stays empty.

For each source requirement group `G` (a set of source form indices), the output
group is `G' = union over f in G of [start_f, start_f + L_f)`. An empty `G'` is
invalid (`Requirement`). This preserves the poem meaning: at least one selected
output branch must have a source form in `G`. Groups are remapped by ranges, not
by assuming source numeric indices remain valid. The verified
`catalog::flatten_groups` records each source form's contiguous output range and
proves the exact functional ranges: `ranges[g] = (prefix_len(g), |group_g|)`.
Thus an empty source list yields an empty output and empty ranges, while an empty
inner group in a nonempty list receives a zero-length range at the current
output position. `catalog::remap_requirements` computes the union as the
increasing filter of all covered output indices (hence sorted, duplicate-free and
range-checked). Its contract is exact: all source indices must be in range, each
requirement group must be nonempty, and the selected ranges must cover at least
one output index; valid groups succeed, and every invalid group returns
`Error::Requirement`. On success each remapped group is nonempty, strictly
increasing and exactly the filtered union. `catalog::compile_groups` composes
both steps, exposes the same guarantees, succeeds on `([], [])`, and its
`Err(Error::Requirement)` result is constrained by the same validity predicate.
`expand_forms` calls `compile_groups`; it no longer computes ranges or sorts
requirement indices in ordinary Rust.

## 6. Profile identity and retained circle-foot prefixes

The output `Definition` preserves `id`, `meter`, `source` and `binding`
unchanged; only `forms` and remapped `requirements` differ.
Every output form retains the same `feet` and base identities: for each retained
slot `j`, the compiled units are exactly `F(M)[j].word_spec()`, the same prefix
of the selected circle-derived sequence (`crates/checker/src/profile.rs:26`).
Expansion never changes units, substitutes an equal-surface foot with different
ownership, reorders feet, or changes the circle/rotation/repetition origin. The
checked `profile::new_bound` validates the circle layout and `basis_bound`
before sealing (`profile.rs:128`, `profile.rs:31`); expansion must preserve those
premises.

## 7. Context, class, binding and poem requirements

`context` is opaque at the kernel boundary. The catalog uses `0` ordinary, `1`
tasri, `2` Basit ridf evidence and `3` their combination (`catalog.rs:16`).
Tasri forms are initial-only and give the arud the darb's terminal recipe; Basit
marks conditional cut-darb forms as ridf. These are ordinary catalog
constructions before expansion; expansion copies context and `initial_only`
unchanged.

`class: DarbClass` is copied per form. With `BindingPolicy::FixedDarbClass`, the
first selected poem form fixes the class for every verse; `Unconstrained`
applies no filter (`crates/checker/src/obligations.rs:71`). Context matching,
initial-only eligibility and first-verse position remain kernel checks
(`crates/checker/src/verse.rs:147`, `obligations.rs:89`).
For an absent ajuz, the kernel permits an empty ajuz slot sequence but requires
the external ajuz word and certificate ajuz steps to be empty; a nonempty ajuz
input never matches. Poem groups are remapped as in §5, and every output group
must contain at least one form selected somewhere in the poem
(`obligations::satisfied`, `obligations.rs:53`). Recognition backtracks over
complete assignments and does not commit to the first equal-surface reading.

## 8. Invalid inputs and boundary errors

`MeterProfile::new` checks layout and requirement ranges, then calls the checked
constructor (`meter.rs:55`). At the public boundary:

| Condition | Error |
| --- | --- |
| empty/whitespace `source` | `MissingSource` |
| `feet == 0`, `feet > len(F(M))`, `sadr.len() != feet`, or nonempty `ajuz.len() != feet` | `Layout` |
| any named recipe fails `program::compile` | `Recipe(program::Error)` |
| empty slot alternative list; out-of-range adjacency slot/letter; exclusion violated by the all-pairs table; zero id family/revision; nonbinary or empty compiled word; missing one-watad structure; circle/basis mismatch; other invalid structured profile | `Profile` |
| a source requirement group is empty or names an index outside `forms`, or its remapped output union is empty | `Requirement` |
| `recognize_poem(&[])` | `EmptyPoem` (malformed profile still yields `Profile`) |

Empty-hemistich behavior follows from the word schema: every compiled
alternative is a nonempty binary word. Empty sadr input never matches. Empty ajuz
input matches only an absent-ajuz form; for a non-single-line form an empty ajuz
is not a valid layout or input. An empty slot alternative list is not "no
change"; it is a `Profile` error.

## 9. Proof boundary and trust

The verified `expand::assignments` proves, relative to §§2–4 and the
half-assignment ordering in §5: **soundness** (every emitted tuple is a total
in-range assignment of declared alternatives and satisfies every declared rule
on the actual compiled actions), **completeness** (every compatible total
assignment over the same slots and rules appears exactly once), and **source
order** (lexicographic by source alternative indices, duplicates preserved).

The verified `checker::catalog` layer proves the adapter facts:

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
`adjacency::check <=> adjacency::valid` (`adjacency.rs:72`);
`normalize::compile` against `normalize_spec` and `normalize_spec_valid`
(`crates/checker/src/normalize.rs:1518`, `normalize.rs:1061`);
`structured::compile_profile` validity characterization and `projected`
(`crates/checker/src/structured.rs:162`, `structured.rs:191`);
`structured::projection_equivalent` (`structured.rs:175`); and the sealed
`profile::new_bound` invariant (`profile.rs:99`).

What remains ordinary Rust: catalog tables and licence decisions
(`catalog::hashimi`, `body`, `usage_licences`, `branch`, `optional_zihaf`);
recipe vocabulary and choice of base; grouping source forms into per-form branch
vectors; calls to `lower_slots`, `assignments`, `expand_form_choices` and
`compile_groups`; error mapping; `Definition`/`MeterProfile` construction and
public APIs; CLI and display; and meter/circle name tables. The adapter chooses
which declared data to pass to the checked functions, but the selection, range,
metadata and requirement calculations are proved. Historical permission and
source completeness remain outside the proof.

The pinned boundary: workspace Rust is 1.92.0, while Verus uses its separate
pinned Rust 1.98.1 toolchain and checks the same crate roots; source outside
`verus!` remains ordinary Rust even when its crate is passed to Verus
(`formal/README.md`). Only the expansion and catalog-compilation theorems, not
their callers or tests, belong to the proved boundary.

```text
PROVED RELATIVE TO INPUTS (Verus, pinned toolchain)
  inputs: ordered compiled program tables per slot; adjacency rules;
          source recipe groups; source requirement groups
  EXPANSION_THEOREM: soundness + completeness of compatible total
          assignments; lexicographic source order; duplicate indices
          preserved
  CATALOG_THEOREM: source-index recipe selection; exact sadr x ajuz product;
          metadata/authority copying; contiguous source-form ranges;
          exact requirement union/filter membership; absent ajuz preserved
  reused: adjacency::check/valid; normalize::compile == normalize_spec;
          normalize_spec_valid; structured::compile_profile/projected;
          projection_equivalent; profile::new_bound invariant;
          obligations derives_from/trace/satisfied
  NOT proved: historical permission, source interpretation, or catalog
          completeness against Arabic usage

ORDINARY RUST ADAPTERS (outside Verus)
  catalog tables/licences: hashimi, body, usage_licences, branch,
      optional_zihaf
  grouping source forms and calling lower_slots / assignments /
      expand_form_choices / compile_groups
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
| Declared catalog adjacency constraints are rechecked; conflicting pairs and out-of-range rules reject; Mujtathth preserves both Kaf/Khabn directions and arud endpoint scope | proof `adjacency::check <=> adjacency::valid` (`adjacency.rs:28`, `:72`); independent tests `crates/lugha/tests/adjacency.rs:23`, `:60`, `:72` |
| Independent canonical words for all 16 meters; obligatory changes and single-line layouts are not guessed | independent tests `crates/lugha/tests/source_examples.rs:19`, `:74`; not proof of source completeness |
| Restricted Hazaj accepts 36 of 625 tuples; forged identity/context/form/steps reject | independent test `crates/lugha/tests/verus_conformance.rs:83`, `:128`; kernel legality is proved by `verse::legal` |
| Poem requirements backtrack across equal surfaces; missing witness, empty group and invalid index fail; Wafir needs an unaltered shortened arud somewhere | proof `obligations::derives_from`, `trace`, `satisfied`; independent tests `crates/lugha/tests/obligations.rs:24`, `:102`, `catalog.rs:71` |
| Original recipes normalize exactly as `normalize_spec` and are mechanically valid when eligible | proof `normalize::compile` and `normalize_spec_valid` |
| Half expansion soundness, completeness, uniqueness and source order | proof `expand::assignments`/`enumerate` (`crates/checker/src/expand.rs`); negative controls `expand-completeness`, `expand-adjacency`, `expand-skip-first`, `expand-order-reversed` |
| Independent expansion oracle, repeated equal alternatives, empty half, rule direction and out-of-range behavior | independent proptest and deterministic tests `crates/lugha/tests/expand.rs` |
| Recipe selection, exact sadr x ajuz product, metadata/class/context/feet preservation, absent ajuz and duplicate source indices | proof `catalog::select_recipes`, `catalog::expand_form_choices`, `catalog::expanded_form`; negative controls `catalog-sadr-alternative`, `catalog-ajuz-alternative`, `catalog-invented-recipe`, `catalog-source-index`, `catalog-context`, `catalog-class`; independent test `crates/lugha/tests/catalog_compilation.rs` |
| Catalog output order, duplicates, metadata/identity/prefix preservation against the declared catalog fingerprint | independent fingerprint regression `crates/lugha/tests/baseline_fingerprint.rs` |
| Exact source-form ranges, empty source-list coverage, and exact requirement union/filter membership | proof `catalog::flatten_groups`, `catalog::remap_requirements`, `catalog::compile_groups`; negative controls `catalog-requirement-index`, `catalog-empty-invented-form`; independent deterministic/property tests `catalog_compilation.rs` |
| A requirement group that is empty, names an out-of-range source index, or has an empty total covered union rejects; a zero-length selected range is allowed when another selected range contributes output | proof `catalog::remap_requirements`, `catalog::compile_groups`; negative control `catalog-requirement-empty-output`; independent tests `catalog_compilation.rs`; public profile error tests `verus_conformance.rs` |
| Layout/MissingSource and EmptyPoem errors | independent tests `verus_conformance.rs:232`, `:227` |
