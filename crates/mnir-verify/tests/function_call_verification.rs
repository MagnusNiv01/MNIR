use std::collections::HashSet;

use mnir_core::{IntrinsicType, MnirProgram};
use mnir_verify::{
    CallArgumentTypeMismatch, Diagnostic, DiagnosticCode, DiagnosticPrimarySubject,
    DiagnosticSeverity, VerificationError, VerificationRuleSet, verify_with_rule_set,
};

// AR-CALL-035, AR-CALL-044 through AR-CALL-047, AR-CALL-049, AR-CALL-050,
// and MNIR-CALL-086 through MNIR-CALL-094. The bodyless target verifies
// without execution or purity inference and remains explicitly sequenced.
#[test]
fn applicability_and_v0_4_evidence_binding_are_explicit() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let target = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(caller).unwrap();
    let call = tx.add_call_expression(block, target, Vec::new()).unwrap();
    tx.set_effect_sequence(block, vec![call]).unwrap();
    tx.set_return(block, call).unwrap();
    let snapshot = tx.commit().unwrap();

    for rule_set in [
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
    ] {
        let error = verify_with_rule_set(&snapshot, rule_set).unwrap_err();
        assert_eq!(
            error,
            VerificationError::RuleSetNotApplicable {
                requested_rule_set: rule_set,
            }
        );
        assert!(error.diagnostics().is_none());
    }

    let verified = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4,
    )
    .unwrap();
    assert_eq!(verified.program_id(), snapshot.program_id());
    assert_eq!(verified.revision_id(), snapshot.revision_id());
    assert_eq!(
        verified.rule_set(),
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4
    );

    let legacy_snapshot = MnirProgram::new().unwrap().snapshot();
    let legacy = verify_with_rule_set(
        &legacy_snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
    )
    .unwrap();
    assert_ne!(legacy.rule_set(), verified.rule_set());
}

// AR-CALL-036 through AR-CALL-041, AR-CALL-054, and AR-CALL-058.
#[test]
fn call_diagnostics_have_normative_target_and_ordered_zero_based_payloads() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let target = tx.add_function(module, IntrinsicType::Unit).unwrap();
    tx.add_parameter(target, IntrinsicType::Int32).unwrap();
    tx.add_parameter(target, IntrinsicType::Int64).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(caller).unwrap();
    let bool_left = tx.add_bool_literal(block, true).unwrap();
    let bool_right = tx.add_bool_literal(block, false).unwrap();
    let unavailable = tx.add_add_expression(block, bool_left, bool_right).unwrap();
    let mismatching = tx.add_int32_literal(block, 7).unwrap();
    let extra = tx.add_bool_literal(block, true).unwrap();
    let call = tx
        .add_call_expression(block, target, vec![unavailable, mismatching, extra])
        .unwrap();
    tx.set_effect_sequence(block, vec![call]).unwrap();
    tx.set_return(block, call).unwrap();
    let snapshot = tx.commit().unwrap();

    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4,
    )
    .unwrap_err();
    let diagnostics = error.diagnostics().unwrap();
    let codes: HashSet<_> = diagnostics.iter().map(Diagnostic::code).collect();
    assert!(codes.contains(&DiagnosticCode::ArithmeticUnsupportedOperandType));
    assert!(codes.contains(&DiagnosticCode::CallArgumentCountMismatch));
    assert!(codes.contains(&DiagnosticCode::CallArgumentTypeUnavailable));
    assert!(codes.contains(&DiagnosticCode::CallArgumentTypeMismatch));

    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::CallArgumentCountMismatch {
            expression_id,
            function_id,
            expected_count: 2,
            actual_count: 3,
        } if *expression_id == call && *function_id == target && *function_id != caller
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::CallArgumentTypeUnavailable {
            expression_id,
            function_id,
            argument_indices,
        } if *expression_id == call && *function_id == target && argument_indices == &[0]
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::CallArgumentTypeMismatch {
            expression_id,
            function_id,
            mismatches,
        } if *expression_id == call
            && *function_id == target
            && mismatches == &[CallArgumentTypeMismatch {
                argument_index: 1,
                expected_type: IntrinsicType::Int64,
                actual_type: IntrinsicType::Int32,
            }]
    )));
    for diagnostic in diagnostics.iter().filter(|diagnostic| {
        matches!(
            diagnostic.code(),
            DiagnosticCode::CallArgumentCountMismatch
                | DiagnosticCode::CallArgumentTypeUnavailable
                | DiagnosticCode::CallArgumentTypeMismatch
        )
    }) {
        assert_eq!(diagnostic.severity(), DiagnosticSeverity::Error);
        assert_eq!(
            diagnostic.primary_subject(),
            DiagnosticPrimarySubject::Expression(call)
        );
    }
    assert_eq!(
        DiagnosticCode::CallArgumentCountMismatch.as_str(),
        "MNIR-DIAG-011"
    );
    assert_eq!(
        DiagnosticCode::CallArgumentTypeUnavailable.as_str(),
        "MNIR-DIAG-012"
    );
    assert_eq!(
        DiagnosticCode::CallArgumentTypeMismatch.as_str(),
        "MNIR-DIAG-013"
    );
}

