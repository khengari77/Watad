//! Named recipes are linguistic data; normalization and ownership checks live
//! in the shared Verus-checked original-coordinate engine.
pub use watad_checker::named::Change;
pub use watad_checker::normalize::{Error, compile};
