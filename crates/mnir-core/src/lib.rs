#![forbid(unsafe_code)]

//! Canonical data model for MNIR Program Model 0.1, Type System Foundations
//! 0.1, Functions and Parameters 0.1, and Expressions and Basic Function
//! Bodies 0.1, Arithmetic Expressions 0.1, and Comparison Expressions 0.1.
//!
//! This crate implements the Program, Module, identity, revision, presentation
//! metadata, snapshot, fork, and controlled mutation concepts defined by
//! `docs/specification/01-program-model.md`, the closed intrinsic type set
//! defined by `docs/specification/02-type-system-foundations.md`, and Function
//! signatures with ordered Parameters defined by
//! `docs/specification/03-functions-and-parameters.md`, and the minimal body,
//! Block, Expression, and Return model defined by
//! `docs/specification/04-expressions-and-basic-function-bodies.md`, extended
//! with arithmetic operators and dependency/type rules from
//! `docs/specification/05-arithmetic-expressions.md`, and with comparison
//! operators from `docs/specification/07-comparison-expressions.md`.
//!
//! It intentionally contains no semantic verifier, EasyH support, general
//! type abstraction, arithmetic evaluation, control flow beyond Return,
//! invocation, serialization, or execution model.

mod ids;
mod intrinsic;
mod presentation;
mod program;

pub use ids::{
    BlockId, ExpressionId, FunctionId, IdentifierCategory, ModuleId, ParameterId, ProgramId,
    RevisionId,
};
pub use intrinsic::IntrinsicType;
pub use presentation::PresentationMetadata;
pub use program::{
    Block, Expression, ExpressionKind, ExpressionTypeError, Function, FunctionBody, MnirProgram,
    Module, MutationError, MutationTransaction, Parameter, ProgramSnapshot, StructuralError,
    TransactionState,
};
