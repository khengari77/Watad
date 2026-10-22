# Formal verification

Watad uses Rust and Verus to check the annotated production core. Specifications
and executable functions share source files under `crates/`; `formal/` contains
the verification driver and pinned toolchain metadata.

## Toolchains

- Workspace: Rust 1.92.0, as specified in `rust-toolchain.toml`.
- Verifier: `0.2026.10.04.426d8b0`, commit
  `426d8b01e7ffb910a36c15e1f870ff985bb61eef`.
- Verifier Rust toolchain: 1.98.1.

The release URL and SHA-256 digest are in
[verus/toolchain.json](verus/toolchain.json). The
[CI workflow](../.github/workflows/ci.yml) installs and verifies that release.
Use the [official installation instructions](https://github.com/verus-lang/verus/blob/release/0.2026.10.04.426d8b0/INSTALL.md).
The driver checks the version, commit and compiler pin before verification.

## Run

Install the pinned verifier and its Rust toolchain, and fetch workspace
packages with `cargo fetch --locked`. Then run:

```sh
python3 -u formal/check-verus.py --verus /absolute/path/to/verus --negative-control
```

If the verifier's compiler is in an isolated rustup installation, add
`--rustup-home /absolute/path/to/rustup`. That setting applies to Verus;
Cargo keeps the workspace toolchain. The driver installs nothing, uses
`--locked --offline` for Cargo integration, and checks proof mutations in
isolated temporary copies. `CARGO_TARGET_DIR` can place build outputs outside
this repository.

## Checked contracts

| Component | Contract |
| --- | --- |
| Primitive words and rotations | Original words, one-watad named feet, primitive periods, disjoint circle orbits, bounded repetition and cyclic rotation |
| Layout and immutable profiles | Circle tiling, retained original-base binding and source/lowered correspondence |
| Morphology and named normalization | Original-coordinate edits, ownership protection, exact normalization result and validity under mechanical eligibility |
| Adjacency and expansion | Original-letter exclusions; sound, complete, ordered enumeration of compatible alternatives |
| Catalog compilation | Recipe selection, exact half products, metadata copying, contiguous ranges and exact requirement remapping |
| Recognition | Exact finite derivation, complete input coverage and checked span witnesses |
| Verse and poem checking | Profile/context authority, initial-only forms, shared classes and poem-wide existence witnesses |

Successful recognition supplies a certificate satisfying the declared relation.
A complete negative exhausts the finite declared search. Invalid profiles and
empty poems are errors at the public profile boundary. There is no heuristic
cutoff reported as no-match.

## Mutation controls

`--negative-control` changes guards or outputs in temporary copies and requires
proof-obligation rejection. Controls cover authority, alphabet/input bounds,
ownership, adjacency, compiler completeness, circle/base provenance,
normalization coordinates/preconditions/suffixes, alternative ordering and
coverage, metadata copying, requirement validity and empty-source output.
Compilation failure, timeout or unrelated diagnostics do not count as the
intended proof rejection. The driver prints results and diagnostics.

## Trust boundary

The checker proves properties relative to declared profiles. Circle seeds,
names, source permissions and historical interpretations are domain facts.
Meter-specific catalog assembly, public adapters and display include ordinary
Rust outside annotated proof bodies; tests support those integration boundaries.
The meter-tiling generator is untrusted, with its outputs independently checked
by the circle-bound layout constructor.

Mechanical eligibility is not source permission. Normalization validity requires
the declared positional/suffix eligibility premise; a mechanically valid change
is not thereby licensed in a historical meter. External contexts such as تصريع
and ردف remain caller claims.

Verus, SMT solving, the relevant preverified library specifications, the official
proof-erasure macro and Rust/LLVM are trusted tools. Project verification runs
with `--no-cheating`; project `assume`, `admit`, `external_body` and
`assume_specification` are rejected. Allocation failure, stack/memory limits and
the operating environment remain outside mathematical termination contracts.
Exact recognition can require exponential search.

Source files outside `verus!` are ordinary Rust even when their crate root is
passed to Verus. See [the API](../docs/API.md) and
[source coverage](../spec/profiles/classical-v1.md) for user-visible premises.
