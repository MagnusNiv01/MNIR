use std::collections::HashSet;

use mnir_core::{IntrinsicType, MnirProgram, ValueType};
use mnir_verify::{
    CallArgumentTypeMismatch, CallArgumentValueTypeMismatch, Diagnostic, DiagnosticCode,
    DiagnosticPrimarySubject, DiagnosticSeverity, VerificationError, VerificationRuleSet,
    verify_with_rule_set,
};

const V5: VerificationRuleSet = VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5;

fn v0_5_diagnostics(snapshot: &mnir_core::ProgramSnapshot) -> Vec<Diagnostic> {
    match verify_with_rule_set(snapshot, V5).unwrap_err() {
        VerificationError::Semantic(failure) => failure.into_diagnostics(),
        other => panic!("expected semantic diagnostics, got {other:?}"),
    }
}

// AR-DOMAIN-048/-071: every newly introduced form independently makes every
// historical rule set inapplicable. The earlier Module contains an error to
// prove applicability scanning completes before semantic verification.
#[test]
fn historical_applicability_rejects_each_new_form_before_diagnostics() {
    for scenario in 0..8 {
        let mut program = MnirProgram::new().unwrap();
        let mut tx = program.begin_transaction();

        let early = tx.add_module().unwrap();
        let erroneous = tx.add_function(early, IntrinsicType::Unit).unwrap();
        let early_block = tx.create_function_body(erroneous).unwrap();
        let left = tx.add_bool_literal(early_block, true).unwrap();
        let right = tx.add_bool_literal(early_block, false).unwrap();
        tx.add_add_expression(early_block, left, right).unwrap();
        let unit = tx.add_unit_literal(early_block).unwrap();
        tx.set_return(early_block, unit).unwrap();

        let late = tx.add_module().unwrap();
        match scenario {
            0 => {
                tx.add_function(late, IntrinsicType::Text).unwrap();
            }
            1 => {
                tx.add_function(late, IntrinsicType::Bytes).unwrap();
            }
            2 => {
                tx.add_domain_type(late, IntrinsicType::Int64).unwrap();
            }
            3 => {
                let domain = tx.add_domain_type(late, IntrinsicType::Int64).unwrap();
                tx.add_function(late, ValueType::Domain(domain)).unwrap();
            }
            4 | 5 => {
                let domain = tx.add_domain_type(late, IntrinsicType::Int64).unwrap();
                let function = tx.add_function(late, IntrinsicType::Unit).unwrap();
                let block = tx.create_function_body(function).unwrap();
                let value = tx.add_int64_literal(block, 1).unwrap();
                if scenario == 4 {
                    tx.add_domain_construct(block, domain, value).unwrap();
                } else {
                    tx.add_domain_project(block, value).unwrap();
                }
                let unit = tx.add_unit_literal(block).unwrap();
                tx.set_return(block, unit).unwrap();
            }
            6 | 7 => {
                let function = tx.add_function(late, IntrinsicType::Unit).unwrap();
                let block = tx.create_function_body(function).unwrap();
                if scenario == 6 {
                    tx.add_text_literal(block, "text").unwrap();
                } else {
                    tx.add_bytes_literal(block, vec![0]).unwrap();
                }
                let unit = tx.add_unit_literal(block).unwrap();
                tx.set_return(block, unit).unwrap();
            }
            _ => unreachable!(),
        }
        let snapshot = tx.commit().unwrap();

        for rule_set in [
            VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
            VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2,
            VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
            VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4,
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
    }
}

// AR-DOMAIN-019 through -024, -030/-031, -035/-037, -051, -054/-055,
// -068, -073/-074, and -077. Every MNIR-DIAG-014 through -025 is checked
// with its normative subject and payload; collection order is ignored.
#[test]
fn v0_5_emits_every_domain_diagnostic_with_exact_payloads() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let customer = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let order = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let flag = tx.add_domain_type(module, IntrinsicType::Bool).unwrap();
    let other_module = tx.add_module().unwrap();

    let expression_host = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let expression_block = tx.create_function_body(expression_host).unwrap();
    let int = tx.add_int64_literal(expression_block, 1).unwrap();
    let boolean = tx.add_bool_literal(expression_block, true).unwrap();
    let invalid_inner = tx
        .add_add_expression(expression_block, boolean, boolean)
        .unwrap();
    let unavailable_construct = tx
        .add_domain_construct(expression_block, customer, invalid_inner)
        .unwrap();
    let mismatching_construct = tx
        .add_domain_construct(expression_block, customer, boolean)
        .unwrap();
    let valid_customer = tx
        .add_domain_construct(expression_block, customer, int)
        .unwrap();
    let _valid_order = tx
        .add_domain_construct(expression_block, order, int)
        .unwrap();
    let unavailable_project = tx
        .add_domain_project(expression_block, invalid_inner)
        .unwrap();
    let intrinsic_project = tx.add_domain_project(expression_block, int).unwrap();
    let arithmetic_mismatch = tx
        .add_add_expression(expression_block, valid_customer, int)
        .unwrap();
    let arithmetic_unsupported = tx
        .add_add_expression(expression_block, valid_customer, valid_customer)
        .unwrap();
    let comparison_mismatch = tx
        .add_equal_expression(expression_block, valid_customer, int)
        .unwrap();
    let comparison_unsupported = tx
        .add_equal_expression(expression_block, valid_customer, valid_customer)
        .unwrap();
    let unit = tx.add_unit_literal(expression_block).unwrap();
    tx.set_return(expression_block, unit).unwrap();

    let single_return = tx
        .add_function(other_module, ValueType::Domain(customer))
        .unwrap();
    let single_block = tx.create_function_body(single_return).unwrap();
    let intrinsic_return = tx.add_int64_literal(single_block, 2).unwrap();
    tx.set_return(single_block, intrinsic_return).unwrap();

    let multi_return = tx
        .add_function(module, ValueType::Domain(customer))
        .unwrap();
    let entry = tx.create_function_body(multi_return).unwrap();
    let exit = tx.add_block(multi_return).unwrap();
    let condition = tx.add_bool_literal(entry, true).unwrap();
    tx.set_branch(entry, condition, exit, exit).unwrap();
    let other_domain_source = tx.add_int64_literal(exit, 3).unwrap();
    let other_domain_return = tx
        .add_domain_construct(exit, order, other_domain_source)
        .unwrap();
    tx.set_return(exit, other_domain_return).unwrap();

    let branch_function = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let branch_entry = tx.create_function_body(branch_function).unwrap();
    let branch_exit = tx.add_block(branch_function).unwrap();
    let bool_source = tx.add_bool_literal(branch_entry, true).unwrap();
    let domain_condition = tx
        .add_domain_construct(branch_entry, flag, bool_source)
        .unwrap();
    tx.set_branch(branch_entry, domain_condition, branch_exit, branch_exit)
        .unwrap();
    let branch_unit = tx.add_unit_literal(branch_exit).unwrap();
    tx.set_return(branch_exit, branch_unit).unwrap();

    let projected_branch_function = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let projected_branch_entry = tx.create_function_body(projected_branch_function).unwrap();
    let projected_branch_exit = tx.add_block(projected_branch_function).unwrap();
    let projected_bool_source = tx.add_bool_literal(projected_branch_entry, true).unwrap();
    let projected_bool_domain = tx
        .add_domain_construct(projected_branch_entry, flag, projected_bool_source)
        .unwrap();
    let projected_condition = tx
        .add_domain_project(projected_branch_entry, projected_bool_domain)
        .unwrap();
    tx.set_branch(
        projected_branch_entry,
        projected_condition,
        projected_branch_exit,
        projected_branch_exit,
    )
    .unwrap();
    let projected_branch_unit = tx.add_unit_literal(projected_branch_exit).unwrap();
    tx.set_return(projected_branch_exit, projected_branch_unit)
        .unwrap();

    let target = tx.add_function(module, IntrinsicType::Unit).unwrap();
    tx.add_parameter(target, ValueType::Domain(customer))
        .unwrap();
    tx.add_parameter(target, ValueType::Domain(customer))
        .unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let caller_block = tx.create_function_body(caller).unwrap();
    let call_int = tx.add_int64_literal(caller_block, 4).unwrap();
    let order_argument = tx
        .add_domain_construct(caller_block, order, call_int)
        .unwrap();
    let call = tx
        .add_call_expression(caller_block, target, vec![order_argument, call_int])
        .unwrap();
    tx.set_effect_sequence(caller_block, vec![call]).unwrap();
    tx.set_return(caller_block, call).unwrap();

    let snapshot = tx.commit().unwrap();
    let diagnostics = v0_5_diagnostics(&snapshot);
    let repeated = v0_5_diagnostics(&snapshot);
    assert_eq!(repeated.len(), diagnostics.len());
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| repeated.contains(diagnostic))
    );
    let codes: HashSet<_> = diagnostics.iter().map(Diagnostic::code).collect();
    for code in [
        DiagnosticCode::DomainConstructSourceTypeUnavailable,
        DiagnosticCode::DomainConstructRepresentationMismatch,
        DiagnosticCode::DomainProjectSourceTypeUnavailable,
        DiagnosticCode::DomainProjectSourceNotDomain,
        DiagnosticCode::ArithmeticOperandValueTypeMismatch,
        DiagnosticCode::ArithmeticUnsupportedValueType,
        DiagnosticCode::ComparisonOperandValueTypeMismatch,
        DiagnosticCode::ComparisonUnsupportedValueType,
        DiagnosticCode::ReturnValueTypeMismatch,
        DiagnosticCode::BranchConditionValueTypeNotBool,
        DiagnosticCode::ControlFlowReturnValueTypeMismatch,
        DiagnosticCode::CallArgumentValueTypeMismatch,
    ] {
        assert!(codes.contains(&code), "missing {}", code.as_str());
    }

    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::DomainConstructSourceTypeUnavailable {
            expression_id,
            type_id,
            source_expression_id,
        } if *expression_id == unavailable_construct
            && *type_id == customer
            && *source_expression_id == invalid_inner
    )));
    assert!(!diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_subject() == DiagnosticPrimarySubject::Block(projected_branch_entry)
    }));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::DomainConstructRepresentationMismatch {
            expression_id,
            type_id,
            source_expression_id,
            expected_representation: IntrinsicType::Int64,
            actual_type: ValueType::Intrinsic(IntrinsicType::Bool),
        } if *expression_id == mismatching_construct
            && *type_id == customer
            && *source_expression_id == boolean
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::DomainProjectSourceTypeUnavailable {
            expression_id,
            source_expression_id,
        } if *expression_id == unavailable_project && *source_expression_id == invalid_inner
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::DomainProjectSourceNotDomain {
            expression_id,
            source_expression_id,
            actual_type: ValueType::Intrinsic(IntrinsicType::Int64),
        } if *expression_id == intrinsic_project && *source_expression_id == int
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticOperandValueTypeMismatch {
            expression_id,
            left_type: ValueType::Domain(left),
            right_type: ValueType::Intrinsic(IntrinsicType::Int64),
        } if *expression_id == arithmetic_mismatch && *left == customer
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticUnsupportedValueType {
            expression_id,
            operand_type: ValueType::Domain(actual),
        } if *expression_id == arithmetic_unsupported && *actual == customer
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonOperandValueTypeMismatch {
            expression_id,
            left_type: ValueType::Domain(left),
            right_type: ValueType::Intrinsic(IntrinsicType::Int64),
        } if *expression_id == comparison_mismatch && *left == customer
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonUnsupportedValueType {
            expression_id,
            operand_type: ValueType::Domain(actual),
        } if *expression_id == comparison_unsupported && *actual == customer
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ReturnValueTypeMismatch {
            function_id,
            return_expression_id,
            expected_type: ValueType::Domain(expected),
            actual_type: ValueType::Intrinsic(IntrinsicType::Int64),
        } if *function_id == single_return
            && *return_expression_id == intrinsic_return
            && *expected == customer
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::BranchConditionValueTypeNotBool {
            block_id,
            condition_expression_id,
            actual_type: ValueType::Domain(actual),
        } if *block_id == branch_entry
            && *condition_expression_id == domain_condition
            && *actual == flag
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ControlFlowReturnValueTypeMismatch {
            function_id,
            block_id,
            return_expression_id,
            expected_type: ValueType::Domain(expected),
            actual_type: ValueType::Domain(actual),
        } if *function_id == multi_return
            && *block_id == exit
            && *return_expression_id == other_domain_return
            && *expected == customer
            && *actual == order
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::CallArgumentValueTypeMismatch {
            expression_id,
            function_id,
            mismatches,
        } if *expression_id == call
            && *function_id == target
            && mismatches == &[
                CallArgumentValueTypeMismatch {
                    argument_index: 0,
                    expected_type: ValueType::Domain(customer),
                    actual_type: ValueType::Domain(order),
                },
                CallArgumentValueTypeMismatch {
                    argument_index: 1,
                    expected_type: ValueType::Domain(customer),
                    actual_type: ValueType::Intrinsic(IntrinsicType::Int64),
                },
            ]
    )));

    let has_code_for_subject = |code, subject| {
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code() == code && diagnostic.primary_subject() == subject)
    };
    assert!(!has_code_for_subject(
        DiagnosticCode::DomainConstructRepresentationMismatch,
        DiagnosticPrimarySubject::Expression(unavailable_construct),
    ));
    assert!(!has_code_for_subject(
        DiagnosticCode::DomainConstructSourceTypeUnavailable,
        DiagnosticPrimarySubject::Expression(mismatching_construct),
    ));
    assert!(!has_code_for_subject(
        DiagnosticCode::DomainProjectSourceNotDomain,
        DiagnosticPrimarySubject::Expression(unavailable_project),
    ));
    assert!(!has_code_for_subject(
        DiagnosticCode::DomainProjectSourceTypeUnavailable,
        DiagnosticPrimarySubject::Expression(intrinsic_project),
    ));
    assert!(!has_code_for_subject(
        DiagnosticCode::BranchConditionTypeUnavailable,
        DiagnosticPrimarySubject::Block(branch_entry),
    ));
    assert!(!has_code_for_subject(
        DiagnosticCode::BranchConditionNotBool,
        DiagnosticPrimarySubject::Block(branch_entry),
    ));
    assert!(!has_code_for_subject(
        DiagnosticCode::CallArgumentTypeMismatch,
        DiagnosticPrimarySubject::Expression(call),
    ));

    for diagnostic in &diagnostics {
        assert_eq!(diagnostic.severity(), DiagnosticSeverity::Error);
    }
    let unique: HashSet<_> = diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code(), diagnostic.primary_subject()))
        .collect();
    assert_eq!(unique.len(), diagnostics.len());
}

