use std::collections::HashSet;

use mnir_core::{
    BlockId, ExpressionId, ExpressionKind, ExpressionTypeError, FunctionId, IntrinsicType,
    MnirProgram, ModuleId, MutationError, ParameterId, TransactionState,
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

fn add_committed_unit_body(
    program: &mut MnirProgram,
    function_id: FunctionId,
) -> (BlockId, ExpressionId) {
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let expression_id = transaction.add_unit_literal(block_id).unwrap();
    transaction.set_return(block_id, expression_id).unwrap();
    transaction.commit().unwrap();
    (block_id, expression_id)
}

// AR-ARITH-001, AR-ARITH-002, AR-ARITH-003, AR-ARITH-004,
// AR-ARITH-005, and AR-ARITH-006.
#[test]
fn all_arithmetic_operators_preserve_identity_operands_and_supported_types() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Int32, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let i32_left = transaction.add_int32_literal(block_id, 12).unwrap();
    let i32_right = transaction.add_int32_literal(block_id, 5).unwrap();
    let i64_left = transaction.add_int64_literal(block_id, 12).unwrap();
    let i64_right = transaction.add_int64_literal(block_id, 5).unwrap();
    let add = transaction
        .add_add_expression(block_id, i32_left, i32_right)
        .unwrap();
    let add_i64 = transaction
        .add_add_expression(block_id, i64_left, i64_right)
        .unwrap();
    let subtract = transaction
        .add_subtract_expression(block_id, i32_left, i32_right)
        .unwrap();
    let multiply = transaction
        .add_multiply_expression(block_id, i32_left, i32_right)
        .unwrap();
    let divide = transaction
        .add_divide_expression(block_id, i32_left, i32_right)
        .unwrap();
    let remainder = transaction
        .add_remainder_expression(block_id, i32_left, i32_right)
        .unwrap();
    assert!(transaction.is_expression_id_provisional(add));
    transaction.set_return(block_id, add).unwrap();
    transaction.commit().unwrap();

    assert_eq!(
        program.expression(add).unwrap().kind(),
        &ExpressionKind::Add {
            left: i32_left,
            right: i32_right,
        }
    );
    assert_eq!(
        program.expression(subtract).unwrap().kind(),
        &ExpressionKind::Subtract {
            left: i32_left,
            right: i32_right,
        }
    );
    assert_eq!(
        program.expression(multiply).unwrap().kind(),
        &ExpressionKind::Multiply {
            left: i32_left,
            right: i32_right,
        }
    );
    assert_eq!(
        program.expression(divide).unwrap().kind(),
        &ExpressionKind::Divide {
            left: i32_left,
            right: i32_right,
        }
    );
    assert_eq!(
        program.expression(remainder).unwrap().kind(),
        &ExpressionKind::Remainder {
            left: i32_left,
            right: i32_right,
        }
    );
    for expression_id in [add, subtract, multiply, divide, remainder] {
        assert_eq!(
            program.expression_type(expression_id),
            Some(Ok(IntrinsicType::Int32))
        );
    }
    assert_eq!(
        program.expression_type(add_i64),
        Some(Ok(IntrinsicType::Int64))
    );
}

// AR-ARITH-007 and MNIR-ARITH-014/-022/-031/-032/-055 through -057.
#[test]
fn mixed_integer_arithmetic_commits_both_orientations_as_type_mismatch() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let int32 = transaction.add_int32_literal(block_id, 1).unwrap();
    let int64 = transaction.add_int64_literal(block_id, 2).unwrap();
    let first = transaction
        .add_add_expression(block_id, int32, int64)
        .unwrap();
    let second = transaction
        .add_add_expression(block_id, int64, int32)
        .unwrap();
    transaction.set_return(block_id, first).unwrap();
    transaction.commit().unwrap();

    for expression_id in [first, second] {
        assert_eq!(
            program.expression_type(expression_id),
            Some(Err(ExpressionTypeError::OperandTypeMismatch {
                expression_id,
            }))
        );
    }
    assert!(program.validate_structure().is_ok());
}

