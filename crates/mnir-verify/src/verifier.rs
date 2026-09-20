use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use mnir_core::{
    Block, ExpressionId, ExpressionKind, ExpressionTypeError, Function, IntrinsicType,
    ProgramSnapshot, StructuralError, Terminator, ValueType,
};

use crate::diagnostic::{
    CallArgumentTypeMismatch, CallArgumentValueTypeMismatch, Diagnostic, DiagnosticCode,
    DiagnosticPrimarySubject,
};
use crate::verified::{VerificationRuleSet, VerifiedProgram};

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

#[derive(Debug, Eq, PartialEq)]
pub enum VerificationError {
    StructuralInput(StructuralError),
    RuleSetNotApplicable {
        requested_rule_set: VerificationRuleSet,
    },
    Semantic(VerificationFailure),
}

impl VerificationError {
    #[must_use]
    pub fn diagnostics(&self) -> Option<&[Diagnostic]> {
        match self {
            Self::Semantic(failure) => Some(failure.diagnostics()),
            Self::StructuralInput(_) | Self::RuleSetNotApplicable { .. } => None,
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
            Self::Semantic(failure) => Some(failure),
            Self::RuleSetNotApplicable { .. } => None,
        }
    }
}

pub fn verify(snapshot: &ProgramSnapshot) -> Result<VerifiedProgram, VerificationError> {
    verify_with_rule_set(
        snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
    )
}

pub fn verify_with_rule_set(
    snapshot: &ProgramSnapshot,
    rule_set: VerificationRuleSet,
) -> Result<VerifiedProgram, VerificationError> {
    snapshot
        .validate_structure()
        .map_err(VerificationError::StructuralInput)?;

    if !rule_set_is_applicable(snapshot, rule_set) {
        return Err(VerificationError::RuleSetNotApplicable {
            requested_rule_set: rule_set,
        });
    }

    let mut diagnostics = Vec::new();
    let mut seen = HashSet::new();
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
    if rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5 {
        return true;
    }

    !snapshot.modules().any(|module| {
        module.domain_type_count() != 0
            || module.functions().any(|function| {
                unsupported_historical_value_type(*function.return_type())
                    || function
                        .parameters()
                        .iter()
                        .any(|parameter| unsupported_historical_value_type(*parameter.value_type()))
                    || function.body().is_some_and(|body| {
                        let has_domain_form = body.blocks().any(|block| {
                            block.expressions().any(|expression| {
                                matches!(
                                    expression.kind(),
                                    ExpressionKind::TextLiteral(_)
                                        | ExpressionKind::BytesLiteral(_)
                                        | ExpressionKind::DomainConstruct { .. }
                                        | ExpressionKind::DomainProject { .. }
                                )
                            })
                        });
                        let has_call = body.blocks().any(|block| {
                            block.expressions().any(|expression| {
                                matches!(expression.kind(), ExpressionKind::Call { .. })
                            })
                        });
                        let has_cfg = body.block_count() > 1
                            || body.blocks().any(|block| {
                                matches!(block.terminator(), Some(Terminator::Branch { .. }))
                            });
                        let has_comparison = body.blocks().any(|block| {
                            block
                                .expressions()
                                .any(|expression| comparison_operands(expression.kind()).is_some())
                        });
                        has_domain_form
                            || (rule_set
                                != VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4
                                && has_call)
                            || (matches!(
                                rule_set,
                                VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1
                                    | VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2
                            ) && has_cfg)
                            || (rule_set
                                == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1
                                && has_comparison)
                    })
            })
    })
}

fn unsupported_historical_value_type(value_type: ValueType) -> bool {
    !matches!(
        value_type,
        ValueType::Intrinsic(
            IntrinsicType::Int32 | IntrinsicType::Int64 | IntrinsicType::Bool | IntrinsicType::Unit
        )
    )
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
        verify_block_expressions(snapshot, block, rule_set, diagnostics, seen);
        match block.terminator() {
            Some(Terminator::Return { expression }) => verify_return(
                snapshot,
                function,
                block,
                *expression,
                body.block_count() > 1,
                rule_set,
                diagnostics,
                seen,
            ),
            Some(Terminator::Branch { condition, .. }) => {
                verify_branch(snapshot, block, *condition, rule_set, diagnostics, seen)
            }
            None => {}
        }
    }
}

fn verify_block_expressions(
    snapshot: &ProgramSnapshot,
    block: &Block,
    rule_set: VerificationRuleSet,
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    for expression in block.expressions() {
        let id = expression.id();
        let inspection = expression_type(snapshot, id);
        let diagnostic = if arithmetic_operands(expression.kind()).is_some() {
            arithmetic_diagnostic(id, inspection)
        } else if rule_set != VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1
            && comparison_operands(expression.kind()).is_some()
        {
            comparison_diagnostic(id, inspection)
        } else {
            None
        };
        if let Some(diagnostic) = diagnostic {
            collect(diagnostic, diagnostics, seen);
        }

        if rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5 {
            verify_domain_expression(snapshot, expression.kind(), id, diagnostics, seen);
        }

        if matches!(
            rule_set,
            VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4
                | VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5
        ) && let ExpressionKind::Call { target, arguments } = expression.kind()
        {
            verify_call(
                snapshot,
                id,
                *target,
                arguments,
                rule_set,
                diagnostics,
                seen,
            );
        }
    }
}

