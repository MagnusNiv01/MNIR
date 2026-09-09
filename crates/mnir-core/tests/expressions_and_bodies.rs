use std::any::TypeId;
use std::collections::HashSet;

use mnir_core::{
    BlockId, ExpressionId, ExpressionKind, ExpressionTypeError, FunctionId, IntrinsicType,
    MnirProgram, ModuleId, MutationError, ParameterId, ProgramId, RevisionId, StructuralError,
    TransactionState,
};

fn add_module_and_function(
    program: &mut MnirProgram,
    return_type: IntrinsicType,
    parameter_types: Vec<IntrinsicType>,
) -> (ModuleId, FunctionId, Vec<ParameterId>) {
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let function_id = transaction.add_function(module_id, return_type).unwrap();
    let parameters = parameter_types
        .into_iter()
        .map(|intrinsic_type| {
            transaction
                .add_parameter(function_id, intrinsic_type)
                .unwrap()
        })
        .collect();
    transaction.commit().unwrap();
    (module_id, function_id, parameters)
}

fn add_unit_body(program: &mut MnirProgram, function_id: FunctionId) -> (BlockId, ExpressionId) {
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let expression_id = transaction.add_unit_literal(block_id).unwrap();
    transaction.set_return(block_id, expression_id).unwrap();
    transaction.commit().unwrap();
    (block_id, expression_id)
}

// AR-EXPR-001, AR-EXPR-002, AR-EXPR-003, and AR-EXPR-041.
#[test]
fn body_is_optional_created_once_and_publicly_inspectable() {
    let mut program = MnirProgram::new().unwrap();
    let (_, bodyless_function, _) =
        add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    assert!(!program.function(bodyless_function).unwrap().has_body());
    assert!(program.validate_structure().is_ok());

    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(bodyless_function).unwrap();
    assert!(transaction.is_block_id_provisional(block_id));
    assert_eq!(transaction.block(block_id).unwrap().expression_count(), 0);
    let expression_id = transaction.add_unit_literal(block_id).unwrap();
    transaction.set_return(block_id, expression_id).unwrap();
    transaction.commit().unwrap();

    let function = program.function(bodyless_function).unwrap();
    let body = function.body().unwrap();
    assert_eq!(body.block_id(), block_id);
    assert_eq!(body.block().expression_count(), 1);
    assert_eq!(
        body.block().expression(expression_id).unwrap().id(),
        expression_id
    );
    assert_eq!(body.block().expressions().count(), 1);

    let source_revision = program.revision_id();
    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.create_function_body(bodyless_function),
        Err(MutationError::FunctionBodyAlreadyExists(bodyless_function))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    assert!(transaction.commit().is_err());
    drop(transaction);
    assert_eq!(program.revision_id(), source_revision);
}

