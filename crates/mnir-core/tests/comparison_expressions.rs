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

// AR-CMP-001 through AR-CMP-006, AR-CMP-014 through AR-CMP-016, and
// AR-CMP-021; MNIR-CMP-005 through -010, -017/-018, -021 through -030,
// -034, -044 through -052, -063 through -066.
#[test]
fn comparison_kinds_preserve_operands_and_derive_bool_for_supported_types() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Bool, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let i32_left = transaction.add_int32_literal(block_id, -4).unwrap();
    let i32_right = transaction.add_int32_literal(block_id, 9).unwrap();
    let i64_left = transaction.add_int64_literal(block_id, -4).unwrap();
    let i64_right = transaction.add_int64_literal(block_id, 9).unwrap();
    let bool_left = transaction.add_bool_literal(block_id, false).unwrap();
    let bool_right = transaction.add_bool_literal(block_id, true).unwrap();
    let unit = transaction.add_unit_literal(block_id).unwrap();

    let equal_i32 = transaction
        .add_equal_expression(block_id, i32_left, i32_right)
        .unwrap();
    let not_equal_i64 = transaction
        .add_not_equal_expression(block_id, i64_left, i64_right)
        .unwrap();
    let equal_bool = transaction
        .add_equal_expression(block_id, bool_left, bool_right)
        .unwrap();
    let equal_unit = transaction
        .add_equal_expression(block_id, unit, unit)
        .unwrap();
    let less = transaction
        .add_less_than_expression(block_id, i32_left, i32_right)
        .unwrap();
    let less_equal = transaction
        .add_less_than_or_equal_expression(block_id, i32_left, i32_right)
        .unwrap();
    let greater = transaction
        .add_greater_than_expression(block_id, i32_left, i32_right)
        .unwrap();
    let greater_equal = transaction
        .add_greater_than_or_equal_expression(block_id, i32_left, i32_right)
        .unwrap();
    let less_i64 = transaction
        .add_less_than_expression(block_id, i64_left, i64_right)
        .unwrap();
    let reverse = transaction
        .add_greater_than_expression(block_id, i32_right, i32_left)
        .unwrap();
    let identical = transaction
        .add_equal_expression(block_id, i32_left, i32_left)
        .unwrap();
    let nested = transaction
        .add_equal_expression(block_id, greater, bool_right)
        .unwrap();

    assert!(transaction.is_expression_id_provisional(greater));
    assert_eq!(
        transaction.expression_type(nested),
        Some(Ok(IntrinsicType::Bool))
    );
    transaction.set_return(block_id, greater).unwrap();
    transaction.commit().unwrap();

    let expected = [
        (
            equal_i32,
            ExpressionKind::Equal {
                left: i32_left,
                right: i32_right,
            },
        ),
        (
            not_equal_i64,
            ExpressionKind::NotEqual {
                left: i64_left,
                right: i64_right,
            },
        ),
        (
            less,
            ExpressionKind::LessThan {
                left: i32_left,
                right: i32_right,
            },
        ),
        (
            less_equal,
            ExpressionKind::LessThanOrEqual {
                left: i32_left,
                right: i32_right,
            },
        ),
        (
            greater,
            ExpressionKind::GreaterThan {
                left: i32_left,
                right: i32_right,
            },
        ),
        (
            greater_equal,
            ExpressionKind::GreaterThanOrEqual {
                left: i32_left,
                right: i32_right,
            },
        ),
        (
            reverse,
            ExpressionKind::GreaterThan {
                left: i32_right,
                right: i32_left,
            },
        ),
        (
            identical,
            ExpressionKind::Equal {
                left: i32_left,
                right: i32_left,
            },
        ),
    ];
    for (id, kind) in expected {
        assert_eq!(program.expression(id).unwrap().kind(), &kind);
    }

    for id in [
        equal_i32,
        not_equal_i64,
        equal_bool,
        equal_unit,
        less,
        less_equal,
        greater,
        greater_equal,
        less_i64,
        reverse,
        identical,
        nested,
    ] {
        assert_eq!(program.expression_type(id), Some(Ok(IntrinsicType::Bool)));
    }
    assert_eq!(
        program.block(block_id).unwrap().return_expression_id(),
        Some(greater)
    );
    assert!(program.validate_structure().is_ok());
}