fn verify_domain_expression(
    snapshot: &ProgramSnapshot,
    kind: &ExpressionKind,
    expression_id: ExpressionId,
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    let diagnostic = match kind {
        ExpressionKind::DomainConstruct { type_id, value } => {
            let representation = snapshot
                .domain_type(*type_id)
                .expect("structural validation guarantees Domain Type")
                .representation();
            match expression_type(snapshot, *value) {
                Ok(ValueType::Intrinsic(actual)) if actual == representation => None,
                Ok(actual_type) => Some(Diagnostic::DomainConstructRepresentationMismatch {
                    expression_id,
                    type_id: *type_id,
                    source_expression_id: *value,
                    expected_representation: representation,
                    actual_type,
                }),
                Err(_) => Some(Diagnostic::DomainConstructSourceTypeUnavailable {
                    expression_id,
                    type_id: *type_id,
                    source_expression_id: *value,
                }),
            }
        }
        ExpressionKind::DomainProject { value } => match expression_type(snapshot, expression_id) {
            Ok(_) => None,
            Err(ExpressionTypeError::DomainProjectSourceNotDomain { actual_type, .. }) => {
                Some(Diagnostic::DomainProjectSourceNotDomain {
                    expression_id,
                    source_expression_id: *value,
                    actual_type,
                })
            }
            Err(_) => Some(Diagnostic::DomainProjectSourceTypeUnavailable {
                expression_id,
                source_expression_id: *value,
            }),
        },
        _ => None,
    };
    if let Some(diagnostic) = diagnostic {
        collect(diagnostic, diagnostics, seen);
    }
}

fn arithmetic_diagnostic(
    expression_id: ExpressionId,
    inspection: Result<ValueType, ExpressionTypeError>,
) -> Option<Diagnostic> {
    match inspection {
        Ok(_) => None,
        Err(ExpressionTypeError::OperandTypeMismatch {
            left_type,
            right_type,
            ..
        }) => match (left_type, right_type) {
            (ValueType::Intrinsic(left_type), ValueType::Intrinsic(right_type)) => {
                Some(Diagnostic::ArithmeticOperandTypeMismatch {
                    expression_id,
                    left_type,
                    right_type,
                })
            }
            _ => Some(Diagnostic::ArithmeticOperandValueTypeMismatch {
                expression_id,
                left_type,
                right_type,
            }),
        },
        Err(ExpressionTypeError::UnsupportedOperandType { operand_type, .. }) => match operand_type
        {
            ValueType::Intrinsic(operand_type) => {
                Some(Diagnostic::ArithmeticUnsupportedOperandType {
                    expression_id,
                    operand_type,
                })
            }
            ValueType::Domain(_) => Some(Diagnostic::ArithmeticUnsupportedValueType {
                expression_id,
                operand_type,
            }),
        },
        Err(_) => Some(Diagnostic::ArithmeticOperandTypeUnavailable { expression_id }),
    }
}

fn comparison_diagnostic(
    expression_id: ExpressionId,
    inspection: Result<ValueType, ExpressionTypeError>,
) -> Option<Diagnostic> {
    match inspection {
        Ok(_) => None,
        Err(ExpressionTypeError::OperandTypeMismatch {
            left_type,
            right_type,
            ..
        }) => match (left_type, right_type) {
            (ValueType::Intrinsic(left_type), ValueType::Intrinsic(right_type)) => {
                Some(Diagnostic::ComparisonOperandTypeMismatch {
                    expression_id,
                    left_type,
                    right_type,
                })
            }
            _ => Some(Diagnostic::ComparisonOperandValueTypeMismatch {
                expression_id,
                left_type,
                right_type,
            }),
        },
        Err(ExpressionTypeError::UnsupportedOperandType { operand_type, .. }) => match operand_type
        {
            ValueType::Intrinsic(operand_type) => {
                Some(Diagnostic::ComparisonUnsupportedOperandType {
                    expression_id,
                    operand_type,
                })
            }
            ValueType::Domain(_) => Some(Diagnostic::ComparisonUnsupportedValueType {
                expression_id,
                operand_type,
            }),
        },
        Err(_) => Some(Diagnostic::ComparisonOperandTypeUnavailable { expression_id }),
    }
}

