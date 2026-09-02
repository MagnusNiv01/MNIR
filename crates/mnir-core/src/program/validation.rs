use std::collections::{HashMap, HashSet, VecDeque};

use crate::ids::{BlockId, ExpressionId, FunctionId, ModuleId, ParameterId};

use super::body::{Block, ExpressionKind};
use super::error::StructuralError;
use super::model::{Module, RevisionState};

pub(super) fn validate_revision_state(state: &RevisionState) -> Result<(), StructuralError> {
    validate_modules(
        &state.modules,
        &state.committed_module_ids,
        &state.committed_function_ids,
        &state.committed_parameter_ids,
        &state.committed_block_ids,
        &state.committed_expression_ids,
    )
}

pub(super) fn validate_modules(
    modules: &HashMap<ModuleId, Module>,
    committed_module_ids: &HashSet<ModuleId>,
    committed_function_ids: &HashSet<FunctionId>,
    committed_parameter_ids: &HashSet<ParameterId>,
    committed_block_ids: &HashSet<BlockId>,
    committed_expression_ids: &HashSet<ExpressionId>,
) -> Result<(), StructuralError> {
    let mut seen_function_ids = HashSet::new();
    let mut seen_parameter_ids = HashSet::new();
    let mut seen_block_ids = HashSet::new();
    let mut seen_expression_ids = HashSet::new();

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

            let Some(body) = &function.body else {
                continue;
            };
            let block = &body.block;
            if !committed_block_ids.contains(&block.id) {
                return Err(StructuralError::BlockIdentityNotCommitted(block.id));
            }
            if !seen_block_ids.insert(block.id) {
                return Err(StructuralError::DuplicateBlockIdentity(block.id));
            }

            for (&collection_id, expression) in &block.expressions {
                if collection_id != expression.id {
                    return Err(StructuralError::ExpressionIdentityMismatch {
                        collection_id,
                        expression_id: expression.id,
                    });
                }
                if !committed_expression_ids.contains(&expression.id) {
                    return Err(StructuralError::ExpressionIdentityNotCommitted(
                        expression.id,
                    ));
                }
                if !seen_expression_ids.insert(expression.id) {
                    return Err(StructuralError::DuplicateExpressionIdentity(expression.id));
                }
                if let ExpressionKind::ParameterReference(parameter_id) = expression.kind
                    && function.parameter(parameter_id).is_none()
                {
                    return Err(StructuralError::DanglingParameterReference {
                        expression_id: expression.id,
                        parameter_id,
                        function_id: function.id,
                    });
                }
                if let Some((left, right)) = expression.kind.arithmetic_operands() {
                    for operand_id in [left, right] {
                        if !block.expressions.contains_key(&operand_id) {
                            return Err(StructuralError::ArithmeticOperandNotInBlock {
                                expression_id: expression.id,
                                operand_id,
                                block_id: block.id,
                            });
                        }
                    }
                }
            }

            validate_acyclic_expression_dependencies(block)?;

            let Some(return_expression_id) = block.return_expression_id else {
                return Err(StructuralError::UnterminatedBlock(block.id));
            };
            if !block.expressions.contains_key(&return_expression_id) {
                return Err(StructuralError::ReturnExpressionNotInBlock {
                    block_id: block.id,
                    expression_id: return_expression_id,
                });
            }
        }
    }

    Ok(())
}

/// Uses Kahn's algorithm so structural validation does not depend on recursive
/// call depth. Duplicate left/right operand edges are retained deliberately;
/// `Add(E1, E1)` therefore decrements both dependencies without being mistaken
/// for a cycle (`MNIR-ARITH-009`, `MNIR-ARITH-096`).
fn validate_acyclic_expression_dependencies(block: &Block) -> Result<(), StructuralError> {
    let mut dependency_count = HashMap::with_capacity(block.expressions.len());
    let mut dependents: HashMap<ExpressionId, Vec<ExpressionId>> = HashMap::new();

    for expression in block.expressions.values() {
        let Some((left, right)) = expression.kind.arithmetic_operands() else {
            dependency_count.insert(expression.id, 0_usize);
            continue;
        };
        dependency_count.insert(expression.id, 2);
        dependents.entry(left).or_default().push(expression.id);
        dependents.entry(right).or_default().push(expression.id);
    }

    let mut ready: VecDeque<_> = dependency_count
        .iter()
        .filter_map(|(&id, &count)| (count == 0).then_some(id))
        .collect();
    let mut processed = 0_usize;

    while let Some(id) = ready.pop_front() {
        processed += 1;
        if let Some(expression_dependents) = dependents.get(&id) {
            for dependent_id in expression_dependents {
                let Some(count) = dependency_count.get_mut(dependent_id) else {
                    continue;
                };
                *count -= 1;
                if *count == 0 {
                    ready.push_back(*dependent_id);
                }
            }
        }
    }

    if processed == block.expressions.len() {
        Ok(())
    } else {
        Err(StructuralError::CyclicExpressionDependency(block.id))
    }
}