// AR-CMP-007 through AR-CMP-010 and AR-CMP-017; MNIR-CMP-019,
// -031 through -043, -063/-064.
#[test]
fn invalid_comparisons_commit_with_deterministic_typed_outcomes() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Bool, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let int32 = transaction.add_int32_literal(block_id, 1).unwrap();
    let int64 = transaction.add_int64_literal(block_id, 2).unwrap();
    let boolean = transaction.add_bool_literal(block_id, true).unwrap();
    let unit = transaction.add_unit_literal(block_id).unwrap();

    let equality_forward = transaction
        .add_equal_expression(block_id, int32, int64)
        .unwrap();
    let equality_reverse = transaction
        .add_equal_expression(block_id, int64, int32)
        .unwrap();
    let ordering_forward = transaction
        .add_less_than_expression(block_id, int32, int64)
        .unwrap();
    let ordering_reverse = transaction
        .add_greater_than_expression(block_id, int64, int32)
        .unwrap();
    let bool_ordering = transaction
        .add_less_than_expression(block_id, boolean, boolean)
        .unwrap();
    let unit_ordering = transaction
        .add_greater_than_expression(block_id, unit, unit)
        .unwrap();
    let outer = transaction
        .add_equal_expression(block_id, equality_forward, boolean)
        .unwrap();

    assert_eq!(transaction.state(), TransactionState::Active);
    transaction.set_return(block_id, equality_forward).unwrap();
    transaction.commit().unwrap();

    for expression_id in [
        equality_forward,
        equality_reverse,
        ordering_forward,
        ordering_reverse,
    ] {
        assert_eq!(
            program.expression_type(expression_id),
            Some(Err(ExpressionTypeError::OperandTypeMismatch {
                expression_id,
            }))
        );
    }
    for expression_id in [bool_ordering, unit_ordering] {
        assert_eq!(
            program.expression_type(expression_id),
            Some(Err(ExpressionTypeError::UnsupportedOperandType {
                expression_id,
            }))
        );
    }
    assert_eq!(
        program.expression_type(outer),
        Some(Err(ExpressionTypeError::OperandTypeUnavailable {
            expression_id: outer,
        }))
    );
    assert!(program.validate_structure().is_ok());
}

fn program_with_two_blocks() -> (MnirProgram, BlockId, ExpressionId, ExpressionId) {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, first_function, _) =
        add_module_and_function(&mut program, IntrinsicType::Bool, vec![]);
    let mut transaction = program.begin_transaction();
    let second_function = transaction
        .add_function(module_id, IntrinsicType::Bool)
        .unwrap();
    let first_block = transaction.create_function_body(first_function).unwrap();
    let second_block = transaction.create_function_body(second_function).unwrap();
    let first_expression = transaction.add_bool_literal(first_block, true).unwrap();
    let second_expression = transaction.add_bool_literal(second_block, false).unwrap();
    transaction
        .set_return(first_block, first_expression)
        .unwrap();
    transaction
        .set_return(second_block, second_expression)
        .unwrap();
    transaction.commit().unwrap();
    (program, first_block, first_expression, second_expression)
}

// AR-CMP-011; MNIR-CMP-012/-020. Both operand positions poison atomically.
#[test]
fn cross_block_comparison_operands_fail_in_both_positions() {
    for foreign_on_left in [true, false] {
        let (mut program, block_id, local, foreign) = program_with_two_blocks();
        let source_revision = program.revision_id();
        let source_allocation_state = program.allocation_counter_state();
        let mut transaction = program.begin_transaction();
        let provisional = transaction.add_bool_literal(block_id, true).unwrap();
        let (left, right) = if foreign_on_left {
            (foreign, local)
        } else {
            (local, foreign)
        };
        assert_eq!(
            transaction.add_equal_expression(block_id, left, right),
            Err(MutationError::ExpressionNotOwnedByBlock {
                expression_id: foreign,
                block_id,
            })
        );
        assert_eq!(transaction.state(), TransactionState::Failed);
        assert!(transaction.commit().is_err());
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision);
        assert_ne!(program.allocation_counter_state(), source_allocation_state);
        assert!(program.expression(provisional).is_none());
    }
}

