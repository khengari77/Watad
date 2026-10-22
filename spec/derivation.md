# Form-bound structural derivation contract

Implemented in `watad_lugha::derivation`. This is an executable structural
boundary for one foot, not the formal certificate checker or an attestation
profile. It introduces no new linguistic permissions beyond the mechanics and
position-scope decisions already specified in [layer2.md](transformations.md).

## Inputs and scope

`derive_structural_foot(form, terminal, foot_index, operation)` takes a generated
`MeterForm`, an arud/darb terminal role, a zero-based index, and either no
operation or one existing `Op`. A compound zihaf is one simultaneous operation;
its components refer to the original foot. There is no list of sequential edits.

The function derives the base identity and `PositionContext` from the actual
form. It does not accept caller-supplied base feet, position flags, or
`LetterOrigin` records. Experimental forms remain structurally constructible;
success does not establish classical attestation.

The rules are:

1. An index outside `0..foot_count` fails, including `usize::MAX`.
2. No operation returns exactly the original foot's expanded beats.
3. A simple or compound zihaf must pass the existing ownership-checked API:
   the targets belong to sababs, exist, and meet their letter-state preconditions.
4. An illa must have decided position scope. Unknown scope is an explicit error,
   including at terminal positions; forbidden scope is a position error.
5. The terminal role is applied only at the actual terminal index. A one-foot
   form retains both its initial fact and its terminal role. Caller choice of
   arud/darb cannot promote an interior foot to a terminal position.
6. An illa with allowed position scope must still pass its structural mechanics.
   Range and state failures remain distinguishable through the error types.

`StructuralDerivation` has private fields and is constructed only after these
checks. Its read-only trace and positional view expose what was computed. This
is ordinary Rust invariant enforcement, with tests, not a theorem in a proof
assistant.

## Trace and replay

`FootTrace` has public fields and is untrusted input. It records:

- The claimed generated form and hemistich terminal role.
- The original foot index and structural `FootId`.
- The optional operation against that original foot.
- The complete claimed output beat sequence for this foot.

`check_structural_trace(expected_form, expected_terminal, trace)` checks context
identity, recomputes the original foot, replays the structural derivation, and
compares the entire output. Equal beat patterns do not allow substituting a
different base identity. Missing, extra, or altered output beats fail.

Form comparison uses the active segment sequence and terminal-shape label.
Unused private array padding is not part of that identity. Segment boundaries
are retained: different segment decompositions are not automatically equivalent,
even if they happen to expand to the same word. A terminal-shape label is checked
as metadata; the label does not itself alter generated feet.

Replay is relative to externally selected context. A candidate trace cannot
authorize itself by changing its own form or terminal role. Conversely, changing
the expected context intentionally asks a different structural question; no
authenticated profile or historical provenance is implied by the trace.

## Error distinctions

| Failure | Error |
| --- | --- |
| Claimed form differs from expected active segments or shape label | `FormMismatch` |
| Claimed hemistich end differs from the expected end | `TerminalMismatch` |
| Requested index does not exist | `FootOutOfRange` |
| Claimed base differs from the form's base | `BaseMismatch` |
| Zihaf targets a watad, a missing letter, or a wrong-state letter | Wrapped `ZihafError` |
| Illa position scope is undecided | `UnknownIllaScope` |
| Illa position scope is forbidden | `IllaPosition` |
| Illa structural precondition fails | Wrapped `IllaError` |
| Replayed beats differ from the claimed output | `OutputMismatch` |

Context and base claims are checked before replay. Within replay, the existing
structural routines determine the first reported failure; errors do not enumerate
all problems in an invalid request.

## Relationship to profile recognition

Structural replay does not inspect `ATTESTED` as an exhaustive permission matrix.
A structurally successful foot can still be absent from a selected meter profile.
Whole-verse permissions, contexts, terminal pairing, span coverage and poem
requirements are checked by `watad_lugha::meter::MeterProfile` and its
certificate path. See [the Rust API](../docs/API.md) and
[verification](../formal/README.md) for that boundary.
