use std::collections::{HashMap, HashSet};

use crate::ids::{FunctionId, ModuleId, ParameterId};

use super::error::StructuralError;
use super::model::{Module, RevisionState};

pub(super) fn validate_revision_state(state: &RevisionState) -> Result<(), StructuralError> {
    validate_modules(
        &state.modules,
        &state.committed_module_ids,
        &state.committed_function_ids,
        &state.committed_parameter_ids,
    )
}

pub(super) fn validate_modules(
    modules: &HashMap<ModuleId, Module>,
    committed_module_ids: &HashSet<ModuleId>,
    committed_function_ids: &HashSet<FunctionId>,
    committed_parameter_ids: &HashSet<ParameterId>,
) -> Result<(), StructuralError> {
    let mut seen_function_ids = HashSet::new();
    let mut seen_parameter_ids = HashSet::new();

    for (&collection_id, module) in modules {
        if collection_id != module.id {
            return Err(StructuralError::ModuleIdentityMismatch {
                collection_id,
                module_id: module.id,
            });
        }

        if !committed_module_ids.contains(&collection_id) {
            return Err(StructuralError::ModuleIdentityNotCommitted(collection_id));
        }

        for (&function_collection_id, function) in &module.functions {
            if function_collection_id != function.id {
                return Err(StructuralError::FunctionIdentityMismatch {
                    collection_id: function_collection_id,
                    function_id: function.id,
                });
            }
            if !committed_function_ids.contains(&function.id) {
                return Err(StructuralError::FunctionIdentityNotCommitted(function.id));
            }
            if !seen_function_ids.insert(function.id) {
                return Err(StructuralError::DuplicateFunctionIdentity(function.id));
            }

            for parameter in &function.parameters {
                if !committed_parameter_ids.contains(&parameter.id) {
                    return Err(StructuralError::ParameterIdentityNotCommitted(parameter.id));
                }
                if !seen_parameter_ids.insert(parameter.id) {
                    return Err(StructuralError::DuplicateParameterIdentity(parameter.id));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::IntrinsicType;
    use crate::ids::{FunctionId, ModuleId};

    use super::super::error::{MutationError, StructuralError, TransactionState};
    use super::super::model::MnirProgram;

    // MNIR-CORE-035 through MNIR-CORE-037 and MNIR-CORE-042.
    #[test]
    fn structurally_invalid_internal_candidate_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let source_program_id = program.program_id();
        let source_revision_id = program.revision_id();
        let source_module_count = program.module_count();

        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        let module = transaction
            .working_modules_mut()
            .remove(&module_id)
            .unwrap();
        let mismatched_collection_id = ModuleId(module_id.0 + 1);
        transaction
            .working_modules_mut()
            .insert(mismatched_collection_id, module);

        assert!(matches!(
            transaction.commit(),
            Err(MutationError::StructuralViolation(
                StructuralError::ModuleIdentityMismatch { .. }
            ))
        ));
        assert_eq!(transaction.state(), TransactionState::Failed);
        drop(transaction);

        assert_eq!(program.program_id(), source_program_id);
        assert_eq!(program.revision_id(), source_revision_id);
        assert_eq!(program.module_count(), source_module_count);
        assert!(program.validate_structure().is_ok());
    }

    // AR-FUNC-034 and MNIR-FUNC-041 through MNIR-FUNC-043.
    #[test]
    fn structurally_invalid_function_candidate_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        transaction.commit().unwrap();
        let source_revision_id = program.revision_id();

        let mut transaction = program.begin_transaction();
        let function_id = transaction
            .add_function(module_id, IntrinsicType::Unit)
            .unwrap();
        let function = transaction
            .working_modules_mut()
            .get_mut(&module_id)
            .unwrap()
            .functions
            .remove(&function_id)
            .unwrap();
        let mismatched_collection_id = FunctionId(function_id.0 + 1);
        transaction
            .working_modules_mut()
            .get_mut(&module_id)
            .unwrap()
            .functions
            .insert(mismatched_collection_id, function);

        assert!(matches!(
            transaction.commit(),
            Err(MutationError::StructuralViolation(
                StructuralError::FunctionIdentityMismatch { .. }
            ))
        ));
        assert_eq!(transaction.state(), TransactionState::Failed);
        drop(transaction);

        assert_eq!(program.revision_id(), source_revision_id);
        assert_eq!(program.module(module_id).unwrap().function_count(), 0);
        assert!(program.validate_structure().is_ok());
    }

    // MNIR-FUNC-016, MNIR-FUNC-017, MNIR-FUNC-053, and MNIR-FUNC-054.
    #[test]
    fn duplicate_parameter_ownership_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        let first_function = transaction
            .add_function(module_id, IntrinsicType::Unit)
            .unwrap();
        transaction
            .add_parameter(first_function, IntrinsicType::Bool)
            .unwrap();
        transaction.commit().unwrap();
        let source_revision_id = program.revision_id();

        let mut transaction = program.begin_transaction();
        let second_function = transaction
            .add_function(module_id, IntrinsicType::Unit)
            .unwrap();
        let duplicated_parameter = transaction
            .working_modules_mut()
            .get(&module_id)
            .unwrap()
            .functions
            .get(&first_function)
            .unwrap()
            .parameters[0]
            .copied();
        transaction
            .working_modules_mut()
            .get_mut(&module_id)
            .unwrap()
            .functions
            .get_mut(&second_function)
            .unwrap()
            .parameters
            .push(duplicated_parameter);

        assert!(matches!(
            transaction.commit(),
            Err(MutationError::StructuralViolation(
                StructuralError::DuplicateParameterIdentity(_)
            ))
        ));
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision_id);
        assert!(program.function(second_function).is_none());
        assert!(program.validate_structure().is_ok());
    }

    // MNIR-FUNC-002, MNIR-FUNC-006, MNIR-FUNC-007, and MNIR-FUNC-054.
    #[test]
    fn duplicate_function_ownership_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let first_module = transaction.add_module().unwrap();
        let second_module = transaction.add_module().unwrap();
        let function_id = transaction
            .add_function(first_module, IntrinsicType::Unit)
            .unwrap();
        transaction.commit().unwrap();
        let source_revision_id = program.revision_id();

        let mut transaction = program.begin_transaction();
        let duplicated_function = transaction
            .working_modules_mut()
            .get(&first_module)
            .unwrap()
            .functions
            .get(&function_id)
            .unwrap()
            .copied();
        transaction
            .working_modules_mut()
            .get_mut(&second_module)
            .unwrap()
            .functions
            .insert(function_id, duplicated_function);

        assert!(matches!(
            transaction.commit(),
            Err(MutationError::StructuralViolation(
                StructuralError::DuplicateFunctionIdentity(_)
            ))
        ));
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision_id);
        assert_eq!(program.module(second_module).unwrap().function_count(), 0);
        assert!(program.validate_structure().is_ok());
    }
}