// AR-ARITH-008, AR-ARITH-009, and part of AR-ARITH-031.
#[test]
fn bool_and_unit_arithmetic_are_structural_but_have_unsupported_type_outcomes() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let bool_left = transaction.add_bool_literal(block_id, false).unwrap();
    let bool_right = transaction.add_bool_literal(block_id, true).unwrap();
    let unit = transaction.add_unit_literal(block_id).unwrap();
    let bool_add = transaction
        .add_add_expression(block_id, bool_left, bool_right)
        .unwrap();
    let unit_multiply = transaction
        .add_multiply_expression(block_id, unit, unit)
        .unwrap();
    transaction.set_return(block_id, unit_multiply).unwrap();
    transaction.commit().unwrap();

    for expression_id in [bool_add, unit_multiply] {
        assert_eq!(
            program.expression_type(expression_id),
            Some(Err(ExpressionTypeError::UnsupportedOperandType {
                expression_id,
            }))
        );
    }
    assert!(program.validate_structure().is_ok());
}

fn program_with_two_body_blocks() -> (
    MnirProgram,
    ModuleId,
    FunctionId,
    BlockId,
    ExpressionId,
    ExpressionId,
) {
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
    let second_expression = transaction.add_unit_literal(second_block).unwrap();
    transaction
        .set_return(first_block, first_expression)
        .unwrap();
    transaction
        .set_return(second_block, second_expression)
        .unwrap();
    transaction.commit().unwrap();
    (
        program,
        module_id,
        first_function,
        first_block,
        first_expression,
        second_expression,
    )
}

// AR-ARITH-010: both left-foreign and right-foreign cases are atomic.
#[test]
fn cross_block_operands_fail_and_poison_both_positions() {
    for foreign_on_left in [true, false] {
        let (mut program, _, _, block_id, local, foreign) = program_with_two_body_blocks();
        let source_revision = program.revision_id();
        let source_history = program.committed_expression_id_count();
        let mut transaction = program.begin_transaction();
        let provisional = transaction.add_unit_literal(block_id).unwrap();
        let (left, right) = if foreign_on_left {
            (foreign, local)
        } else {
            (local, foreign)
        };
        assert_eq!(
            transaction.add_add_expression(block_id, left, right),
            Err(MutationError::ExpressionNotOwnedByBlock {
                expression_id: foreign,
                block_id,
            })
        );
        assert_eq!(transaction.state(), TransactionState::Failed);
        assert!(transaction.commit().is_err());
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision);
        assert_eq!(program.committed_expression_id_count(), source_history);
        assert!(program.expression(provisional).is_none());
    }
}

fn retired_body_identity_program() -> (MnirProgram, ModuleId, FunctionId, BlockId, ExpressionId) {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, function_id, _) =
        add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let (retired_block, retired_expression) = add_committed_unit_body(&mut program, function_id);
    let mut transaction = program.begin_transaction();
    transaction.remove_function_body(function_id).unwrap();
    transaction.commit().unwrap();
    (
        program,
        module_id,
        function_id,
        retired_block,
        retired_expression,
    )
}

// AR-ARITH-011: both left-unknown and right-unknown cases are atomic.
#[test]
fn unknown_operands_fail_and_poison_both_positions() {
    for unknown_on_left in [true, false] {
        let (mut program, _, function_id, _, unknown) = retired_body_identity_program();
        let source_revision = program.revision_id();
        let source_history = program.committed_expression_id_count();
        let mut transaction = program.begin_transaction();
        let block_id = transaction.create_function_body(function_id).unwrap();
        let local = transaction.add_unit_literal(block_id).unwrap();
        let (left, right) = if unknown_on_left {
            (unknown, local)
        } else {
            (local, unknown)
        };
        assert_eq!(
            transaction.add_add_expression(block_id, left, right),
            Err(MutationError::UnknownExpression(unknown))
        );
        assert_eq!(transaction.state(), TransactionState::Failed);
        assert!(transaction.commit().is_err());
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision);
        assert_eq!(program.committed_expression_id_count(), source_history);
        assert!(program.block(block_id).is_none());
    }
}

