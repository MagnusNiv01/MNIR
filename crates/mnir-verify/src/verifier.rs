use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use mnir_core::{
    Block, ExpressionId, ExpressionKind, Function, IntrinsicType, ProgramSnapshot, StructuralError,
};

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticPrimarySubject};
use crate::verified::{VerificationRuleSet, VerifiedProgram};

/// The complete semantic failure from one verification run.
#[derive(Debug, Eq, PartialEq)]
pub struct VerificationFailure {
    diagnostics: Vec<Diagnostic>,
}

impl VerificationFailure {
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    #[must_use]
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

impl fmt::Display for VerificationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "semantic verification produced {} error diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

impl Error for VerificationFailure {}

/// A typed distinction between invalid input, rule-set inapplicability, and
/// semantic failure.
#[derive(Debug, Eq, PartialEq)]
pub enum VerificationError {
    StructuralInput(StructuralError),
    RuleSetNotApplicable {
        requested_rule_set: VerificationRuleSet,
    },
    Semantic(VerificationFailure),
}

impl VerificationError {
    /// Semantic diagnostics are present only for semantic verification failure.
    #[must_use]
    pub fn diagnostics(&self) -> Option<&[Diagnostic]> {
        match self {
            Self::StructuralInput(_) => None,
            Self::RuleSetNotApplicable { .. } => None,
            Self::Semantic(failure) => Some(failure.diagnostics()),
        }
    }
}

impl fmt::Display for VerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StructuralInput(error) => {
                write!(formatter, "invalid verification input: {error}")
            }
            Self::RuleSetNotApplicable { requested_rule_set } => write!(
                formatter,
                "verification rule set {} is not applicable to this Program revision",
                requested_rule_set.as_str()
            ),
            Self::Semantic(failure) => failure.fmt(formatter),
        }
    }
}

impl Error for VerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::StructuralInput(error) => Some(error),
            Self::RuleSetNotApplicable { .. } => None,
            Self::Semantic(failure) => Some(failure),
        }
    }
}

/// Verifies one immutable committed Program revision under the fixed 0.1 rule set.
pub fn verify(snapshot: &ProgramSnapshot) -> Result<VerifiedProgram, VerificationError> {
    verify_with_rule_set(
        snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
    )
}

/// Verifies one immutable revision under an explicitly selected rule set.
pub fn verify_with_rule_set(
    snapshot: &ProgramSnapshot,
    rule_set: VerificationRuleSet,
) -> Result<VerifiedProgram, VerificationError> {
    snapshot
        .validate_structure()
        .map_err(VerificationError::StructuralInput)?;

    // Applicability is scanned across the complete revision before any V0_1
    // semantic check can produce a result (`MNIR-CMP-101` through -105).
    if rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1
        && contains_comparison_expression(snapshot)
    {
        return Err(VerificationError::RuleSetNotApplicable {
            requested_rule_set: rule_set,
        });
    }

    let mut diagnostics = Vec::new();
    let mut seen = HashSet::new();

    // Full traversal is intentionally based only on public read-only mnir-core
    // inspection (`MNIR-VERIFY-027`, -037, -055, -067, -087 through -089).
    for module in snapshot.modules() {
        for function in module.functions() {
            verify_function(function, rule_set, &mut diagnostics, &mut seen);
        }
    }

    if diagnostics.is_empty() {
        Ok(VerifiedProgram::new(snapshot.clone(), rule_set))
    } else {
        Err(VerificationError::Semantic(VerificationFailure {
            diagnostics,
        }))
    }
}

fn contains_comparison_expression(snapshot: &ProgramSnapshot) -> bool {
    snapshot.modules().any(|module| {
        module.functions().any(|function| {
            function.body().is_some_and(|body| {
                body.block()
                    .expressions()
                    .any(|expression| comparison_operands(expression.kind()).is_some())
            })
        })
    })
}

