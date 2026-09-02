use mnir_core::{ProgramId, ProgramSnapshot, RevisionId};

/// Identity of the fixed semantic verification rule set implemented here.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VerificationRuleSet {
    SemanticVerificationAndDiagnosticsV0_1,
}

impl VerificationRuleSet {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SemanticVerificationAndDiagnosticsV0_1 => {
                "MNIR Semantic Verification and Diagnostics 0.1"
            }
        }
    }
}

/// Immutable evidence that one exact Program revision passed version 0.1.
///
/// Fields are private and no public constructor exists, so ordinary callers
/// cannot manufacture verification evidence without [`crate::verify`]
/// (`MNIR-VERIFY-056`, -057, -085; `AR-VERIFY-022`).
///
/// ```compile_fail
/// use mnir_core::MnirProgram;
/// use mnir_verify::{VerificationRuleSet, VerifiedProgram};
///
/// let snapshot = MnirProgram::new().unwrap().snapshot();
/// let _forged = VerifiedProgram {
///     snapshot,
///     rule_set: VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
/// };
/// ```
#[derive(Clone, Debug)]
pub struct VerifiedProgram {
    snapshot: ProgramSnapshot,
    rule_set: VerificationRuleSet,
}

impl VerifiedProgram {
    pub(crate) const fn new(snapshot: ProgramSnapshot) -> Self {
        Self {
            snapshot,
            rule_set: VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
        }
    }

    #[must_use]
    pub const fn program_id(&self) -> ProgramId {
        self.snapshot.program_id()
    }

    #[must_use]
    pub fn revision_id(&self) -> RevisionId {
        self.snapshot.revision_id()
    }

    #[must_use]
    pub const fn rule_set(&self) -> VerificationRuleSet {
        self.rule_set
    }

    /// Provides read-only access to the exact successfully verified contents.
    #[must_use]
    pub const fn snapshot(&self) -> &ProgramSnapshot {
        &self.snapshot
    }
}
