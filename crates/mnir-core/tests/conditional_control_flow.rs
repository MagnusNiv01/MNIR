use mnir_core::{
    BlockId, ExpressionId, FunctionId, IntrinsicType, MnirProgram, ModuleId, MutationError,
    MutationTransaction, StructuralError, Terminator, TransactionState,
};

// AR-CFG-001 through AR-CFG-008, AR-CFG-017, AR-CFG-038, and AR-CFG-048.
#[test]
fn cfg_construction_replacement_and_read_only_inspection() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Int32)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let true_block = transaction.add_block(function).unwrap();
    let false_block = transaction.add_block(function).unwrap();
    assert_ne!(entry, true_block);
    assert_ne!(entry, false_block);
    assert_ne!(true_block, false_block);
    assert!(transaction.is_block_id_provisional(true_block));

    let condition = transaction.add_bool_literal(entry, true).unwrap();
    let entry_value = transaction.add_int32_literal(entry, 0).unwrap();
    transaction.set_return(entry, entry_value).unwrap();
    transaction
        .set_branch(entry, condition, true_block, false_block)
        .unwrap();
    assert!(matches!(
        transaction.block(entry).unwrap().terminator(),
        Some(Terminator::Branch {
            condition: actual_condition,
            true_block: actual_true,
            false_block: actual_false,
        }) if *actual_condition == condition
            && *actual_true == true_block
            && *actual_false == false_block
    ));

    let true_value = transaction.add_int32_literal(true_block, 1).unwrap();
    transaction.set_return(true_block, true_value).unwrap();
    let false_value = transaction.add_int32_literal(false_block, 2).unwrap();
    transaction.set_return(false_block, false_value).unwrap();

    // Return replaces Branch, then Branch replaces Return without creating a
    // separate terminator identity (MNIR-CFG-026, MNIR-CFG-033).
    transaction.set_return(entry, entry_value).unwrap();
    assert_eq!(
        transaction.block(entry).unwrap().return_expression_id(),
        Some(entry_value)
    );
    transaction
        .set_branch(entry, condition, true_block, false_block)
        .unwrap();

    let snapshot = transaction.commit().unwrap();
    let body = snapshot.function(function).unwrap().body().unwrap();
    assert_eq!(body.entry_block_id(), entry);
    assert_eq!(body.block_id(), entry);
    assert_eq!(body.block().id(), entry);
    assert_eq!(body.block_count(), 3);
    assert_eq!(body.blocks().count(), 3);
    assert_eq!(body.block_by_id(true_block).unwrap().id(), true_block);
    assert!(program.is_block_id_committed(true_block));
}

// AR-CFG-008: duplicate semantic edges are allowed and do not alone form a cycle.
#[test]
fn identical_branch_targets_are_valid() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let target = transaction.add_block(function).unwrap();
    let condition = transaction.add_bool_literal(entry, false).unwrap();
    transaction
        .set_branch(entry, condition, target, target)
        .unwrap();
    let unit = transaction.add_unit_literal(target).unwrap();
    transaction.set_return(target, unit).unwrap();
    transaction.commit().unwrap();
    assert!(program.validate_structure().is_ok());
}

// AR-CFG-009 through AR-CFG-012 and AR-CFG-043.
#[test]
fn set_branch_rejects_unknown_and_foreign_references_atomically() {
    fn fixture() -> (
        MnirProgram,
        BlockId,
        BlockId,
        ExpressionId,
        FunctionId,
        BlockId,
        ExpressionId,
    ) {
        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let module = transaction.add_module().unwrap();
        let first = transaction
            .add_function(module, IntrinsicType::Unit)
            .unwrap();
        let first_entry = transaction.create_function_body(first).unwrap();
        let first_target = transaction.add_block(first).unwrap();
        let condition = transaction.add_bool_literal(first_entry, true).unwrap();
        let target_unit = transaction.add_unit_literal(first_target).unwrap();
        transaction.set_return(first_target, target_unit).unwrap();
        transaction
            .set_branch(first_entry, condition, first_target, first_target)
            .unwrap();
        let second = transaction
            .add_function(module, IntrinsicType::Unit)
            .unwrap();
        let second_entry = transaction.create_function_body(second).unwrap();
        let second_unit = transaction.add_unit_literal(second_entry).unwrap();
        transaction.set_return(second_entry, second_unit).unwrap();
        let removed_function = transaction
            .add_function(module, IntrinsicType::Unit)
            .unwrap();
        let removed_block = transaction.create_function_body(removed_function).unwrap();
        let removed_expression = transaction.add_unit_literal(removed_block).unwrap();
        transaction
            .set_return(removed_block, removed_expression)
            .unwrap();
        transaction.remove_function(removed_function).unwrap();
        transaction.commit().unwrap();
        (
            program,
            first_entry,
            first_target,
            condition,
            second,
            removed_block,
            removed_expression,
        )
    }

    let (mut program, entry, target, _, _, _, unknown_expression) = fixture();
    let revision = program.revision_id();
    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.set_branch(entry, unknown_expression, target, target),
        Err(MutationError::UnknownExpression(unknown_expression))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    drop(transaction);
    assert_eq!(program.revision_id(), revision);
    assert!(matches!(
        program.block(entry).unwrap().terminator(),
        Some(Terminator::Branch { .. })
    ));

    let (mut program, entry, target, _, second, _, _) = fixture();
    let foreign_entry = program.function(second).unwrap().body().unwrap().block_id();
    let foreign_condition = program
        .block(foreign_entry)
        .unwrap()
        .expressions()
        .next()
        .unwrap()
        .id();
    let mut transaction = program.begin_transaction();
    assert!(matches!(
        transaction.set_branch(entry, foreign_condition, target, target),
        Err(MutationError::ExpressionNotOwnedByBlock { .. })
    ));

    let (mut program, entry, target, condition, _, unknown_block, _) = fixture();
    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.set_branch(entry, condition, unknown_block, target),
        Err(MutationError::UnknownBlock(unknown_block))
    );

    let (mut program, entry, target, condition, _, unknown_block, _) = fixture();
    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.set_branch(entry, condition, target, unknown_block),
        Err(MutationError::UnknownBlock(unknown_block))
    );

    let (mut program, entry, target, condition, second, _, _) = fixture();
    let foreign_entry = program.function(second).unwrap().body().unwrap().block_id();
    let mut transaction = program.begin_transaction();
    assert!(matches!(
        transaction.set_branch(entry, condition, foreign_entry, target),
        Err(MutationError::BlockNotOwnedByFunctionBody { .. })
    ));

    let (mut program, _, target, condition, _, unknown_block, _) = fixture();
    let revision = program.revision_id();
    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.set_branch(unknown_block, condition, target, target),
        Err(MutationError::UnknownBlock(unknown_block))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    drop(transaction);
    assert_eq!(program.revision_id(), revision);
}

