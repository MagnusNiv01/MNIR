#![forbid(unsafe_code)]

//! Semantic verification and machine-readable diagnostics for MNIR rule sets
//! V0_1, V0_2, V0_3, and V0_4.
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
//! V0_4 adds direct effectful Calls and argument diagnostics while preserving
//! all earlier rule-set applicability boundaries.
//! The crate deliberately contains no evaluator, constant folder, arithmetic
//! fault analysis, future verification domains, warning system, suppression
//! profile, or generic source-location/node abstraction.

mod diagnostic;
mod verified;
mod verifier;

pub use diagnostic::{
    CallArgumentTypeMismatch, Diagnostic, DiagnosticCode, DiagnosticPrimarySubject,
    DiagnosticSeverity,
};
pub use verified::{VerificationRuleSet, VerifiedProgram};
pub use verifier::{VerificationError, VerificationFailure, verify, verify_with_rule_set};
