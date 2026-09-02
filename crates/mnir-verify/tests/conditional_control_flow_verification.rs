use mnir_core::{IntrinsicType, MnirProgram};
use mnir_verify::{
    Diagnostic, DiagnosticCode, DiagnosticPrimarySubject, VerificationError, VerificationRuleSet,
    verify_with_rule_set,
};

fn valid_conditional_snapshot() -> mnir_core::ProgramSnapshot {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let true_block = transaction.add_block(function).unwrap();
    let false_block = transaction.add_block(function).unwrap();
    let condition = transaction.add_bool_literal(entry, true).unwrap();
    transaction
        .set_branch(entry, condition, true_block, false_block)
        .unwrap();
    for block in [true_block, false_block] {
        let unit = transaction.add_unit_literal(block).unwrap();
        transaction.set_return(block, unit).unwrap();
    }
    transaction.commit().unwrap()
}

// AR-CFG-024 through AR-CFG-027, AR-CFG-030, and AR-CFG-044.
#[test]
fn rule_set_applicability_and_v0_3_binding_are_explicit() {
    let snapshot = valid_conditional_snapshot();
    for rule_set in [
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2,
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
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
    )
    .unwrap();
    assert_eq!(
        verified.rule_set(),
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3
    );
    assert_ne!(
        verified.rule_set(),
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1
    );
    assert_ne!(
        verified.rule_set(),
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2
    );

    let legacy_snapshot = MnirProgram::new().unwrap().snapshot();
    for legacy_rule_set in [
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2,
    ] {
        let legacy_evidence = verify_with_rule_set(&legacy_snapshot, legacy_rule_set).unwrap();
        assert_eq!(legacy_evidence.rule_set(), legacy_rule_set);
        assert_ne!(
            legacy_evidence.rule_set(),
            VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3
        );
    }
}

// AR-CFG-028 and MNIR-CFG-083/-084/-087/-088/-089.
#[test]
fn non_bool_branch_reports_diag_009_with_block_subject() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let target = transaction.add_block(function).unwrap();
    let condition = transaction.add_int32_literal(entry, 1).unwrap();
    transaction
        .set_branch(entry, condition, target, target)
        .unwrap();
    let unit = transaction.add_unit_literal(target).unwrap();
    transaction.set_return(target, unit).unwrap();
    let snapshot = transaction.commit().unwrap();

    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
    )
    .unwrap_err();
    let diagnostics = error.diagnostics().unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert!(matches!(
        &diagnostics[0],
        Diagnostic::BranchConditionNotBool {
            block_id,
            condition_expression_id,
            actual_type: IntrinsicType::Int32,
        } if *block_id == entry && *condition_expression_id == condition
    ));
    assert_eq!(
        diagnostics[0].primary_subject(),
        DiagnosticPrimarySubject::Block(entry)
    );
    assert_eq!(diagnostics[0].code().as_str(), "MNIR-DIAG-009");
}

// AR-CFG-029 and MNIR-CFG-085/-086/-089/-098.
#[test]
fn unavailable_branch_reports_underlying_expression_and_diag_008() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let target = transaction.add_block(function).unwrap();
    let left = transaction.add_bool_literal(entry, true).unwrap();
    let right = transaction.add_bool_literal(entry, false).unwrap();
    let condition = transaction.add_add_expression(entry, left, right).unwrap();
    transaction
        .set_branch(entry, condition, target, target)
        .unwrap();
    let unit = transaction.add_unit_literal(target).unwrap();
    transaction.set_return(target, unit).unwrap();
    let snapshot = transaction.commit().unwrap();

    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
    )
    .unwrap_err();
    let diagnostics = error.diagnostics().unwrap();
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticUnsupportedOperandType { expression_id, .. }
            if *expression_id == condition
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::BranchConditionTypeUnavailable {
            block_id,
            condition_expression_id,
        } if *block_id == entry && *condition_expression_id == condition
    )));
    assert!(
        !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::BranchConditionNotBool)
    );
}

// AR-CFG-030 through AR-CFG-034 and MNIR-CFG-090 through -094.
#[test]
fn return_diagnostics_are_block_scoped_only_for_multi_block_bodies() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Int32)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let first_return = transaction.add_block(function).unwrap();
    let second_return = transaction.add_block(function).unwrap();
    let condition = transaction.add_bool_literal(entry, true).unwrap();
    transaction
        .set_branch(entry, condition, first_return, second_return)
        .unwrap();
    let bool_value = transaction.add_bool_literal(first_return, true).unwrap();
    transaction.set_return(first_return, bool_value).unwrap();
    let unit_value = transaction.add_unit_literal(second_return).unwrap();
    transaction.set_return(second_return, unit_value).unwrap();
    let snapshot = transaction.commit().unwrap();

    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
    )
    .unwrap_err();
    let diagnostics = error.diagnostics().unwrap();
    assert_eq!(diagnostics.len(), 2);
    for (block, expression, actual_type) in [
        (first_return, bool_value, IntrinsicType::Bool),
        (second_return, unit_value, IntrinsicType::Unit),
    ] {
        assert!(diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            Diagnostic::ControlFlowReturnTypeMismatch {
                function_id,
                block_id,
                return_expression_id,
                expected_type: IntrinsicType::Int32,
                actual_type: found_type,
            } if *function_id == function && *block_id == block
                && *return_expression_id == expression && found_type == &actual_type
        )));
    }

    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Int32)
        .unwrap();
    let block = transaction.create_function_body(function).unwrap();
    let returned = transaction.add_bool_literal(block, true).unwrap();
    transaction.set_return(block, returned).unwrap();
    let snapshot = transaction.commit().unwrap();
    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
    )
    .unwrap_err();
    assert!(matches!(
        error.diagnostics().unwrap(),
        [Diagnostic::ReturnTypeMismatch { .. }]
    ));
}

