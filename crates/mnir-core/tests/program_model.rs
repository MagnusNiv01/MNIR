use std::any::TypeId;

use mnir_core::{MnirProgram, ModuleId, MutationError, ProgramId, RevisionId, TransactionState};

fn new_program() -> MnirProgram {
    MnirProgram::new().expect("the process-local ProgramId allocator should have capacity")
}

fn add_and_commit(program: &mut MnirProgram) -> ModuleId {
    let mut transaction = program.begin_transaction();
    let id = transaction
        .add_module()
        .expect("adding a module to an active transaction should succeed");
    transaction
        .commit()
        .expect("the structurally valid transaction should commit");
    id
}

// AR-CORE-001; MNIR-CORE-001, MNIR-CORE-002, MNIR-CORE-040.
#[test]
fn ar_core_001_empty_program_is_structurally_valid() {
    let program = new_program();

    assert_eq!(program.module_count(), 0);
    assert_eq!(program.committed_module_id_count(), 0);
    assert!(program.validate_structure().is_ok());
}

// AR-CORE-002; MNIR-CORE-030, MNIR-CORE-034, MNIR-CORE-038.
#[test]
fn ar_core_002_add_module_commits_a_new_revision() {
    let mut program = new_program();
    let program_id = program.program_id();
    let source_revision = program.revision_id();

    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    let committed = transaction.commit().unwrap();

    assert_eq!(transaction.state(), TransactionState::Committed);
    assert_eq!(committed.program_id(), program_id);
    assert_ne!(committed.revision_id(), source_revision);
    assert_eq!(committed.module(module_id).unwrap().id(), module_id);
    assert!(committed.validate_structure().is_ok());
}

// AR-CORE-003; MNIR-CORE-005. The compile-fail doctest on ModuleId also
// demonstrates that these categories cannot be interchanged.
#[test]
fn ar_core_003_identifier_categories_are_distinct_rust_types() {
    assert_ne!(TypeId::of::<ProgramId>(), TypeId::of::<RevisionId>());
    assert_ne!(TypeId::of::<ProgramId>(), TypeId::of::<ModuleId>());
    assert_ne!(TypeId::of::<RevisionId>(), TypeId::of::<ModuleId>());
}

// AR-CORE-004; MNIR-CORE-011, MNIR-CORE-064.
#[test]
fn ar_core_004_module_ids_are_unique_within_a_program_lineage() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();

    let first = transaction.add_module().unwrap();
    let second = transaction.add_module().unwrap();

    assert_ne!(first, second);
    let committed = transaction.commit().unwrap();
    assert_eq!(committed.module_count(), 2);
}

// AR-CORE-005; MNIR-CORE-013, MNIR-CORE-015, MNIR-CORE-055.
#[test]
fn ar_core_005_removed_module_identity_is_not_reused() {
    let mut program = new_program();
    let first = add_and_commit(&mut program);

    let mut transaction = program.begin_transaction();
    transaction.remove_module(first).unwrap();
    let replacement = transaction.add_module().unwrap();
    transaction.commit().unwrap();

    assert_ne!(first, replacement);
    assert!(program.is_module_id_committed(first));
    assert!(program.is_module_id_committed(replacement));
}

// AR-CORE-006; MNIR-CORE-014, MNIR-CORE-022 through MNIR-CORE-024.
#[test]
fn ar_core_006_presentation_updates_preserve_semantic_identity() {
    let mut program = new_program();
    let module_id = add_and_commit(&mut program);
    let program_id = program.program_id();
    let source_revision = program.revision_id();

    let mut transaction = program.begin_transaction();
    transaction
        .set_module_preferred_name(module_id, Some("orders".to_owned()))
        .unwrap();
    transaction
        .set_module_documentation(module_id, Some("Order processing".to_owned()))
        .unwrap();
    let committed = transaction.commit().unwrap();

    let module = committed.module(module_id).unwrap();
    assert_eq!(committed.program_id(), program_id);
    assert_eq!(module.id(), module_id);
    assert_ne!(committed.revision_id(), source_revision);
    assert_eq!(module.presentation().preferred_name(), Some("orders"));
    assert_eq!(
        module.presentation().documentation(),
        Some("Order processing")
    );
}

