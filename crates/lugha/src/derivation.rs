//! Form-bound structural derivations, not a classical-meter legality oracle.
//!
//! The base foot and position are derived from a form and an index. Simple and
//! compound zihaf use the ownership-checked mechanics; illa also pass the
//! position-scope gate. Unknown scope stays an error.
//!
//! A trace records one operation (or none) against the original foot. A compound
//! is one simultaneous operation, not a sequence with shifting coordinates.
//! Profile permissions, terminal pairing, and poem constraints remain outside
//! this boundary. Contract: `spec/derivation.md`.

use watad_daira::{MeterForm, PositionContext, TerminalRole};
use watad_tahwil::{IllaScope, apply_compound_checked, apply_zihaf_checked, illa_scope};
use watad_types::{Beat, Foot, FootId, Role};

use crate::{Illa, IllaError, Op, ZihafError, apply_illa, beats_of};

/// Untrusted, replayable claims about a single original foot.
///
/// Public fields allow callers to construct or deserialize candidate traces.
/// Use [`check_structural_trace`] with the expected form and hemistich end; the
/// claimed base, context, and output do not authorize themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootTrace {
    /// Claimed generated form, including its terminal-shape label.
    pub form: MeterForm,
    /// Whether the hemistich ends in arud or darb.
    pub terminal: TerminalRole,
    /// Zero-based foot index in the form.
    pub foot_index: usize,
    /// Claimed original structural identity, not just its beat pattern.
    pub base: FootId,
    /// One operation against the original foot; `None` leaves it unchanged.
    pub operation: Option<Op>,
    /// Claimed complete output of this foot, without input spans.
    pub output: Box<[Beat]>,
}

/// A successful structural derivation bound to its form and position.
///
/// Fields are private and can only be populated by the structural checks. This
/// is an ordinary Rust result, not a formal proof of metrical legality.
#[derive(Debug, Clone)]
pub struct StructuralDerivation {
    trace: FootTrace,
    position: PositionContext,
}

impl StructuralDerivation {
    /// The computed output beats.
    pub fn beats(&self) -> &[Beat] {
        &self.trace.output
    }

    /// The positional facts derived from the form, never supplied by the caller.
    pub fn position(&self) -> PositionContext {
        self.position
    }

    /// A read-only view of the trace; a modified copy must be checked again.
    pub fn trace(&self) -> &FootTrace {
        &self.trace
    }

    /// Consume the result to obtain an ordinary, untrusted candidate trace.
    pub fn into_trace(self) -> FootTrace {
        self.trace
    }
}

/// A failure to derive or replay a structural foot transformation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DerivationError {
    /// The requested foot does not exist in the form.
    #[error("foot index {index} is outside a form with {foot_count} feet")]
    FootOutOfRange {
        /// Rejected index.
        index: usize,
        /// Actual number of feet.
        foot_count: usize,
    },
    /// A defined illa cannot apply at the derived nonterminal position.
    #[error("{illa:?} is forbidden at foot {foot_index} by the position gate")]
    IllaPosition {
        /// Requested operation.
        illa: Illa,
        /// Nonterminal index in the form.
        foot_index: usize,
    },
    /// An unresolved illa must not become allowed merely by choosing a position.
    #[error("the position scope of {illa:?} is not yet decided")]
    UnknownIllaScope {
        /// Unresolved operation.
        illa: Illa,
    },
    /// Zihaf ownership, range, or letter-state failure.
    #[error(transparent)]
    Zihaf(#[from] ZihafError),
    /// Illa structural precondition failure.
    #[error(transparent)]
    Illa(#[from] IllaError),
    /// The trace belongs to a different form from the one being checked.
    #[error("the trace form differs from the expected form")]
    FormMismatch,
    /// A trace may not select its own hemistich end.
    #[error("the trace terminal role differs from the expected terminal role")]
    TerminalMismatch,
    /// Even equal-surface feet retain different structural identities.
    #[error("the form requires {expected:?}, but the trace claims {claimed:?}")]
    BaseMismatch {
        /// Original identity derived from the form.
        expected: FootId,
        /// Identity claimed by the trace.
        claimed: FootId,
    },
    /// Replaying the operation does not reproduce the claimed output.
    #[error("the trace output differs from the replayed beats")]
    OutputMismatch,
}

/// Derive at most one operation at an actual foot of a generated form.
///
/// The terminal role is required, including for a single-foot form. Supplying
/// it never turns a nonterminal index into a terminal position. Success says
/// nothing about attestation of the form or permission in a linguistic profile.
pub fn derive_structural_foot(
    form: &MeterForm,
    terminal: TerminalRole,
    foot_index: usize,
    operation: Option<Op>,
) -> Result<StructuralDerivation, DerivationError> {
    let out_of_range = || DerivationError::FootOutOfRange {
        index: foot_index,
        foot_count: form.foot_count(),
    };
    let base = form.try_foot_at(foot_index).ok_or_else(out_of_range)?;
    let position = form
        .position_context(foot_index, Some(terminal))
        .ok_or_else(out_of_range)?;
    let foot = Foot::of(base);
    let output = match operation {
        None => beats_of(foot),
        Some(Op::Zihaf(z)) => apply_zihaf_checked(foot, z)?,
        Some(Op::Compound(c)) => apply_compound_checked(foot, c)?,
        Some(Op::Illa(illa)) => {
            let role = if illa == Illa::Kharm {
                if position.is_initial {
                    Role::Ibtida
                } else {
                    Role::Hashw
                }
            } else {
                match position.endpoint {
                    Some(end) => end.role(),
                    None if position.is_initial => Role::Ibtida,
                    None => Role::Hashw,
                }
            };
            match illa_scope(role, illa) {
                IllaScope::Allowed => apply_illa(foot, illa)?,
                IllaScope::Forbidden => {
                    return Err(DerivationError::IllaPosition { illa, foot_index });
                }
                IllaScope::Unknown => {
                    return Err(DerivationError::UnknownIllaScope { illa });
                }
            }
        }
    };
    Ok(StructuralDerivation {
        trace: FootTrace {
            form: *form,
            terminal,
            foot_index,
            base,
            operation,
            output,
        },
        position,
    })
}

/// Replay all claims against an externally selected form and hemistich end.
///
/// Form identity compares active segments and the shape label, ignoring unused
/// private array padding. Only one original-foot operation is represented; no
/// sequential transformation or full verse certificate is accepted here.
pub fn check_structural_trace(
    form: &MeterForm,
    terminal: TerminalRole,
    trace: &FootTrace,
) -> Result<StructuralDerivation, DerivationError> {
    if form.segments() != trace.form.segments() || form.terminal() != trace.form.terminal() {
        return Err(DerivationError::FormMismatch);
    }
    if terminal != trace.terminal {
        return Err(DerivationError::TerminalMismatch);
    }
    let base = form
        .try_foot_at(trace.foot_index)
        .ok_or(DerivationError::FootOutOfRange {
            index: trace.foot_index,
            foot_count: form.foot_count(),
        })?;
    if base != trace.base {
        return Err(DerivationError::BaseMismatch {
            expected: base,
            claimed: trace.base,
        });
    }
    let derived = derive_structural_foot(form, terminal, trace.foot_index, trace.operation)?;
    if derived.beats() != trace.output.as_ref() {
        return Err(DerivationError::OutputMismatch);
    }
    Ok(derived)
}
