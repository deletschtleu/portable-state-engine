//! Domain-independent engine core (`FR-001`..`FR-013`, `FR-016`).
//!
//! This module contains only pure contracts, error categories, replay logic,
//! store traits, and generic command orchestration. It must never import
//! domain-specific types or infrastructure adapters.

pub mod contracts;
pub mod errors;