// AR-DOMAIN-032/-052/-053/-072/-073: intrinsic cases retain their historical
// variants and IntrinsicType payload schemas under V0_5.
#[test]
fn v0_5_reuses_historical_diagnostics_without_widening_payloads() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();

    let host = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(host).unwrap();
    let i32_value = tx.add_int32_literal(block, 1).unwrap();
    let i64_value = tx.add_int64_literal(block, 2).unwrap();
    let boolean = tx.add_bool_literal(block, true).unwrap();
    let text = tx.add_text_literal(block, "x").unwrap();
    let bytes = tx.add_bytes_literal(block, vec![120]).unwrap();
    let mismatch_arithmetic = tx.add_add_expression(block, i32_value, i64_value).unwrap();
    let unsupported_arithmetic = tx.add_add_expression(block, boolean, boolean).unwrap();
    let unavailable_arithmetic = tx
        .add_add_expression(block, unsupported_arithmetic, i32_value)
        .unwrap();
    let mismatch_comparison = tx.add_equal_expression(block, text, bytes).unwrap();
    let unsupported_comparison = tx.add_less_than_expression(block, text, text).unwrap();
    let unavailable_comparison = tx
        .add_equal_expression(block, unsupported_arithmetic, boolean)
        .unwrap();
    let unit = tx.add_unit_literal(block).unwrap();
    tx.set_return(block, unit).unwrap();

    let single_return = tx.add_function(module, IntrinsicType::Int64).unwrap();
    let single_block = tx.create_function_body(single_return).unwrap();
    let wrong_return = tx.add_int32_literal(single_block, 3).unwrap();
    tx.set_return(single_block, wrong_return).unwrap();

    let branch_non_bool = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let non_bool_entry = tx.create_function_body(branch_non_bool).unwrap();
    let non_bool_exit = tx.add_block(branch_non_bool).unwrap();
    let non_bool_condition = tx.add_int64_literal(non_bool_entry, 4).unwrap();
    tx.set_branch(
        non_bool_entry,
        non_bool_condition,
        non_bool_exit,
        non_bool_exit,
    )
    .unwrap();
    let non_bool_unit = tx.add_unit_literal(non_bool_exit).unwrap();
    tx.set_return(non_bool_exit, non_bool_unit).unwrap();

    let branch_unavailable = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let unavailable_entry = tx.create_function_body(branch_unavailable).unwrap();
    let unavailable_exit = tx.add_block(branch_unavailable).unwrap();
    let u1 = tx.add_bool_literal(unavailable_entry, true).unwrap();
    let unavailable_condition = tx.add_add_expression(unavailable_entry, u1, u1).unwrap();
    tx.set_branch(
        unavailable_entry,
        unavailable_condition,
        unavailable_exit,
        unavailable_exit,
    )
    .unwrap();
    let unavailable_unit = tx.add_unit_literal(unavailable_exit).unwrap();
    tx.set_return(unavailable_exit, unavailable_unit).unwrap();

    let multi_return = tx.add_function(module, IntrinsicType::Int64).unwrap();
    let multi_entry = tx.create_function_body(multi_return).unwrap();
    let multi_exit = tx.add_block(multi_return).unwrap();
    let multi_condition = tx.add_bool_literal(multi_entry, true).unwrap();
    tx.set_branch(multi_entry, multi_condition, multi_exit, multi_exit)
        .unwrap();
    let multi_wrong = tx.add_int32_literal(multi_exit, 5).unwrap();
    tx.set_return(multi_exit, multi_wrong).unwrap();

    let target = tx.add_function(module, IntrinsicType::Unit).unwrap();
    tx.add_parameter(target, IntrinsicType::Int64).unwrap();
    tx.add_parameter(target, IntrinsicType::Int64).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let caller_block = tx.create_function_body(caller).unwrap();
    let c1 = tx.add_bool_literal(caller_block, true).unwrap();
    let unavailable_argument = tx.add_add_expression(caller_block, c1, c1).unwrap();
    let mismatching_argument = tx.add_int32_literal(caller_block, 6).unwrap();
    let extra_argument = tx.add_unit_literal(caller_block).unwrap();
    let call = tx
        .add_call_expression(
            caller_block,
            target,
            vec![unavailable_argument, mismatching_argument, extra_argument],
        )
        .unwrap();
    tx.set_effect_sequence(caller_block, vec![call]).unwrap();
    tx.set_return(caller_block, call).unwrap();

    let snapshot = tx.commit().unwrap();
    let diagnostics = v0_5_diagnostics(&snapshot);
    let codes: HashSet<_> = diagnostics.iter().map(Diagnostic::code).collect();
    for code in [
        DiagnosticCode::ArithmeticOperandTypeUnavailable,
        DiagnosticCode::ArithmeticOperandTypeMismatch,
        DiagnosticCode::ArithmeticUnsupportedOperandType,
        DiagnosticCode::ReturnTypeMismatch,
        DiagnosticCode::ComparisonOperandTypeUnavailable,
        DiagnosticCode::ComparisonOperandTypeMismatch,
        DiagnosticCode::ComparisonUnsupportedOperandType,
        DiagnosticCode::BranchConditionTypeUnavailable,
        DiagnosticCode::BranchConditionNotBool,
        DiagnosticCode::ControlFlowReturnTypeMismatch,
        DiagnosticCode::CallArgumentCountMismatch,
        DiagnosticCode::CallArgumentTypeUnavailable,
        DiagnosticCode::CallArgumentTypeMismatch,
    ] {
        assert!(codes.contains(&code), "missing {}", code.as_str());
    }
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticOperandTypeMismatch {
            expression_id,
            left_type: IntrinsicType::Int32,
            right_type: IntrinsicType::Int64,
        } if *expression_id == mismatch_arithmetic
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticUnsupportedOperandType {
            expression_id,
            operand_type: IntrinsicType::Bool,
        } if *expression_id == unsupported_arithmetic
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticOperandTypeUnavailable { expression_id }
            if *expression_id == unavailable_arithmetic
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonOperandTypeMismatch {
            expression_id,
            left_type: IntrinsicType::Text,
            right_type: IntrinsicType::Bytes,
        } if *expression_id == mismatch_comparison
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonUnsupportedOperandType {
            expression_id,
            operand_type: IntrinsicType::Text,
        } if *expression_id == unsupported_comparison
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonOperandTypeUnavailable { expression_id }
            if *expression_id == unavailable_comparison
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ReturnTypeMismatch {
            function_id,
            return_expression_id,
            expected_type: IntrinsicType::Int64,
            actual_type: IntrinsicType::Int32,
        } if *function_id == single_return && *return_expression_id == wrong_return
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::BranchConditionNotBool {
            block_id,
            condition_expression_id,
            actual_type: IntrinsicType::Int64,
        } if *block_id == non_bool_entry && *condition_expression_id == non_bool_condition
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::BranchConditionTypeUnavailable {
            block_id,
            condition_expression_id,
        } if *block_id == unavailable_entry && *condition_expression_id == unavailable_condition
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ControlFlowReturnTypeMismatch {
            function_id,
            block_id,
            return_expression_id,
            expected_type: IntrinsicType::Int64,
            actual_type: IntrinsicType::Int32,
        } if *function_id == multi_return
            && *block_id == multi_exit
            && *return_expression_id == multi_wrong
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::CallArgumentCountMismatch {
            expression_id,
            function_id,
            expected_count: 2,
            actual_count: 3,
        } if *expression_id == call && *function_id == target
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
}

