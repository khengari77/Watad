# Command-line reference

Use `cargo run -p watad-cli -- COMMAND` from the repository. The binary is named
`watad`. Meter selectors accept an Arabic name or an ID from `watad meters`
(0–21, in circle order). `/` and `1` mean متحرك; `0` means ساكن.

## Meters and examples

```sh
watad meters
watad example الهزج
watad example الهزج 1
watad example المستطيل --circle
```

`meters` prints ID, name, status and usage-branch count. This counts compact
source forms, including contexts and binding classes, rather than every
possible rhythm. Foot alternatives and correlation rules remain within forms.
The optional example
form index defaults to zero. An example prints `SADR | AJUZ` using the selected
form's first alternatives; required context is reported separately. Generated
examples illustrate declared profiles and are not quotations from poetry.

## Verse recognition

```text
watad scan SADR AJUZ [--meter METER] [--circle] [--initial] [--context N]
```

```sh
watad scan '//0/0/0 //0/0/0' '//0/0/0 //0/0/0' --meter الهزج
```

Quote spaces. Without `--meter`, scans all usage profiles. `--circle` selects
unchanged generated forms, including unused rotations. Unused rotations require
that option. For a declared single-line form, pass `''` as AJUZ.

Matches show meter, profile family/revision, branch, class, original feet,
changes and half-open beat spans. Certificates replay before display.

## Poem recognition

```text
watad poem FILE [--meter METER] [--circle] [--context N]
```

Each nonblank file line is `[CONTEXT:] SADR | AJUZ`:

```text
//0/0/0 //0/0/0 | //0/0/0 //0/0/0
0: //0/0/0 //0/0/0 | //0/0/0 //0/0/0
```

A context prefix overrides `--context` (default zero). Blank lines are ignored.
Initial position comes from the first nonblank verse; `--initial` does not
position verses in a poem. A match satisfies shared-class and existence
requirements across the complete assignment. Every certificate replays before
the verses are displayed.

## Contexts

Built-in usage profiles define `0` ordinary, `1` initial تصريع, `2` supplied
ردف evidence for Basit's conditional cut darb, and `3` both. The caller supplies
these facts. Beat input does not establish rhyme letters or تصريع.

## Exit status

| Code | Meaning |
| --- | --- |
| `0` | Successful match/check or informational command |
| `1` | No match in the selected profiles |
| `2` | Argument, beat-syntax, file or profile error |

Ordinary Arabic spelling is not accepted as beat syntax. No-match is relative
to the selected source convention.