// AR-CALL-042: absent positions have no argument Expression and therefore no
// unavailable/mismatch diagnostic.
#[test]
fn missing_arguments_only_report_count_mismatch() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let target = tx.add_function(module, IntrinsicType::Unit).unwrap();
    tx.add_parameter(target, IntrinsicType::Int32).unwrap();
    tx.add_parameter(target, IntrinsicType::Bool).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(caller).unwrap();
    let argument = tx.add_int32_literal(block, 1).unwrap();
    let call = tx
        .add_call_expression(block, target, vec![argument])
        .unwrap();
    tx.set_effect_sequence(block, vec![call]).unwrap();
    tx.set_return(block, call).unwrap();
    let snapshot = tx.commit().unwrap();

    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4,
    )
    .unwrap_err();
    assert!(matches!(
        error.diagnostics().unwrap(),
        [Diagnostic::CallArgumentCountMismatch {
            expression_id,
            function_id,
            expected_count: 2,
            actual_count: 1,
        }] if *expression_id == call && *function_id == target
    ));
}

// AR-CALL-043 and MNIR-CALL-110 through MNIR-CALL-112. Argument diagnostics
// do not invalidate the Call's target-derived result type for arithmetic or
// Return verification.
#[test]
fn invalid_arguments_do_not_hide_a_valid_call_result_type() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let target = tx.add_function(module, IntrinsicType::Int32).unwrap();
    tx.add_parameter(target, IntrinsicType::Bool).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Int32).unwrap();
    let block = tx.create_function_body(caller).unwrap();
    let argument = tx.add_int32_literal(block, 1).unwrap();
    let call = tx
        .add_call_expression(block, target, vec![argument])
        .unwrap();
    tx.set_effect_sequence(block, vec![call]).unwrap();
    let one = tx.add_int32_literal(block, 1).unwrap();
    let sum = tx.add_add_expression(block, call, one).unwrap();
    tx.set_return(block, sum).unwrap();
    let snapshot = tx.commit().unwrap();

    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4,
    )
    .unwrap_err();
    assert!(matches!(
        error.diagnostics().unwrap(),
        [Diagnostic::CallArgumentTypeMismatch { expression_id, .. }]
            if *expression_id == call
    ));
}

// MNIR-CALL-111: inherited Branch verification consumes the declared result
// type of a Call without executing it.
#[test]
fn call_result_type_participates_in_branch_verification() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let target = tx.add_function(module, IntrinsicType::Int32).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let entry = tx.create_function_body(caller).unwrap();
    let successor = tx.add_block(caller).unwrap();
    let call = tx.add_call_expression(entry, target, Vec::new()).unwrap();
    tx.set_effect_sequence(entry, vec![call]).unwrap();
    tx.set_branch(entry, call, successor, successor).unwrap();
    let unit = tx.add_unit_literal(successor).unwrap();
    tx.set_return(successor, unit).unwrap();
    let snapshot = tx.commit().unwrap();

    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4,
    )
    .unwrap_err();
    assert!(matches!(
        error.diagnostics().unwrap(),
        [Diagnostic::BranchConditionNotBool {
            block_id,
            condition_expression_id,
            actual_type: IntrinsicType::Int32,
        }] if *block_id == entry && *condition_expression_id == call
    ));
}

// AR-CALL-048 and AR-CALL-058: V0_4 traverses every Module, Function, and
// Block. The test compares a set because diagnostic collection order is not
// semantic.
#[test]
fn v0_4_traverses_calls_across_modules_functions_and_blocks() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let targets_module = tx.add_module().unwrap();
    let callers_module = tx.add_module().unwrap();
    let target = tx
        .add_function(targets_module, IntrinsicType::Bool)
        .unwrap();
    tx.add_parameter(target, IntrinsicType::Bool).unwrap();

    let first_caller = tx
        .add_function(callers_module, IntrinsicType::Unit)
        .unwrap();
    let first_block = tx.create_function_body(first_caller).unwrap();
    let first_call = tx
        .add_call_expression(first_block, target, Vec::new())
        .unwrap();
    tx.set_effect_sequence(first_block, vec![first_call])
        .unwrap();
    let unit = tx.add_unit_literal(first_block).unwrap();
    tx.set_return(first_block, unit).unwrap();

    let second_caller = tx
        .add_function(callers_module, IntrinsicType::Unit)
        .unwrap();
    let entry = tx.create_function_body(second_caller).unwrap();
    let successor = tx.add_block(second_caller).unwrap();
    let condition = tx.add_bool_literal(entry, true).unwrap();
    tx.set_branch(entry, condition, successor, successor)
        .unwrap();
    let second_call = tx
        .add_call_expression(successor, target, Vec::new())
        .unwrap();
    tx.set_effect_sequence(successor, vec![second_call])
        .unwrap();
    let unit = tx.add_unit_literal(successor).unwrap();
    tx.set_return(successor, unit).unwrap();
    let snapshot = tx.commit().unwrap();

    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4,
    )
    .unwrap_err();
    let diagnosed_calls: HashSet<_> = error
        .diagnostics()
        .unwrap()
        .iter()
        .filter_map(|diagnostic| match diagnostic {
            Diagnostic::CallArgumentCountMismatch { expression_id, .. } => Some(*expression_id),
            _ => None,
        })
        .collect();
    assert_eq!(diagnosed_calls, HashSet::from([first_call, second_call]));
}
