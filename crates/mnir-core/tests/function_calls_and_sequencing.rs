use mnir_core::{
    ExpressionKind, ExpressionTypeError, IntrinsicType, MnirProgram, MutationError,
    StructuralError, Terminator, TransactionState,
};

// AR-CALL-001 through AR-CALL-005, AR-CALL-009 through AR-CALL-011,
// AR-CALL-014, AR-CALL-017, AR-CALL-027, and AR-CALL-028.
#[test]
fn calls_preserve_semantic_data_and_support_required_target_shapes() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let caller_module = tx.add_module().unwrap();
    let target_module = tx.add_module().unwrap();
    let target = tx
        .add_function(target_module, IntrinsicType::Int32)
        .unwrap();
    let parameter = tx.add_parameter(target, IntrinsicType::Int32).unwrap();
    let zero_target = tx.add_function(target_module, IntrinsicType::Unit).unwrap();
    let caller = tx
        .add_function(caller_module, IntrinsicType::Int32)
        .unwrap();
    let block = tx.create_function_body(caller).unwrap();
    assert!(tx.block(block).unwrap().effect_sequence().is_empty());

    let first = tx.add_int32_literal(block, 10).unwrap();
    let second = tx.add_int32_literal(block, 20).unwrap();
    let call = tx
        .add_call_expression(block, target, vec![first, second, first])
        .unwrap();
    let zero_call = tx
        .add_call_expression(block, zero_target, Vec::new())
        .unwrap();
    // Recursive calls are allowed; only Expression dependency cycles are not.
    let recursive = tx.add_call_expression(block, caller, vec![first]).unwrap();
    tx.set_effect_sequence(block, vec![call, zero_call, recursive])
        .unwrap();
    tx.set_return(block, call).unwrap();

    assert_eq!(tx.expression_type(call), Some(Ok(IntrinsicType::Int32)));
    assert!(matches!(
        tx.expression(call).unwrap().kind(),
        ExpressionKind::Call { target: actual, arguments }
            if *actual == target && arguments == &[first, second, first]
    ));
    assert!(matches!(
        tx.expression(zero_call).unwrap().kind(),
        ExpressionKind::Call { target: actual, arguments }
            if *actual == zero_target && arguments.is_empty()
    ));
    let snapshot = tx.commit().unwrap();
    assert_eq!(
        snapshot.block(block).unwrap().effect_sequence(),
        &[call, zero_call, recursive]
    );
    assert_eq!(
        parameter,
        snapshot.function(target).unwrap().parameters()[0].id()
    );
}

// AR-CALL-006 through AR-CALL-008 and AR-CALL-018 through AR-CALL-021.
#[test]
fn invalid_call_and_sequence_inputs_poison_atomically() {
    fn fixture() -> (
        MnirProgram,
        mnir_core::BlockId,
        mnir_core::BlockId,
        mnir_core::FunctionId,
        mnir_core::FunctionId,
        mnir_core::ExpressionId,
        mnir_core::ExpressionId,
        mnir_core::ExpressionId,
        mnir_core::ExpressionId,
    ) {
        let mut program = MnirProgram::new().unwrap();
        let mut tx = program.begin_transaction();
        let module = tx.add_module().unwrap();
        let target = tx.add_function(module, IntrinsicType::Unit).unwrap();
        let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
        let block = tx.create_function_body(caller).unwrap();
        let pure = tx.add_unit_literal(block).unwrap();
        tx.set_return(block, pure).unwrap();
        let foreign_owner = tx.add_function(module, IntrinsicType::Unit).unwrap();
        let foreign_block = tx.create_function_body(foreign_owner).unwrap();
        let foreign = tx.add_unit_literal(foreign_block).unwrap();
        let foreign_call = tx
            .add_call_expression(foreign_block, target, Vec::new())
            .unwrap();
        tx.set_effect_sequence(foreign_block, vec![foreign_call])
            .unwrap();
        tx.set_return(foreign_block, foreign).unwrap();
        let removed_target = tx.add_function(module, IntrinsicType::Unit).unwrap();
        let removed_block = tx.create_function_body(removed_target).unwrap();
        let retired = tx.add_unit_literal(removed_block).unwrap();
        tx.set_return(removed_block, retired).unwrap();
        tx.remove_function(removed_target).unwrap();
        tx.commit().unwrap();
        (
            program,
            block,
            foreign_block,
            target,
            removed_target,
            pure,
            foreign,
            foreign_call,
            retired,
        )
    }

    let (mut program, block, _, _, unknown_target, pure, _, _, _) = fixture();
    let mut tx = program.begin_transaction();
    assert_eq!(
        tx.add_call_expression(block, unknown_target, vec![pure]),
        Err(MutationError::UnknownFunction(unknown_target))
    );
    assert_eq!(tx.state(), TransactionState::Failed);
    assert_eq!(tx.block(block).unwrap().expression_count(), 1);

    let (mut program, block, _, target, _, _, _, _, retired) = fixture();
    let mut tx = program.begin_transaction();
    assert_eq!(
        tx.add_call_expression(block, target, vec![retired]),
        Err(MutationError::UnknownExpression(retired))
    );

    let (mut program, block, _, target, _, _, foreign, _, _) = fixture();
    let mut tx = program.begin_transaction();
    assert!(matches!(
        tx.add_call_expression(block, target, vec![foreign]),
        Err(MutationError::ExpressionNotOwnedByBlock { .. })
    ));

    let (mut program, block, _, target, _, _, _, _, _) = fixture();
    let mut tx = program.begin_transaction();
    let call = tx.add_call_expression(block, target, Vec::new()).unwrap();
    assert_eq!(
        tx.set_effect_sequence(block, vec![call, call]),
        Err(MutationError::DuplicateEffectSequenceEntry(call))
    );

    let (mut program, block, _, _, _, pure, _, _, _) = fixture();
    let mut tx = program.begin_transaction();
    assert_eq!(
        tx.set_effect_sequence(block, vec![pure]),
        Err(MutationError::EffectSequenceEntryNotCall(pure))
    );

    let (mut program, block, _, _, _, _, _, _, retired) = fixture();
    let mut tx = program.begin_transaction();
    assert_eq!(
        tx.set_effect_sequence(block, vec![retired]),
        Err(MutationError::UnknownExpression(retired))
    );

    let (mut program, block, _, target, _, _, _, foreign_call, _) = fixture();
    let mut tx = program.begin_transaction();
    let _call = tx.add_call_expression(block, target, Vec::new()).unwrap();
    assert!(matches!(
        tx.set_effect_sequence(block, vec![foreign_call]),
        Err(MutationError::ExpressionNotOwnedByBlock { .. })
    ));
}

