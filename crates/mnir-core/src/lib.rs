#![forbid(unsafe_code)]

//! Canonical data model for MNIR Program Model 0.1, Type System Foundations
//! 0.1, Functions and Parameters 0.1, and Expressions and Basic Function
//! Bodies 0.1, Arithmetic Expressions 0.1, Comparison Expressions 0.1, and
//! Conditional Control Flow 0.1.
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
//! Conditional Control Flow extends Function bodies with multiple Blocks,
//! entry identity, and Return/Branch terminators as defined by
//! `docs/specification/08-conditional-control-flow.md`. Direct effectful Call
//! Expressions and per-Block EffectSequences are defined by
//! `docs/specification/09-function-calls-and-sequencing-foundations.md`.
//!
//! It intentionally contains no semantic verifier, EasyH support, general
//! type abstraction, arithmetic or comparison evaluation, loops, Call
//! execution, serialization, or execution model.

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
    Terminator, TransactionState,
};