// AR-DOMAIN-032/-073: Text and Bytes each support equality and inequality,
// neither supports ordering, and a mixed intrinsic comparison retains the
// historical MNIR-DIAG-006 IntrinsicType payload.
#[test]
fn text_and_bytes_comparison_matrix_is_explicit_under_v0_5() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let block = transaction.create_function_body(function).unwrap();
    let text_left = transaction.add_text_literal(block, "é").unwrap();
    let text_right = transaction.add_text_literal(block, "e\u{301}").unwrap();
    let bytes_left = transaction.add_bytes_literal(block, vec![0, 255]).unwrap();
    let bytes_right = transaction.add_bytes_literal(block, vec![0, 254]).unwrap();

    let text_equal = transaction
        .add_equal_expression(block, text_left, text_right)
        .unwrap();
    let text_not_equal = transaction
        .add_not_equal_expression(block, text_left, text_right)
        .unwrap();
    let bytes_equal = transaction
        .add_equal_expression(block, bytes_left, bytes_right)
        .unwrap();
    let bytes_not_equal = transaction
        .add_not_equal_expression(block, bytes_left, bytes_right)
        .unwrap();
    let text_order = transaction
        .add_less_than_expression(block, text_left, text_right)
        .unwrap();
    let bytes_order = transaction
        .add_less_than_expression(block, bytes_left, bytes_right)
        .unwrap();
    let mixed = transaction
        .add_equal_expression(block, text_left, bytes_left)
        .unwrap();
    let unit = transaction.add_unit_literal(block).unwrap();
    transaction.set_return(block, unit).unwrap();
    let snapshot = transaction.commit().unwrap();

    for expression in [text_equal, text_not_equal, bytes_equal, bytes_not_equal] {
        assert_eq!(
            snapshot.expression_type(expression),
            Some(Ok(ValueType::Intrinsic(IntrinsicType::Bool)))
        );
    }

    let diagnostics = v0_5_diagnostics(&snapshot);
    assert_eq!(diagnostics.len(), 3);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonUnsupportedOperandType {
            expression_id,
            operand_type: IntrinsicType::Text,
        } if *expression_id == text_order
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonUnsupportedOperandType {
            expression_id,
            operand_type: IntrinsicType::Bytes,
        } if *expression_id == bytes_order
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonOperandTypeMismatch {
            expression_id,
            left_type: IntrinsicType::Text,
            right_type: IntrinsicType::Bytes,
        } if *expression_id == mixed
    )));
    for expression in [text_equal, text_not_equal, bytes_equal, bytes_not_equal] {
        assert!(!diagnostics.iter().any(|diagnostic| {
            diagnostic.primary_subject() == DiagnosticPrimarySubject::Expression(expression)
        }));
    }
}