// AR-ARITH-012.
#[test]
fn arithmetic_construction_against_unknown_block_fails_and_poisons() {
    let (mut program, module_id, _, unknown_block, unknown_expression) =
        retired_body_identity_program();
    let source_revision = program.revision_id();
    let mut transaction = program.begin_transaction();
    let provisional_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    assert_eq!(
        transaction.add_add_expression(unknown_block, unknown_expression, unknown_expression),
        Err(MutationError::UnknownBlock(unknown_block))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    assert!(transaction.commit().is_err());
    drop(transaction);
    assert_eq!(program.revision_id(), source_revision);
    assert!(program.function(provisional_function).is_none());
}

// AR-ARITH-013, AR-ARITH-016, and AR-ARITH-032.
#[test]
fn nested_ordered_and_identical_operands_are_valid_and_derived() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Int32, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let first = transaction.add_int32_literal(block_id, 3).unwrap();
    let second = transaction.add_int32_literal(block_id, 4).unwrap();
    let add = transaction
        .add_add_expression(block_id, first, second)
        .unwrap();
    let nested = transaction
        .add_multiply_expression(block_id, add, first)
        .unwrap();
    let forward = transaction
        .add_subtract_expression(block_id, first, second)
        .unwrap();
    let reverse = transaction
        .add_subtract_expression(block_id, second, first)
        .unwrap();
    let repeated = transaction
        .add_add_expression(block_id, first, first)
        .unwrap();
    transaction.set_return(block_id, nested).unwrap();
    transaction.commit().unwrap();

    assert_eq!(
        program.expression_type(nested),
        Some(Ok(IntrinsicType::Int32))
    );
    assert_eq!(
        program.expression(forward).unwrap().kind(),
        &ExpressionKind::Subtract {
            left: first,
            right: second,
        }
    );
    assert_eq!(
        program.expression(reverse).unwrap().kind(),
        &ExpressionKind::Subtract {
            left: second,
            right: first,
        }
    );
    assert_eq!(
        program.expression(repeated).unwrap().kind(),
        &ExpressionKind::Add {
            left: first,
            right: first,
        }
    );
    assert!(program.validate_structure().is_ok());
}

// AR-ARITH-014 and MNIR-ARITH-094/-095.
#[test]
fn nested_arithmetic_type_failures_become_operand_unavailable() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let int32 = transaction.add_int32_literal(block_id, 1).unwrap();
    let int64 = transaction.add_int64_literal(block_id, 2).unwrap();
    let bool_value = transaction.add_bool_literal(block_id, true).unwrap();
    let mismatch = transaction
        .add_add_expression(block_id, int32, int64)
        .unwrap();
    let unsupported = transaction
        .add_add_expression(block_id, bool_value, bool_value)
        .unwrap();
    let outer_mismatch = transaction
        .add_multiply_expression(block_id, mismatch, int32)
        .unwrap();
    let outer_unsupported = transaction
        .add_multiply_expression(block_id, unsupported, int32)
        .unwrap();
    transaction.set_return(block_id, outer_mismatch).unwrap();
    transaction.commit().unwrap();

    for expression_id in [outer_mismatch, outer_unsupported] {
        assert_eq!(
            program.expression_type(expression_id),
            Some(Err(ExpressionTypeError::OperandTypeUnavailable {
                expression_id,
            }))
        );
    }
}

// AR-ARITH-015 and the unavailable arm of AR-ARITH-031.
#[test]
fn dangling_parameter_type_failure_propagates_without_poisoning() {
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
    let literal = transaction.add_int32_literal(block_id, 1).unwrap();
    let add = transaction
        .add_add_expression(block_id, reference, literal)
        .unwrap();
    let unavailable_on_right = transaction
        .add_add_expression(block_id, literal, reference)
        .unwrap();
    let both_unavailable = transaction
        .add_add_expression(block_id, reference, reference)
        .unwrap();
    transaction.set_return(block_id, add).unwrap();
    transaction.commit().unwrap();

    let mut transaction = program.begin_transaction();
    transaction.remove_parameter(parameter_id).unwrap();
    assert!(matches!(
        transaction.expression_type(reference),
        Some(Err(ExpressionTypeError::UnresolvedParameter { .. }))
    ));
    for expression_id in [add, unavailable_on_right, both_unavailable] {
        assert_eq!(
            transaction.expression_type(expression_id),
            Some(Err(ExpressionTypeError::OperandTypeUnavailable {
                expression_id,
            }))
        );
    }
    assert_eq!(transaction.state(), TransactionState::Active);
    transaction.remove_function_body(function_id).unwrap();
    transaction.commit().unwrap();
    assert!(!program.function(function_id).unwrap().has_body());
}

