#![forbid(unsafe_code)]

//! Canonical data model for MNIR Program Model 0.1 and Type System
//! Foundations 0.1.
//!
//! This crate implements the Program, Module, identity, revision, presentation
//! metadata, snapshot, fork, and controlled mutation concepts defined by
//! `docs/specification/01-program-model.md`, plus the closed intrinsic type set
//! defined by `docs/specification/02-type-system-foundations.md`.
//!
//! It intentionally contains no semantic verifier, EasyH support, general
//! type abstraction, values, expressions, serialization, or execution model.

mod ids;
mod intrinsic;
mod model;

pub use ids::{IdentifierCategory, ModuleId, ProgramId, RevisionId};
pub use intrinsic::IntrinsicType;
pub use model::{
    MnirProgram, Module, MutationError, MutationTransaction, PresentationMetadata, ProgramSnapshot,
    StructuralError, TransactionState,
};