// AR-CALL-012, AR-CALL-013, AR-CALL-032 through AR-CALL-034, and AR-CALL-056.
#[test]
fn target_type_is_live_and_dangling_targets_are_inspectable_and_repairable() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let caller_module = tx.add_module().unwrap();
    let target_module = tx.add_module().unwrap();
    let target = tx
        .add_function(target_module, IntrinsicType::Int32)
        .unwrap();
    let caller = tx
        .add_function(caller_module, IntrinsicType::Int32)
        .unwrap();
    let block = tx.create_function_body(caller).unwrap();
    let call = tx.add_call_expression(block, target, Vec::new()).unwrap();
    tx.set_effect_sequence(block, vec![call]).unwrap();
    tx.set_return(block, call).unwrap();
    tx.commit().unwrap();

    let mut tx = program.begin_transaction();
    tx.set_function_return_type(target, IntrinsicType::Int64)
        .unwrap();
    assert_eq!(tx.expression_type(call), Some(Ok(IntrinsicType::Int64)));
    tx.commit().unwrap();
    assert_eq!(program.expression(call).unwrap().id(), call);
    let mut module_removal_case = program.snapshot().fork().unwrap();

    let revision = program.revision_id();
    let mut tx = program.begin_transaction();
    tx.remove_function(target).unwrap();
    assert_eq!(tx.state(), TransactionState::Active);
    assert_eq!(
        tx.expression_type(call),
        Some(Err(ExpressionTypeError::UnresolvedFunction {
            expression_id: call,
            function_id: target,
        }))
    );
    assert_eq!(tx.state(), TransactionState::Active);
    assert!(matches!(
        tx.commit(),
        Err(MutationError::StructuralViolation(
            StructuralError::DanglingCallTarget { .. }
        ))
    ));
    drop(tx);
    assert_eq!(program.revision_id(), revision);

    let mut tx = program.begin_transaction();
    tx.remove_function(target).unwrap();
    tx.remove_function_body(caller).unwrap();
    tx.commit().unwrap();
    assert!(program.function(caller).unwrap().body().is_none());
    assert!(program.expression(call).is_none());

    let revision = module_removal_case.revision_id();
    let mut tx = module_removal_case.begin_transaction();
    tx.remove_module(target_module).unwrap();
    assert_eq!(tx.state(), TransactionState::Active);
    assert!(matches!(
        tx.expression_type(call),
        Some(Err(ExpressionTypeError::UnresolvedFunction { .. }))
    ));
    assert!(matches!(
        tx.commit(),
        Err(MutationError::StructuralViolation(
            StructuralError::DanglingCallTarget { .. }
        ))
    ));
    drop(tx);
    assert_eq!(module_removal_case.revision_id(), revision);

    let mut tx = module_removal_case.begin_transaction();
    tx.remove_module(target_module).unwrap();
    tx.remove_function_body(caller).unwrap();
    tx.commit().unwrap();
    assert!(
        module_removal_case
            .function(caller)
            .unwrap()
            .body()
            .is_none()
    );
}