fn verify_function(
    function: &Function,
    rule_set: VerificationRuleSet,
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    let Some(body) = function.body() else {
        return;
    };
    let block = body.block();
    let mut memo = HashMap::new();

    for expression in block.expressions() {
        let inspection = inspect_expression(function, block, expression.id(), &mut memo);
        let diagnostic = if arithmetic_operands(expression.kind()).is_some() {
            match inspection {
                TypeInspection::Valid(_) => None,
                TypeInspection::Unavailable => Some(Diagnostic::ArithmeticOperandTypeUnavailable {
                    expression_id: expression.id(),
                }),
                TypeInspection::Mismatch {
                    left_type,
                    right_type,
                } => Some(Diagnostic::ArithmeticOperandTypeMismatch {
                    expression_id: expression.id(),
                    left_type,
                    right_type,
                }),
                TypeInspection::Unsupported(operand_type) => {
                    Some(Diagnostic::ArithmeticUnsupportedOperandType {
                        expression_id: expression.id(),
                        operand_type,
                    })
                }
            }
        } else if rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2
            && comparison_operands(expression.kind()).is_some()
        {
            match inspection {
                TypeInspection::Valid(_) => None,
                TypeInspection::Unavailable => Some(Diagnostic::ComparisonOperandTypeUnavailable {
                    expression_id: expression.id(),
                }),
                TypeInspection::Mismatch {
                    left_type,
                    right_type,
                } => Some(Diagnostic::ComparisonOperandTypeMismatch {
                    expression_id: expression.id(),
                    left_type,
                    right_type,
                }),
                TypeInspection::Unsupported(operand_type) => {
                    Some(Diagnostic::ComparisonUnsupportedOperandType {
                        expression_id: expression.id(),
                        operand_type,
                    })
                }
            }
        } else {
            None
        };
        if let Some(diagnostic) = diagnostic {
            collect(diagnostic, diagnostics, seen);
        }
    }

    let Some(return_expression_id) = block.return_expression_id() else {
        return;
    };
    let TypeInspection::Valid(actual_type) =
        inspect_expression(function, block, return_expression_id, &mut memo)
    else {
        // The applicable operand diagnostic is emitted above. Neither rule set
        // defines a ReturnTypeUnavailable diagnostic (`MNIR-VERIFY-041`,
        // `MNIR-CMP-079`).
        return;
    };

    if &actual_type != function.return_type() {
        collect(
            Diagnostic::ReturnTypeMismatch {
                function_id: function.id(),
                return_expression_id,
                expected_type: copy_intrinsic_type(function.return_type()),
                actual_type,
            },
            diagnostics,
            seen,
        );
    }
}

fn collect(
    diagnostic: Diagnostic,
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    let key = (diagnostic.code(), diagnostic.primary_subject());
    if seen.insert(key) {
        diagnostics.push(diagnostic);
    }
}

#[derive(Debug, Eq, PartialEq)]
enum TypeInspection {
    Valid(IntrinsicType),
    Unavailable,
    Mismatch {
        left_type: IntrinsicType,
        right_type: IntrinsicType,
    },
    Unsupported(IntrinsicType),
}

impl TypeInspection {
    fn copied(&self) -> Self {
        match self {
            Self::Valid(intrinsic_type) => Self::Valid(copy_intrinsic_type(intrinsic_type)),
            Self::Unavailable => Self::Unavailable,
            Self::Mismatch {
                left_type,
                right_type,
            } => Self::Mismatch {
                left_type: copy_intrinsic_type(left_type),
                right_type: copy_intrinsic_type(right_type),
            },
            Self::Unsupported(intrinsic_type) => {
                Self::Unsupported(copy_intrinsic_type(intrinsic_type))
            }
        }
    }
}

fn inspect_expression(
    function: &Function,
    block: &Block,
    expression_id: ExpressionId,
    memo: &mut HashMap<ExpressionId, TypeInspection>,
) -> TypeInspection {
    if let Some(inspection) = memo.get(&expression_id) {
        return inspection.copied();
    }

    let inspection = match block
        .expression(expression_id)
        .map(|expression| expression.kind())
    {
        Some(ExpressionKind::Int32Literal(_)) => TypeInspection::Valid(IntrinsicType::Int32),
        Some(ExpressionKind::Int64Literal(_)) => TypeInspection::Valid(IntrinsicType::Int64),
        Some(ExpressionKind::BoolLiteral(_)) => TypeInspection::Valid(IntrinsicType::Bool),
        Some(ExpressionKind::UnitLiteral) => TypeInspection::Valid(IntrinsicType::Unit),
        Some(ExpressionKind::ParameterReference(parameter_id)) => function
            .parameter(*parameter_id)
            .map(|parameter| TypeInspection::Valid(copy_intrinsic_type(parameter.intrinsic_type())))
            .unwrap_or(TypeInspection::Unavailable),
        Some(
            ExpressionKind::Add { left, right }
            | ExpressionKind::Subtract { left, right }
            | ExpressionKind::Multiply { left, right }
            | ExpressionKind::Divide { left, right }
            | ExpressionKind::Remainder { left, right },
        ) => {
            let left = inspect_expression(function, block, *left, memo);
            let right = inspect_expression(function, block, *right, memo);
            inspect_arithmetic(left, right)
        }
        Some(ExpressionKind::Equal { left, right } | ExpressionKind::NotEqual { left, right }) => {
            let left = inspect_expression(function, block, *left, memo);
            let right = inspect_expression(function, block, *right, memo);
            inspect_comparison(left, right, false)
        }
        Some(
            ExpressionKind::LessThan { left, right }
            | ExpressionKind::LessThanOrEqual { left, right }
            | ExpressionKind::GreaterThan { left, right }
            | ExpressionKind::GreaterThanOrEqual { left, right },
        ) => {
            let left = inspect_expression(function, block, *left, memo);
            let right = inspect_expression(function, block, *right, memo);
            inspect_comparison(left, right, true)
        }
        None => TypeInspection::Unavailable,
    };

    memo.insert(expression_id, inspection.copied());
    inspection
}