// AR-CORE-007; MNIR-CORE-035, MNIR-CORE-061, MNIR-CORE-063.
#[test]
fn ar_core_007_failed_mutation_is_atomic_and_poisoned() {
    let mut program = new_program();
    let retired_id = add_and_commit(&mut program);

    let mut removal = program.begin_transaction();
    removal.remove_module(retired_id).unwrap();
    removal.commit().unwrap();

    let source = program.snapshot();
    let mut transaction = program.begin_transaction();
    let provisional_id = transaction.add_module().unwrap();
    assert!(transaction.module(provisional_id).is_some());

    assert_eq!(
        transaction.remove_module(retired_id),
        Err(MutationError::UnknownModule(retired_id))
    );
    assert_eq!(transaction.state(), TransactionState::Failed);
    assert!(matches!(
        transaction.commit(),
        Err(MutationError::TransactionNotActive(
            TransactionState::Failed
        ))
    ));
    transaction.discard().unwrap();
    drop(transaction);

    assert_eq!(program.revision_id(), source.revision_id());
    assert_eq!(program.module_count(), source.module_count());
    assert!(program.module(provisional_id).is_none());
}

// AR-CORE-008; MNIR-CORE-034, MNIR-CORE-038.
#[test]
fn ar_core_008_remove_module_commits_a_valid_revision() {
    let mut program = new_program();
    let module_id = add_and_commit(&mut program);
    let source_revision = program.revision_id();

    let mut transaction = program.begin_transaction();
    transaction.remove_module(module_id).unwrap();
    let committed = transaction.commit().unwrap();

    assert_ne!(committed.revision_id(), source_revision);
    assert!(committed.module(module_id).is_none());
    assert!(committed.validate_structure().is_ok());
}

// AR-CORE-009; MNIR-CORE-030, MNIR-CORE-039.
#[test]
fn ar_core_009_empty_no_op_commit_changes_only_revision() {
    let mut program = new_program();
    let mut setup = program.begin_transaction();
    let module_id = setup.add_module().unwrap();
    setup
        .set_module_preferred_name(module_id, Some("stable name".to_owned()))
        .unwrap();
    setup
        .set_module_documentation(module_id, Some("stable documentation".to_owned()))
        .unwrap();
    setup.commit().unwrap();
    drop(setup);

    let program_id = program.program_id();
    let source_revision = program.revision_id();
    let module_count = program.module_count();
    let history_count = program.committed_module_id_count();

    let mut transaction = program.begin_transaction();
    let committed = transaction.commit().unwrap();

    assert_eq!(committed.program_id(), program_id);
    assert_ne!(committed.revision_id(), source_revision);
    assert_eq!(committed.module_count(), module_count);
    let module = committed.module(module_id).unwrap();
    assert_eq!(module.presentation().preferred_name(), Some("stable name"));
    assert_eq!(
        module.presentation().documentation(),
        Some("stable documentation")
    );
    assert_eq!(program.committed_module_id_count(), history_count);
}

// AR-CORE-010; MNIR-CORE-003, MNIR-CORE-043, MNIR-CORE-044.
#[test]
fn ar_core_010_program_model_uses_only_semantic_rust_apis() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    transaction
        .set_module_preferred_name(module_id, Some("human label".to_owned()))
        .unwrap();
    let committed = transaction.commit().unwrap();

    assert!(committed.module(module_id).is_some());
}

// AR-CORE-011; MNIR-CORE-050, MNIR-CORE-052.
#[test]
fn ar_core_011_mnir_core_manifest_has_no_easyh_dependency() {
    const MANIFEST: &str = include_str!("../Cargo.toml");

    assert!(!MANIFEST.contains("easyh-render"));
    assert!(!MANIFEST.contains("mnir-verify"));
    assert!(!MANIFEST.contains("mnir-cli"));
}

// AR-CORE-012; MNIR-CORE-061 through MNIR-CORE-063.
#[test]
fn ar_core_012_operations_observe_sequential_working_state() {
    let mut program = new_program();
    let mut creation = program.begin_transaction();
    let module_id = creation.add_module().unwrap();
    creation
        .set_module_preferred_name(module_id, Some("new module".to_owned()))
        .unwrap();
    let committed = creation.commit().unwrap();
    assert_eq!(
        committed
            .module(module_id)
            .unwrap()
            .presentation()
            .preferred_name(),
        Some("new module")
    );
    drop(creation);

    let source_revision = program.revision_id();
    let mut removal = program.begin_transaction();
    removal.remove_module(module_id).unwrap();
    assert_eq!(
        removal.set_module_preferred_name(module_id, Some("invalid".to_owned())),
        Err(MutationError::UnknownModule(module_id))
    );
    assert_eq!(removal.state(), TransactionState::Failed);
    removal.discard().unwrap();
    drop(removal);

    assert_eq!(program.revision_id(), source_revision);
    assert!(program.module(module_id).is_some());
}