fn program_with_retired_body() -> (MnirProgram, FunctionId, BlockId, ExpressionId) {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, _) = add_module_and_function(&mut program, IntrinsicType::Bool, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let expression_id = transaction.add_bool_literal(block_id, true).unwrap();
    transaction.set_return(block_id, expression_id).unwrap();
    transaction.commit().unwrap();
    let mut transaction = program.begin_transaction();
    transaction.remove_function_body(function_id).unwrap();
    transaction.commit().unwrap();
    (program, function_id, block_id, expression_id)
}

// AR-CMP-012 and AR-CMP-013; MNIR-CMP-011/-018/-020.
#[test]
fn unknown_comparison_operands_and_block_fail_atomically() {
    for unknown_on_left in [true, false] {
        let (mut program, function_id, _, unknown_expression) = program_with_retired_body();
        let source_revision = program.revision_id();
        let mut transaction = program.begin_transaction();
        let block_id = transaction.create_function_body(function_id).unwrap();
        let local = transaction.add_bool_literal(block_id, true).unwrap();
        let (left, right) = if unknown_on_left {
            (unknown_expression, local)
        } else {
            (local, unknown_expression)
        };
        assert_eq!(
            transaction.add_not_equal_expression(block_id, left, right),
            Err(MutationError::UnknownExpression(unknown_expression))
        );
        assert_eq!(transaction.state(), TransactionState::Failed);
        assert!(transaction.commit().is_err());
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision);
        assert!(program.block(block_id).is_none());
    }

    let (mut program, function_id, unknown_block, unknown_expression) = program_with_retired_body();
    let source_revision = program.revision_id();
    let mut transaction = program.begin_transaction();
    let provisional_block = transaction.create_function_body(function_id).unwrap();
    assert_eq!(
        transaction.add_equal_expression(unknown_block, unknown_expression, unknown_expression),
        Err(MutationError::UnknownBlock(unknown_block))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    assert!(transaction.commit().is_err());
    drop(transaction);
    assert_eq!(program.revision_id(), source_revision);
    assert!(program.block(provisional_block).is_none());
}

// MNIR-CMP-035/-039/-040/-041. Read-only inspection of a temporarily
// dangling dependency remains non-poisoning and repairable.
#[test]
fn dangling_parameter_comparison_type_is_unavailable_without_poisoning() {
    let mut program = MnirProgram::new().unwrap();
    let (_, function_id, parameters) = add_module_and_function(
        &mut program,
        IntrinsicType::Bool,
        vec![IntrinsicType::Int32],
    );
    let parameter_id = parameters[0];
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let reference = transaction
        .add_parameter_reference(block_id, parameter_id)
        .unwrap();
    let literal = transaction.add_int32_literal(block_id, 1).unwrap();
    let comparison = transaction
        .add_equal_expression(block_id, reference, literal)
        .unwrap();
    transaction.set_return(block_id, comparison).unwrap();
    transaction.commit().unwrap();

    let mut transaction = program.begin_transaction();
    transaction.remove_parameter(parameter_id).unwrap();
    assert!(matches!(
        transaction.expression_type(reference),
        Some(Err(ExpressionTypeError::UnresolvedParameter { .. }))
    ));
    assert_eq!(
        transaction.expression_type(comparison),
        Some(Err(ExpressionTypeError::OperandTypeUnavailable {
            expression_id: comparison,
        }))
    );
    assert_eq!(transaction.state(), TransactionState::Active);
    transaction.remove_function_body(function_id).unwrap();
    transaction.commit().unwrap();
    assert!(program.function(function_id).unwrap().body().is_none());
}