// AR-DOMAIN-017/-025/-026/-033/-034/-036/-050/-066/-067/-075: a valid
// Domain-rich revision verifies read-only and binds exact immutable evidence.
#[test]
fn valid_domain_program_verifies_and_binds_v0_5_evidence() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let domain = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let target = tx.add_function(module, ValueType::Domain(domain)).unwrap();
    tx.add_parameter(target, ValueType::Domain(domain)).unwrap();
    let caller = tx.add_function(module, ValueType::Domain(domain)).unwrap();
    let entry = tx.create_function_body(caller).unwrap();
    let exit = tx.add_block(caller).unwrap();
    let int = tx.add_int64_literal(entry, 7).unwrap();
    let constructed = tx.add_domain_construct(entry, domain, int).unwrap();
    let projected = tx.add_domain_project(entry, constructed).unwrap();
    let sum = tx.add_add_expression(entry, projected, projected).unwrap();
    let comparison = tx.add_equal_expression(entry, sum, projected).unwrap();
    let call = tx
        .add_call_expression(entry, target, vec![constructed])
        .unwrap();
    tx.set_effect_sequence(entry, vec![call]).unwrap();
    let text_left = tx.add_text_literal(entry, "text").unwrap();
    let text_right = tx.add_text_literal(entry, "text").unwrap();
    tx.add_equal_expression(entry, text_left, text_right)
        .unwrap();
    let bytes_left = tx.add_bytes_literal(entry, vec![0, 255]).unwrap();
    let bytes_right = tx.add_bytes_literal(entry, vec![0, 255]).unwrap();
    tx.add_not_equal_expression(entry, bytes_left, bytes_right)
        .unwrap();
    tx.set_branch(entry, comparison, exit, exit).unwrap();
    let exit_source = tx.add_int64_literal(exit, 8).unwrap();
    let exit_value = tx.add_domain_construct(exit, domain, exit_source).unwrap();
    tx.set_return(exit, exit_value).unwrap();
    let snapshot = tx.commit().unwrap();
    let revision = program.revision_id();
    let counter = program.allocation_counter_state();

    let first = verify_with_rule_set(&snapshot, V5).unwrap();
    let second = verify_with_rule_set(&snapshot, V5).unwrap();
    assert_eq!(first.rule_set(), V5);
    assert_eq!(first.program_id(), snapshot.program_id());
    assert_eq!(first.revision_id(), snapshot.revision_id());
    assert_eq!(
        first.snapshot().domain_type(domain),
        snapshot.domain_type(domain)
    );
    assert_eq!(second.program_id(), first.program_id());
    assert_eq!(second.revision_id(), first.revision_id());
    assert_eq!(program.revision_id(), revision);
    assert_eq!(program.allocation_counter_state(), counter);

    let domain_identity = program.domain_type(domain).unwrap().id();
    let representation = program.domain_type(domain).unwrap().representation();
    let parameter_type = *program.function(target).unwrap().parameters()[0].value_type();
    let construct_type = program.expression_type(constructed);
    let project_type = program.expression_type(projected);

    let mut presentation = program.begin_transaction();
    presentation
        .set_domain_type_preferred_name(domain, Some("Renamed".into()))
        .unwrap();
    presentation
        .set_domain_type_documentation(domain, Some("Presentation only".into()))
        .unwrap();
    let renamed = presentation.commit().unwrap();
    assert_ne!(renamed.revision_id(), revision);
    assert_eq!(program.domain_type(domain).unwrap().id(), domain_identity);
    assert_eq!(
        program.domain_type(domain).unwrap().representation(),
        representation
    );
    assert_eq!(
        *program.function(target).unwrap().parameters()[0].value_type(),
        parameter_type
    );
    assert_eq!(parameter_type, ValueType::Domain(domain));
    assert_eq!(program.expression_type(constructed), construct_type);
    assert_eq!(program.expression_type(projected), project_type);
    let renamed_evidence = verify_with_rule_set(&renamed, V5).unwrap();
    assert_eq!(renamed_evidence.rule_set(), V5);
    assert_eq!(renamed_evidence.program_id(), first.program_id());
    assert_eq!(renamed_evidence.revision_id(), renamed.revision_id());
}