// AR-CORE-013; MNIR-CORE-054, MNIR-CORE-055.
#[test]
fn ar_core_013_committed_module_id_remains_retired_across_revisions() {
    let mut program = new_program();
    let retired_id = add_and_commit(&mut program);

    let mut removal = program.begin_transaction();
    removal.remove_module(retired_id).unwrap();
    removal.commit().unwrap();
    drop(removal);

    let replacement_id = add_and_commit(&mut program);

    assert_ne!(replacement_id, retired_id);
    assert!(program.is_module_id_committed(retired_id));
    assert_eq!(program.committed_module_id_count(), 2);
}

// AR-CORE-014; MNIR-CORE-064 through MNIR-CORE-066.
#[test]
fn ar_core_014_provisional_identity_becomes_committed_only_on_commit() {
    let mut program = new_program();
    assert_eq!(program.committed_module_id_count(), 0);

    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    assert!(transaction.is_module_id_provisional(module_id));
    assert!(transaction.module(module_id).is_some());

    let committed = transaction.commit().unwrap();
    assert!(!transaction.is_module_id_provisional(module_id));
    assert!(committed.module(module_id).is_some());
    drop(transaction);

    assert!(program.is_module_id_committed(module_id));
}

// AR-CORE-015; MNIR-CORE-009, MNIR-CORE-010, MNIR-CORE-058,
// MNIR-CORE-059. The compile-fail doctest on ProgramSnapshot demonstrates
// that snapshots expose no same-lineage mutation entry point.
#[test]
fn ar_core_015_historical_snapshot_requires_a_new_lineage_to_continue() {
    let mut program = new_program();
    let historical = program.snapshot();
    let source_program_id = historical.program_id();

    add_and_commit(&mut program);
    assert_ne!(historical.revision_id(), program.revision_id());
    assert_eq!(historical.module_count(), 0);

    let fork = historical.fork().unwrap();
    assert_ne!(fork.program_id(), source_program_id);
    assert_eq!(fork.module_count(), historical.module_count());
    assert!(fork.validate_structure().is_ok());
}

// MNIR-CORE-009, MNIR-CORE-016, MNIR-CORE-031.
#[test]
fn snapshots_preserve_revision_contents_after_later_commits() {
    let mut program = new_program();
    let first_snapshot = program.snapshot();
    let first_program_id = first_snapshot.program_id();
    let first_revision_id = first_snapshot.revision_id();

    let module_id = add_and_commit(&mut program);

    assert_eq!(first_snapshot.program_id(), first_program_id);
    assert_eq!(first_snapshot.revision_id(), first_revision_id);
    assert_eq!(first_snapshot.module_count(), 0);
    assert!(first_snapshot.module(module_id).is_none());
    assert!(program.module(module_id).is_some());
}

// MNIR-CORE-017. Preserving raw ModuleId values on fork is this
// implementation's allowed choice.
#[test]
fn fork_preserves_module_ids_but_changes_program_identity() {
    let mut source = new_program();
    let module_id = add_and_commit(&mut source);
    let snapshot = source.snapshot();

    let fork = snapshot.fork().unwrap();

    assert_ne!(fork.program_id(), source.program_id());
    assert_eq!(fork.module(module_id).unwrap().id(), module_id);
    assert!(fork.is_module_id_committed(module_id));
}

// MNIR-CORE-060.
#[test]
fn committed_and_discarded_transactions_are_terminal() {
    let mut committed_program = new_program();
    let mut committed = committed_program.begin_transaction();
    committed.commit().unwrap();
    assert_eq!(committed.state(), TransactionState::Committed);
    assert_eq!(
        committed.add_module(),
        Err(MutationError::TransactionNotActive(
            TransactionState::Committed
        ))
    );

    let mut discarded_program = new_program();
    let mut discarded = discarded_program.begin_transaction();
    discarded.discard().unwrap();
    assert_eq!(discarded.state(), TransactionState::Discarded);
    assert!(matches!(
        discarded.commit(),
        Err(MutationError::TransactionNotActive(
            TransactionState::Discarded
        ))
    ));
}

// MNIR-CORE-065. A created-then-removed Module does not remain in the
// committed collection, but its successfully committed identity is retired.
#[test]
fn created_then_removed_module_id_enters_committed_history() {
    let mut program = new_program();
    let mut transaction = program.begin_transaction();
    let module_id = transaction.add_module().unwrap();
    transaction.remove_module(module_id).unwrap();
    transaction.commit().unwrap();
    drop(transaction);

    assert_eq!(program.module_count(), 0);
    assert!(program.is_module_id_committed(module_id));
    assert_eq!(program.committed_module_id_count(), 1);
}
