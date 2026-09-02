#![forbid(unsafe_code)]

//! Semantic verification and machine-readable diagnostics for MNIR 0.1.
//!
//! Verification consumes an immutable [`mnir_core::ProgramSnapshot`], never
//! mutates or repairs it, and either returns revision-bound [`VerifiedProgram`]
//! evidence or all applicable version 0.1 semantic diagnostics.
//!
//! The crate deliberately contains no evaluator, constant folder, arithmetic
//! fault analysis, future verification domains, warning system, suppression
//! profile, or generic source-location/node abstraction.

mod diagnostic;
mod verified;
mod verifier;

pub use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticPrimarySubject, DiagnosticSeverity};
pub use verified::{VerificationRuleSet, VerifiedProgram};
pub use verifier::{VerificationError, VerificationFailure, verify};
