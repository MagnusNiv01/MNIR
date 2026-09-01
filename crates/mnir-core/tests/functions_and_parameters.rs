use std::any::TypeId;
use std::collections::HashSet;

use mnir_core::{
    FunctionId, IntrinsicType, MnirProgram, ModuleId, MutationError, ParameterId, ProgramId,
    RevisionId, TransactionState,
};

fn add_committed_module(program: &mut MnirProgram) -> ModuleId {
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    transaction.commit().unwrap();
    module_id
}

fn add_committed_function(
    program: &mut MnirProgram,
    module_id: ModuleId,
    return_type: IntrinsicType,
    parameter_types: Vec<IntrinsicType>,
) -> (FunctionId, Vec<ParameterId>) {
    let mut transaction = program.begin_transaction();
    let function_id = transaction.add_function(module_id, return_type).unwrap();
    let parameter_ids = parameter_types
        .into_iter()
        .map(|intrinsic_type| {
            transaction
                .add_parameter(function_id, intrinsic_type)
                .unwrap()
        })
        .collect();
    transaction.commit().unwrap();
    (function_id, parameter_ids)
}

// AR-FUNC-001, AR-FUNC-002, and AR-FUNC-020.
#[test]
fn functions_are_created_in_requested_modules_with_unique_identity() {
    let mut program = MnirProgram::new().unwrap();
    let first_module = add_committed_module(&mut program);
    let second_module = add_committed_module(&mut program);
    let source_revision = program.revision_id();

    let mut transaction = program.begin_transaction();
    let first = transaction
        .add_function(first_module, IntrinsicType::Int32)
        .unwrap();
    let second = transaction
        .add_function(second_module, IntrinsicType::Unit)
        .unwrap();
    let committed = transaction.commit().unwrap();

    assert_ne!(first, second);
    assert_ne!(committed.revision_id(), source_revision);
    assert_eq!(
        committed
            .module(first_module)
            .unwrap()
            .function(first)
            .unwrap()
            .return_type(),
        &IntrinsicType::Int32
    );
    let zero_parameter = committed
        .module(second_module)
        .unwrap()
        .function(second)
        .unwrap();
    assert_eq!(zero_parameter.return_type(), &IntrinsicType::Unit);
    assert_eq!(zero_parameter.parameter_count(), 0);
}

// AR-FUNC-003 and MNIR-FUNC-007 through MNIR-FUNC-009.
#[test]
fn removed_function_identity_is_never_reused() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (removed, _) = add_committed_function(&mut program, module_id, IntrinsicType::Unit, vec![]);

    let mut transaction = program.begin_transaction();
    transaction.remove_function(removed).unwrap();
    transaction.commit().unwrap();

    let (replacement, _) =
        add_committed_function(&mut program, module_id, IntrinsicType::Unit, vec![]);
    assert_ne!(removed, replacement);
    assert!(program.is_function_id_committed(removed));
}

// AR-FUNC-004 and AR-FUNC-005.
#[test]
fn function_presentation_and_return_type_mutate_without_changing_identity() {
    let mut program = MnirProgram::new().unwrap();
    let program_id = program.program_id();
    let module_id = add_committed_module(&mut program);
    let (function_id, _) =
        add_committed_function(&mut program, module_id, IntrinsicType::Int32, vec![]);
    let source_revision = program.revision_id();

    let mut transaction = program.begin_transaction();
    transaction
        .set_function_preferred_name(function_id, Some("compute".into()))
        .unwrap();
    transaction
        .set_function_documentation(function_id, Some("A signature".into()))
        .unwrap();
    transaction
        .set_function_return_type(function_id, IntrinsicType::Bool)
        .unwrap();
    let committed = transaction.commit().unwrap();

    let function = committed
        .module(module_id)
        .unwrap()
        .function(function_id)
        .unwrap();
    assert_eq!(committed.program_id(), program_id);
    assert_ne!(committed.revision_id(), source_revision);
    assert_eq!(function.id(), function_id);
    assert_eq!(function.return_type(), &IntrinsicType::Bool);
    assert_eq!(function.presentation().preferred_name(), Some("compute"));
    assert_eq!(function.presentation().documentation(), Some("A signature"));
}

