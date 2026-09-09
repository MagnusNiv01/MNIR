use std::any::TypeId;
use std::collections::HashSet;

use mnir_core::{
    AllocationCounterState, AllocationNamespaceId, BlockId, ExpressionId, ExpressionKind,
    FunctionId, IntrinsicType, MnirProgram, ModuleId, ParameterId, ProgramId, RevisionId,
    Terminator,
};

#[derive(Clone, Copy)]
struct CompleteIds {
    module: ModuleId,
    callee: FunctionId,
    caller: FunctionId,
    parameter: ParameterId,
    entry: BlockId,
    true_block: BlockId,
    false_block: BlockId,
    parameter_reference: ExpressionId,
    literal: ExpressionId,
    arithmetic: ExpressionId,
    comparison: ExpressionId,
    call: ExpressionId,
    true_value: ExpressionId,
    false_value: ExpressionId,
}

fn complete_program() -> (MnirProgram, CompleteIds) {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let callee = transaction
        .add_function(module, IntrinsicType::Int32)
        .unwrap();
    transaction
        .add_parameter(callee, IntrinsicType::Int32)
        .unwrap();
    let caller = transaction
        .add_function(module, IntrinsicType::Int32)
        .unwrap();
    let parameter = transaction
        .add_parameter(caller, IntrinsicType::Int32)
        .unwrap();
    let entry = transaction.create_function_body(caller).unwrap();
    let true_block = transaction.add_block(caller).unwrap();
    let false_block = transaction.add_block(caller).unwrap();
    let parameter_reference = transaction
        .add_parameter_reference(entry, parameter)
        .unwrap();
    let literal = transaction.add_int32_literal(entry, 1).unwrap();
    let arithmetic = transaction
        .add_add_expression(entry, parameter_reference, literal)
        .unwrap();
    let comparison = transaction
        .add_less_than_expression(entry, arithmetic, literal)
        .unwrap();
    let call = transaction
        .add_call_expression(entry, callee, vec![arithmetic])
        .unwrap();
    transaction.set_effect_sequence(entry, vec![call]).unwrap();
    let true_value = transaction.add_int32_literal(true_block, 2).unwrap();
    let false_value = transaction.add_int32_literal(false_block, 3).unwrap();
    transaction
        .set_branch(entry, comparison, true_block, false_block)
        .unwrap();
    transaction.set_return(true_block, true_value).unwrap();
    transaction.set_return(false_block, false_value).unwrap();
    transaction.commit().unwrap();
    drop(transaction);

    (
        program,
        CompleteIds {
            module,
            callee,
            caller,
            parameter,
            entry,
            true_block,
            false_block,
            parameter_reference,
            literal,
            arithmetic,
            comparison,
            call,
            true_value,
            false_value,
        },
    )
}

// AR-PSI-001/-002/-004; MNIR-PSI-003 through MNIR-PSI-006.
// The compile-fail doctests on the public ID types additionally demonstrate
// non-interchangeability at API boundaries.
#[test]
fn persistent_id_categories_are_typed_and_component_inspectable() {
    let categories = HashSet::from([
        TypeId::of::<AllocationNamespaceId>(),
        TypeId::of::<ProgramId>(),
        TypeId::of::<RevisionId>(),
        TypeId::of::<ModuleId>(),
        TypeId::of::<FunctionId>(),
        TypeId::of::<ParameterId>(),
        TypeId::of::<BlockId>(),
        TypeId::of::<ExpressionId>(),
    ]);
    assert_eq!(categories.len(), 8);

    let (program, ids) = complete_program();
    let namespace = program.allocation_namespace_id();
    for actual in [
        ids.module.namespace_id(),
        ids.callee.namespace_id(),
        ids.parameter.namespace_id(),
        ids.entry.namespace_id(),
        ids.literal.namespace_id(),
    ] {
        assert_eq!(actual, namespace);
    }
    assert!(ids.module.counter() < ids.callee.counter());
    assert!(ids.callee.counter() < ids.parameter.counter());
    assert!(ids.parameter.counter() < ids.entry.counter());
    assert!(ids.entry.counter() < ids.literal.counter());
}

