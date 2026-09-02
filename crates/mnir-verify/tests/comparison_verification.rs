use std::collections::HashSet;

use mnir_core::{FunctionId, IntrinsicType, MnirProgram, ModuleId, ParameterId, ProgramSnapshot};
use mnir_verify::{
    Diagnostic, DiagnosticCode, DiagnosticPrimarySubject, DiagnosticSeverity, VerificationError,
    VerificationFailure, VerificationRuleSet, verify, verify_with_rule_set,
};

const V0_1: VerificationRuleSet = VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1;
const V0_2: VerificationRuleSet = VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2;

fn add_function(
    transaction: &mut mnir_core::MutationTransaction<'_>,
    module_id: ModuleId,
    return_type: IntrinsicType,
    parameter_types: Vec<IntrinsicType>,
) -> (FunctionId, Vec<ParameterId>) {
    let function_id = transaction.add_function(module_id, return_type).unwrap();
    let parameters = parameter_types
        .into_iter()
        .map(|intrinsic_type| {
            transaction
                .add_parameter(function_id, intrinsic_type)
                .unwrap()
        })
        .collect();
    (function_id, parameters)
}

fn v2_failure(snapshot: &ProgramSnapshot) -> VerificationFailure {
    match verify_with_rule_set(snapshot, V0_2) {
        Err(VerificationError::Semantic(failure)) => failure,
        Err(error) => panic!("test setup produced non-semantic verification error: {error}"),
        Ok(_) => panic!("test setup unexpectedly verified successfully"),
    }
}

fn assert_v0_1_not_applicable(snapshot: &ProgramSnapshot) {
    for result in [verify(snapshot), verify_with_rule_set(snapshot, V0_1)] {
        let error = result.unwrap_err();
        assert_eq!(error.diagnostics(), None);
        assert_eq!(
            error,
            VerificationError::RuleSetNotApplicable {
                requested_rule_set: V0_1,
            }
        );
    }
}

// AR-CMP-023, AR-CMP-030, AR-CMP-031, and AR-CMP-037;
// MNIR-CMP-002 through -004, -078, -082, -084 through -086, -106, -110/-111.
#[test]
fn v0_2_verifies_mixed_graphs_and_binds_explicit_rule_set_identity() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, parameters) = add_function(
        &mut transaction,
        module_id,
        IntrinsicType::Bool,
        vec![IntrinsicType::Int32, IntrinsicType::Int32],
    );
    let block_id = transaction.create_function_body(function_id).unwrap();
    let left = transaction
        .add_parameter_reference(block_id, parameters[0])
        .unwrap();
    let right = transaction
        .add_parameter_reference(block_id, parameters[1])
        .unwrap();
    let sum = transaction
        .add_add_expression(block_id, left, right)
        .unwrap();
    let arithmetic_to_comparison = transaction
        .add_greater_than_expression(block_id, sum, left)
        .unwrap();
    let bool_value = transaction.add_bool_literal(block_id, true).unwrap();
    let comparison_to_equality = transaction
        .add_equal_expression(block_id, arithmetic_to_comparison, bool_value)
        .unwrap();
    transaction
        .set_return(block_id, comparison_to_equality)
        .unwrap();
    let snapshot = transaction.commit().unwrap();

    let verified = verify_with_rule_set(&snapshot, V0_2).unwrap();
    assert_eq!(verified.program_id(), snapshot.program_id());
    assert_eq!(verified.revision_id(), snapshot.revision_id());
    assert_eq!(verified.rule_set(), V0_2);
    assert_eq!(verified.snapshot().revision_id(), snapshot.revision_id());
    assert_eq!(
        snapshot.expression_type(arithmetic_to_comparison),
        Some(Ok(IntrinsicType::Bool))
    );
    assert_eq!(
        snapshot.expression_type(comparison_to_equality),
        Some(Ok(IntrinsicType::Bool))
    );

    let empty = MnirProgram::new().unwrap().snapshot();
    let verified_v0_1 = verify(&empty).unwrap();
    let verified_v0_2 = verify_with_rule_set(&empty, V0_2).unwrap();
    assert_eq!(verified_v0_1.rule_set(), V0_1);
    assert_eq!(verified_v0_2.rule_set(), V0_2);
    assert_ne!(verified_v0_1.rule_set(), verified_v0_2.rule_set());
}