// AR-FUNC-006, AR-FUNC-007, and AR-FUNC-008.
#[test]
fn parameters_are_program_unique_and_ordered_per_function() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let mut transaction = program.begin_transaction();
    let first_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let first_a = transaction
        .add_parameter(first_function, IntrinsicType::Int32)
        .unwrap();
    transaction
        .set_parameter_preferred_name(first_a, Some("a".into()))
        .unwrap();
    let first_b = transaction
        .add_parameter(first_function, IntrinsicType::Bool)
        .unwrap();
    transaction
        .set_parameter_preferred_name(first_b, Some("b".into()))
        .unwrap();
    let second_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let second_a = transaction
        .add_parameter(second_function, IntrinsicType::Bool)
        .unwrap();
    let second_b = transaction
        .add_parameter(second_function, IntrinsicType::Int32)
        .unwrap();
    let committed = transaction.commit().unwrap();

    let first = committed.function(first_function).unwrap();
    let second = committed.function(second_function).unwrap();
    assert_eq!(first.parameters()[0].id(), first_a);
    assert_eq!(
        first.parameters()[0].presentation().preferred_name(),
        Some("a")
    );
    assert_eq!(
        first.parameters()[0].intrinsic_type(),
        &IntrinsicType::Int32
    );
    assert_eq!(first.parameters()[1].id(), first_b);
    assert_eq!(
        first.parameters()[1].presentation().preferred_name(),
        Some("b")
    );
    assert_eq!(first.parameters()[1].intrinsic_type(), &IntrinsicType::Bool);
    assert_eq!(
        second.parameters()[0].intrinsic_type(),
        &IntrinsicType::Bool
    );
    assert_eq!(
        second.parameters()[1].intrinsic_type(),
        &IntrinsicType::Int32
    );
    assert_eq!(
        HashSet::from([first_a, first_b, second_a, second_b]).len(),
        4
    );
    assert_ne!(
        first
            .parameters()
            .iter()
            .map(|parameter| parameter.intrinsic_type())
            .collect::<Vec<_>>(),
        second
            .parameters()
            .iter()
            .map(|parameter| parameter.intrinsic_type())
            .collect::<Vec<_>>()
    );
}

// AR-FUNC-009 and AR-FUNC-010.
#[test]
fn removed_parameters_and_function_descendants_are_never_reused() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (removed_function, removed_parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![IntrinsicType::Int32, IntrinsicType::Bool],
    );

    let mut transaction = program.begin_transaction();
    transaction.remove_parameter(removed_parameters[0]).unwrap();
    transaction.commit().unwrap();
    let mut transaction = program.begin_transaction();
    transaction.remove_function(removed_function).unwrap();
    transaction.commit().unwrap();

    let (_, replacements) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![IntrinsicType::Int64, IntrinsicType::Unit],
    );
    assert!(
        replacements
            .iter()
            .all(|replacement| !removed_parameters.contains(replacement))
    );
    assert!(
        removed_parameters
            .iter()
            .all(|id| program.is_parameter_id_committed(*id))
    );
}

// AR-FUNC-011 and AR-FUNC-012.
#[test]
fn parameter_metadata_and_type_mutate_without_changing_identity() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (_, parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![IntrinsicType::Int32],
    );
    let parameter_id = parameters[0];

    let mut transaction = program.begin_transaction();
    transaction
        .set_parameter_preferred_name(parameter_id, Some("value".into()))
        .unwrap();
    transaction
        .set_parameter_documentation(parameter_id, Some("input".into()))
        .unwrap();
    transaction
        .set_parameter_type(parameter_id, IntrinsicType::Int64)
        .unwrap();
    let committed = transaction.commit().unwrap();

    let parameter = committed.parameter(parameter_id).unwrap();
    assert_eq!(parameter.id(), parameter_id);
    assert_eq!(parameter.intrinsic_type(), &IntrinsicType::Int64);
    assert_eq!(parameter.presentation().preferred_name(), Some("value"));
    assert_eq!(parameter.presentation().documentation(), Some("input"));
}

// AR-FUNC-013.
#[test]
fn removing_parameter_preserves_remaining_relative_order() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (function_id, parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![
            IntrinsicType::Int32,
            IntrinsicType::Bool,
            IntrinsicType::Int64,
        ],
    );

    let mut transaction = program.begin_transaction();
    transaction.remove_parameter(parameters[1]).unwrap();
    let committed = transaction.commit().unwrap();
    let remaining: Vec<_> = committed
        .function(function_id)
        .unwrap()
        .parameters()
        .iter()
        .map(|parameter| parameter.id())
        .collect();
    assert_eq!(remaining, vec![parameters[0], parameters[2]]);
}