// AR-ARITH-017, AR-ARITH-018, and MNIR-ARITH-067/-068.
#[test]
fn arithmetic_identities_are_unique_retired_and_removed_by_cascades() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, first_function, _) =
        add_module_and_function(&mut program, IntrinsicType::Unit, vec![]);
    let mut transaction = program.begin_transaction();
    let second_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let third_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let first_block = transaction.create_function_body(first_function).unwrap();
    let second_block = transaction.create_function_body(second_function).unwrap();
    let third_block = transaction.create_function_body(third_function).unwrap();
    let first_literal = transaction.add_unit_literal(first_block).unwrap();
    let second_literal = transaction.add_unit_literal(second_block).unwrap();
    let third_literal = transaction.add_unit_literal(third_block).unwrap();
    let first_arithmetic = transaction
        .add_add_expression(first_block, first_literal, first_literal)
        .unwrap();
    let second_arithmetic = transaction
        .add_add_expression(second_block, second_literal, second_literal)
        .unwrap();
    let third_arithmetic = transaction
        .add_add_expression(third_block, third_literal, third_literal)
        .unwrap();
    let all_ids = [
        first_literal,
        second_literal,
        third_literal,
        first_arithmetic,
        second_arithmetic,
        third_arithmetic,
    ];
    assert_eq!(all_ids.into_iter().collect::<HashSet<_>>().len(), 6);
    assert!(
        all_ids
            .iter()
            .all(|id| transaction.is_expression_id_provisional(*id))
    );
    transaction
        .set_return(first_block, first_arithmetic)
        .unwrap();
    transaction
        .set_return(second_block, second_arithmetic)
        .unwrap();
    transaction
        .set_return(third_block, third_arithmetic)
        .unwrap();
    transaction.commit().unwrap();

    let mut transaction = program.begin_transaction();
    transaction.remove_function_body(first_function).unwrap();
    transaction.remove_function(second_function).unwrap();
    transaction.commit().unwrap();
    for id in [
        first_literal,
        first_arithmetic,
        second_literal,
        second_arithmetic,
    ] {
        assert!(program.expression(id).is_none());
        assert!(program.is_expression_id_committed(id));
    }
    assert!(program.expression(third_arithmetic).is_some());

    let mut transaction = program.begin_transaction();
    transaction.remove_module(module_id).unwrap();
    transaction.commit().unwrap();
    for id in [third_literal, third_arithmetic] {
        assert!(program.expression(id).is_none());
        assert!(program.is_expression_id_committed(id));
    }

    let mut transaction = program.begin_transaction();
    let new_module = transaction.add_module().unwrap();
    let new_function = transaction
        .add_function(new_module, IntrinsicType::Unit)
        .unwrap();
    let new_block = transaction.create_function_body(new_function).unwrap();
    let new_literal = transaction.add_unit_literal(new_block).unwrap();
    let replacement = transaction
        .add_add_expression(new_block, new_literal, new_literal)
        .unwrap();
    transaction.set_return(new_block, replacement).unwrap();
    transaction.commit().unwrap();
    assert!(!all_ids.contains(&replacement));
}

// AR-ARITH-019 and AR-ARITH-020; MNIR-ARITH-064 through -066.
#[test]
fn snapshots_and_forks_preserve_arithmetic_references() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, function_id, parameters) = add_module_and_function(
        &mut program,
        IntrinsicType::Int32,
        vec![IntrinsicType::Int32, IntrinsicType::Int32],
    );
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let left = transaction
        .add_parameter_reference(block_id, parameters[0])
        .unwrap();
    let right = transaction
        .add_parameter_reference(block_id, parameters[1])
        .unwrap();
    let add = transaction
        .add_add_expression(block_id, left, right)
        .unwrap();
    transaction.set_return(block_id, add).unwrap();
    transaction.commit().unwrap();
    let snapshot = program.snapshot();
    assert_eq!(
        snapshot.expression(add).unwrap().kind(),
        &ExpressionKind::Add { left, right }
    );

    let mut fork = snapshot.fork().unwrap();
    assert_ne!(fork.program_id(), program.program_id());
    assert_eq!(
        fork.expression(add).unwrap().kind(),
        &ExpressionKind::Add { left, right }
    );
    assert_eq!(fork.expression_type(add), Some(Ok(IntrinsicType::Int32)));
    assert!(fork.validate_structure().is_ok());
    let mut transaction = fork.begin_transaction();
    transaction.remove_module(module_id).unwrap();
    transaction.commit().unwrap();
    assert!(fork.expression(add).is_none());
    assert!(fork.is_expression_id_committed(add));
}