// AR-CMP-018 through AR-CMP-020; MNIR-CMP-006, -057 through -060.
#[test]
fn comparison_identity_snapshot_fork_and_removal_semantics_are_preserved() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, function_id, _) =
        add_module_and_function(&mut program, IntrinsicType::Bool, vec![]);
    let mut transaction = program.begin_transaction();
    let block_id = transaction.create_function_body(function_id).unwrap();
    let left = transaction.add_int32_literal(block_id, 1).unwrap();
    let right = transaction.add_int32_literal(block_id, 2).unwrap();
    let comparison = transaction
        .add_less_than_expression(block_id, left, right)
        .unwrap();
    transaction.set_return(block_id, comparison).unwrap();
    let snapshot = transaction.commit().unwrap();

    // MNIR-CMP-067 and MNIR-PSI-060: an empty no-op commit preserves all
    // comparison data and allocator state while allocating a revision.
    let allocation_state = program.allocation_counter_state();
    let revision_before_no_op = program.revision_id();
    let mut transaction = program.begin_transaction();
    transaction.commit().unwrap();
    assert_ne!(program.revision_id(), revision_before_no_op);
    assert_eq!(program.allocation_counter_state(), allocation_state);
    assert_eq!(
        program.expression(comparison).unwrap().kind(),
        &ExpressionKind::LessThan { left, right }
    );

    let ids: HashSet<_> = snapshot
        .block(block_id)
        .unwrap()
        .expressions()
        .map(|expression| expression.id())
        .collect();
    assert_eq!(ids.len(), 3);
    assert_eq!(
        snapshot.expression(comparison).unwrap().kind(),
        &ExpressionKind::LessThan { left, right }
    );

    let fork = snapshot.fork().unwrap();
    assert_ne!(fork.program_id(), snapshot.program_id());
    assert_eq!(
        fork.expression(comparison).unwrap().kind(),
        &ExpressionKind::LessThan { left, right }
    );
    assert_eq!(
        fork.expression_type(comparison),
        Some(Ok(IntrinsicType::Bool))
    );

    let mut transaction = program.begin_transaction();
    transaction.remove_function_body(function_id).unwrap();
    transaction.commit().unwrap();
    assert!(program.expression(comparison).is_none());

    let mut transaction = program.begin_transaction();
    let function_replacement_block = transaction.create_function_body(function_id).unwrap();
    let function_replacement_literal = transaction
        .add_bool_literal(function_replacement_block, true)
        .unwrap();
    let function_replacement_comparison = transaction
        .add_equal_expression(
            function_replacement_block,
            function_replacement_literal,
            function_replacement_literal,
        )
        .unwrap();
    transaction
        .set_return(function_replacement_block, function_replacement_comparison)
        .unwrap();
    transaction.commit().unwrap();
    assert_ne!(function_replacement_comparison, comparison);

    let mut transaction = program.begin_transaction();
    transaction.remove_function(function_id).unwrap();
    transaction.commit().unwrap();
    assert!(
        program
            .expression(function_replacement_comparison)
            .is_none()
    );

    let mut transaction = program.begin_transaction();
    let module_replacement = transaction
        .add_function(module_id, IntrinsicType::Bool)
        .unwrap();
    let replacement_block = transaction
        .create_function_body(module_replacement)
        .unwrap();
    let replacement_literal = transaction
        .add_bool_literal(replacement_block, true)
        .unwrap();
    let replacement_comparison = transaction
        .add_equal_expression(replacement_block, replacement_literal, replacement_literal)
        .unwrap();
    transaction
        .set_return(replacement_block, replacement_comparison)
        .unwrap();
    transaction.commit().unwrap();

    let mut transaction = program.begin_transaction();
    transaction.remove_module(module_id).unwrap();
    transaction.commit().unwrap();
    assert!(program.expression(replacement_comparison).is_none());
}

// AR-CMP-033/-034 are conformance-inspection requirements: this crate adds no
// evaluator, constant folder, generic Comparable/Ordered trait, operator
// overloading, or comparison-specific identity. AR-CMP-035 is demonstrated by
// the complete unchanged workspace suite. AR-CMP-038 is covered by the private
// cross-family corruption test in program::validation.