// AR-CALL-015, AR-CALL-016, AR-CALL-022 through AR-CALL-025, and AR-CALL-055.
// The dependency conflict is intentionally detected at structural commit.
#[test]
fn explicit_sequence_replacement_and_dependency_order_are_enforced() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let int_target = tx.add_function(module, IntrinsicType::Int32).unwrap();
    let sink = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(caller).unwrap();
    let first = tx
        .add_call_expression(block, int_target, Vec::new())
        .unwrap();
    tx.set_effect_sequence(block, vec![first]).unwrap();
    let one = tx.add_int32_literal(block, 1).unwrap();
    let intermediate = tx.add_add_expression(block, first, one).unwrap();
    let second = tx
        .add_call_expression(block, sink, vec![intermediate])
        .unwrap();
    assert_eq!(tx.state(), TransactionState::Active);
    assert_eq!(tx.block(block).unwrap().effect_sequence(), &[first]);
    tx.set_effect_sequence(block, vec![second, first]).unwrap();
    assert_eq!(tx.block(block).unwrap().effect_sequence(), &[second, first]);
    let unit = tx.add_unit_literal(block).unwrap();
    tx.set_return(block, unit).unwrap();
    assert!(matches!(
        tx.commit(),
        Err(MutationError::StructuralViolation(
            StructuralError::EffectSequenceOrderConflict { .. }
        ))
    ));

    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let target = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(caller).unwrap();
    let call = tx.add_call_expression(block, target, Vec::new()).unwrap();
    let unit = tx.add_unit_literal(block).unwrap();
    tx.set_return(block, unit).unwrap();
    assert!(matches!(
        tx.commit(),
        Err(MutationError::StructuralViolation(
            StructuralError::CallMissingFromEffectSequence { expression_id, .. }
        )) if expression_id == call
    ));
}

// AR-CALL-026, AR-CALL-030, AR-CALL-031, and AR-CALL-059.
#[test]
fn effect_order_is_snapshot_and_fork_preserved_semantic_state() {
    let mut base = MnirProgram::new().unwrap();
    let mut tx = base.begin_transaction();
    let module = tx.add_module().unwrap();
    let target = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(caller).unwrap();
    let first = tx.add_call_expression(block, target, Vec::new()).unwrap();
    let second = tx.add_call_expression(block, target, Vec::new()).unwrap();
    tx.set_effect_sequence(block, vec![first, second]).unwrap();
    let unit = tx.add_unit_literal(block).unwrap();
    tx.set_return(block, unit).unwrap();
    let snapshot = tx.commit().unwrap();

    assert_eq!(
        snapshot.block(block).unwrap().effect_sequence(),
        &[first, second]
    );
    let preserved = snapshot.fork().unwrap();
    assert_eq!(
        preserved.block(block).unwrap(),
        snapshot.block(block).unwrap()
    );

    let mut opposite = snapshot.fork().unwrap();
    let mut tx = opposite.begin_transaction();
    tx.set_effect_sequence(block, vec![second, first]).unwrap();
    tx.commit().unwrap();
    assert_ne!(
        snapshot.block(block).unwrap(),
        opposite.block(block).unwrap()
    );
    assert_eq!(
        opposite.block(block).unwrap().effect_sequence(),
        &[second, first]
    );
}

// AR-CALL-029 and AR-CALL-057.
#[test]
fn effects_are_block_local_and_removal_cascades_retire_call_identity() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let target = tx.add_function(module, IntrinsicType::Bool).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let entry = tx.create_function_body(caller).unwrap();
    let successor = tx.add_block(caller).unwrap();
    let condition_call = tx.add_call_expression(entry, target, Vec::new()).unwrap();
    tx.set_effect_sequence(entry, vec![condition_call]).unwrap();
    tx.set_branch(entry, condition_call, successor, successor)
        .unwrap();
    let unit_target = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let successor_call = tx
        .add_call_expression(successor, unit_target, Vec::new())
        .unwrap();
    tx.set_effect_sequence(successor, vec![successor_call])
        .unwrap();
    let unit = tx.add_unit_literal(successor).unwrap();
    tx.set_return(successor, unit).unwrap();
    tx.commit().unwrap();
    assert_eq!(
        program.block(entry).unwrap().effect_sequence(),
        &[condition_call]
    );
    assert_eq!(
        program.block(successor).unwrap().effect_sequence(),
        &[successor_call]
    );

    let mut tx = program.begin_transaction();
    tx.set_return(entry, condition_call).unwrap();
    tx.remove_block(successor).unwrap();
    tx.commit().unwrap();
    assert!(program.block(successor).is_none());
    assert!(program.expression(successor_call).is_none());
    assert!(matches!(
        program.block(entry).unwrap().terminator(),
        Some(Terminator::Return { .. })
    ));

    let mut tx = program.begin_transaction();
    tx.remove_function_body(caller).unwrap();
    tx.commit().unwrap();
    assert!(program.expression(condition_call).is_none());
}
