use std::collections::{HashMap, HashSet, VecDeque};

use crate::ids::{BlockId, ExpressionId, FunctionId, ModuleId, ParameterId};

use super::body::{Block, ExpressionKind, Terminator};
use super::error::StructuralError;
use super::model::{Module, RevisionState, find_function};

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
            if body.blocks.is_empty() {
                return Err(StructuralError::FunctionBodyHasNoBlocks(function.id));
            }
            if !body.blocks.contains_key(&body.entry_block_id) {
                return Err(StructuralError::EntryBlockNotInBody {
                    function_id: function.id,
                    entry_block_id: body.entry_block_id,
                });
            }

            for (&block_collection_id, block) in &body.blocks {
                if block_collection_id != block.id {
                    return Err(StructuralError::BlockIdentityMismatch {
                        collection_id: block_collection_id,
                        block_id: block.id,
                    });
                }
                if !committed_block_ids.contains(&block.id) {
                    return Err(StructuralError::BlockIdentityNotCommitted(block.id));
                }
                if !seen_block_ids.insert(block.id) {
                    return Err(StructuralError::DuplicateBlockIdentity(block.id));
                }

                validate_block_expressions(
                    modules,
                    function,
                    block,
                    committed_expression_ids,
                    &mut seen_expression_ids,
                )?;
                validate_acyclic_expression_dependencies(block)?;
                validate_effect_sequence(block)?;

                match block.terminator {
                    None => return Err(StructuralError::UnterminatedBlock(block.id)),
                    Some(Terminator::Return { expression }) => {
                        if !block.expressions.contains_key(&expression) {
                            return Err(StructuralError::ReturnExpressionNotInBlock {
                                block_id: block.id,
                                expression_id: expression,
                            });
                        }
                    }
                    Some(Terminator::Branch {
                        condition,
                        true_block,
                        false_block,
                    }) => {
                        if !block.expressions.contains_key(&condition) {
                            return Err(StructuralError::BranchConditionNotInBlock {
                                block_id: block.id,
                                expression_id: condition,
                            });
                        }
                        for target_block_id in [true_block, false_block] {
                            if !body.blocks.contains_key(&target_block_id) {
                                return Err(StructuralError::BranchTargetNotInBody {
                                    block_id: block.id,
                                    target_block_id,
                                });
                            }
                        }
                    }
                }
            }

            validate_control_flow(function.id, body.entry_block_id, &body.blocks)?;
        }
    }

    Ok(())
}

