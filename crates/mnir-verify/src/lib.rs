#![forbid(unsafe_code)]

//! Semantic verification and machine-readable diagnostics for MNIR rule sets
//! V0_1 and V0_2.
//!
//! Verification consumes an immutable [`mnir_core::ProgramSnapshot`], never
//! mutates or repairs it, and either returns revision-bound [`VerifiedProgram`]
//! evidence or all applicable semantic diagnostics. The existing [`verify`]
//! convenience function remains fixed to V0_1; [`verify_with_rule_set`] makes
//! V0_2 selection explicit.
//!
//! V0_2 adds Comparison Expressions without changing V0_1 applicability or
//! semantics. The crate deliberately contains no evaluator, constant folder,
//! arithmetic fault analysis, future verification domains, warning system,
//! suppression profile, or generic source-location/node abstraction.

mod diagnostic;
mod verified;
mod verifier;

pub use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticPrimarySubject, DiagnosticSeverity};
pub use verified::{VerificationRuleSet, VerifiedProgram};
pub use verifier::{VerificationError, VerificationFailure, verify, verify_with_rule_set};
