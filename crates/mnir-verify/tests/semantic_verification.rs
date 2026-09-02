use std::collections::HashSet;

use mnir_core::{FunctionId, IntrinsicType, MnirProgram, ModuleId, ParameterId, ProgramSnapshot};
use mnir_verify::{
    Diagnostic, DiagnosticCode, DiagnosticPrimarySubject, DiagnosticSeverity, VerificationError,
    VerificationFailure, VerificationRuleSet, verify,
};

fn new_program() -> MnirProgram {
    MnirProgram::new().expect("the process-local ProgramId allocator should have capacity")
}

fn semantic_failure(snapshot: &ProgramSnapshot) -> VerificationFailure {
    match verify(snapshot) {
        Err(VerificationError::Semantic(failure)) => failure,
        Err(VerificationError::StructuralInput(error)) => {
            panic!("test setup unexpectedly produced structural invalidity: {error}")
        }
        Ok(_) => panic!("test setup unexpectedly verified successfully"),
    }
}

fn add_function(
    transaction: &mut mnir_core::MutationTransaction<'_>,
    module_id: ModuleId,
    return_type: IntrinsicType,
    parameter_types: Vec<IntrinsicType>,
) -> (FunctionId, Vec<ParameterId>) {
    let function_id = transaction.add_function(module_id, return_type).unwrap();
    let parameter_ids = parameter_types
        .into_iter()
        .map(|intrinsic_type| {
            transaction
                .add_parameter(function_id, intrinsic_type)
                .unwrap()
        })
        .collect();
    (function_id, parameter_ids)
}

// AR-VERIFY-001, AR-VERIFY-002, and AR-VERIFY-016;
// MNIR-VERIFY-004 through -011, -017, -043, -044, -069, and -070.
#[test]
fn empty_and_bodyless_programs_verify_without_mutation_or_revision_creation() {
    let mut program = new_program();
    let empty = program.snapshot();
    let verified_empty = verify(&empty).unwrap();

    assert_eq!(verified_empty.program_id(), empty.program_id());
    assert_eq!(verified_empty.revision_id(), empty.revision_id());
    assert_eq!(program.revision_id(), empty.revision_id());
    assert_eq!(program.module_count(), 0);

    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    add_function(&mut transaction, module_id, IntrinsicType::Unit, vec![]);
    let bodyless = transaction.commit().unwrap();
    drop(transaction);

    let revision_before = program.revision_id();
    assert!(verify(&bodyless).is_ok());
    assert_eq!(program.revision_id(), revision_before);
}

// AR-VERIFY-003 and AR-VERIFY-004; MNIR-VERIFY-027, -028, -037, and -038.
#[test]
fn valid_identity_and_arithmetic_functions_verify() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();

    let (identity, parameters) = add_function(
        &mut transaction,
        module_id,
        IntrinsicType::Int32,
        vec![IntrinsicType::Int32],
    );
    let identity_block = transaction.create_function_body(identity).unwrap();
    let value = transaction
        .add_parameter_reference(identity_block, parameters[0])
        .unwrap();
    transaction.set_return(identity_block, value).unwrap();

    let (add, parameters) = add_function(
        &mut transaction,
        module_id,
        IntrinsicType::Int32,
        vec![IntrinsicType::Int32, IntrinsicType::Int32],
    );
    let add_block = transaction.create_function_body(add).unwrap();
    let left = transaction
        .add_parameter_reference(add_block, parameters[0])
        .unwrap();
    let right = transaction
        .add_parameter_reference(add_block, parameters[1])
        .unwrap();
    let sum = transaction
        .add_add_expression(add_block, left, right)
        .unwrap();
    transaction.set_return(add_block, sum).unwrap();

    let snapshot = transaction.commit().unwrap();
    assert!(verify(&snapshot).is_ok());
}

// AR-VERIFY-005, AR-VERIFY-012, AR-VERIFY-013, AR-VERIFY-021, and
// AR-VERIFY-024; MNIR-VERIFY-031, -032, -035, -052, -060, and -061.
#[test]
fn operand_mismatch_has_one_machine_readable_payload_and_no_verified_program() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, _) = add_function(&mut transaction, module_id, IntrinsicType::Unit, vec![]);
    let block_id = transaction.create_function_body(function_id).unwrap();
    let left = transaction.add_int32_literal(block_id, 1).unwrap();
    let right = transaction.add_int64_literal(block_id, 2).unwrap();
    let mismatch = transaction
        .add_add_expression(block_id, left, right)
        .unwrap();
    transaction.set_return(block_id, mismatch).unwrap();
    let snapshot = transaction.commit().unwrap();

    let failure = semantic_failure(&snapshot);
    assert_eq!(failure.len(), 1);
    assert!(matches!(
        &failure.diagnostics()[0],
        Diagnostic::ArithmeticOperandTypeMismatch {
            expression_id,
            left_type: IntrinsicType::Int32,
            right_type: IntrinsicType::Int64,
        } if *expression_id == mismatch
    ));
    assert_eq!(
        failure.diagnostics()[0].primary_subject(),
        DiagnosticPrimarySubject::Expression(mismatch)
    );
}