// AR-FUNC-014 and AR-FUNC-015.
#[test]
fn provisional_identities_support_sequential_operations() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let mut transaction = program.begin_transaction();
    let function_id = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    assert!(transaction.is_function_id_provisional(function_id));
    transaction
        .set_function_preferred_name(function_id, Some("f".into()))
        .unwrap();
    let parameter_id = transaction
        .add_parameter(function_id, IntrinsicType::Int32)
        .unwrap();
    assert!(transaction.is_parameter_id_provisional(parameter_id));
    transaction
        .set_parameter_preferred_name(parameter_id, Some("p".into()))
        .unwrap();
    transaction
        .set_parameter_type(parameter_id, IntrinsicType::Bool)
        .unwrap();
    let committed = transaction.commit().unwrap();

    assert_eq!(
        committed
            .function(function_id)
            .unwrap()
            .presentation()
            .preferred_name(),
        Some("f")
    );
    let parameter = committed.parameter(parameter_id).unwrap();
    assert_eq!(parameter.id(), parameter_id);
    assert_eq!(parameter.presentation().preferred_name(), Some("p"));
    assert_eq!(parameter.intrinsic_type(), &IntrinsicType::Bool);
}

// MNIR-FUNC-041: failed and discarded transactions publish neither entity nor
// provisional allocation history.
#[test]
fn failed_and_discarded_provisional_identities_do_not_become_committed() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let initial_function_history = program.committed_function_id_count();
    let initial_parameter_history = program.committed_parameter_id_count();

    let mut transaction = program.begin_transaction();
    let failed_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let failed_parameter = transaction
        .add_parameter(failed_function, IntrinsicType::Bool)
        .unwrap();
    transaction.remove_function(failed_function).unwrap();
    assert_eq!(
        transaction.remove_function(failed_function),
        Err(MutationError::UnknownFunction(failed_function))
    );
    transaction.discard().unwrap();
    drop(transaction);
    assert!(!program.is_function_id_committed(failed_function));
    assert!(!program.is_parameter_id_committed(failed_parameter));

    let mut transaction = program.begin_transaction();
    let discarded_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let discarded_parameter = transaction
        .add_parameter(discarded_function, IntrinsicType::Bool)
        .unwrap();
    transaction.discard().unwrap();
    drop(transaction);
    assert!(!program.is_function_id_committed(discarded_function));
    assert!(!program.is_parameter_id_committed(discarded_parameter));
    assert_eq!(
        program.committed_function_id_count(),
        initial_function_history
    );
    assert_eq!(
        program.committed_parameter_id_count(),
        initial_parameter_history
    );
}

// AR-FUNC-016.
#[test]
fn unknown_function_poisons_transaction_atomically() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (unknown, _) = add_committed_function(&mut program, module_id, IntrinsicType::Unit, vec![]);
    let mut transaction = program.begin_transaction();
    transaction.remove_function(unknown).unwrap();
    transaction.commit().unwrap();
    let source_revision = program.revision_id();

    let mut transaction = program.begin_transaction();
    let provisional = transaction
        .add_function(module_id, IntrinsicType::Bool)
        .unwrap();
    assert_eq!(
        transaction.set_function_return_type(unknown, IntrinsicType::Int64),
        Err(MutationError::UnknownFunction(unknown))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    assert!(matches!(
        transaction.commit(),
        Err(MutationError::TransactionNotActive(
            TransactionState::Failed
        ))
    ));
    drop(transaction);
    assert_eq!(program.revision_id(), source_revision);
    assert!(program.function(provisional).is_none());
}

// AR-FUNC-017.
#[test]
fn unknown_parameter_poisons_transaction_atomically() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (function_id, parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![IntrinsicType::Int32],
    );
    let unknown = parameters[0];
    let mut transaction = program.begin_transaction();
    transaction.remove_parameter(unknown).unwrap();
    transaction.commit().unwrap();
    let source_revision = program.revision_id();

    let mut transaction = program.begin_transaction();
    let provisional = transaction
        .add_parameter(function_id, IntrinsicType::Bool)
        .unwrap();
    assert_eq!(
        transaction.set_parameter_type(unknown, IntrinsicType::Int64),
        Err(MutationError::UnknownParameter(unknown))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    assert!(transaction.commit().is_err());
    drop(transaction);
    assert_eq!(program.revision_id(), source_revision);
    assert!(program.parameter(provisional).is_none());
}