// AR-CMP-022 and MNIR-CMP-064 through -066, -078.
#[test]
fn v0_2_reports_return_mismatch_for_valid_bool_comparison() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, _) = add_function(&mut transaction, module_id, IntrinsicType::Int32, vec![]);
    let block_id = transaction.create_function_body(function_id).unwrap();
    let left = transaction.add_int32_literal(block_id, 1).unwrap();
    let right = transaction.add_int32_literal(block_id, 2).unwrap();
    let comparison = transaction
        .add_equal_expression(block_id, left, right)
        .unwrap();
    transaction.set_return(block_id, comparison).unwrap();
    let snapshot = transaction.commit().unwrap();

    let failure = v2_failure(&snapshot);
    assert_eq!(failure.len(), 1);
    assert!(matches!(
        &failure.diagnostics()[0],
        Diagnostic::ReturnTypeMismatch {
            function_id: actual_function,
            return_expression_id,
            expected_type: IntrinsicType::Int32,
            actual_type: IntrinsicType::Bool,
        } if *actual_function == function_id && *return_expression_id == comparison
    ));
}

// AR-CMP-024 through AR-CMP-028 and AR-CMP-032;
// MNIR-CMP-068 through -079, -087/-088.
#[test]
fn v0_2_comparison_diagnostics_are_complete_exclusive_and_machine_readable() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let second_module_id = transaction.add_module().unwrap();

    let (first_function, _) =
        add_function(&mut transaction, module_id, IntrinsicType::Bool, vec![]);
    let first_block = transaction.create_function_body(first_function).unwrap();
    let int32 = transaction.add_int32_literal(first_block, 1).unwrap();
    let int64 = transaction.add_int64_literal(first_block, 2).unwrap();
    let boolean = transaction.add_bool_literal(first_block, true).unwrap();
    let mismatch = transaction
        .add_equal_expression(first_block, int32, int64)
        .unwrap();
    let unsupported = transaction
        .add_less_than_expression(first_block, boolean, boolean)
        .unwrap();
    let unavailable = transaction
        .add_equal_expression(first_block, mismatch, boolean)
        .unwrap();
    // The invalid comparisons above are deliberately unreturned (AR-CMP-028).
    transaction.set_return(first_block, boolean).unwrap();

    let (second_function, _) = add_function(
        &mut transaction,
        second_module_id,
        IntrinsicType::Int32,
        vec![],
    );
    let second_block = transaction.create_function_body(second_function).unwrap();
    let return_left = transaction.add_int32_literal(second_block, 1).unwrap();
    let return_right = transaction.add_int64_literal(second_block, 2).unwrap();
    let invalid_return = transaction
        .add_not_equal_expression(second_block, return_left, return_right)
        .unwrap();
    transaction
        .set_return(second_block, invalid_return)
        .unwrap();

    let (arithmetic_function, _) = add_function(
        &mut transaction,
        second_module_id,
        IntrinsicType::Bool,
        vec![],
    );
    let arithmetic_block = transaction
        .create_function_body(arithmetic_function)
        .unwrap();
    let arithmetic_left = transaction.add_int32_literal(arithmetic_block, 1).unwrap();
    let arithmetic_right = transaction.add_int64_literal(arithmetic_block, 2).unwrap();
    let arithmetic_mismatch = transaction
        .add_add_expression(arithmetic_block, arithmetic_left, arithmetic_right)
        .unwrap();
    let arithmetic_return = transaction
        .add_bool_literal(arithmetic_block, true)
        .unwrap();
    transaction
        .set_return(arithmetic_block, arithmetic_return)
        .unwrap();
    let snapshot = transaction.commit().unwrap();

    let failure = v2_failure(&snapshot);
    let repeated = v2_failure(&snapshot);
    let semantic_keys = |failure: &VerificationFailure| {
        failure
            .diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.code(), diagnostic.primary_subject()))
            .collect::<HashSet<_>>()
    };
    assert_eq!(semantic_keys(&failure), semantic_keys(&repeated));
    assert_eq!(failure.len(), 5);
    assert!(failure.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonOperandTypeMismatch {
            expression_id,
            left_type: IntrinsicType::Int32,
            right_type: IntrinsicType::Int64,
        } if *expression_id == mismatch
    )));
    assert_eq!(
        snapshot.expression(unsupported).unwrap().kind(),
        &mnir_core::ExpressionKind::LessThan {
            left: boolean,
            right: boolean,
        }
    );
    assert!(failure.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonUnsupportedOperandType {
            expression_id,
            operand_type: IntrinsicType::Bool,
        } if *expression_id == unsupported
    )));
    assert!(failure.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonOperandTypeUnavailable { expression_id }
            if *expression_id == unavailable
    )));
    assert!(failure.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ComparisonOperandTypeMismatch {
            expression_id,
            left_type: IntrinsicType::Int32,
            right_type: IntrinsicType::Int64,
        } if *expression_id == invalid_return
    )));
    assert!(failure.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticOperandTypeMismatch { expression_id, .. }
            if *expression_id == arithmetic_mismatch
    )));
    assert!(
        !failure
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::ReturnTypeMismatch)
    );

    let codes: HashSet<_> = failure
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code())
        .collect();
    assert!(codes.contains(&DiagnosticCode::ComparisonOperandTypeUnavailable));
    assert!(codes.contains(&DiagnosticCode::ComparisonOperandTypeMismatch));
    assert!(codes.contains(&DiagnosticCode::ComparisonUnsupportedOperandType));
    assert_eq!(
        DiagnosticCode::ComparisonOperandTypeUnavailable.as_str(),
        "MNIR-DIAG-005"
    );
    assert_eq!(
        DiagnosticCode::ComparisonOperandTypeMismatch.as_str(),
        "MNIR-DIAG-006"
    );
    assert_eq!(
        DiagnosticCode::ComparisonUnsupportedOperandType.as_str(),
        "MNIR-DIAG-007"
    );
    for diagnostic in failure.diagnostics() {
        assert_eq!(diagnostic.severity(), DiagnosticSeverity::Error);
        if matches!(
            diagnostic.code(),
            DiagnosticCode::ComparisonOperandTypeUnavailable
                | DiagnosticCode::ComparisonOperandTypeMismatch
                | DiagnosticCode::ComparisonUnsupportedOperandType
        ) {
            assert!(matches!(
                diagnostic.primary_subject(),
                DiagnosticPrimarySubject::Expression(_)
            ));
        }
    }
    let unique: HashSet<_> = failure
        .diagnostics()
        .iter()
        .map(|diagnostic| (diagnostic.code(), diagnostic.primary_subject()))
        .collect();
    assert_eq!(unique.len(), failure.len());
}

