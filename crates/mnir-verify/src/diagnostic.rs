use std::fmt;

use mnir_core::{BlockId, ExpressionId, FunctionId, IntrinsicType};

/// Stable machine-readable diagnostic categories defined by verification rule
/// sets V0_1 through V0_4.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticCode {
    ArithmeticOperandTypeUnavailable,
    ArithmeticOperandTypeMismatch,
    ArithmeticUnsupportedOperandType,
    ReturnTypeMismatch,
    ComparisonOperandTypeUnavailable,
    ComparisonOperandTypeMismatch,
    ComparisonUnsupportedOperandType,
    BranchConditionTypeUnavailable,
    BranchConditionNotBool,
    ControlFlowReturnTypeMismatch,
    CallArgumentCountMismatch,
    CallArgumentTypeUnavailable,
    CallArgumentTypeMismatch,
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
            Self::ComparisonOperandTypeUnavailable => "MNIR-DIAG-005",
            Self::ComparisonOperandTypeMismatch => "MNIR-DIAG-006",
            Self::ComparisonUnsupportedOperandType => "MNIR-DIAG-007",
            Self::BranchConditionTypeUnavailable => "MNIR-DIAG-008",
            Self::BranchConditionNotBool => "MNIR-DIAG-009",
            Self::ControlFlowReturnTypeMismatch => "MNIR-DIAG-010",
            Self::CallArgumentCountMismatch => "MNIR-DIAG-011",
            Self::CallArgumentTypeUnavailable => "MNIR-DIAG-012",
            Self::CallArgumentTypeMismatch => "MNIR-DIAG-013",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The only diagnostic severity defined by V0_1 through V0_4.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
}

/// The identity paired with a code for duplicate prevention.
///
/// This is deliberately limited to the concrete subjects defined by
/// the active rule sets; it is not a generic node abstraction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticPrimarySubject {
    Expression(ExpressionId),
    Function(FunctionId),
    Block(BlockId),
}

/// One zero-based Call argument position whose derived intrinsic type differs
/// from the corresponding target Parameter type.
#[derive(Debug, Eq, PartialEq)]
pub struct CallArgumentTypeMismatch {
    pub argument_index: usize,
    pub expected_type: IntrinsicType,
    pub actual_type: IntrinsicType,
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
    ComparisonOperandTypeUnavailable {
        expression_id: ExpressionId,
    },
    ComparisonOperandTypeMismatch {
        expression_id: ExpressionId,
        left_type: IntrinsicType,
        right_type: IntrinsicType,
    },
    ComparisonUnsupportedOperandType {
        expression_id: ExpressionId,
        operand_type: IntrinsicType,
    },
    BranchConditionTypeUnavailable {
        block_id: BlockId,
        condition_expression_id: ExpressionId,
    },
    BranchConditionNotBool {
        block_id: BlockId,
        condition_expression_id: ExpressionId,
        actual_type: IntrinsicType,
    },
    ControlFlowReturnTypeMismatch {
        function_id: FunctionId,
        block_id: BlockId,
        return_expression_id: ExpressionId,
        expected_type: IntrinsicType,
        actual_type: IntrinsicType,
    },
    CallArgumentCountMismatch {
        expression_id: ExpressionId,
        function_id: FunctionId,
        expected_count: usize,
        actual_count: usize,
    },
    CallArgumentTypeUnavailable {
        expression_id: ExpressionId,
        function_id: FunctionId,
        argument_indices: Vec<usize>,
    },
    CallArgumentTypeMismatch {
        expression_id: ExpressionId,
        function_id: FunctionId,
        mismatches: Vec<CallArgumentTypeMismatch>,
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
            Self::ComparisonOperandTypeUnavailable { .. } => {
                DiagnosticCode::ComparisonOperandTypeUnavailable
            }
            Self::ComparisonOperandTypeMismatch { .. } => {
                DiagnosticCode::ComparisonOperandTypeMismatch
            }
            Self::ComparisonUnsupportedOperandType { .. } => {
                DiagnosticCode::ComparisonUnsupportedOperandType
            }
            Self::BranchConditionTypeUnavailable { .. } => {
                DiagnosticCode::BranchConditionTypeUnavailable
            }
            Self::BranchConditionNotBool { .. } => DiagnosticCode::BranchConditionNotBool,
            Self::ControlFlowReturnTypeMismatch { .. } => {
                DiagnosticCode::ControlFlowReturnTypeMismatch
            }
            Self::CallArgumentCountMismatch { .. } => DiagnosticCode::CallArgumentCountMismatch,
            Self::CallArgumentTypeUnavailable { .. } => DiagnosticCode::CallArgumentTypeUnavailable,
            Self::CallArgumentTypeMismatch { .. } => DiagnosticCode::CallArgumentTypeMismatch,
        }
    }

    /// Every currently defined diagnostic has Error severity.
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
            | Self::ArithmeticUnsupportedOperandType { expression_id, .. }
            | Self::ComparisonOperandTypeUnavailable { expression_id }
            | Self::ComparisonOperandTypeMismatch { expression_id, .. }
            | Self::ComparisonUnsupportedOperandType { expression_id, .. }
            | Self::CallArgumentCountMismatch { expression_id, .. }
            | Self::CallArgumentTypeUnavailable { expression_id, .. }
            | Self::CallArgumentTypeMismatch { expression_id, .. } => {
                DiagnosticPrimarySubject::Expression(*expression_id)
            }
            Self::ReturnTypeMismatch { function_id, .. } => {
                DiagnosticPrimarySubject::Function(*function_id)
            }
            Self::BranchConditionTypeUnavailable { block_id, .. }
            | Self::BranchConditionNotBool { block_id, .. }
            | Self::ControlFlowReturnTypeMismatch { block_id, .. } => {
                DiagnosticPrimarySubject::Block(*block_id)
            }
        }
    }
}