// AR-PSI-006/-007; MNIR-PSI-012/-013/-019/-030/-031.
#[test]
fn independent_programs_and_shared_category_allocation_are_distinct() {
    let first = MnirProgram::new().unwrap();
    let second = MnirProgram::new().unwrap();
    assert_ne!(first.program_id(), second.program_id());
    assert_ne!(
        first.allocation_namespace_id(),
        second.allocation_namespace_id()
    );

    let (program, ids) = complete_program();
    let counters = vec![
        ids.module.counter(),
        ids.callee.counter(),
        ids.caller.counter(),
        ids.parameter.counter(),
        ids.entry.counter(),
        ids.literal.counter(),
    ];
    let mut sorted = counters.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), counters.len());
    assert!(matches!(
        program.allocation_counter_state(),
        AllocationCounterState::Available(next) if next > *sorted.last().unwrap()
    ));
}

// AR-PSI-003/-005/-008/-010/-011/-012/-030; MNIR-PSI-007/-009 and
// MNIR-PSI-055/-061 through MNIR-PSI-065/-070 through MNIR-PSI-073.
#[test]
fn snapshot_and_fork_preserve_all_ids_and_reference_families_exactly() {
    let (program, ids) = complete_program();
    let allocation_state = program.allocation_counter_state();
    let snapshot = program.snapshot();
    assert_eq!(snapshot.allocation_counter_state(), allocation_state);
    assert_eq!(program.allocation_counter_state(), allocation_state);
    let fork = snapshot.fork().unwrap();

    assert_ne!(snapshot.program_id(), fork.program_id());
    assert_eq!(fork.module(ids.module).unwrap().id(), ids.module);
    assert_eq!(fork.function(ids.callee).unwrap().id(), ids.callee);
    assert_eq!(fork.function(ids.caller).unwrap().id(), ids.caller);
    assert_eq!(fork.parameter(ids.parameter).unwrap().id(), ids.parameter);
    for block in [ids.entry, ids.true_block, ids.false_block] {
        assert_eq!(fork.block(block).unwrap().id(), block);
    }
    for expression in [
        ids.parameter_reference,
        ids.literal,
        ids.arithmetic,
        ids.comparison,
        ids.call,
        ids.true_value,
        ids.false_value,
    ] {
        assert_eq!(fork.expression(expression).unwrap().id(), expression);
        assert_eq!(
            fork.expression(expression).unwrap().kind(),
            snapshot.expression(expression).unwrap().kind()
        );
    }
    assert_eq!(
        fork.block(ids.entry).unwrap().terminator(),
        Some(&Terminator::Branch {
            condition: ids.comparison,
            true_block: ids.true_block,
            false_block: ids.false_block,
        })
    );
    assert_eq!(
        fork.block(ids.entry).unwrap().effect_sequence(),
        &[ids.call]
    );
    assert_eq!(
        fork.expression(ids.parameter_reference).unwrap().kind(),
        &ExpressionKind::ParameterReference(ids.parameter)
    );
    assert_eq!(
        fork.expression(ids.arithmetic).unwrap().kind(),
        &ExpressionKind::Add {
            left: ids.parameter_reference,
            right: ids.literal,
        }
    );
    assert_eq!(
        fork.expression(ids.call).unwrap().kind(),
        &ExpressionKind::Call {
            target: ids.callee,
            arguments: vec![ids.arithmetic],
        }
    );
}

