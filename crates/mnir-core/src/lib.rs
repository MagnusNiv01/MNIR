#![forbid(unsafe_code)]

//! Canonical data model for MNIR Program Model 0.1, Type System Foundations
//! 0.1, and Functions and Parameters 0.1.
//!
//! This crate implements the Program, Module, identity, revision, presentation
//! metadata, snapshot, fork, and controlled mutation concepts defined by
//! `docs/specification/01-program-model.md`, the closed intrinsic type set
//! defined by `docs/specification/02-type-system-foundations.md`, and Function
//! signatures with ordered Parameters defined by
//! `docs/specification/03-functions-and-parameters.md`.
//!
//! It intentionally contains no semantic verifier, EasyH support, general
//! type abstraction, Function bodies, expressions, invocation, serialization,
//! or execution model.

mod ids;
mod intrinsic;
mod presentation;
mod program;

pub use ids::{FunctionId, IdentifierCategory, ModuleId, ParameterId, ProgramId, RevisionId};
pub use intrinsic::IntrinsicType;
pub use presentation::PresentationMetadata;
pub use program::{
    Function, MnirProgram, Module, MutationError, MutationTransaction, Parameter, ProgramSnapshot,
    StructuralError, TransactionState,
};