// AR-FUNC-018.
#[test]
fn removed_function_and_its_parameters_are_immediately_unavailable() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (function_id, parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![IntrinsicType::Int32],
    );

    let mut transaction = program.begin_transaction();
    transaction.remove_function(function_id).unwrap();
    assert_eq!(
        transaction.set_function_return_type(function_id, IntrinsicType::Bool),
        Err(MutationError::UnknownFunction(function_id))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    transaction.discard().unwrap();

    let mut transaction = program.begin_transaction();
    transaction.remove_function(function_id).unwrap();
    assert_eq!(
        transaction.set_parameter_type(parameters[0], IntrinsicType::Bool),
        Err(MutationError::UnknownParameter(parameters[0]))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
}

// AR-FUNC-019.
#[test]
fn duplicate_presentation_names_are_allowed() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let mut transaction = program.begin_transaction();
    let first = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let second = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    transaction
        .set_function_preferred_name(first, Some("same".into()))
        .unwrap();
    transaction
        .set_function_preferred_name(second, Some("same".into()))
        .unwrap();
    let first_parameter = transaction
        .add_parameter(first, IntrinsicType::Int32)
        .unwrap();
    let second_parameter = transaction
        .add_parameter(first, IntrinsicType::Bool)
        .unwrap();
    transaction
        .set_parameter_preferred_name(first_parameter, Some("same".into()))
        .unwrap();
    transaction
        .set_parameter_preferred_name(second_parameter, Some("same".into()))
        .unwrap();
    assert!(transaction.commit().is_ok());
}

// AR-FUNC-021 and AR-FUNC-022.
#[test]
fn snapshots_and_forks_preserve_contents_and_retire_preserved_raw_ids() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (function_id, parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Bool,
        vec![IntrinsicType::Int32, IntrinsicType::Int64],
    );
    let snapshot = program.snapshot();
    assert_eq!(snapshot.function(function_id).unwrap().id(), function_id);
    assert_eq!(
        snapshot.parameter(parameters[0]).unwrap().id(),
        parameters[0]
    );

    let mut fork = snapshot.fork().unwrap();
    assert_ne!(fork.program_id(), program.program_id());
    assert_eq!(
        fork.function(function_id).unwrap().return_type(),
        &IntrinsicType::Bool
    );
    assert_eq!(
        fork.parameter(parameters[1]).unwrap().intrinsic_type(),
        &IntrinsicType::Int64
    );

    let mut transaction = fork.begin_transaction();
    transaction.remove_function(function_id).unwrap();
    let replacement = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let replacement_parameter = transaction
        .add_parameter(replacement, IntrinsicType::Unit)
        .unwrap();
    transaction.commit().unwrap();
    assert_ne!(replacement, function_id);
    assert!(!parameters.contains(&replacement_parameter));
}

// AR-FUNC-026.
#[test]
fn sibling_functions_are_identity_addressed_not_position_addressed() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let mut transaction = program.begin_transaction();
    let ids: HashSet<_> = [
        IntrinsicType::Int32,
        IntrinsicType::Bool,
        IntrinsicType::Unit,
    ]
    .into_iter()
    .map(|return_type| transaction.add_function(module_id, return_type).unwrap())
    .collect();
    let committed = transaction.commit().unwrap();
    let module = committed.module(module_id).unwrap();
    let observed: HashSet<_> = module.functions().map(|function| function.id()).collect();
    assert_eq!(observed, ids);
    assert!(ids.iter().all(|id| module.function(*id).is_some()));
}

// AR-FUNC-027.
#[test]
fn module_removal_cascades_to_committed_functions_and_parameters() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (first, first_parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![IntrinsicType::Int32],
    );
    let (second, second_parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Bool,
        vec![IntrinsicType::Bool],
    );

    let mut transaction = program.begin_transaction();
    transaction.remove_module(module_id).unwrap();
    let committed = transaction.commit().unwrap();
    assert!(committed.module(module_id).is_none());
    assert!(committed.function(first).is_none());
    assert!(committed.function(second).is_none());
    assert!(committed.parameter(first_parameters[0]).is_none());
    assert!(committed.parameter(second_parameters[0]).is_none());
    assert!(program.is_function_id_committed(first));
    assert!(program.is_function_id_committed(second));
    assert!(program.is_parameter_id_committed(first_parameters[0]));
    assert!(program.is_parameter_id_committed(second_parameters[0]));

    let replacement_module = add_committed_module(&mut program);
    let (replacement, replacement_parameters) = add_committed_function(
        &mut program,
        replacement_module,
        IntrinsicType::Unit,
        vec![IntrinsicType::Int32],
    );
    assert_ne!(replacement, first);
    assert_ne!(replacement, second);
    assert_ne!(replacement_parameters[0], first_parameters[0]);
    assert_ne!(replacement_parameters[0], second_parameters[0]);
}

