use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use mnir_core::{
    Block, ExpressionId, ExpressionKind, Function, IntrinsicType, ProgramSnapshot, StructuralError,
    Terminator,
};

use crate::diagnostic::{
    CallArgumentTypeMismatch, Diagnostic, DiagnosticCode, DiagnosticPrimarySubject,
};
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

    // Applicability is scanned across the complete revision before semantic
    // verification begins (`MNIR-CFG-075` through `MNIR-CFG-077`, -129, -130).
    if !rule_set_is_applicable(snapshot, rule_set) {
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
            verify_function(snapshot, function, rule_set, &mut diagnostics, &mut seen);
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

fn rule_set_is_applicable(snapshot: &ProgramSnapshot, rule_set: VerificationRuleSet) -> bool {
    if rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4 {
        return true;
    }

    !snapshot.modules().any(|module| {
        module.functions().any(|function| {
            function.body().is_some_and(|body| {
                let has_call = body.blocks().any(|block| {
                    block
                        .expressions()
                        .any(|expression| matches!(expression.kind(), ExpressionKind::Call { .. }))
                });
                let has_conditional_control_flow = body.block_count() > 1
                    || body
                        .blocks()
                        .any(|block| matches!(block.terminator(), Some(Terminator::Branch { .. })));
                let has_comparison = body.blocks().any(|block| {
                    block
                        .expressions()
                        .any(|expression| comparison_operands(expression.kind()).is_some())
                });
                has_call
                    || (rule_set != VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3
                        && has_conditional_control_flow)
                    || (rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1
                        && has_comparison)
            })
        })
    })
}

fn verify_function(
    snapshot: &ProgramSnapshot,
    function: &Function,
    rule_set: VerificationRuleSet,
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    let Some(body) = function.body() else {
        return;
    };
    for block in body.blocks() {
        verify_block_expressions(snapshot, function, block, rule_set, diagnostics, seen);

        match block.terminator() {
            Some(Terminator::Return { expression }) => verify_return(
                function,
                block,
                *expression,
                body.block_count() > 1,
                snapshot,
                diagnostics,
                seen,
            ),
            Some(Terminator::Branch { condition, .. }) => {
                let mut memo = HashMap::new();
                let diagnostic =
                    match inspect_expression(snapshot, function, block, *condition, &mut memo) {
                        TypeInspection::Valid(IntrinsicType::Bool) => None,
                        TypeInspection::Valid(actual_type) => {
                            Some(Diagnostic::BranchConditionNotBool {
                                block_id: block.id(),
                                condition_expression_id: *condition,
                                actual_type,
                            })
                        }
                        TypeInspection::Unavailable
                        | TypeInspection::Mismatch { .. }
                        | TypeInspection::Unsupported(_) => {
                            Some(Diagnostic::BranchConditionTypeUnavailable {
                                block_id: block.id(),
                                condition_expression_id: *condition,
                            })
                        }
                    };
                if let Some(diagnostic) = diagnostic {
                    collect(diagnostic, diagnostics, seen);
                }
            }
            None => {}
        }
    }
}

fn verify_block_expressions(
    snapshot: &ProgramSnapshot,
    function: &Function,
    block: &Block,
    rule_set: VerificationRuleSet,
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    let mut memo = HashMap::new();
    for expression in block.expressions() {
        let inspection = inspect_expression(snapshot, function, block, expression.id(), &mut memo);
        let diagnostic = if arithmetic_operands(expression.kind()).is_some() {
            arithmetic_diagnostic(expression.id(), inspection)
        } else if rule_set != VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1
            && comparison_operands(expression.kind()).is_some()
        {
            comparison_diagnostic(expression.id(), inspection)
        } else {
            None
        };
        if let Some(diagnostic) = diagnostic {
            collect(diagnostic, diagnostics, seen);
        }
        if rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4
            && let ExpressionKind::Call { target, arguments } = expression.kind()
        {
            verify_call(
                snapshot,
                function,
                block,
                expression.id(),
                *target,
                arguments,
                diagnostics,
                seen,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_call(
    snapshot: &ProgramSnapshot,
    containing_function: &Function,
    block: &Block,
    expression_id: ExpressionId,
    target_function_id: mnir_core::FunctionId,
    arguments: &[ExpressionId],
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    let target = snapshot
        .function(target_function_id)
        .expect("structural validation guarantees a resolved Call target");

    if arguments.len() != target.parameter_count() {
        collect(
            Diagnostic::CallArgumentCountMismatch {
                expression_id,
                function_id: target_function_id,
                expected_count: target.parameter_count(),
                actual_count: arguments.len(),
            },
            diagnostics,
            seen,
        );
    }

    let mut unavailable = Vec::new();
    let mut mismatches = Vec::new();
    let mut memo = HashMap::new();
    for (argument_index, (&argument_id, parameter)) in
        arguments.iter().zip(target.parameters()).enumerate()
    {
        match inspect_expression(snapshot, containing_function, block, argument_id, &mut memo) {
            TypeInspection::Valid(actual_type) => {
                if &actual_type != parameter.intrinsic_type() {
                    mismatches.push(CallArgumentTypeMismatch {
                        argument_index,
                        expected_type: copy_intrinsic_type(parameter.intrinsic_type()),
                        actual_type,
                    });
                }
            }
            TypeInspection::Unavailable
            | TypeInspection::Mismatch { .. }
            | TypeInspection::Unsupported(_) => unavailable.push(argument_index),
        }
    }

    if !unavailable.is_empty() {
        collect(
            Diagnostic::CallArgumentTypeUnavailable {
                expression_id,
                function_id: target_function_id,
                argument_indices: unavailable,
            },
            diagnostics,
            seen,
        );
    }
    if !mismatches.is_empty() {
        collect(
            Diagnostic::CallArgumentTypeMismatch {
                expression_id,
                function_id: target_function_id,
                mismatches,
            },
            diagnostics,
            seen,
        );
    }
}

fn arithmetic_diagnostic(
    expression_id: ExpressionId,
    inspection: TypeInspection,
) -> Option<Diagnostic> {
    match inspection {
        TypeInspection::Valid(_) => None,
        TypeInspection::Unavailable => {
            Some(Diagnostic::ArithmeticOperandTypeUnavailable { expression_id })
        }
        TypeInspection::Mismatch {
            left_type,
            right_type,
        } => Some(Diagnostic::ArithmeticOperandTypeMismatch {
            expression_id,
            left_type,
            right_type,
        }),
        TypeInspection::Unsupported(operand_type) => {
            Some(Diagnostic::ArithmeticUnsupportedOperandType {
                expression_id,
                operand_type,
            })
        }
    }
}

fn comparison_diagnostic(
    expression_id: ExpressionId,
    inspection: TypeInspection,
) -> Option<Diagnostic> {
    match inspection {
        TypeInspection::Valid(_) => None,
        TypeInspection::Unavailable => {
            Some(Diagnostic::ComparisonOperandTypeUnavailable { expression_id })
        }
        TypeInspection::Mismatch {
            left_type,
            right_type,
        } => Some(Diagnostic::ComparisonOperandTypeMismatch {
            expression_id,
            left_type,
            right_type,
        }),
        TypeInspection::Unsupported(operand_type) => {
            Some(Diagnostic::ComparisonUnsupportedOperandType {
                expression_id,
                operand_type,
            })
        }
    }
}

fn verify_return(
    function: &Function,
    block: &Block,
    return_expression_id: ExpressionId,
    multi_block: bool,
    snapshot: &ProgramSnapshot,
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    let TypeInspection::Valid(actual_type) = inspect_expression(
        snapshot,
        function,
        block,
        return_expression_id,
        &mut HashMap::new(),
    ) else {
        return;
    };
    if &actual_type == function.return_type() {
        return;
    }

    let diagnostic = if multi_block {
        Diagnostic::ControlFlowReturnTypeMismatch {
            function_id: function.id(),
            block_id: block.id(),
            return_expression_id,
            expected_type: copy_intrinsic_type(function.return_type()),
            actual_type,
        }
    } else {
        Diagnostic::ReturnTypeMismatch {
            function_id: function.id(),
            return_expression_id,
            expected_type: copy_intrinsic_type(function.return_type()),
            actual_type,
        }
    };
    collect(diagnostic, diagnostics, seen);
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
    snapshot: &ProgramSnapshot,
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
            let left = inspect_expression(snapshot, function, block, *left, memo);
            let right = inspect_expression(snapshot, function, block, *right, memo);
            inspect_arithmetic(left, right)
        }
        Some(ExpressionKind::Equal { left, right } | ExpressionKind::NotEqual { left, right }) => {
            let left = inspect_expression(snapshot, function, block, *left, memo);
            let right = inspect_expression(snapshot, function, block, *right, memo);
            inspect_comparison(left, right, false)
        }
        Some(
            ExpressionKind::LessThan { left, right }
            | ExpressionKind::LessThanOrEqual { left, right }
            | ExpressionKind::GreaterThan { left, right }
            | ExpressionKind::GreaterThanOrEqual { left, right },
        ) => {
            let left = inspect_expression(snapshot, function, block, *left, memo);
            let right = inspect_expression(snapshot, function, block, *right, memo);
            inspect_comparison(left, right, true)
        }
        Some(ExpressionKind::Call { target, .. }) => snapshot
            .function(*target)
            .map(|target| TypeInspection::Valid(copy_intrinsic_type(target.return_type())))
            .unwrap_or(TypeInspection::Unavailable),
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
        | ExpressionKind::ParameterReference(_)
        | ExpressionKind::Call { .. } => None,
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
        | ExpressionKind::Remainder { .. }
        | ExpressionKind::Call { .. } => None,
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
