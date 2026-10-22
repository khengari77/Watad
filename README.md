<div align="center">

# وتد · Watad

**Arabic prosody, modeled through structure.**

Rust libraries and a command-line tool for the circles, meters, and transformations of علم العروض.

[Quick start](#quick-start) · [Rust API](docs/API.md) · [CLI](docs/CLI.md) · [Verification](formal/README.md)

</div>

Watad models سبب and وتد explicitly, derives meters from al-Khalīl's five
circles, and recognizes verse against versioned profiles. Each match includes
a certificate that can be checked against the original input.

Two feet can have the same rhythm and different structure. مستفعلن and
مستفعِ لن share a beat pattern but place their وتد differently. Watad preserves
that distinction when applying زحاف and عِلّة.

## Features

- Five circles, 22 rotations, and usage profiles for the sixteen used meters.
- Structural transformations with original-letter coordinates and ownership checks.
- Exact beat recognition and replayable span certificates.
- Whole-poem checks for meter, ضرب class, context, and existence requirements.
- A Rust mathematical core whose annotated executable source is checked by Verus.

Input is **explicit prosodic beats**: `/` or `1` for متحرك, and `0` for ساكن.
Ordinary Arabic spelling → الكتابة العروضية conversion is outside the current API.

## Quick start

Install the Rust toolchain in [rust-toolchain.toml](rust-toolchain.toml), then
run these commands from the repository:

```sh
cargo run -p watad-cli -- meters
cargo run -p watad-cli -- example الهزج
cargo run -p watad-cli -- scan '//0/0/0 //0/0/0' '//0/0/0 //0/0/0' --meter الهزج
```

For a poem, save one verse per line in `poem.beats`:

```text
//0/0/0 //0/0/0 | //0/0/0 //0/0/0
//0/0/0 //0/0/0 | //0/0/0 //0/0/0
```

```sh
cargo run -p watad-cli -- poem poem.beats --meter الهزج
```

Use `--circle` to select unchanged circle forms, including unused rotations.
Usage profiles and circle profiles are distinct authorities.

## Libraries

| Crate | Purpose |
| --- | --- |
| `watad-types` | Beats, سبب/وتد units, and structural foot identities |
| `watad-daira` | Circles, rotations, meters, and retained layouts |
| `watad-tahwil` | Mechanical زحاف and عِلّة operations |
| `watad-lugha` | Source profiles, named recipes, and `MeterProfile` |
| `watad-checker` | Verified structural, recognition, and certificate contracts |
| `watad-matcher` | Beat parsing, verse recognition, and explanations |
| `watad-qasida` | Whole-poem recognition and certificate replay |

Runnable examples and generated symbol documentation:

```sh
cargo run -p watad-matcher --example scan
cargo run -p watad-qasida --example poem
cargo doc --workspace --no-deps --open
```

See the [API guide](docs/API.md) for profile selection, error semantics,
contexts, certificates, and custom definitions. Workspace version `0.0.0`
does not promise stable APIs or a stable certificate serialization format.

## Guarantees and scope

Recognition exhausts the selected finite profile. A negative result means no
derivation exists in that declared language. Certificates are checked against
externally selected authority. Poem recognition searches complete assignments,
including ambiguous verses and required witnesses.

Historical source interpretation and coverage of competing schools are separate
from mathematical correctness for declared rules. Contexts such as تصريع and
ردف are supplied by the caller; beat patterns do not establish rhyme evidence.
Allocation failure and process resource limits remain environmental limits.

- [Profile coverage and source conventions](spec/profiles/classical-v1.md)
- [Types and structural identity](docs/TYPES.md)
- [Proof scope and reproduction](formal/README.md)

## Build and check

```sh
cargo fmt --all --check
PROPTEST_CASES=512 cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
```

## License

[MIT](LICENSE).
