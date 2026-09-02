use std::fmt;

use mnir_core::{ExpressionId, FunctionId, IntrinsicType};

/// Stable machine-readable diagnostic categories defined by version 0.1.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticCode {
    ArithmeticOperandTypeUnavailable,
    ArithmeticOperandTypeMismatch,
    ArithmeticUnsupportedOperandType,
    ReturnTypeMismatch,
}

impl DiagnosticCode {
    /// Returns the stable specification code for this diagnostic category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ArithmeticOperandTypeUnavailable => "MNIR-DIAG-001",
            Self::ArithmeticOperandTypeMismatch => "MNIR-DIAG-002",
            Self::ArithmeticUnsupportedOperandType => "MNIR-DIAG-003",
            Self::ReturnTypeMismatch => "MNIR-DIAG-004",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The only diagnostic severity defined by version 0.1.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
}

/// The identity paired with a code for duplicate prevention.
///
/// This is deliberately limited to the concrete subjects defined by
/// `MNIR-VERIFY-086`; it is not a generic node abstraction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticPrimarySubject {
    Expression(ExpressionId),
    Function(FunctionId),
}

/// One semantic verification failure with its complete normative payload.
#[derive(Debug, Eq, PartialEq)]
pub enum Diagnostic {
    ArithmeticOperandTypeUnavailable {
        expression_id: ExpressionId,
    },
    ArithmeticOperandTypeMismatch {
        expression_id: ExpressionId,
        left_type: IntrinsicType,
        right_type: IntrinsicType,
    },
    ArithmeticUnsupportedOperandType {
        expression_id: ExpressionId,
        operand_type: IntrinsicType,
    },
    ReturnTypeMismatch {
        function_id: FunctionId,
        return_expression_id: ExpressionId,
        expected_type: IntrinsicType,
        actual_type: IntrinsicType,
    },
}

impl Diagnostic {
    /// Returns the stable category identity (`MNIR-VERIFY-022`, -063).
    #[must_use]
    pub const fn code(&self) -> DiagnosticCode {
        match self {
            Self::ArithmeticOperandTypeUnavailable { .. } => {
                DiagnosticCode::ArithmeticOperandTypeUnavailable
            }
            Self::ArithmeticOperandTypeMismatch { .. } => {
                DiagnosticCode::ArithmeticOperandTypeMismatch
            }
            Self::ArithmeticUnsupportedOperandType { .. } => {
                DiagnosticCode::ArithmeticUnsupportedOperandType
            }
            Self::ReturnTypeMismatch { .. } => DiagnosticCode::ReturnTypeMismatch,
        }
    }

    /// Every version 0.1 diagnostic has Error severity (`MNIR-VERIFY-020`).
    #[must_use]
    pub const fn severity(&self) -> DiagnosticSeverity {
        DiagnosticSeverity::Error
    }

    /// Returns the primary subject used for duplicate prevention.
    #[must_use]
    pub const fn primary_subject(&self) -> DiagnosticPrimarySubject {
        match self {
            Self::ArithmeticOperandTypeUnavailable { expression_id }
            | Self::ArithmeticOperandTypeMismatch { expression_id, .. }
            | Self::ArithmeticUnsupportedOperandType { expression_id, .. } => {
                DiagnosticPrimarySubject::Expression(*expression_id)
            }
            Self::ReturnTypeMismatch { function_id, .. } => {
                DiagnosticPrimarySubject::Function(*function_id)
            }
        }
    }
}