#[cfg(test)]
mod tests {
    use crate::IntrinsicType;
    use crate::ids::{ExpressionId, FunctionId, ModuleId};

    use super::super::body::ExpressionKind;
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

    // AR-EXPR-028 and MNIR-EXPR-038/-072/-073. This test is deliberately
    // colocated with private structural state instead of weakening the API.
    #[test]
    fn structurally_corrupt_return_reference_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        let function_id = transaction
            .add_function(module_id, IntrinsicType::Unit)
            .unwrap();
        let block_id = transaction.create_function_body(function_id).unwrap();
        let expression_id = transaction.add_unit_literal(block_id).unwrap();
        transaction.set_return(block_id, expression_id).unwrap();
        transaction.commit().unwrap();
        let source_revision = program.revision_id();

        let mut transaction = program.begin_transaction();
        let corrupt_expression_id = ExpressionId(expression_id.0 + 1_000);
        transaction
            .working_modules_mut()
            .get_mut(&module_id)
            .unwrap()
            .functions
            .get_mut(&function_id)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .block
            .return_expression_id = Some(corrupt_expression_id);

        assert!(matches!(
            transaction.commit(),
            Err(MutationError::StructuralViolation(
                StructuralError::ReturnExpressionNotInBlock {
                    block_id: actual_block_id,
                    expression_id: actual_expression_id,
                }
            )) if actual_block_id == block_id && actual_expression_id == corrupt_expression_id
        ));
        assert_eq!(transaction.state(), TransactionState::Failed);
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision);
        assert_eq!(
            program.block(block_id).unwrap().return_expression_id(),
            Some(expression_id)
        );
        assert!(program.validate_structure().is_ok());
    }

    // AR-ARITH-027 and MNIR-ARITH-009/-011/-069/-070. Only this private test
    // can retarget operands; the production API keeps Expressions immutable.
    #[test]
    fn cyclic_expression_dependency_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let source_revision = program.revision_id();
        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        let function_id = transaction
            .add_function(module_id, IntrinsicType::Int32)
            .unwrap();
        let block_id = transaction.create_function_body(function_id).unwrap();
        let first = transaction.add_int32_literal(block_id, 1).unwrap();
        let second = transaction.add_int32_literal(block_id, 2).unwrap();
        transaction.set_return(block_id, first).unwrap();

        let block = &mut transaction
            .working_modules_mut()
            .get_mut(&module_id)
            .unwrap()
            .functions
            .get_mut(&function_id)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .block;
        block.expressions.get_mut(&first).unwrap().kind = ExpressionKind::Add {
            left: second,
            right: second,
        };
        block.expressions.get_mut(&second).unwrap().kind = ExpressionKind::Multiply {
            left: first,
            right: first,
        };

        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::CyclicExpressionDependency(
                block_id
            ))
        );
        assert_eq!(transaction.state(), TransactionState::Failed);
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision);
        assert!(program.function(function_id).is_none());
        assert!(program.validate_structure().is_ok());
    }

    // MNIR-ARITH-006/-007/-063/-069/-070. Structural validation remains a
    // defense even though safe construction rejects missing/foreign operands.
    #[test]
    fn structurally_corrupt_arithmetic_operand_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let source_revision = program.revision_id();
        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        let function_id = transaction
            .add_function(module_id, IntrinsicType::Int32)
            .unwrap();
        let block_id = transaction.create_function_body(function_id).unwrap();
        let expression_id = transaction.add_int32_literal(block_id, 1).unwrap();
        transaction.set_return(block_id, expression_id).unwrap();
        let missing_operand = ExpressionId(expression_id.0 + 1_000);
        transaction
            .working_modules_mut()
            .get_mut(&module_id)
            .unwrap()
            .functions
            .get_mut(&function_id)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .block
            .expressions
            .get_mut(&expression_id)
            .unwrap()
            .kind = ExpressionKind::Add {
            left: missing_operand,
            right: expression_id,
        };

        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::ArithmeticOperandNotInBlock {
                expression_id,
                operand_id: missing_operand,
                block_id,
            })
        );
        assert_eq!(transaction.state(), TransactionState::Failed);
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision);
        assert!(program.function(function_id).is_none());
        assert!(program.validate_structure().is_ok());
    }
}