// AR-CFG-013 and AR-CFG-014.
#[test]
fn commit_rejects_unterminated_and_unreachable_blocks() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let unit = transaction.add_unit_literal(entry).unwrap();
    transaction.set_return(entry, unit).unwrap();
    let unterminated = transaction.add_block(function).unwrap();
    assert_eq!(
        transaction.commit().unwrap_err(),
        MutationError::StructuralViolation(StructuralError::UnterminatedBlock(unterminated))
    );

    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let entry_unit = transaction.add_unit_literal(entry).unwrap();
    transaction.set_return(entry, entry_unit).unwrap();
    let unreachable = transaction.add_block(function).unwrap();
    let unit = transaction.add_unit_literal(unreachable).unwrap();
    transaction.set_return(unreachable, unit).unwrap();
    assert!(matches!(
        transaction.commit(),
        Err(MutationError::StructuralViolation(
            StructuralError::UnreachableBlock { block_id, .. }
        )) if block_id == unreachable
    ));
}

// AR-CFG-015 and AR-CFG-016.
#[test]
fn commit_rejects_direct_and_indirect_cfg_cycles() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let condition = transaction.add_bool_literal(entry, true).unwrap();
    transaction
        .set_branch(entry, condition, entry, entry)
        .unwrap();
    assert_eq!(
        transaction.commit().unwrap_err(),
        MutationError::StructuralViolation(StructuralError::CyclicControlFlow(function))
    );

    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let second = transaction.add_block(function).unwrap();
    let first_condition = transaction.add_bool_literal(entry, true).unwrap();
    let second_condition = transaction.add_bool_literal(second, false).unwrap();
    transaction
        .set_branch(entry, first_condition, second, second)
        .unwrap();
    transaction
        .set_branch(second, second_condition, entry, entry)
        .unwrap();
    assert_eq!(
        transaction.commit().unwrap_err(),
        MutationError::StructuralViolation(StructuralError::CyclicControlFlow(function))
    );
}

// AR-CFG-018 through AR-CFG-021, AR-CFG-039, and MNIR-CFG-063 through -068.
#[test]
fn block_removal_is_repairable_and_preserves_allocation_history() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let removed = transaction.add_block(function).unwrap();
    let condition = transaction.add_bool_literal(entry, true).unwrap();
    let entry_unit = transaction.add_unit_literal(entry).unwrap();
    let removed_expression = transaction.add_unit_literal(removed).unwrap();
    transaction.set_return(removed, removed_expression).unwrap();
    transaction
        .set_branch(entry, condition, removed, removed)
        .unwrap();
    transaction.remove_block(removed).unwrap();
    assert_eq!(transaction.state(), TransactionState::Active);
    assert!(transaction.expression(removed_expression).is_none());
    transaction.set_return(entry, entry_unit).unwrap();
    transaction.commit().unwrap();
    assert!(program.is_block_id_committed(removed));
    assert!(program.is_expression_id_committed(removed_expression));

    let mut transaction = program.begin_transaction();
    let next = transaction.add_block(function).unwrap();
    assert_ne!(next, removed);
    let unit = transaction.add_unit_literal(next).unwrap();
    transaction.set_return(next, unit).unwrap();
    let condition = transaction.add_bool_literal(entry, true).unwrap();
    transaction
        .set_branch(entry, condition, next, next)
        .unwrap();
    transaction.commit().unwrap();

    let revision = program.revision_id();
    let mut transaction = program.begin_transaction();
    transaction.remove_block(next).unwrap();
    assert_eq!(transaction.state(), TransactionState::Active);
    assert_eq!(
        transaction.commit().unwrap_err(),
        MutationError::StructuralViolation(StructuralError::BranchTargetNotInBody {
            block_id: entry,
            target_block_id: next,
        })
    );
    drop(transaction);
    assert_eq!(program.revision_id(), revision);
    assert!(program.block(next).is_some());

    let mut transaction = program.begin_transaction();
    transaction.remove_block(next).unwrap();
    let replacement = transaction.add_unit_literal(entry).unwrap();
    transaction.set_return(entry, replacement).unwrap();
    transaction.commit().unwrap();
    assert!(program.block(next).is_none());

    let mut transaction = program.begin_transaction();
    let later = transaction.add_block(function).unwrap();
    assert_ne!(later, next);
    transaction.discard().unwrap();

    let mut transaction = program.begin_transaction();
    assert_eq!(
        transaction.remove_block(entry),
        Err(MutationError::EntryBlockCannotBeRemoved(entry))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
}