// AR-EXPR-004, AR-EXPR-005, AR-EXPR-006, AR-EXPR-007, AR-EXPR-032, AR-EXPR-033,
// AR-EXPR-040, and MNIR-EXPR-019 through MNIR-EXPR-028.
#[test]
fn every_literal_preserves_data_has_derived_type_and_uses_range_safe_inputs() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let int32_min = transaction.add_int32_literal(block_id, i32::MIN).unwrap();
    let int32_max = transaction.add_int32_literal(block_id, i32::MAX).unwrap();
    let int64_min = transaction.add_int64_literal(block_id, i64::MIN).unwrap();
    let int64_max = transaction.add_int64_literal(block_id, i64::MAX).unwrap();
    let false_id = transaction.add_bool_literal(block_id, false).unwrap();
    let true_id = transaction.add_bool_literal(block_id, true).unwrap();
    let unit_id = transaction.add_unit_literal(block_id).unwrap();
    let ids = [
        int32_min, int32_max, int64_min, int64_max, false_id, true_id, unit_id,
    ];
    assert_eq!(ids.into_iter().collect::<HashSet<_>>().len(), ids.len());
    assert!(
        ids.iter()
            .all(|id| transaction.is_expression_id_provisional(*id))
    );
    transaction.set_return(block_id, unit_id).unwrap();
    transaction.commit().unwrap();

    assert_eq!(
        program.expression(int32_min).unwrap().kind(),
        &ExpressionKind::Int32Literal(i32::MIN)
    );
    assert_eq!(
        program.expression(int32_max).unwrap().kind(),
        &ExpressionKind::Int32Literal(i32::MAX)
    );
    assert_eq!(
        program.expression(int64_min).unwrap().kind(),
        &ExpressionKind::Int64Literal(i64::MIN)
    );
    assert_eq!(
        program.expression(int64_max).unwrap().kind(),
        &ExpressionKind::Int64Literal(i64::MAX)
    );
    assert_eq!(
        program.expression(false_id).unwrap().kind(),
        &ExpressionKind::BoolLiteral(false)
    );
    assert_eq!(
        program.expression(true_id).unwrap().kind(),
        &ExpressionKind::BoolLiteral(true)
    );
    assert_eq!(
        program.expression(unit_id).unwrap().kind(),
        &ExpressionKind::UnitLiteral
    );
    for id in [int32_min, int32_max] {
        assert_eq!(program.expression_type(id), Some(Ok(IntrinsicType::Int32)));
    }
    for id in [int64_min, int64_max] {
        assert_eq!(program.expression_type(id), Some(Ok(IntrinsicType::Int64)));
    }
    for id in [false_id, true_id] {
        assert_eq!(program.expression_type(id), Some(Ok(IntrinsicType::Bool)));
    }
    assert_eq!(
        program.expression_type(unit_id),
        Some(Ok(IntrinsicType::Unit))
    );
}

// AR-EXPR-008, AR-EXPR-009, and AR-EXPR-010.
#[test]
fn parameter_reference_requires_same_function_and_can_be_returned() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, function_id, parameters) = add_module_and_function(
        &mut program,
        IntrinsicType::Int32,
        vec![IntrinsicType::Int32],
    );
    let mut transaction = program.begin_transaction();
    let other_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let other_parameter = transaction
        .add_parameter(other_function, IntrinsicType::Bool)
        .unwrap();
    transaction.commit().unwrap();

    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let reference = transaction
        .add_parameter_reference(block_id, parameters[0])
        .unwrap();
    transaction.set_return(block_id, reference).unwrap();
    transaction.commit().unwrap();
    assert_eq!(
        program.expression(reference).unwrap().kind(),
        &ExpressionKind::ParameterReference(parameters[0])
    );
    assert_eq!(
        program.expression_type(reference),
        Some(Ok(IntrinsicType::Int32))
    );
    assert_eq!(
        program.block(block_id).unwrap().return_expression_id(),
        Some(reference)
    );

    let mut transaction = program.begin_transaction();
    transaction.remove_function_body(function_id).unwrap();
    let new_block = transaction.create_function_body(function_id).unwrap();
    assert_eq!(
        transaction.add_parameter_reference(new_block, other_parameter),
        Err(MutationError::ParameterNotOwnedByFunction {
            parameter_id: other_parameter,
            function_id,
        })
    );
    assert_eq!(transaction.state(), TransactionState::Failed);

    let mut transaction = program.begin_transaction();
    transaction.remove_parameter(other_parameter).unwrap();
    transaction.commit().unwrap();
    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.add_parameter_reference(block_id, other_parameter),
        Err(MutationError::UnknownParameter(other_parameter))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
}

// AR-EXPR-011, AR-EXPR-012, AR-EXPR-038, and MNIR-EXPR-039/-101.
#[test]
fn return_replaces_without_identity_and_rejects_invalid_targets() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, first_function, _) =
        add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let mut transaction = program.begin_transaction();
    let second_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let first_block = transaction.create_function_body(first_function).unwrap();
    let second_block = transaction.create_function_body(second_function).unwrap();
    let first_expression = transaction.add_unit_literal(first_block).unwrap();
    let replacement = transaction.add_unit_literal(first_block).unwrap();
    let foreign = transaction.add_unit_literal(second_block).unwrap();
    transaction
        .set_return(first_block, first_expression)
        .unwrap();
    transaction.set_return(first_block, replacement).unwrap();
    transaction.set_return(second_block, foreign).unwrap();
    transaction.commit().unwrap();
    assert_eq!(
        program.block(first_block).unwrap().return_expression_id(),
        Some(replacement)
    );
    assert_eq!(program.block(first_block).unwrap().expression_count(), 2);

    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.set_return(first_block, foreign),
        Err(MutationError::ExpressionNotOwnedByBlock {
            expression_id: foreign,
            block_id: first_block,
        })
    );
    assert_eq!(transaction.state(), TransactionState::Failed);

    let mut transaction = program.begin_transaction();
    transaction.remove_function(second_function).unwrap();
    transaction.commit().unwrap();
    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.set_return(second_block, replacement),
        Err(MutationError::UnknownBlock(second_block))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);

    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.set_return(first_block, foreign),
        Err(MutationError::UnknownExpression(foreign))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
}

