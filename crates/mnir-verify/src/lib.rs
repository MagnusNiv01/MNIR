#![forbid(unsafe_code)]

//! Semantic verification and machine-readable diagnostics for MNIR rule sets
//! V0_1, V0_2, and V0_3.
//!
//! Verification consumes an immutable [`mnir_core::ProgramSnapshot`], never
//! mutates or repairs it, and either returns revision-bound [`VerifiedProgram`]
//! evidence or all applicable semantic diagnostics. The existing [`verify`]
//! convenience function remains fixed to V0_1; [`verify_with_rule_set`] makes
//! later rule-set selection explicit.
//!
//! V0_2 adds Comparison Expressions without changing V0_1 applicability or
//! semantics. V0_3 adds Conditional Control Flow while preserving both older
//! rule sets and rejecting inapplicable CFG revisions before semantic work.
//! The crate deliberately contains no evaluator, constant folder, arithmetic
//! fault analysis, future verification domains, warning system, suppression
//! profile, or generic source-location/node abstraction.

mod diagnostic;
mod verified;
mod verifier;

pub use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticPrimarySubject, DiagnosticSeverity};
pub use verified::{VerificationRuleSet, VerifiedProgram};
pub use verifier::{VerificationError, VerificationFailure, verify, verify_with_rule_set};