// AR-CFG-022 and AR-CFG-023.
#[test]
fn snapshot_and_fork_preserve_cfg_references() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let target = transaction.add_block(function).unwrap();
    let condition = transaction.add_bool_literal(entry, true).unwrap();
    transaction
        .set_branch(entry, condition, target, target)
        .unwrap();
    let unit = transaction.add_unit_literal(target).unwrap();
    transaction.set_return(target, unit).unwrap();
    let snapshot = transaction.commit().unwrap();
    let fork = snapshot.fork().unwrap();

    for inspected in [&snapshot, &fork.snapshot()] {
        let body = inspected.function(function).unwrap().body().unwrap();
        assert_eq!(body.entry_block_id(), entry);
        assert!(body.block_by_id(target).is_some());
        assert_eq!(
            body.block().terminator(),
            Some(&Terminator::Branch {
                condition,
                true_block: target,
                false_block: target,
            })
        );
    }
}

// MNIR-CFG-063 through MNIR-CFG-066: every ownership-level cascade removes
// all CFG state while retaining committed allocation history.
#[test]
fn body_function_and_module_removal_cascade_all_cfg_state() {
    fn add_cfg(
        transaction: &mut MutationTransaction<'_>,
        module: ModuleId,
    ) -> (FunctionId, [BlockId; 2], [ExpressionId; 2]) {
        let function = transaction
            .add_function(module, IntrinsicType::Unit)
            .unwrap();
        let entry = transaction.create_function_body(function).unwrap();
        let target = transaction.add_block(function).unwrap();
        let condition = transaction.add_bool_literal(entry, true).unwrap();
        transaction
            .set_branch(entry, condition, target, target)
            .unwrap();
        let unit = transaction.add_unit_literal(target).unwrap();
        transaction.set_return(target, unit).unwrap();
        (function, [entry, target], [condition, unit])
    }

    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let body_module = transaction.add_module().unwrap();
    let function_module = transaction.add_module().unwrap();
    let removed_module = transaction.add_module().unwrap();
    let (body_function, body_blocks, body_expressions) = add_cfg(&mut transaction, body_module);
    let (removed_function, function_blocks, function_expressions) =
        add_cfg(&mut transaction, function_module);
    let (_, module_blocks, module_expressions) = add_cfg(&mut transaction, removed_module);
    transaction.commit().unwrap();

    let mut transaction = program.begin_transaction();
    transaction.remove_function_body(body_function).unwrap();
    transaction.remove_function(removed_function).unwrap();
    transaction.remove_module(removed_module).unwrap();
    transaction.commit().unwrap();

    for block in body_blocks
        .into_iter()
        .chain(function_blocks)
        .chain(module_blocks)
    {
        assert!(program.block(block).is_none());
        assert!(program.is_block_id_committed(block));
    }
    for expression in body_expressions
        .into_iter()
        .chain(function_expressions)
        .chain(module_expressions)
    {
        assert!(program.expression(expression).is_none());
        assert!(program.is_expression_id_committed(expression));
    }
}

// AR-CFG-041 and AR-CFG-042.
#[test]
fn add_and_remove_block_target_failures_poison_transactions() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let unknown_function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    transaction.remove_function(unknown_function).unwrap();
    assert_eq!(
        transaction.add_block(unknown_function),
        Err(MutationError::UnknownFunction(unknown_function))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);

    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    assert_eq!(
        transaction.add_block(function),
        Err(MutationError::FunctionBodyAbsent(function))
    );
    assert!(transaction.function(function).unwrap().body().is_none());

    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let entry = transaction.create_function_body(function).unwrap();
    let unit = transaction.add_unit_literal(entry).unwrap();
    transaction.set_return(entry, unit).unwrap();
    let removed = transaction.add_block(function).unwrap();
    transaction.remove_block(removed).unwrap();
    assert_eq!(
        transaction.remove_block(removed),
        Err(MutationError::UnknownBlock(removed))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
}