fn inspect_arithmetic(left: TypeInspection, right: TypeInspection) -> TypeInspection {
    let (TypeInspection::Valid(left_type), TypeInspection::Valid(right_type)) = (left, right)
    else {
        return TypeInspection::Unavailable;
    };

    if left_type != right_type {
        return TypeInspection::Mismatch {
            left_type,
            right_type,
        };
    }

    match left_type {
        IntrinsicType::Int32 => TypeInspection::Valid(IntrinsicType::Int32),
        IntrinsicType::Int64 => TypeInspection::Valid(IntrinsicType::Int64),
        IntrinsicType::Bool => TypeInspection::Unsupported(IntrinsicType::Bool),
        IntrinsicType::Unit => TypeInspection::Unsupported(IntrinsicType::Unit),
    }
}

fn inspect_comparison(
    left: TypeInspection,
    right: TypeInspection,
    ordering: bool,
) -> TypeInspection {
    let (TypeInspection::Valid(left_type), TypeInspection::Valid(right_type)) = (left, right)
    else {
        return TypeInspection::Unavailable;
    };

    if left_type != right_type {
        return TypeInspection::Mismatch {
            left_type,
            right_type,
        };
    }

    if ordering {
        match left_type {
            IntrinsicType::Int32 | IntrinsicType::Int64 => {}
            IntrinsicType::Bool => return TypeInspection::Unsupported(IntrinsicType::Bool),
            IntrinsicType::Unit => return TypeInspection::Unsupported(IntrinsicType::Unit),
        }
    }

    TypeInspection::Valid(IntrinsicType::Bool)
}

fn arithmetic_operands(kind: &ExpressionKind) -> Option<(ExpressionId, ExpressionId)> {
    match kind {
        ExpressionKind::Add { left, right }
        | ExpressionKind::Subtract { left, right }
        | ExpressionKind::Multiply { left, right }
        | ExpressionKind::Divide { left, right }
        | ExpressionKind::Remainder { left, right } => Some((*left, *right)),
        ExpressionKind::Equal { .. }
        | ExpressionKind::NotEqual { .. }
        | ExpressionKind::LessThan { .. }
        | ExpressionKind::LessThanOrEqual { .. }
        | ExpressionKind::GreaterThan { .. }
        | ExpressionKind::GreaterThanOrEqual { .. }
        | ExpressionKind::Int32Literal(_)
        | ExpressionKind::Int64Literal(_)
        | ExpressionKind::BoolLiteral(_)
        | ExpressionKind::UnitLiteral
        | ExpressionKind::ParameterReference(_) => None,
    }
}

fn comparison_operands(kind: &ExpressionKind) -> Option<(ExpressionId, ExpressionId)> {
    match kind {
        ExpressionKind::Equal { left, right }
        | ExpressionKind::NotEqual { left, right }
        | ExpressionKind::LessThan { left, right }
        | ExpressionKind::LessThanOrEqual { left, right }
        | ExpressionKind::GreaterThan { left, right }
        | ExpressionKind::GreaterThanOrEqual { left, right } => Some((*left, *right)),
        ExpressionKind::Int32Literal(_)
        | ExpressionKind::Int64Literal(_)
        | ExpressionKind::BoolLiteral(_)
        | ExpressionKind::UnitLiteral
        | ExpressionKind::ParameterReference(_)
        | ExpressionKind::Add { .. }
        | ExpressionKind::Subtract { .. }
        | ExpressionKind::Multiply { .. }
        | ExpressionKind::Divide { .. }
        | ExpressionKind::Remainder { .. } => None,
    }
}

fn copy_intrinsic_type(intrinsic_type: &IntrinsicType) -> IntrinsicType {
    match intrinsic_type {
        IntrinsicType::Int32 => IntrinsicType::Int32,
        IntrinsicType::Int64 => IntrinsicType::Int64,
        IntrinsicType::Bool => IntrinsicType::Bool,
        IntrinsicType::Unit => IntrinsicType::Unit,
    }
}