// AR-PSI-013 through AR-PSI-016 and AR-PSI-031; MNIR-PSI-066 through
// MNIR-PSI-069. Safe allocation exposes no namespace-selection input.
#[test]
fn forks_have_independent_fresh_authorities_while_source_is_unchanged() {
    let (mut source, ids) = complete_program();
    let snapshot = source.snapshot();
    let source_program_id = source.program_id();
    let source_namespace = source.allocation_namespace_id();
    let source_counter_before_fork = source.allocation_counter_state();
    let AllocationCounterState::Available(source_next_counter) = source_counter_before_fork else {
        panic!("the test lineage should have available allocation capacity");
    };

    let mut first_fork = snapshot.fork().unwrap();
    let mut second_fork = snapshot.fork().unwrap();
    let first_namespace = first_fork.allocation_namespace_id();
    let first_counter_before_source_allocation = first_fork.allocation_counter_state();
    let AllocationCounterState::Available(first_next_counter) =
        first_counter_before_source_allocation
    else {
        panic!("the first fork should have available allocation capacity");
    };
    let second_namespace = second_fork.allocation_namespace_id();
    let second_counter_before_allocations = second_fork.allocation_counter_state();
    let AllocationCounterState::Available(second_next_counter) = second_counter_before_allocations
    else {
        panic!("the second fork should have available allocation capacity");
    };

    assert_eq!(source.program_id(), source_program_id);
    assert_eq!(source.allocation_namespace_id(), source_namespace);
    assert_eq!(
        source.allocation_counter_state(),
        source_counter_before_fork
    );
    assert_ne!(first_fork.program_id(), source_program_id);
    assert_ne!(second_fork.program_id(), source_program_id);
    assert_ne!(first_namespace, second_namespace);
    assert_ne!(first_namespace, source_namespace);
    assert_ne!(second_namespace, source_namespace);
    assert_eq!(first_fork.function(ids.caller).unwrap().id(), ids.caller);
    assert_eq!(second_fork.function(ids.caller).unwrap().id(), ids.caller);

    let mut source_transaction = source.begin_transaction();
    let source_new = source_transaction.add_module().unwrap();
    source_transaction.commit().unwrap();
    drop(source_transaction);
    let source_counter_after_source_allocation = source.allocation_counter_state();
    assert_eq!(source.allocation_namespace_id(), source_namespace);
    assert_eq!(source_new.namespace_id(), source_namespace);
    assert_eq!(source_new.counter(), source_next_counter);
    assert_eq!(
        source_counter_after_source_allocation,
        AllocationCounterState::Available(source_next_counter + 1)
    );
    assert_eq!(first_fork.allocation_namespace_id(), first_namespace);
    assert_eq!(
        first_fork.allocation_counter_state(),
        first_counter_before_source_allocation
    );
    assert_eq!(second_fork.allocation_namespace_id(), second_namespace);
    assert_eq!(
        second_fork.allocation_counter_state(),
        second_counter_before_allocations
    );

    let mut first_transaction = first_fork.begin_transaction();
    let first_new = first_transaction.add_module().unwrap();
    first_transaction.commit().unwrap();
    drop(first_transaction);
    assert_eq!(first_fork.allocation_namespace_id(), first_namespace);
    assert_eq!(first_new.namespace_id(), first_namespace);
    assert_eq!(first_new.counter(), first_next_counter);
    assert_eq!(
        first_fork.allocation_counter_state(),
        AllocationCounterState::Available(first_next_counter + 1)
    );
    assert_eq!(source.allocation_namespace_id(), source_namespace);
    assert_eq!(
        source.allocation_counter_state(),
        source_counter_after_source_allocation
    );
    assert_eq!(second_fork.allocation_namespace_id(), second_namespace);
    assert_eq!(
        second_fork.allocation_counter_state(),
        second_counter_before_allocations
    );

    let mut second_transaction = second_fork.begin_transaction();
    let second_new = second_transaction.add_module().unwrap();
    second_transaction.commit().unwrap();
    drop(second_transaction);

    assert_eq!(second_fork.allocation_namespace_id(), second_namespace);
    assert_eq!(second_new.namespace_id(), second_namespace);
    assert_eq!(second_new.counter(), second_next_counter);
    assert_eq!(
        second_fork.allocation_counter_state(),
        AllocationCounterState::Available(second_next_counter + 1)
    );
    assert_eq!(
        source.allocation_counter_state(),
        source_counter_after_source_allocation
    );
    assert_eq!(
        first_fork.allocation_counter_state(),
        AllocationCounterState::Available(first_next_counter + 1)
    );
    assert_ne!(first_new.namespace_id(), ids.module.namespace_id());
    assert_ne!(first_new, second_new);
}