// AR-EXPR-013 and MNIR-EXPR-051/-052/-073.
#[test]
fn unterminated_body_is_allowed_while_active_but_commit_is_atomic_rejection() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let source_revision = program.revision_id();
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    assert_eq!(
        transaction.block(block_id).unwrap().return_expression_id(),
        None
    );
    assert_eq!(transaction.state(), TransactionState::Active);
    assert!(matches!(
        transaction.commit(),
        Err(MutationError::StructuralViolation(
            StructuralError::UnterminatedBlock(actual_block_id)
        )) if actual_block_id == block_id
    ));
    assert_eq!(transaction.state(), TransactionState::Failed);
    drop(transaction);
    assert_eq!(program.revision_id(), source_revision);
    assert!(!program.function(function_id).unwrap().has_body());
}

// AR-EXPR-014 and MNIR-EXPR-040 through MNIR-EXPR-043.
#[test]
fn structurally_valid_return_type_mismatch_is_representable() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Bool, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let expression_id = transaction.add_int32_literal(block_id, 7).unwrap();
    transaction.set_return(block_id, expression_id).unwrap();
    transaction.commit().unwrap();
    assert_eq!(
        program.function(function_id).unwrap().return_type(),
        &IntrinsicType::Bool
    );
    assert_eq!(
        program.expression_type(expression_id),
        Some(Ok(IntrinsicType::Int32))
    );
    assert!(program.validate_structure().is_ok());
}

// AR-EXPR-015, AR-EXPR-016, AR-EXPR-017, and AR-EXPR-043.
#[test]
fn parameter_reference_type_is_derived_live_and_dangling_inspection_does_not_poison() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, parameters) = add_module_and_function(
        &mut program,
        IntrinsicType::Int32,
        vec![IntrinsicType::Int32],
    );
    let parameter_id = parameters[0];
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let reference = transaction
        .add_parameter_reference(block_id, parameter_id)
        .unwrap();
    transaction.set_return(block_id, reference).unwrap();
    transaction.commit().unwrap();

    let mut transaction = program.begin_transaction();
    transaction
        .set_parameter_type(parameter_id, IntrinsicType::Int64)
        .unwrap();
    assert_eq!(
        transaction.expression_type(reference),
        Some(Ok(IntrinsicType::Int64))
    );
    transaction.commit().unwrap();
    assert_eq!(
        program.expression_type(reference),
        Some(Ok(IntrinsicType::Int64))
    );

    let source_revision = program.revision_id();
    let mut transaction = program.begin_transaction();
    transaction.remove_parameter(parameter_id).unwrap();
    assert_eq!(
        transaction.expression_type(reference),
        Some(Err(ExpressionTypeError::UnresolvedParameter {
            expression_id: reference,
            parameter_id,
        }))
    );
    assert_eq!(transaction.state(), TransactionState::Active);
    assert!(matches!(
        transaction.commit(),
        Err(MutationError::StructuralViolation(
            StructuralError::DanglingParameterReference { .. }
        ))
    ));
    drop(transaction);
    assert_eq!(program.revision_id(), source_revision);

    let mut transaction = program.begin_transaction();
    transaction.remove_parameter(parameter_id).unwrap();
    assert!(transaction.expression_type(reference).unwrap().is_err());
    assert_eq!(transaction.state(), TransactionState::Active);
    transaction.remove_function_body(function_id).unwrap();
    transaction.commit().unwrap();
    assert!(program.parameter(parameter_id).is_none());
    assert!(!program.function(function_id).unwrap().has_body());
}