fn validate_block_expressions(
    modules: &HashMap<ModuleId, Module>,
    function: &super::model::Function,
    block: &Block,
    committed_expression_ids: &HashSet<ExpressionId>,
    seen_expression_ids: &mut HashSet<ExpressionId>,
) -> Result<(), StructuralError> {
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
        if let ExpressionKind::ParameterReference(parameter_id) = &expression.kind
            && function.parameter(*parameter_id).is_none()
        {
            return Err(StructuralError::DanglingParameterReference {
                expression_id: expression.id,
                parameter_id: *parameter_id,
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
        if let Some((left, right)) = expression.kind.comparison_operands() {
            for operand_id in [left, right] {
                if !block.expressions.contains_key(&operand_id) {
                    return Err(StructuralError::ComparisonOperandNotInBlock {
                        expression_id: expression.id,
                        operand_id,
                        block_id: block.id,
                    });
                }
            }
        }
        if let ExpressionKind::Call { target, arguments } = &expression.kind {
            if find_function(modules, *target).is_none() {
                return Err(StructuralError::DanglingCallTarget {
                    expression_id: expression.id,
                    function_id: *target,
                });
            }
            for &argument_id in arguments {
                if !block.expressions.contains_key(&argument_id) {
                    return Err(StructuralError::CallArgumentNotInBlock {
                        expression_id: expression.id,
                        argument_id,
                        block_id: block.id,
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_effect_sequence(block: &Block) -> Result<(), StructuralError> {
    let mut positions = HashMap::with_capacity(block.effect_sequence.len());
    for (position, &expression_id) in block.effect_sequence.iter().enumerate() {
        let Some(expression) = block.expressions.get(&expression_id) else {
            return Err(StructuralError::EffectSequenceEntryNotInBlock {
                block_id: block.id,
                expression_id,
            });
        };
        if !expression.kind.is_call() {
            return Err(StructuralError::EffectSequenceEntryNotCall {
                block_id: block.id,
                expression_id,
            });
        }
        if positions.insert(expression_id, position).is_some() {
            return Err(StructuralError::DuplicateEffectSequenceEntry {
                block_id: block.id,
                expression_id,
            });
        }
    }

    for expression in block.expressions.values() {
        if expression.kind.is_call() && !positions.contains_key(&expression.id) {
            return Err(StructuralError::CallMissingFromEffectSequence {
                block_id: block.id,
                expression_id: expression.id,
            });
        }
    }

    // A Call dependency can be separated from its consumer by any number of
    // pure Expressions. Every reachable Call dependency must precede the
    // consuming Call in the semantic effect order (`MNIR-CALL-056` through
    // `MNIR-CALL-060`).
    for expression in block
        .expressions
        .values()
        .filter(|expression| expression.kind.is_call())
    {
        let dependent_position = positions[&expression.id];
        let mut visited = HashSet::new();
        let mut pending = expression.kind.dependencies();
        while let Some(dependency_id) = pending.pop() {
            if !visited.insert(dependency_id) {
                continue;
            }
            let dependency = block
                .expressions
                .get(&dependency_id)
                .expect("validated Expression dependency");
            if dependency.kind.is_call() && positions[&dependency_id] >= dependent_position {
                return Err(StructuralError::EffectSequenceOrderConflict {
                    block_id: block.id,
                    dependency_id,
                    dependent_id: expression.id,
                });
            }
            pending.extend(dependency.kind.dependencies());
        }
    }

    Ok(())
}

fn validate_control_flow(
    function_id: FunctionId,
    entry_block_id: BlockId,
    blocks: &HashMap<BlockId, Block>,
) -> Result<(), StructuralError> {
    let mut reachable = HashSet::new();
    let mut pending = vec![entry_block_id];
    while let Some(block_id) = pending.pop() {
        if !reachable.insert(block_id) {
            continue;
        }
        if let Some(Terminator::Branch {
            true_block,
            false_block,
            ..
        }) = blocks.get(&block_id).and_then(|block| block.terminator)
        {
            pending.push(true_block);
            pending.push(false_block);
        }
    }
    if let Some(&block_id) = blocks.keys().find(|id| !reachable.contains(id)) {
        return Err(StructuralError::UnreachableBlock {
            function_id,
            block_id,
        });
    }

    let mut indegree = HashMap::with_capacity(blocks.len());
    for &block_id in blocks.keys() {
        indegree.insert(block_id, 0_usize);
    }
    for block in blocks.values() {
        if let Some(Terminator::Branch {
            true_block,
            false_block,
            ..
        }) = block.terminator
        {
            *indegree
                .get_mut(&true_block)
                .expect("validated Branch target") += 1;
            *indegree
                .get_mut(&false_block)
                .expect("validated Branch target") += 1;
        }
    }
    let mut ready: VecDeque<_> = indegree
        .iter()
        .filter_map(|(&id, &count)| (count == 0).then_some(id))
        .collect();
    let mut processed = 0;
    while let Some(block_id) = ready.pop_front() {
        processed += 1;
        if let Some(Terminator::Branch {
            true_block,
            false_block,
            ..
        }) = blocks.get(&block_id).and_then(|block| block.terminator)
        {
            for target in [true_block, false_block] {
                let count = indegree.get_mut(&target).expect("validated Branch target");
                *count -= 1;
                if *count == 0 {
                    ready.push_back(target);
                }
            }
        }
    }
    if processed == blocks.len() {
        Ok(())
    } else {
        Err(StructuralError::CyclicControlFlow(function_id))
    }
}

/// Uses Kahn's algorithm so structural validation does not depend on recursive
/// dependency depth. Duplicate operand/argument edges are retained
/// deliberately and therefore cannot be mistaken for a cycle
/// (`MNIR-ARITH-009`, `MNIR-CMP-014`, `MNIR-CALL-131`, `MNIR-CALL-132`).
fn validate_acyclic_expression_dependencies(block: &Block) -> Result<(), StructuralError> {
    let mut dependency_count = HashMap::with_capacity(block.expressions.len());
    let mut dependents: HashMap<ExpressionId, Vec<ExpressionId>> = HashMap::new();

    for expression in block.expressions.values() {
        let dependencies = expression.kind.dependencies();
        dependency_count.insert(expression.id, dependencies.len());
        for dependency_id in dependencies {
            dependents
                .entry(dependency_id)
                .or_default()
                .push(expression.id);
        }
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
    use crate::ids::{BlockId, ExpressionId, FunctionId, ModuleId};

    use super::super::body::{ExpressionKind, Terminator};
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
            .blocks
            .get_mut(&block_id)
            .unwrap()
            .terminator = Some(Terminator::Return {
            expression: corrupt_expression_id,
        });

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
            .blocks
            .get_mut(&block_id)
            .unwrap();
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
            .blocks
            .get_mut(&block_id)
            .unwrap()
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

    // AR-CMP-038 and MNIR-CMP-014/-015/-016/-061/-062. The public API cannot
    // retarget either family; this private corruption verifies the shared
    // dependency graph rejects a cross-family cycle atomically.
    #[test]
    fn cross_family_expression_cycle_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let source_revision = program.revision_id();
        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        let function_id = transaction
            .add_function(module_id, IntrinsicType::Bool)
            .unwrap();
        let block_id = transaction.create_function_body(function_id).unwrap();
        let arithmetic = transaction.add_int32_literal(block_id, 1).unwrap();
        let comparison = transaction.add_int32_literal(block_id, 2).unwrap();
        transaction.set_return(block_id, comparison).unwrap();

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
            .blocks
            .get_mut(&block_id)
            .unwrap();
        block.expressions.get_mut(&arithmetic).unwrap().kind = ExpressionKind::Add {
            left: comparison,
            right: comparison,
        };
        block.expressions.get_mut(&comparison).unwrap().kind = ExpressionKind::Equal {
            left: arithmetic,
            right: arithmetic,
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

    // MNIR-CMP-011/-012/-061/-062. Structural validation remains a defense
    // even though controlled construction rejects missing operands.
    #[test]
    fn structurally_corrupt_comparison_operand_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let source_revision = program.revision_id();
        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        let function_id = transaction
            .add_function(module_id, IntrinsicType::Bool)
            .unwrap();
        let block_id = transaction.create_function_body(function_id).unwrap();
        let expression_id = transaction.add_bool_literal(block_id, true).unwrap();
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
            .blocks
            .get_mut(&block_id)
            .unwrap()
            .expressions
            .get_mut(&expression_id)
            .unwrap()
            .kind = ExpressionKind::Equal {
            left: missing_operand,
            right: expression_id,
        };

        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::ComparisonOperandNotInBlock {
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

    // AR-CFG-046 and MNIR-CFG-004/-073/-074/-131. Corrupt entry state is
    // reachable only from this private test support.
    #[test]
    fn corrupt_entry_reference_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        let function_id = transaction
            .add_function(module_id, IntrinsicType::Unit)
            .unwrap();
        let entry = transaction.create_function_body(function_id).unwrap();
        let unit = transaction.add_unit_literal(entry).unwrap();
        transaction.set_return(entry, unit).unwrap();
        transaction.commit().unwrap();
        let revision = program.revision_id();

        let mut transaction = program.begin_transaction();
        let missing_entry = BlockId(entry.0 + 10_000);
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
            .entry_block_id = missing_entry;
        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::EntryBlockNotInBody {
                function_id,
                entry_block_id: missing_entry,
            })
        );
        drop(transaction);
        assert_eq!(program.revision_id(), revision);
        assert_eq!(
            program
                .function(function_id)
                .unwrap()
                .body()
                .unwrap()
                .block_id(),
            entry
        );
    }

    // AR-CFG-047 and MNIR-CFG-029/-030/-073/-074/-132.
    #[test]
    fn corrupt_branch_references_cannot_commit() {
        fn program_with_branch() -> (
            MnirProgram,
            ModuleId,
            FunctionId,
            BlockId,
            BlockId,
            ExpressionId,
        ) {
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
            transaction.commit().unwrap();
            (program, module, function, entry, target, condition)
        }

        let (mut program, module, function, entry, target, condition) = program_with_branch();
        let missing_expression = ExpressionId(condition.0 + 10_000);
        let mut transaction = program.begin_transaction();
        transaction
            .working_modules_mut()
            .get_mut(&module)
            .unwrap()
            .functions
            .get_mut(&function)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .blocks
            .get_mut(&entry)
            .unwrap()
            .terminator = Some(Terminator::Branch {
            condition: missing_expression,
            true_block: target,
            false_block: target,
        });
        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::BranchConditionNotInBlock {
                block_id: entry,
                expression_id: missing_expression,
            })
        );

        let (mut program, module, function, entry, _, condition) = program_with_branch();
        let missing_target = BlockId(entry.0 + 10_000);
        let mut transaction = program.begin_transaction();
        transaction
            .working_modules_mut()
            .get_mut(&module)
            .unwrap()
            .functions
            .get_mut(&function)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .blocks
            .get_mut(&entry)
            .unwrap()
            .terminator = Some(Terminator::Branch {
            condition,
            true_block: missing_target,
            false_block: missing_target,
        });
        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::BranchTargetNotInBody {
                block_id: entry,
                target_block_id: missing_target,
            })
        );

        let (mut program, module, function, entry, target, condition) = program_with_branch();
        let mut setup = program.begin_transaction();
        let foreign_function = setup.add_function(module, IntrinsicType::Unit).unwrap();
        let foreign_block = setup.create_function_body(foreign_function).unwrap();
        let foreign_unit = setup.add_unit_literal(foreign_block).unwrap();
        setup.set_return(foreign_block, foreign_unit).unwrap();
        setup.commit().unwrap();
        let mut transaction = program.begin_transaction();
        transaction
            .working_modules_mut()
            .get_mut(&module)
            .unwrap()
            .functions
            .get_mut(&function)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .blocks
            .get_mut(&entry)
            .unwrap()
            .terminator = Some(Terminator::Branch {
            condition,
            true_block: foreign_block,
            false_block: target,
        });
        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::BranchTargetNotInBody {
                block_id: entry,
                target_block_id: foreign_block,
            })
        );
    }

    // AR-CALL-051 and MNIR-CALL-039 through MNIR-CALL-042, MNIR-CALL-055,
    // MNIR-CALL-059, MNIR-CALL-068, and MNIR-CALL-069. Production mutation
    // rejects most of these states earlier; private access exercises the
    // commit-time defense without weakening encapsulation.
    #[test]
    fn corrupt_effect_sequences_cannot_commit() {
        fn fixture() -> (
            MnirProgram,
            ModuleId,
            FunctionId,
            BlockId,
            ExpressionId,
            ExpressionId,
            ExpressionId,
        ) {
            let mut program = MnirProgram::new().unwrap();
            let mut transaction = program.begin_transaction();
            let module = transaction.add_module().unwrap();
            let target = transaction
                .add_function(module, IntrinsicType::Unit)
                .unwrap();
            let caller = transaction
                .add_function(module, IntrinsicType::Unit)
                .unwrap();
            let block = transaction.create_function_body(caller).unwrap();
            let call = transaction
                .add_call_expression(block, target, Vec::new())
                .unwrap();
            let pure = transaction.add_unit_literal(block).unwrap();
            transaction.set_effect_sequence(block, vec![call]).unwrap();
            transaction.set_return(block, pure).unwrap();

            let foreign = transaction
                .add_function(module, IntrinsicType::Unit)
                .unwrap();
            let foreign_block = transaction.create_function_body(foreign).unwrap();
            let foreign_call = transaction
                .add_call_expression(foreign_block, target, Vec::new())
                .unwrap();
            transaction
                .set_effect_sequence(foreign_block, vec![foreign_call])
                .unwrap();
            let foreign_unit = transaction.add_unit_literal(foreign_block).unwrap();
            transaction.set_return(foreign_block, foreign_unit).unwrap();
            transaction.commit().unwrap();
            (program, module, caller, block, call, pure, foreign_call)
        }

        let (mut program, module, caller, block, call, _, _) = fixture();
        let mut transaction = program.begin_transaction();
        transaction
            .working_modules_mut()
            .get_mut(&module)
            .unwrap()
            .functions
            .get_mut(&caller)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .blocks
            .get_mut(&block)
            .unwrap()
            .effect_sequence
            .clear();
        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::CallMissingFromEffectSequence {
                block_id: block,
                expression_id: call,
            })
        );

        let (mut program, module, caller, block, call, pure, _) = fixture();
        let mut transaction = program.begin_transaction();
        transaction
            .working_modules_mut()
            .get_mut(&module)
            .unwrap()
            .functions
            .get_mut(&caller)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .blocks
            .get_mut(&block)
            .unwrap()
            .effect_sequence = vec![call, pure];
        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::EffectSequenceEntryNotCall {
                block_id: block,
                expression_id: pure,
            })
        );

        let (mut program, module, caller, block, call, _, foreign_call) = fixture();
        let mut transaction = program.begin_transaction();
        transaction
            .working_modules_mut()
            .get_mut(&module)
            .unwrap()
            .functions
            .get_mut(&caller)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .blocks
            .get_mut(&block)
            .unwrap()
            .effect_sequence = vec![call, foreign_call];
        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::EffectSequenceEntryNotInBlock {
                block_id: block,
                expression_id: foreign_call,
            })
        );

        let (mut program, module, caller, block, call, _, _) = fixture();
        let mut transaction = program.begin_transaction();
        transaction
            .working_modules_mut()
            .get_mut(&module)
            .unwrap()
            .functions
            .get_mut(&caller)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .blocks
            .get_mut(&block)
            .unwrap()
            .effect_sequence = vec![call, call];
        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::DuplicateEffectSequenceEntry {
                block_id: block,
                expression_id: call,
            })
        );

        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let module = transaction.add_module().unwrap();
        let producer = transaction
            .add_function(module, IntrinsicType::Int32)
            .unwrap();
        let consumer = transaction
            .add_function(module, IntrinsicType::Unit)
            .unwrap();
        let caller = transaction
            .add_function(module, IntrinsicType::Unit)
            .unwrap();
        let block = transaction.create_function_body(caller).unwrap();
        let first = transaction
            .add_call_expression(block, producer, Vec::new())
            .unwrap();
        let second = transaction
            .add_call_expression(block, consumer, vec![first])
            .unwrap();
        transaction
            .set_effect_sequence(block, vec![first, second])
            .unwrap();
        let unit = transaction.add_unit_literal(block).unwrap();
        transaction.set_return(block, unit).unwrap();
        transaction.commit().unwrap();

        let mut transaction = program.begin_transaction();
        transaction
            .working_modules_mut()
            .get_mut(&module)
            .unwrap()
            .functions
            .get_mut(&caller)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .blocks
            .get_mut(&block)
            .unwrap()
            .effect_sequence = vec![second, first];
        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::EffectSequenceOrderConflict {
                block_id: block,
                dependency_id: first,
                dependent_id: second,
            })
        );
    }

    // AR-CALL-053 and MNIR-CALL-131/-132: Call argument edges share the same
    // DAG with arithmetic and comparison dependencies.
    #[test]
    fn mixed_call_and_arithmetic_dependency_cycle_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let module = transaction.add_module().unwrap();
        let target = transaction
            .add_function(module, IntrinsicType::Int32)
            .unwrap();
        let caller = transaction
            .add_function(module, IntrinsicType::Int32)
            .unwrap();
        let block = transaction.create_function_body(caller).unwrap();
        let call = transaction
            .add_call_expression(block, target, Vec::new())
            .unwrap();
        let literal = transaction.add_int32_literal(block, 1).unwrap();
        let arithmetic = transaction
            .add_add_expression(block, call, literal)
            .unwrap();
        transaction.set_effect_sequence(block, vec![call]).unwrap();
        transaction.set_return(block, arithmetic).unwrap();
        transaction
            .working_modules_mut()
            .get_mut(&module)
            .unwrap()
            .functions
            .get_mut(&caller)
            .unwrap()
            .body
            .as_mut()
            .unwrap()
            .blocks
            .get_mut(&block)
            .unwrap()
            .expressions
            .get_mut(&call)
            .unwrap()
            .kind = ExpressionKind::Call {
            target,
            arguments: vec![arithmetic],
        };

        assert_eq!(
            transaction.commit().unwrap_err(),
            MutationError::StructuralViolation(StructuralError::CyclicExpressionDependency(block))
        );
    }
}