// AR-PSI-017/-024/-025/-027; MNIR-PSI-044/-049/-050/-052/-053.
#[test]
fn removed_ids_are_not_reused_and_create_remove_is_a_semantic_no_op() {
    let (mut program, old) = complete_program();
    let source_revision = program.revision_id();
    let source_module_count = program.module_count();
    let mut transaction = program.begin_transaction();
    let temporary = transaction.add_module().unwrap();
    transaction.remove_module(temporary).unwrap();
    assert!(transaction.is_semantic_no_op());
    transaction.commit().unwrap();
    drop(transaction);
    assert_ne!(program.revision_id(), source_revision);
    assert_eq!(program.module_count(), source_module_count);

    let mut removal = program.begin_transaction();
    removal.remove_module(old.module).unwrap();
    removal.commit().unwrap();
    drop(removal);

    let mut replacement = program.begin_transaction();
    let module = replacement.add_module().unwrap();
    let function = replacement
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let parameter = replacement
        .add_parameter(function, IntrinsicType::Unit)
        .unwrap();
    let block = replacement.create_function_body(function).unwrap();
    let expression = replacement.add_unit_literal(block).unwrap();
    replacement.set_return(block, expression).unwrap();
    replacement.commit().unwrap();

    assert_ne!(module, old.module);
    assert_ne!(function, old.caller);
    assert_ne!(parameter, old.parameter);
    assert_ne!(block, old.entry);
    assert_ne!(expression, old.literal);
    assert_eq!(module.namespace_id(), old.module.namespace_id());
    assert_eq!(function.namespace_id(), old.caller.namespace_id());
    assert_eq!(parameter.namespace_id(), old.parameter.namespace_id());
    assert_eq!(block.namespace_id(), old.entry.namespace_id());
    assert_eq!(expression.namespace_id(), old.literal.namespace_id());
    assert!(module.counter() > old.module.counter());
    assert!(function.counter() > old.caller.counter());
    assert!(parameter.counter() > old.parameter.counter());
    assert!(block.counter() > old.entry.counter());
    assert!(expression.counter() > old.literal.counter());
    assert!(module.counter() > temporary.counter());
}

// AR-PSI-018/-019; MNIR-PSI-010/-038/-039.
#[test]
fn updates_preserve_ids_and_provisional_ids_address_working_entities() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Int32)
        .unwrap();
    let parameter = transaction
        .add_parameter(function, IntrinsicType::Int32)
        .unwrap();
    assert!(transaction.is_module_id_provisional(module));
    assert!(transaction.is_function_id_provisional(function));
    assert!(transaction.is_parameter_id_provisional(parameter));
    assert_eq!(transaction.module(module).unwrap().id(), module);
    transaction.commit().unwrap();
    drop(transaction);

    let mut update = program.begin_transaction();
    update
        .set_module_preferred_name(module, Some("renamed".to_owned()))
        .unwrap();
    update
        .set_function_return_type(function, IntrinsicType::Int64)
        .unwrap();
    update
        .set_parameter_type(parameter, IntrinsicType::Bool)
        .unwrap();
    update.commit().unwrap();

    assert_eq!(program.module(module).unwrap().id(), module);
    assert_eq!(program.function(function).unwrap().id(), function);
    assert_eq!(program.parameter(parameter).unwrap().id(), parameter);
}

// AR-PSI-028/-029/-036/-038: persistence/restoration, import, package,
// trust, signature, diff, merge, and remapping APIs remain outside this
// non-persistent increment. The manifest/source inspection also guards crate
// boundaries, absence of unsafe Rust, and absence of speculative frameworks.
#[test]
fn implementation_scope_and_dependencies_remain_bounded() {
    const MANIFEST: &str = include_str!("../Cargo.toml");
    const IDS: &str = include_str!("../src/ids.rs");
    const MODEL: &str = include_str!("../src/program/model.rs");
    const TRANSACTION: &str = include_str!("../src/program/transaction.rs");

    assert!(!MANIFEST.contains("mnir-verify"));
    assert!(!MANIFEST.contains("easyh-render"));
    assert!(!IDS.contains("unsafe "));
    assert!(!MODEL.contains("unsafe "));
    assert!(!TRANSACTION.contains("unsafe "));
    assert!(!MODEL.contains("pub fn restore"));
    assert!(!MODEL.contains("pub fn import"));
    assert!(!MODEL.contains("pub fn merge"));
    assert!(!MODEL.contains("pub fn remap"));
}