// AR-EXPR-018, AR-EXPR-020, AR-EXPR-021, and AR-EXPR-044
// (Function cascade arm).
#[test]
fn function_and_body_removal_cascade_and_retire_committed_identities() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, function_id, _) =
        add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let (old_block, old_expression) = add_unit_body(&mut program, function_id);
    let mut transaction = program.begin_transaction();
    transaction.remove_function_body(function_id).unwrap();
    transaction.commit().unwrap();
    assert!(program.block(old_block).is_none());
    assert!(program.expression(old_expression).is_none());
    let (new_block, new_expression) = add_unit_body(&mut program, function_id);
    assert_ne!(old_block, new_block);
    assert_ne!(old_expression, new_expression);

    let mut transaction = program.begin_transaction();
    transaction.remove_function(function_id).unwrap();
    transaction.commit().unwrap();
    assert!(program.function(function_id).is_none());
    assert!(program.block(new_block).is_none());
    assert!(program.expression(new_expression).is_none());

    let mut transaction = program.begin_transaction();
    let replacement_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let replacement_block = transaction
        .create_function_body(replacement_function)
        .unwrap();
    let replacement_expression = transaction.add_unit_literal(replacement_block).unwrap();
    transaction
        .set_return(replacement_block, replacement_expression)
        .unwrap();
    transaction.commit().unwrap();
    assert_ne!(replacement_block, new_block);
    assert_ne!(replacement_expression, new_expression);
}

// AR-EXPR-019 and AR-EXPR-044 (Module cascade arm).
#[test]
fn module_removal_cascades_bodies_without_enabling_identity_reuse() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, function_id, _) =
        add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let (block_id, expression_id) = add_unit_body(&mut program, function_id);
    let mut transaction = program.begin_transaction();
    transaction.remove_module(module_id).unwrap();
    transaction.commit().unwrap();
    assert!(program.module(module_id).is_none());
    assert!(program.function(function_id).is_none());
    assert!(program.block(block_id).is_none());
    assert!(program.expression(expression_id).is_none());
}

// AR-EXPR-022, MNIR-EXPR-061, MNIR-EXPR-080, and MNIR-PSI-032.
#[test]
fn allocated_then_removed_body_still_consumes_identities_and_advances_revision() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let source_revision = program.revision_id();
    let old_allocation_state = program.allocation_counter_state();
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let expression_id = transaction.add_unit_literal(block_id).unwrap();
    transaction.remove_function_body(function_id).unwrap();
    transaction.commit().unwrap();
    assert_ne!(program.revision_id(), source_revision);
    assert_ne!(program.allocation_counter_state(), old_allocation_state);

    let (next_block, next_expression) = add_unit_body(&mut program, function_id);
    assert_ne!(block_id, next_block);
    assert_ne!(expression_id, next_expression);
}

// AR-EXPR-023 and MNIR-EXPR-062 as superseded by MNIR-PSI-033/-034.
#[test]
fn failed_and_discarded_body_identities_are_absent_but_consumed() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let initial_allocation_state = program.allocation_counter_state();

    let mut transaction = program.begin_transaction();
    let failed_block = transaction.create_function_body(function_id).unwrap();
    let failed_expression = transaction.add_unit_literal(failed_block).unwrap();
    assert!(transaction.commit().is_err());
    transaction.discard().unwrap();
    drop(transaction);
    assert!(program.block(failed_block).is_none());
    assert!(program.expression(failed_expression).is_none());

    let mut transaction = program.begin_transaction();
    let discarded_block = transaction.create_function_body(function_id).unwrap();
    let discarded_expression = transaction.add_unit_literal(discarded_block).unwrap();
    transaction.discard().unwrap();
    drop(transaction);
    assert!(program.block(discarded_block).is_none());
    assert!(program.expression(discarded_expression).is_none());
    assert_ne!(program.allocation_counter_state(), initial_allocation_state);
    assert!(discarded_block.counter() > failed_expression.counter());
    assert!(discarded_expression.counter() > discarded_block.counter());
}