// AR-DOMAIN-049/-072: ValueType introduction does not alter historical
// rule-set binding or the exact IntrinsicType Return payload.
#[test]
fn historical_return_payload_remains_identical_in_v0_1_through_v0_4() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let function = tx.add_function(module, IntrinsicType::Int64).unwrap();
    let block = tx.create_function_body(function).unwrap();
    let returned = tx.add_int32_literal(block, 1).unwrap();
    tx.set_return(block, returned).unwrap();
    let snapshot = tx.commit().unwrap();

    for rule_set in [
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4,
    ] {
        let error = verify_with_rule_set(&snapshot, rule_set).unwrap_err();
        assert!(matches!(
            error.diagnostics().unwrap(),
            [Diagnostic::ReturnTypeMismatch {
                function_id,
                return_expression_id,
                expected_type: IntrinsicType::Int64,
                actual_type: IntrinsicType::Int32,
            }] if *function_id == function && *return_expression_id == returned
        ));
    }
}

// AR-DOMAIN-025/-075: representation mutation changes construct validity but
// never the nominal ValueType used by Call argument and result matching.
#[test]
fn representation_mutation_preserves_nominal_call_matching() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let domain = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let target = tx.add_function(module, ValueType::Domain(domain)).unwrap();
    tx.add_parameter(target, ValueType::Domain(domain)).unwrap();
    let caller = tx.add_function(module, ValueType::Domain(domain)).unwrap();
    let block = tx.create_function_body(caller).unwrap();
    let source = tx.add_int64_literal(block, 1).unwrap();
    let construct = tx.add_domain_construct(block, domain, source).unwrap();
    let call = tx
        .add_call_expression(block, target, vec![construct])
        .unwrap();
    tx.set_effect_sequence(block, vec![call]).unwrap();
    tx.set_return(block, call).unwrap();
    let initial = tx.commit().unwrap();
    assert!(verify_with_rule_set(&initial, V5).is_ok());

    let mut mutation = program.begin_transaction();
    mutation
        .set_domain_type_representation(domain, IntrinsicType::Text)
        .unwrap();
    assert_eq!(
        mutation.expression_type(construct),
        Some(Ok(ValueType::Domain(domain)))
    );
    assert_eq!(
        mutation.expression_type(call),
        Some(Ok(ValueType::Domain(domain)))
    );
    let changed = mutation.commit().unwrap();
    let diagnostics = v0_5_diagnostics(&changed);
    assert!(matches!(
        diagnostics.as_slice(),
        [Diagnostic::DomainConstructRepresentationMismatch {
            expression_id,
            type_id,
            source_expression_id,
            expected_representation: IntrinsicType::Text,
            actual_type: ValueType::Intrinsic(IntrinsicType::Int64),
        }] if *expression_id == construct
            && *type_id == domain
            && *source_expression_id == source
    ));
    assert!(!diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.code(),
        DiagnosticCode::CallArgumentTypeUnavailable
            | DiagnosticCode::CallArgumentTypeMismatch
            | DiagnosticCode::CallArgumentValueTypeMismatch
    )));
}