// AR-FUNC-028.
#[test]
fn removed_module_descendant_references_poison_transactions() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (function_id, parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![IntrinsicType::Bool],
    );

    let mut transaction = program.begin_transaction();
    transaction.remove_module(module_id).unwrap();
    assert_eq!(
        transaction.set_function_return_type(function_id, IntrinsicType::Bool),
        Err(MutationError::UnknownFunction(function_id))
    );
    transaction.discard().unwrap();

    let mut transaction = program.begin_transaction();
    transaction.remove_module(module_id).unwrap();
    assert_eq!(
        transaction.set_parameter_type(parameters[0], IntrinsicType::Int32),
        Err(MutationError::UnknownParameter(parameters[0]))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
}

// AR-FUNC-029.
#[test]
fn module_removal_preserves_unrelated_module_signature_state() {
    let mut program = MnirProgram::new().unwrap();
    let removed_module = add_committed_module(&mut program);
    let preserved_module = add_committed_module(&mut program);
    add_committed_function(
        &mut program,
        removed_module,
        IntrinsicType::Unit,
        vec![IntrinsicType::Int32],
    );
    let (function_id, parameters) = add_committed_function(
        &mut program,
        preserved_module,
        IntrinsicType::Bool,
        vec![IntrinsicType::Int32, IntrinsicType::Int64],
    );
    let mut transaction = program.begin_transaction();
    transaction
        .set_function_preferred_name(function_id, Some("kept".into()))
        .unwrap();
    transaction
        .set_parameter_documentation(parameters[0], Some("kept parameter".into()))
        .unwrap();
    transaction.commit().unwrap();

    let mut transaction = program.begin_transaction();
    transaction.remove_module(removed_module).unwrap();
    let committed = transaction.commit().unwrap();
    let function = committed
        .module(preserved_module)
        .unwrap()
        .function(function_id)
        .unwrap();
    assert_eq!(function.id(), function_id);
    assert_eq!(function.return_type(), &IntrinsicType::Bool);
    assert_eq!(function.presentation().preferred_name(), Some("kept"));
    assert_eq!(
        function
            .parameters()
            .iter()
            .map(|parameter| parameter.id())
            .collect::<Vec<_>>(),
        parameters
    );
    assert_eq!(
        function.parameters()[0].intrinsic_type(),
        &IntrinsicType::Int32
    );
    assert_eq!(
        function.parameters()[1].intrinsic_type(),
        &IntrinsicType::Int64
    );
    assert_eq!(
        function.parameters()[0].presentation().documentation(),
        Some("kept parameter")
    );
}

// AR-FUNC-030.
#[test]
fn create_then_remove_commits_function_and_parameter_allocation_history() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let source_revision = program.revision_id();
    let mut transaction = program.begin_transaction();
    let removed_function = transaction
        .add_function(module_id, IntrinsicType::Unit)
        .unwrap();
    let removed_parameter = transaction
        .add_parameter(removed_function, IntrinsicType::Bool)
        .unwrap();
    transaction.remove_function(removed_function).unwrap();
    transaction.commit().unwrap();

    assert_ne!(program.revision_id(), source_revision);
    assert!(program.function(removed_function).is_none());
    assert!(program.parameter(removed_parameter).is_none());
    assert!(program.is_function_id_committed(removed_function));
    assert!(program.is_parameter_id_committed(removed_parameter));

    let (replacement, replacement_parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![IntrinsicType::Bool],
    );
    assert_ne!(replacement, removed_function);
    assert_ne!(replacement_parameters[0], removed_parameter);
}