// AR-VERIFY-006 and AR-VERIFY-007; MNIR-VERIFY-033 and -034.
#[test]
fn equal_bool_and_unit_operands_produce_unsupported_diagnostics() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, _) = add_function(&mut transaction, module_id, IntrinsicType::Unit, vec![]);
    let block_id = transaction.create_function_body(function_id).unwrap();
    let bool_left = transaction.add_bool_literal(block_id, true).unwrap();
    let bool_right = transaction.add_bool_literal(block_id, false).unwrap();
    let unit = transaction.add_unit_literal(block_id).unwrap();
    let bool_add = transaction
        .add_add_expression(block_id, bool_left, bool_right)
        .unwrap();
    let unit_add = transaction
        .add_add_expression(block_id, unit, unit)
        .unwrap();
    transaction.set_return(block_id, unit_add).unwrap();
    let snapshot = transaction.commit().unwrap();

    let failure = semantic_failure(&snapshot);
    assert_eq!(failure.len(), 2);
    assert!(failure.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticUnsupportedOperandType {
            expression_id,
            operand_type: IntrinsicType::Bool,
        } if *expression_id == bool_add
    )));
    assert!(failure.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticUnsupportedOperandType {
            expression_id,
            operand_type: IntrinsicType::Unit,
        } if *expression_id == unit_add
    )));
}

// AR-VERIFY-008 and AR-VERIFY-010; MNIR-VERIFY-029, -030, -036, and -041.
#[test]
fn nested_type_failure_propagates_unavailable_and_suppresses_return_mismatch() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, _) = add_function(&mut transaction, module_id, IntrinsicType::Int32, vec![]);
    let block_id = transaction.create_function_body(function_id).unwrap();
    let int32 = transaction.add_int32_literal(block_id, 1).unwrap();
    let boolean = transaction.add_bool_literal(block_id, true).unwrap();
    let inner = transaction
        .add_add_expression(block_id, int32, boolean)
        .unwrap();
    let outer = transaction
        .add_multiply_expression(block_id, inner, int32)
        .unwrap();
    transaction.set_return(block_id, outer).unwrap();
    let snapshot = transaction.commit().unwrap();

    let failure = semantic_failure(&snapshot);
    assert_eq!(failure.len(), 2);
    assert!(failure.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticOperandTypeMismatch { expression_id, .. }
            if *expression_id == inner
    )));
    assert!(failure.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        Diagnostic::ArithmeticOperandTypeUnavailable { expression_id }
            if *expression_id == outer
    )));
    assert!(
        !failure
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::ReturnTypeMismatch)
    );
}

// AR-VERIFY-009, AR-VERIFY-021, and AR-VERIFY-036;
// MNIR-VERIFY-025, -039, -040, -042, -065, and -086.
#[test]
fn return_mismatch_payload_and_primary_subject_are_function_bound() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, _) = add_function(&mut transaction, module_id, IntrinsicType::Int32, vec![]);
    let block_id = transaction.create_function_body(function_id).unwrap();
    let returned = transaction.add_bool_literal(block_id, true).unwrap();
    transaction.set_return(block_id, returned).unwrap();
    let snapshot = transaction.commit().unwrap();

    let failure = semantic_failure(&snapshot);
    assert_eq!(failure.len(), 1);
    let diagnostic = &failure.diagnostics()[0];
    assert!(matches!(
        diagnostic,
        Diagnostic::ReturnTypeMismatch {
            function_id: actual_function,
            return_expression_id,
            expected_type: IntrinsicType::Int32,
            actual_type: IntrinsicType::Bool,
        } if *actual_function == function_id && *return_expression_id == returned
    ));
    assert_eq!(
        diagnostic.primary_subject(),
        DiagnosticPrimarySubject::Function(function_id)
    );
}