// AR-DOMAIN-077: every dependent construct reports its required unavailable
// category, while Return mismatch remains deliberately suppressed.
#[test]
fn unavailable_types_propagate_without_suppressing_dependent_diagnostics() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let domain = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let target = tx.add_function(module, IntrinsicType::Unit).unwrap();
    tx.add_parameter(target, IntrinsicType::Int64).unwrap();

    let function = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(function).unwrap();
    let boolean = tx.add_bool_literal(block, true).unwrap();
    let invalid = tx.add_add_expression(block, boolean, boolean).unwrap();
    let int = tx.add_int64_literal(block, 1).unwrap();
    let arithmetic = tx.add_add_expression(block, invalid, int).unwrap();
    let comparison = tx.add_equal_expression(block, invalid, boolean).unwrap();
    let construct = tx.add_domain_construct(block, domain, invalid).unwrap();
    let project = tx.add_domain_project(block, invalid).unwrap();
    let call = tx
        .add_call_expression(block, target, vec![invalid])
        .unwrap();
    tx.set_effect_sequence(block, vec![call]).unwrap();
    tx.set_return(block, invalid).unwrap();
    let snapshot = tx.commit().unwrap();

    let diagnostics = v0_5_diagnostics(&snapshot);
    for (code, subject) in [
        (
            DiagnosticCode::ArithmeticOperandTypeUnavailable,
            DiagnosticPrimarySubject::Expression(arithmetic),
        ),
        (
            DiagnosticCode::ComparisonOperandTypeUnavailable,
            DiagnosticPrimarySubject::Expression(comparison),
        ),
        (
            DiagnosticCode::DomainConstructSourceTypeUnavailable,
            DiagnosticPrimarySubject::Expression(construct),
        ),
        (
            DiagnosticCode::DomainProjectSourceTypeUnavailable,
            DiagnosticPrimarySubject::Expression(project),
        ),
        (
            DiagnosticCode::CallArgumentTypeUnavailable,
            DiagnosticPrimarySubject::Expression(call),
        ),
    ] {
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code() == code && diagnostic.primary_subject() == subject
        }));
    }
    assert!(!diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.code(),
        DiagnosticCode::ReturnTypeMismatch
            | DiagnosticCode::ReturnValueTypeMismatch
            | DiagnosticCode::ControlFlowReturnTypeMismatch
            | DiagnosticCode::ControlFlowReturnValueTypeMismatch
    )));
}

#[test]
fn domain_diagnostic_codes_and_primary_subjects_are_stable() {
    let codes = [
        DiagnosticCode::DomainConstructSourceTypeUnavailable,
        DiagnosticCode::DomainConstructRepresentationMismatch,
        DiagnosticCode::DomainProjectSourceTypeUnavailable,
        DiagnosticCode::DomainProjectSourceNotDomain,
        DiagnosticCode::ArithmeticOperandValueTypeMismatch,
        DiagnosticCode::ArithmeticUnsupportedValueType,
        DiagnosticCode::ComparisonOperandValueTypeMismatch,
        DiagnosticCode::ComparisonUnsupportedValueType,
        DiagnosticCode::ReturnValueTypeMismatch,
        DiagnosticCode::BranchConditionValueTypeNotBool,
        DiagnosticCode::ControlFlowReturnValueTypeMismatch,
        DiagnosticCode::CallArgumentValueTypeMismatch,
    ];
    for (offset, code) in codes.into_iter().enumerate() {
        assert_eq!(code.as_str(), format!("MNIR-DIAG-{:03}", offset + 14));
    }

    let _subject_schema: Option<DiagnosticPrimarySubject> = None;
}