// AR-ARITH-021 and AR-ARITH-022.
#[test]
fn arithmetic_can_be_returned_and_return_mismatch_remains_structural() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, add_function, parameters) = add_module_and_function(
        &mut program,
        IntrinsicType::Int32,
        vec![IntrinsicType::Int32, IntrinsicType::Int32],
    );
    let mut transaction = program.begin_transaction();
    let mismatch_function = transaction
        .add_function(module_id, IntrinsicType::Int32)
        .unwrap();
    let add_block = transaction.create_function_body(add_function).unwrap();
    let left = transaction
        .add_parameter_reference(add_block, parameters[0])
        .unwrap();
    let right = transaction
        .add_parameter_reference(add_block, parameters[1])
        .unwrap();
    let add = transaction
        .add_add_expression(add_block, left, right)
        .unwrap();
    transaction.set_return(add_block, add).unwrap();
    let mismatch_block = transaction.create_function_body(mismatch_function).unwrap();
    let int64_left = transaction.add_int64_literal(mismatch_block, 1).unwrap();
    let int64_right = transaction.add_int64_literal(mismatch_block, 2).unwrap();
    let mismatch_return = transaction
        .add_add_expression(mismatch_block, int64_left, int64_right)
        .unwrap();
    transaction
        .set_return(mismatch_block, mismatch_return)
        .unwrap();
    transaction.commit().unwrap();

    assert_eq!(
        program.block(add_block).unwrap().return_expression_id(),
        Some(add)
    );
    assert_eq!(program.expression_type(add), Some(Ok(IntrinsicType::Int32)));
    assert_eq!(
        program.expression_type(mismatch_return),
        Some(Ok(IntrinsicType::Int64))
    );
    assert_eq!(
        program.function(mismatch_function).unwrap().return_type(),
        &IntrinsicType::Int32
    );
    assert!(program.validate_structure().is_ok());
}

// AR-ARITH-023 and the structural evidence for MNIR-ARITH-058/-059.
#[test]
fn guaranteed_overflow_expression_is_representable_without_evaluation() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Int32, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let maximum = transaction.add_int32_literal(block_id, i32::MAX).unwrap();
    let one = transaction.add_int32_literal(block_id, 1).unwrap();
    let add = transaction
        .add_add_expression(block_id, maximum, one)
        .unwrap();
    transaction.set_return(block_id, add).unwrap();
    transaction.commit().unwrap();
    assert_eq!(
        program.expression(maximum).unwrap().kind(),
        &ExpressionKind::Int32Literal(i32::MAX)
    );
    assert_eq!(program.expression_type(add), Some(Ok(IntrinsicType::Int32)));
}

// AR-ARITH-024, AR-ARITH-025, and AR-ARITH-026 are conformance-inspection
// evidence: the implementation represents only Add, Subtract, Multiply,
// Divide, and Remainder. These names carry the checked mathematical and fault
// semantics of MNIR-ARITH-034 through MNIR-ARITH-059; there is deliberately no
// evaluator, fault value/API, wrapping variant, or saturating variant.
//
// AR-ARITH-027 is completed by the private-state cyclic-candidate test in
// program::validation. AR-ARITH-028 and AR-ARITH-029 are API/conformance
// inspection evidence: no operator/operand retargeting method, interpreter,
// evaluator, constant folder, or backend was introduced. AR-ARITH-030 is
// demonstrated by the full pre-existing workspace test suite. AR-ARITH-031 is
// covered across the valid, unavailable, mismatch, and unsupported tests above.