fn verify_branch(
    snapshot: &ProgramSnapshot,
    block: &Block,
    condition: ExpressionId,
    rule_set: VerificationRuleSet,
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    let diagnostic = match expression_type(snapshot, condition) {
        Ok(ValueType::Intrinsic(IntrinsicType::Bool)) => None,
        Ok(ValueType::Intrinsic(actual_type)) => Some(Diagnostic::BranchConditionNotBool {
            block_id: block.id(),
            condition_expression_id: condition,
            actual_type,
        }),
        Ok(actual_type @ ValueType::Domain(_))
            if rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5 =>
        {
            Some(Diagnostic::BranchConditionValueTypeNotBool {
                block_id: block.id(),
                condition_expression_id: condition,
                actual_type,
            })
        }
        Ok(ValueType::Domain(_)) | Err(_) => Some(Diagnostic::BranchConditionTypeUnavailable {
            block_id: block.id(),
            condition_expression_id: condition,
        }),
    };
    if let Some(diagnostic) = diagnostic {
        collect(diagnostic, diagnostics, seen);
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_return(
    snapshot: &ProgramSnapshot,
    function: &Function,
    block: &Block,
    return_expression_id: ExpressionId,
    multi_block: bool,
    rule_set: VerificationRuleSet,
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    let Ok(actual_type) = expression_type(snapshot, return_expression_id) else {
        return;
    };
    let expected_type = *function.return_type();
    if actual_type == expected_type {
        return;
    }

    let diagnostic = match (multi_block, expected_type, actual_type) {
        (false, ValueType::Intrinsic(expected_type), ValueType::Intrinsic(actual_type)) => {
            Diagnostic::ReturnTypeMismatch {
                function_id: function.id(),
                return_expression_id,
                expected_type,
                actual_type,
            }
        }
        (true, ValueType::Intrinsic(expected_type), ValueType::Intrinsic(actual_type)) => {
            Diagnostic::ControlFlowReturnTypeMismatch {
                function_id: function.id(),
                block_id: block.id(),
                return_expression_id,
                expected_type,
                actual_type,
            }
        }
        (false, expected_type, actual_type)
            if rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5 =>
        {
            Diagnostic::ReturnValueTypeMismatch {
                function_id: function.id(),
                return_expression_id,
                expected_type,
                actual_type,
            }
        }
        (true, expected_type, actual_type)
            if rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5 =>
        {
            Diagnostic::ControlFlowReturnValueTypeMismatch {
                function_id: function.id(),
                block_id: block.id(),
                return_expression_id,
                expected_type,
                actual_type,
            }
        }
        _ => return,
    };
    collect(diagnostic, diagnostics, seen);
}

#[allow(clippy::too_many_arguments)]
fn verify_call(
    snapshot: &ProgramSnapshot,
    expression_id: ExpressionId,
    target_function_id: mnir_core::FunctionId,
    arguments: &[ExpressionId],
    rule_set: VerificationRuleSet,
    diagnostics: &mut Vec<Diagnostic>,
    seen: &mut HashSet<(DiagnosticCode, DiagnosticPrimarySubject)>,
) {
    let target = snapshot
        .function(target_function_id)
        .expect("structural validation guarantees Call target");
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
    for (argument_index, (&argument_id, parameter)) in
        arguments.iter().zip(target.parameters()).enumerate()
    {
        match expression_type(snapshot, argument_id) {
            Ok(actual_type) if actual_type != *parameter.value_type() => {
                mismatches.push(CallArgumentValueTypeMismatch {
                    argument_index,
                    expected_type: *parameter.value_type(),
                    actual_type,
                });
            }
            Ok(_) => {}
            Err(_) => unavailable.push(argument_index),
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
    if mismatches.is_empty() {
        return;
    }

    let all_intrinsic = mismatches.iter().all(|mismatch| {
        matches!(mismatch.expected_type, ValueType::Intrinsic(_))
            && matches!(mismatch.actual_type, ValueType::Intrinsic(_))
    });
    if all_intrinsic {
        let mismatches = mismatches
            .into_iter()
            .map(|mismatch| CallArgumentTypeMismatch {
                argument_index: mismatch.argument_index,
                expected_type: intrinsic(mismatch.expected_type),
                actual_type: intrinsic(mismatch.actual_type),
            })
            .collect();
        collect(
            Diagnostic::CallArgumentTypeMismatch {
                expression_id,
                function_id: target_function_id,
                mismatches,
            },
            diagnostics,
            seen,
        );
    } else if rule_set == VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5 {
        collect(
            Diagnostic::CallArgumentValueTypeMismatch {
                expression_id,
                function_id: target_function_id,
                mismatches,
            },
            diagnostics,
            seen,
        );
    }
}

fn expression_type(
    snapshot: &ProgramSnapshot,
    expression_id: ExpressionId,
) -> Result<ValueType, ExpressionTypeError> {
    snapshot
        .expression_type(expression_id)
        .expect("structural validation guarantees Expression")
}

fn intrinsic(value_type: ValueType) -> IntrinsicType {
    match value_type {
        ValueType::Intrinsic(intrinsic) => intrinsic,
        ValueType::Domain(_) => unreachable!("caller established intrinsic ValueType"),
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

fn arithmetic_operands(kind: &ExpressionKind) -> Option<(ExpressionId, ExpressionId)> {
    match kind {
        ExpressionKind::Add { left, right }
        | ExpressionKind::Subtract { left, right }
        | ExpressionKind::Multiply { left, right }
        | ExpressionKind::Divide { left, right }
        | ExpressionKind::Remainder { left, right } => Some((*left, *right)),
        _ => None,
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
        _ => None,
    }
}