// MNIR-FUNC-080: provisional descendants are removed transitively through
// Module removal, while all provisional identities enter committed history.
#[test]
fn provisional_descendants_removed_with_module_are_committed_to_history() {
    let mut program = MnirProgram::new().unwrap();
    let mut transaction = program.begin_transaction();
    let removed_module = transaction.add_module().unwrap();
    let removed_function = transaction
        .add_function(removed_module, IntrinsicType::Unit)
        .unwrap();
    let removed_parameter = transaction
        .add_parameter(removed_function, IntrinsicType::Int32)
        .unwrap();
    transaction.remove_module(removed_module).unwrap();
    transaction.commit().unwrap();

    assert!(program.module(removed_module).is_none());
    assert!(program.function(removed_function).is_none());
    assert!(program.parameter(removed_parameter).is_none());
    assert!(program.is_module_id_committed(removed_module));
    assert!(program.is_function_id_committed(removed_function));
    assert!(program.is_parameter_id_committed(removed_parameter));

    let module_id = add_committed_module(&mut program);
    let (function_id, parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![IntrinsicType::Int32],
    );
    assert_ne!(module_id, removed_module);
    assert_ne!(function_id, removed_function);
    assert_ne!(parameters[0], removed_parameter);
}

// AR-FUNC-031.
#[test]
fn adding_function_to_unknown_module_poisons_transaction_atomically() {
    let mut program = MnirProgram::new().unwrap();
    let unknown_module = add_committed_module(&mut program);
    let mut transaction = program.begin_transaction();
    transaction.remove_module(unknown_module).unwrap();
    transaction.commit().unwrap();
    let source_revision = program.revision_id();
    let source_count = program.module_count();

    let mut transaction = program.begin_transaction();
    let provisional_module = transaction.add_module().unwrap();
    assert_eq!(
        transaction.add_function(unknown_module, IntrinsicType::Unit),
        Err(MutationError::UnknownModule(unknown_module))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    assert!(transaction.commit().is_err());
    drop(transaction);
    assert_eq!(program.revision_id(), source_revision);
    assert_eq!(program.module_count(), source_count);
    assert!(program.module(provisional_module).is_none());
}

// AR-FUNC-032.
#[test]
fn removed_parameter_is_immediately_unavailable() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (_, parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Unit,
        vec![IntrinsicType::Int32],
    );
    let mut transaction = program.begin_transaction();
    transaction.remove_parameter(parameters[0]).unwrap();
    assert_eq!(
        transaction.set_parameter_documentation(parameters[0], Some("gone".into())),
        Err(MutationError::UnknownParameter(parameters[0]))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
}

// AR-FUNC-033. The FunctionId/ParameterId interchange is additionally covered
// by the compile-fail doctest on FunctionId.
#[test]
fn all_identifier_categories_have_distinct_rust_types() {
    let categories = HashSet::from([
        TypeId::of::<ProgramId>(),
        TypeId::of::<RevisionId>(),
        TypeId::of::<ModuleId>(),
        TypeId::of::<FunctionId>(),
        TypeId::of::<ParameterId>(),
    ]);
    assert_eq!(categories.len(), 5);
}

// AR-FUNC-035.
#[test]
fn removing_function_preserves_unrelated_function_signature_state() {
    let mut program = MnirProgram::new().unwrap();
    let module_id = add_committed_module(&mut program);
    let (removed, _) = add_committed_function(&mut program, module_id, IntrinsicType::Unit, vec![]);
    let (preserved, parameters) = add_committed_function(
        &mut program,
        module_id,
        IntrinsicType::Bool,
        vec![IntrinsicType::Int32, IntrinsicType::Int64],
    );
    let mut transaction = program.begin_transaction();
    transaction
        .set_function_documentation(preserved, Some("preserved".into()))
        .unwrap();
    transaction
        .set_parameter_preferred_name(parameters[0], Some("first".into()))
        .unwrap();
    transaction.commit().unwrap();

    let mut transaction = program.begin_transaction();
    transaction.remove_function(removed).unwrap();
    let committed = transaction.commit().unwrap();
    let function = committed
        .module(module_id)
        .unwrap()
        .function(preserved)
        .unwrap();
    assert_eq!(function.id(), preserved);
    assert_eq!(function.return_type(), &IntrinsicType::Bool);
    assert_eq!(function.presentation().documentation(), Some("preserved"));
    assert_eq!(function.parameters()[0].id(), parameters[0]);
    assert_eq!(function.parameters()[1].id(), parameters[1]);
    assert_eq!(
        function.parameters()[0].intrinsic_type(),
        &IntrinsicType::Int32
    );
    assert_eq!(
        function.parameters()[1].intrinsic_type(),
        &IntrinsicType::Int64
    );
    assert_eq!(
        function.parameters()[0].presentation().preferred_name(),
        Some("first")
    );
}
