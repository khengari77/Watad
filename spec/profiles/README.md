# Meter profiles

The public profile API is `watad_lugha::meter::MeterProfile`, with built-in
source data in `watad_lugha::catalog`.

- [Classical usage, revision 1](classical-v1.md): supported layouts, terminal
  pairs, transformations, contexts and unresolved readings.
- [Restricted Hazaj fixture](hashimi-hazaj-v1.md): an independently specified
  finite test language for the generic profile engine.
- [Rust API](../../docs/API.md): construction, recognition, replay and errors.
- [Catalog compilation](../catalog-compilation.md): recipe expansion and
  requirement-preservation contracts.

Profiles are externally supplied authority. IDs are version labels, not hashes
or authentication. Circle-bound construction checks retained original feet,
programs, exclusions and metadata before sealing the compiled representation.
Equal beat surfaces do not replace different ownership.

Exactness is relative to the selected profile. Historical coverage and source
interpretation remain separate from the mathematical language relation. Caller
contexts such as تصريع and ردف are not inferred from beats.

Independent tests exercise literal source patterns, original-coordinate edits,
finite-language oracles, complete poem assignments, rotations, structural
identity and malformed certificates. Property-test counterexamples use explicit
regression paths and can be committed with focused regression tests.

```sh
PROPTEST_CASES=512 cargo test --workspace --locked
```

See [verification](../../formal/README.md) for the annotated proof boundary.
