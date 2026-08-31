#![forbid(unsafe_code)]

//! Canonical data model for MNIR Program Model 0.1.
//!
//! This crate implements only the Program, Module, identity, revision,
//! presentation metadata, snapshot, fork, and controlled mutation concepts
//! defined by `docs/specification/01-program-model.md`.
//!
//! It intentionally contains no semantic verifier, EasyH support, generic
//! node abstraction, language entities, serialization, or execution model.

mod ids;
mod model;

pub use ids::{IdentifierCategory, ModuleId, ProgramId, RevisionId};
pub use model::{
    MnirProgram, Module, MutationError, MutationTransaction, PresentationMetadata, ProgramSnapshot,
    StructuralError, TransactionState,
};