fn comparison_return_snapshot(return_type: IntrinsicType) -> ProgramSnapshot {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, _) = add_function(&mut transaction, module_id, return_type, vec![]);
    let block_id = transaction.create_function_body(function_id).unwrap();
    let left = transaction.add_int32_literal(block_id, 1).unwrap();
    let right = transaction.add_int32_literal(block_id, 2).unwrap();
    let comparison = transaction
        .add_less_than_expression(block_id, left, right)
        .unwrap();
    transaction.set_return(block_id, comparison).unwrap();
    transaction.commit().unwrap()
}

// AR-CMP-029 cases A through E and AR-CMP-036;
// MNIR-CMP-001, -080/-081/-083, -101 through -109.
#[test]
fn v0_1_applicability_scan_rejects_every_comparison_program_shape() {
    // Case A: a valid comparison returned from Bool.
    assert_v0_1_not_applicable(&comparison_return_snapshot(IntrinsicType::Bool));
    // Case B: a valid comparison returned from Int32, without DIAG-004.
    assert_v0_1_not_applicable(&comparison_return_snapshot(IntrinsicType::Int32));

    // Case C: an invalid, unreturned comparison in another Module. The first
    // Module also contains invalid V0_1 arithmetic, proving no partial semantic
    // diagnostic result is produced before the complete applicability scan.
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let arithmetic_module = transaction.add_module().unwrap();
    let comparison_module = transaction.add_module().unwrap();
    let (arithmetic_function, _) = add_function(
        &mut transaction,
        arithmetic_module,
        IntrinsicType::Bool,
        vec![],
    );
    let arithmetic_block = transaction
        .create_function_body(arithmetic_function)
        .unwrap();
    let int32 = transaction.add_int32_literal(arithmetic_block, 1).unwrap();
    let boolean = transaction
        .add_bool_literal(arithmetic_block, true)
        .unwrap();
    transaction
        .add_add_expression(arithmetic_block, int32, boolean)
        .unwrap();
    transaction.set_return(arithmetic_block, boolean).unwrap();
    let (comparison_function, _) = add_function(
        &mut transaction,
        comparison_module,
        IntrinsicType::Bool,
        vec![],
    );
    let comparison_block = transaction
        .create_function_body(comparison_function)
        .unwrap();
    let int64 = transaction.add_int64_literal(comparison_block, 2).unwrap();
    let comparison_bool = transaction
        .add_bool_literal(comparison_block, false)
        .unwrap();
    transaction
        .add_equal_expression(comparison_block, int64, comparison_bool)
        .unwrap();
    transaction
        .set_return(comparison_block, comparison_bool)
        .unwrap();
    let dead_comparison = transaction.commit().unwrap();
    assert_v0_1_not_applicable(&dead_comparison);

    // Case D: Comparison used as an Arithmetic operand.
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, _) = add_function(&mut transaction, module_id, IntrinsicType::Unit, vec![]);
    let block_id = transaction.create_function_body(function_id).unwrap();
    let first = transaction.add_int32_literal(block_id, 1).unwrap();
    let second = transaction.add_int32_literal(block_id, 2).unwrap();
    let comparison = transaction
        .add_equal_expression(block_id, first, second)
        .unwrap();
    transaction
        .add_add_expression(block_id, comparison, comparison)
        .unwrap();
    let unit = transaction.add_unit_literal(block_id).unwrap();
    transaction.set_return(block_id, unit).unwrap();
    let comparison_in_arithmetic = transaction.commit().unwrap();
    assert_v0_1_not_applicable(&comparison_in_arithmetic);

    // Case E: Arithmetic used as a Comparison operand.
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, _) = add_function(&mut transaction, module_id, IntrinsicType::Bool, vec![]);
    let block_id = transaction.create_function_body(function_id).unwrap();
    let first = transaction.add_int32_literal(block_id, 1).unwrap();
    let second = transaction.add_int32_literal(block_id, 2).unwrap();
    let sum = transaction
        .add_add_expression(block_id, first, second)
        .unwrap();
    let comparison = transaction
        .add_greater_than_expression(block_id, sum, first)
        .unwrap();
    transaction.set_return(block_id, comparison).unwrap();
    let arithmetic_in_comparison = transaction.commit().unwrap();
    assert_v0_1_not_applicable(&arithmetic_in_comparison);
}

// AR-CMP-033 and AR-CMP-034 are conformance-inspection requirements: no
// evaluator, constant folder, Comparable/Ordered abstraction, operator
// overload system, or comparison-specific identity was introduced.
// AR-CMP-035 is demonstrated by the complete pre-existing workspace suite.
// AR-CMP-038 is covered by mnir-core's private structural-corruption test.