// AR-VERIFY-011, AR-VERIFY-013, AR-VERIFY-019, AR-VERIFY-020,
// AR-VERIFY-033, AR-VERIFY-034, and AR-VERIFY-036;
// MNIR-VERIFY-019 through -026, -050 through -055, -063 through -067,
// and -087 through -089.
#[test]
fn complete_cross_module_traversal_collects_all_codes_without_duplicates() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let first_module = transaction.add_module().unwrap();
    let second_module = transaction.add_module().unwrap();

    let (nested_function, _) =
        add_function(&mut transaction, first_module, IntrinsicType::Int32, vec![]);
    let nested_block = transaction.create_function_body(nested_function).unwrap();
    let int32 = transaction.add_int32_literal(nested_block, 1).unwrap();
    let int64 = transaction.add_int64_literal(nested_block, 2).unwrap();
    let inner = transaction
        .add_add_expression(nested_block, int32, int64)
        .unwrap();
    let outer = transaction
        .add_add_expression(nested_block, inner, int32)
        .unwrap();
    transaction.set_return(nested_block, outer).unwrap();

    let (unsupported_function, _) =
        add_function(&mut transaction, second_module, IntrinsicType::Unit, vec![]);
    let unsupported_block = transaction
        .create_function_body(unsupported_function)
        .unwrap();
    let boolean = transaction
        .add_bool_literal(unsupported_block, true)
        .unwrap();
    let unsupported = transaction
        .add_add_expression(unsupported_block, boolean, boolean)
        .unwrap();
    transaction
        .set_return(unsupported_block, unsupported)
        .unwrap();

    let (return_function, _) = add_function(
        &mut transaction,
        second_module,
        IntrinsicType::Int32,
        vec![],
    );
    let return_block = transaction.create_function_body(return_function).unwrap();
    let wrong_return = transaction.add_bool_literal(return_block, false).unwrap();
    transaction.set_return(return_block, wrong_return).unwrap();

    let snapshot = transaction.commit().unwrap();
    let module_ids: HashSet<_> = snapshot.modules().map(|module| module.id()).collect();
    assert_eq!(module_ids, HashSet::from([first_module, second_module]));
    assert_eq!(
        snapshot
            .modules()
            .flat_map(|module| module.functions())
            .count(),
        3
    );

    let failure = semantic_failure(&snapshot);
    let codes: HashSet<_> = failure.diagnostics().iter().map(Diagnostic::code).collect();
    assert_eq!(
        codes,
        HashSet::from([
            DiagnosticCode::ArithmeticOperandTypeUnavailable,
            DiagnosticCode::ArithmeticOperandTypeMismatch,
            DiagnosticCode::ArithmeticUnsupportedOperandType,
            DiagnosticCode::ReturnTypeMismatch,
        ])
    );
    assert_eq!(failure.len(), 4);
    assert!(
        failure
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.severity() == DiagnosticSeverity::Error)
    );

    let unique_keys: HashSet<_> = failure
        .diagnostics()
        .iter()
        .map(|diagnostic| (diagnostic.code(), diagnostic.primary_subject()))
        .collect();
    assert_eq!(unique_keys.len(), failure.len());
    assert_eq!(
        DiagnosticCode::ArithmeticOperandTypeUnavailable.as_str(),
        "MNIR-DIAG-001"
    );
    assert_eq!(
        DiagnosticCode::ArithmeticOperandTypeMismatch.as_str(),
        "MNIR-DIAG-002"
    );
    assert_eq!(
        DiagnosticCode::ArithmeticUnsupportedOperandType.as_str(),
        "MNIR-DIAG-003"
    );
    assert_eq!(DiagnosticCode::ReturnTypeMismatch.as_str(), "MNIR-DIAG-004");
}

// AR-VERIFY-014, AR-VERIFY-015, AR-VERIFY-023, and AR-VERIFY-035;
// MNIR-VERIFY-012 through -016 and -056 through -059.
#[test]
fn verified_program_is_rule_set_and_revision_bound_historical_evidence() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let revision_one = transaction.commit().unwrap();
    drop(transaction);

    let verified_one = verify(&revision_one).unwrap();
    assert_eq!(verified_one.program_id(), revision_one.program_id());
    assert_eq!(verified_one.revision_id(), revision_one.revision_id());
    assert_eq!(
        verified_one.rule_set(),
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1
    );
    assert_eq!(
        verified_one.rule_set().as_str(),
        "MNIR Semantic Verification and Diagnostics 0.1"
    );
    assert!(verified_one.snapshot().module(module_id).is_some());

    let mut transaction = program.begin_transaction();
    transaction
        .set_module_preferred_name(module_id, Some("later".to_owned()))
        .unwrap();
    let revision_two = transaction.commit().unwrap();

    assert_ne!(revision_one.revision_id(), revision_two.revision_id());
    assert_eq!(verified_one.revision_id(), revision_one.revision_id());
    assert_eq!(
        verified_one
            .snapshot()
            .module(module_id)
            .unwrap()
            .presentation()
            .preferred_name(),
        None
    );

    let verified_two = verify(&revision_two).unwrap();
    assert_eq!(verified_two.revision_id(), revision_two.revision_id());
    assert_ne!(verified_one.revision_id(), verified_two.revision_id());
}