// AR-EXPR-024 and AR-EXPR-025; MNIR-EXPR-066/-067/-099.
#[test]
fn snapshots_and_forks_preserve_body_identities_and_references() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, function_id, parameters) =
        add_module_and_function(&mut program, IntrinsicType::Bool, vec![IntrinsicType::Bool]);
    let parameter_id = parameters[0];
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let expression_id = transaction
        .add_parameter_reference(block_id, parameter_id)
        .unwrap();
    transaction.set_return(block_id, expression_id).unwrap();
    transaction.commit().unwrap();
    let snapshot = program.snapshot();
    assert_eq!(
        snapshot.block(block_id).unwrap().return_expression_id(),
        Some(expression_id)
    );
    assert_eq!(
        snapshot.expression(expression_id).unwrap().kind(),
        &ExpressionKind::ParameterReference(parameter_id)
    );

    let mut fork = snapshot.fork().unwrap();
    assert_ne!(fork.program_id(), program.program_id());
    assert_eq!(
        fork.block(block_id).unwrap().return_expression_id(),
        Some(expression_id)
    );
    assert_eq!(
        fork.expression_type(expression_id),
        Some(Ok(IntrinsicType::Bool))
    );
    let mut transaction = fork.begin_transaction();
    transaction.remove_function_body(function_id).unwrap();
    let new_block = transaction.create_function_body(function_id).unwrap();
    let new_expression = transaction.add_unit_literal(new_block).unwrap();
    transaction.set_return(new_block, new_expression).unwrap();
    transaction.commit().unwrap();
    assert_ne!(new_block, block_id);
    assert_ne!(new_expression, expression_id);
    assert!(fork.block(block_id).is_none());
    assert!(fork.expression(expression_id).is_none());
    assert!(fork.module(module_id).is_some());
}

// AR-EXPR-026 and MNIR-EXPR-017/-018/-074.
#[test]
fn expressions_are_identity_addressed_and_enumeration_order_is_not_observed() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let first = transaction.add_int32_literal(block_id, 1).unwrap();
    let second = transaction.add_bool_literal(block_id, true).unwrap();
    let third = transaction.add_unit_literal(block_id).unwrap();
    transaction.set_return(block_id, third).unwrap();
    transaction.commit().unwrap();
    let block = program.block(block_id).unwrap();
    let enumerated: HashSet<_> = block
        .expressions()
        .map(|expression| expression.id())
        .collect();
    assert_eq!(enumerated, HashSet::from([first, second, third]));
    assert_eq!(
        block.expression(first).unwrap().kind(),
        &ExpressionKind::Int32Literal(1)
    );
    assert_eq!(
        block.expression(second).unwrap().kind(),
        &ExpressionKind::BoolLiteral(true)
    );
}

// AR-EXPR-027. Separate opaque Rust types provide compile-time category safety.
#[test]
fn block_and_expression_ids_are_distinct_from_existing_identifier_types() {
    let ids = [
        TypeId::of::<ProgramId>(),
        TypeId::of::<RevisionId>(),
        TypeId::of::<ModuleId>(),
        TypeId::of::<FunctionId>(),
        TypeId::of::<ParameterId>(),
        TypeId::of::<BlockId>(),
        TypeId::of::<ExpressionId>(),
    ];
    assert_eq!(ids.into_iter().collect::<HashSet<_>>().len(), ids.len());
}

// AR-EXPR-034, AR-EXPR-035, AR-EXPR-039, and AR-EXPR-040.
#[test]
fn committed_and_provisional_body_identities_are_program_unique() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, first_function, _) =
        add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let (committed_block, committed_expression) = add_unit_body(&mut program, first_function);
    let mut transaction = program.begin_transaction();
    let second_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let third_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let second_block = transaction.create_function_body(second_function).unwrap();
    let third_block = transaction.create_function_body(third_function).unwrap();
    let second_expression = transaction.add_unit_literal(second_block).unwrap();
    let third_expression = transaction.add_unit_literal(third_block).unwrap();
    assert!(transaction.is_block_id_provisional(second_block));
    assert!(transaction.is_block_id_provisional(third_block));
    assert!(transaction.is_expression_id_provisional(second_expression));
    assert!(transaction.is_expression_id_provisional(third_expression));
    assert_eq!(
        HashSet::from([committed_block, second_block, third_block]).len(),
        3
    );
    assert_eq!(
        HashSet::from([committed_expression, second_expression, third_expression]).len(),
        3
    );
    transaction
        .set_return(second_block, second_expression)
        .unwrap();
    transaction
        .set_return(third_block, third_expression)
        .unwrap();
    transaction.commit().unwrap();
}

