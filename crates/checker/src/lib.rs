//! Profile-driven meter kernels with Verus contracts beside executable Rust.
//!
//! Structural edits, adjacency restrictions, profile lowering, exact finite
//! recognition, input-span certificates and poem obligations share their native
//! implementation with Verus. Linguistic facts and named recipe construction
//! remain external data/adapters. The general API does not dispatch on meter names.
#![no_std]
#![deny(unsafe_code)]

extern crate alloc;

pub mod adapter;
pub mod adjacency;
pub mod catalog;
pub mod expand;
pub mod language;
pub mod layout;
pub mod morphology;
pub mod named;
pub mod normalize;
pub mod obligations;
pub mod poem;
pub mod profile;
pub mod structured;
pub mod verse;