// AR-CFG-033: unavailable Return type leaves the underlying Expression as the
// sole owner of the semantic failure.
#[test]
fn unavailable_return_suppresses_control_flow_return_mismatch() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Int32)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let target = transaction.add_block(function).unwrap();
    let condition = transaction.add_bool_literal(entry, true).unwrap();
    transaction
        .set_branch(entry, condition, target, target)
        .unwrap();
    let left = transaction.add_bool_literal(target, true).unwrap();
    let right = transaction.add_bool_literal(target, false).unwrap();
    let invalid = transaction.add_add_expression(target, left, right).unwrap();
    transaction.set_return(target, invalid).unwrap();
    let snapshot = transaction.commit().unwrap();

    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
    )
    .unwrap_err();
    let diagnostics = error.diagnostics().unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code(),
        DiagnosticCode::ArithmeticUnsupportedOperandType
    );
}

// AR-CFG-035: the specification's target max Function.
#[test]
fn max_function_is_structurally_and_semantically_valid() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Int32)
        .unwrap();
    let a = transaction
        .add_parameter(function, IntrinsicType::Int32)
        .unwrap();
    let b = transaction
        .add_parameter(function, IntrinsicType::Int32)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let true_block = transaction.add_block(function).unwrap();
    let false_block = transaction.add_block(function).unwrap();
    let entry_a = transaction.add_parameter_reference(entry, a).unwrap();
    let entry_b = transaction.add_parameter_reference(entry, b).unwrap();
    let condition = transaction
        .add_greater_than_expression(entry, entry_a, entry_b)
        .unwrap();
    transaction
        .set_branch(entry, condition, true_block, false_block)
        .unwrap();
    let returned_a = transaction.add_parameter_reference(true_block, a).unwrap();
    transaction.set_return(true_block, returned_a).unwrap();
    let returned_b = transaction.add_parameter_reference(false_block, b).unwrap();
    transaction.set_return(false_block, returned_b).unwrap();
    let snapshot = transaction.commit().unwrap();

    verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
    )
    .unwrap();
}

// AR-CFG-036, AR-CFG-037, and MNIR-CFG-095 through -102.
#[test]
fn v0_3_traverses_every_module_function_block_and_successor() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    for literal_condition in [true, false] {
        let module = transaction.add_module().unwrap();
        let function = transaction
            .add_function(module, IntrinsicType::Int32)
            .unwrap();
        let entry = transaction.create_function_body(function).unwrap();
        let true_block = transaction.add_block(function).unwrap();
        let false_block = transaction.add_block(function).unwrap();
        let condition = transaction
            .add_bool_literal(entry, literal_condition)
            .unwrap();
        transaction
            .set_branch(entry, condition, true_block, false_block)
            .unwrap();

        let left = transaction.add_bool_literal(true_block, true).unwrap();
        let right = transaction.add_bool_literal(true_block, false).unwrap();
        transaction
            .add_add_expression(true_block, left, right)
            .unwrap();
        let mismatching_bool = transaction.add_bool_literal(true_block, true).unwrap();
        transaction
            .set_return(true_block, mismatching_bool)
            .unwrap();

        let int_value = transaction.add_int32_literal(false_block, 1).unwrap();
        let bool_value = transaction.add_bool_literal(false_block, true).unwrap();
        transaction
            .add_equal_expression(false_block, int_value, bool_value)
            .unwrap();
        let mismatching_unit = transaction.add_unit_literal(false_block).unwrap();
        transaction
            .set_return(false_block, mismatching_unit)
            .unwrap();
    }
    let snapshot = transaction.commit().unwrap();
    let error = verify_with_rule_set(
        &snapshot,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
    )
    .unwrap_err();
    let diagnostics = error.diagnostics().unwrap();
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code() == DiagnosticCode::ArithmeticUnsupportedOperandType
            })
            .count(),
        2
    );
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code() == DiagnosticCode::ComparisonOperandTypeMismatch
            })
            .count(),
        2
    );
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code() == DiagnosticCode::ControlFlowReturnTypeMismatch
            })
            .count(),
        4
    );
}

// AR-CFG-045: unsupported CFG in any Module wins before semantic work in all
// older rule sets, independent of collection iteration order.
#[test]
fn old_rule_sets_scan_all_modules_before_emitting_diagnostics() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let invalid_module = transaction.add_module().unwrap();
    let invalid_function = transaction
        .add_function(invalid_module, IntrinsicType::Int32)
        .unwrap();
    let invalid_block = transaction.create_function_body(invalid_function).unwrap();
    let left = transaction.add_bool_literal(invalid_block, true).unwrap();
    let right = transaction.add_bool_literal(invalid_block, false).unwrap();
    let bad = transaction
        .add_add_expression(invalid_block, left, right)
        .unwrap();
    transaction.set_return(invalid_block, bad).unwrap();

    let cfg_module = transaction.add_module().unwrap();
    let cfg_function = transaction
        .add_function(cfg_module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(cfg_function).unwrap();
    let target = transaction.add_block(cfg_function).unwrap();
    let condition = transaction.add_bool_literal(entry, true).unwrap();
    transaction
        .set_branch(entry, condition, target, target)
        .unwrap();
    let unit = transaction.add_unit_literal(target).unwrap();
    transaction.set_return(target, unit).unwrap();
    let snapshot = transaction.commit().unwrap();

    for rule_set in [
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2,
    ] {
        let error = verify_with_rule_set(&snapshot, rule_set).unwrap_err();
        assert!(matches!(
            error,
            VerificationError::RuleSetNotApplicable {
                requested_rule_set
            } if requested_rule_set == rule_set
        ));
        assert!(error.diagnostics().is_none());
    }
}