// AR-EXPR-036.
#[test]
fn removing_absent_or_unknown_body_poisons_transaction() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.remove_function_body(function_id),
        Err(MutationError::FunctionBodyAbsent(function_id))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    assert!(transaction.commit().is_err());

    let mut transaction = program.begin_transaction();
    transaction.remove_function(function_id).unwrap();
    transaction.commit().unwrap();
    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.remove_function_body(function_id),
        Err(MutationError::UnknownFunction(function_id))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    assert!(transaction.commit().is_err());

    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.create_function_body(function_id),
        Err(MutationError::UnknownFunction(function_id))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
}

// AR-EXPR-037 and remaining AR-EXPR-038 unknown-identity cases.
#[test]
fn body_mutations_against_unknown_identities_fail_atomically() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, function_id, parameters) =
        add_module_and_function(&mut program, IntrinsicType::Unit, vec![IntrinsicType::Unit]);
    let (unknown_block, unknown_expression) = add_unit_body(&mut program, function_id);
    let mut transaction = program.begin_transaction();
    transaction.remove_function(function_id).unwrap();
    transaction.commit().unwrap();
    let source_revision = program.revision_id();

    let mut transaction = program.begin_transaction();
    let provisional_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    assert_eq!(
        transaction.add_unit_literal(unknown_block),
        Err(MutationError::UnknownBlock(unknown_block))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    assert!(transaction.commit().is_err());
    drop(transaction);
    assert_eq!(program.revision_id(), source_revision);
    assert!(program.function(provisional_function).is_none());

    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.add_parameter_reference(unknown_block, parameters[0]),
        Err(MutationError::UnknownBlock(unknown_block))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);

    let mut transaction = program.begin_transaction();
    let new_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let new_block = transaction.create_function_body(new_function).unwrap();
    let new_expression = transaction.add_unit_literal(new_block).unwrap();
    assert_eq!(
        transaction.set_return(unknown_block, new_expression),
        Err(MutationError::UnknownBlock(unknown_block))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);

    let mut transaction = program.begin_transaction();
    let new_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let new_block = transaction.create_function_body(new_function).unwrap();
    let new_expression = transaction.add_unit_literal(new_block).unwrap();
    assert!(transaction.is_expression_id_provisional(new_expression));
    assert_eq!(
        transaction.set_return(new_block, unknown_expression),
        Err(MutationError::UnknownExpression(unknown_expression))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
}

// AR-EXPR-042 and MNIR-EXPR-096.
#[test]
fn return_presence_is_inspectable_in_working_and_committed_states() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    assert_eq!(
        transaction.block(block_id).unwrap().return_expression_id(),
        None
    );
    let expression_id = transaction.add_unit_literal(block_id).unwrap();
    transaction.set_return(block_id, expression_id).unwrap();
    assert_eq!(
        transaction.block(block_id).unwrap().return_expression_id(),
        Some(expression_id)
    );
    let snapshot = transaction.commit().unwrap();
    assert_eq!(
        snapshot.block(block_id).unwrap().return_expression_id(),
        Some(expression_id)
    );
}

// AR-EXPR-028 is demonstrated by the private-state corruption test in
// program::validation. AR-EXPR-029 is demonstrated by the complete pre-existing
// workspace test suite. AR-EXPR-030 and AR-EXPR-031 are conformance-inspection
// evidence: this increment exposes only the five closed Expression alternatives,
// a single Block, and Return; it adds no arithmetic, conversion, comparison,
// local, assignment, call, branch, loop, generic Node, Statement, or TerminatorId.
