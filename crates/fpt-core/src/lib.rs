//! `fpt-core` — shared foundation types for the fpt-cli workspace.
//!
//! This crate provides the error model ([`AppError`], [`ErrorCode`]),
//! output format definitions ([`OutputFormat`], [`RiskLevel`]),
//! command specification schema ([`CommandSpec`]), and JSON I/O helpers
//! used by both `fpt-domain` and `fpt-cli`.
//!
//! It intentionally has zero ShotGrid-specific logic so that any downstream
//! crate can depend on it without pulling in transport or domain concerns.

#![allow(clippy::result_large_err)]

pub mod error;
pub mod io;
pub mod model;
pub mod spec;

pub use error::{AppError, ErrorCode, ErrorEnvelope, Result};
pub use io::read_json_input;
pub use model::{OutputFormat, RiskLevel};
pub use spec::CommandSpec;
