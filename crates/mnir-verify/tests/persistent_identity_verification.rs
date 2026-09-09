use mnir_core::{IntrinsicType, MnirProgram};
use mnir_verify::{VerificationRuleSet, verify_with_rule_set};

// AR-PSI-033/-034; MNIR-PSI-078 through MNIR-PSI-081.
#[test]
fn inherited_identity_does_not_make_verified_evidence_interchangeable() {
    let mut source = MnirProgram::new().unwrap();
    let mut transaction = source.begin_transaction();
    let module = transaction.add_module().unwrap();
    let function = transaction
        .add_function(module, IntrinsicType::Unit)
        .unwrap();
    let source_snapshot = transaction.commit().unwrap();
    drop(transaction);
    let mut fork = source_snapshot.fork().unwrap();
    let mut fork_transaction = fork.begin_transaction();
    fork_transaction.commit().unwrap();
    drop(fork_transaction);
    let fork_snapshot = fork.snapshot();

    assert_eq!(source_snapshot.function(function).unwrap().id(), function);
    assert_eq!(fork_snapshot.function(function).unwrap().id(), function);
    assert_ne!(source_snapshot.program_id(), fork_snapshot.program_id());
    assert_eq!(source_snapshot.revision_id(), fork_snapshot.revision_id());

    for rule_set in [
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4,
    ] {
        let source_evidence = verify_with_rule_set(&source_snapshot, rule_set).unwrap();
        let fork_evidence = verify_with_rule_set(&fork_snapshot, rule_set).unwrap();

        assert_eq!(source_evidence.program_id(), source_snapshot.program_id());
        assert_eq!(source_evidence.revision_id(), source_snapshot.revision_id());
        assert_eq!(source_evidence.rule_set(), rule_set);
        assert_eq!(fork_evidence.program_id(), fork_snapshot.program_id());
        assert_eq!(fork_evidence.revision_id(), fork_snapshot.revision_id());
        assert_eq!(fork_evidence.rule_set(), rule_set);
        assert_ne!(source_evidence.program_id(), fork_evidence.program_id());
    }
}