// AR-VERIFY-017 and AR-VERIFY-018; MNIR-VERIFY-011, -048, -049, -053, and -054.
#[test]
fn repeated_and_presentation_only_verification_has_equivalent_semantics() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, _) = add_function(&mut transaction, module_id, IntrinsicType::Int32, vec![]);
    let block_id = transaction.create_function_body(function_id).unwrap();
    let wrong_return = transaction.add_bool_literal(block_id, false).unwrap();
    transaction.set_return(block_id, wrong_return).unwrap();
    let first = transaction.commit().unwrap();
    drop(transaction);

    let first_run = semantic_failure(&first);
    let repeated_run = semantic_failure(&first);
    assert_eq!(first_run, repeated_run);

    let mut transaction = program.begin_transaction();
    transaction
        .set_function_documentation(function_id, Some("presentation only".to_owned()))
        .unwrap();
    let second = transaction.commit().unwrap();
    assert_ne!(first.revision_id(), second.revision_id());
    assert_eq!(first_run, semantic_failure(&second));
}

// AR-VERIFY-025, AR-VERIFY-026, and AR-VERIFY-029;
// MNIR-VERIFY-045 through -047 and -076 through -078.
#[test]
fn guaranteed_overflow_and_division_by_zero_verify_without_evaluation() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();

    let (overflow_function, _) =
        add_function(&mut transaction, module_id, IntrinsicType::Int32, vec![]);
    let overflow_block = transaction.create_function_body(overflow_function).unwrap();
    let maximum = transaction
        .add_int32_literal(overflow_block, i32::MAX)
        .unwrap();
    let one = transaction.add_int32_literal(overflow_block, 1).unwrap();
    let overflow = transaction
        .add_add_expression(overflow_block, maximum, one)
        .unwrap();
    transaction.set_return(overflow_block, overflow).unwrap();

    let (division_function, _) =
        add_function(&mut transaction, module_id, IntrinsicType::Int32, vec![]);
    let division_block = transaction.create_function_body(division_function).unwrap();
    let one = transaction.add_int32_literal(division_block, 1).unwrap();
    let zero = transaction.add_int32_literal(division_block, 0).unwrap();
    let division = transaction
        .add_divide_expression(division_block, one, zero)
        .unwrap();
    transaction.set_return(division_block, division).unwrap();

    let snapshot = transaction.commit().unwrap();
    assert!(verify(&snapshot).is_ok());
}

// AR-VERIFY-027 and AR-VERIFY-028; MNIR-VERIFY-027 and -071.
#[test]
fn unreturned_invalid_arithmetic_is_checked_and_names_do_not_change_types() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let (function_id, parameters) = add_function(
        &mut transaction,
        module_id,
        IntrinsicType::Int32,
        vec![IntrinsicType::Bool],
    );
    transaction
        .set_parameter_preferred_name(parameters[0], Some("numeric_count".to_owned()))
        .unwrap();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let parameter = transaction
        .add_parameter_reference(block_id, parameters[0])
        .unwrap();
    let invalid = transaction
        .add_add_expression(block_id, parameter, parameter)
        .unwrap();
    let valid_return = transaction.add_int32_literal(block_id, 0).unwrap();
    transaction.set_return(block_id, valid_return).unwrap();
    let snapshot = transaction.commit().unwrap();

    let failure = semantic_failure(&snapshot);
    assert_eq!(failure.len(), 1);
    assert!(matches!(
        &failure.diagnostics()[0],
        Diagnostic::ArithmeticUnsupportedOperandType {
            expression_id,
            operand_type: IntrinsicType::Bool,
        } if *expression_id == invalid
    ));
}

// AR-VERIFY-030 and AR-VERIFY-031. AR-VERIFY-032 is demonstrated by the full
// workspace acceptance run. Source inspection also confirms that version 0.1
// defines no evaluator, future verifier, warning/suppression system, generic
// diagnostic node, or reverse mnir-core dependency.
#[test]
fn crate_dependency_direction_is_one_way() {
    let verifier_manifest = include_str!("../Cargo.toml");
    let core_manifest = include_str!("../../mnir-core/Cargo.toml");

    assert!(verifier_manifest.contains("mnir-core.workspace = true"));
    assert!(!core_manifest.contains("mnir-verify"));
}
