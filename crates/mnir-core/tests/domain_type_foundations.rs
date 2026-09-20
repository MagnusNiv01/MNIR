use std::collections::HashSet;

use mnir_core::{
    BlockId, ExpressionId, ExpressionKind, ExpressionTypeError, IntrinsicType, MnirProgram,
    MutationError, MutationTransaction, StructuralError, TransactionState, TypeId, ValueType,
};

fn add_all_domain_expression_kinds(
    transaction: &mut MutationTransaction<'_>,
    block: BlockId,
    domain: TypeId,
) -> [ExpressionId; 4] {
    let text = transaction.add_text_literal(block, "x").unwrap();
    let bytes = transaction.add_bytes_literal(block, vec![1]).unwrap();
    let source = transaction.add_int64_literal(block, 1).unwrap();
    let construct = transaction
        .add_domain_construct(block, domain, source)
        .unwrap();
    let project = transaction.add_domain_project(block, construct).unwrap();
    [text, bytes, construct, project]
}

// AR-DOMAIN-001 through -010, -015/-016, -019, -022, -025/-026, -039,
// -061/-062, and -066; MNIR-DOMAIN-003 through -031, -047 through -050,
// -063, -070, and -078.
#[test]
fn domain_types_literals_signatures_and_live_representation_are_inspectable() {
    let all = [
        IntrinsicType::Int32,
        IntrinsicType::Int64,
        IntrinsicType::Bool,
        IntrinsicType::Unit,
        IntrinsicType::Text,
        IntrinsicType::Bytes,
    ];
    for (left_index, left) in all.iter().enumerate() {
        for (right_index, right) in all.iter().enumerate() {
            assert_eq!(left == right, left_index == right_index);
        }
    }

    let mut program = MnirProgram::new().unwrap();
    let namespace = program.allocation_namespace_id();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let customer = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let order = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    assert_eq!(customer.namespace_id(), namespace);
    assert_eq!(order.namespace_id(), namespace);
    assert_ne!(customer, order);
    assert!(customer.counter() < order.counter());
    tx.set_domain_type_preferred_name(customer, Some("Identifier".into()))
        .unwrap();
    tx.set_domain_type_preferred_name(order, Some("Identifier".into()))
        .unwrap();
    tx.set_domain_type_documentation(customer, Some("customer identity".into()))
        .unwrap();

    let function = tx
        .add_function(module, ValueType::Domain(customer))
        .unwrap();
    let parameter = tx
        .add_parameter(function, ValueType::Domain(customer))
        .unwrap();
    let block = tx.create_function_body(function).unwrap();
    let text_empty = tx.add_text_literal(block, "").unwrap();
    let text_non_ascii = tx.add_text_literal(block, "å🙂").unwrap();
    let text_composed = tx.add_text_literal(block, "é").unwrap();
    let text_decomposed = tx.add_text_literal(block, "e\u{301}").unwrap();
    let bytes_empty = tx.add_bytes_literal(block, Vec::<u8>::new()).unwrap();
    let bytes = tx.add_bytes_literal(block, vec![0, 255, 1]).unwrap();
    let source = tx.add_int64_literal(block, 7).unwrap();
    let construct = tx.add_domain_construct(block, customer, source).unwrap();
    let project = tx.add_domain_project(block, construct).unwrap();
    let parameter_reference = tx.add_parameter_reference(block, parameter).unwrap();
    tx.set_return(block, parameter_reference).unwrap();

    assert_eq!(
        tx.expression_type(text_empty),
        Some(Ok(IntrinsicType::Text.into()))
    );
    assert_eq!(
        tx.expression_type(bytes),
        Some(Ok(IntrinsicType::Bytes.into()))
    );
    assert_eq!(
        tx.expression_type(construct),
        Some(Ok(ValueType::Domain(customer)))
    );
    assert_eq!(
        tx.expression_type(project),
        Some(Ok(IntrinsicType::Int64.into()))
    );
    assert_eq!(
        tx.expression_type(parameter_reference),
        Some(Ok(ValueType::Domain(customer)))
    );
    tx.commit().unwrap();

    let block_view = program.block(block).unwrap();
    assert!(
        matches!(block_view.expression(text_empty).unwrap().kind(), ExpressionKind::TextLiteral(value) if value.is_empty())
    );
    assert!(
        matches!(block_view.expression(text_non_ascii).unwrap().kind(), ExpressionKind::TextLiteral(value) if value == "å🙂")
    );
    assert!(
        matches!(block_view.expression(text_composed).unwrap().kind(), ExpressionKind::TextLiteral(value) if value == "é")
    );
    assert!(
        matches!(block_view.expression(text_decomposed).unwrap().kind(), ExpressionKind::TextLiteral(value) if value == "e\u{301}")
    );
    assert_ne!(
        block_view.expression(text_composed).unwrap().kind(),
        block_view.expression(text_decomposed).unwrap().kind()
    );
    assert!(
        matches!(block_view.expression(bytes_empty).unwrap().kind(), ExpressionKind::BytesLiteral(value) if value.is_empty())
    );
    assert!(
        matches!(block_view.expression(bytes).unwrap().kind(), ExpressionKind::BytesLiteral(value) if value == &[0, 255, 1])
    );
    assert!(
        matches!(block_view.expression(construct).unwrap().kind(), ExpressionKind::DomainConstruct { type_id, value } if *type_id == customer && *value == source)
    );
    assert!(
        matches!(block_view.expression(project).unwrap().kind(), ExpressionKind::DomainProject { value } if *value == construct)
    );

    assert_eq!(program.module(module).unwrap().domain_type_count(), 2);
    assert_eq!(
        program
            .domain_type(customer)
            .unwrap()
            .presentation()
            .preferred_name(),
        Some("Identifier")
    );
    assert_eq!(
        *program.function(function).unwrap().return_type(),
        ValueType::Domain(customer)
    );
    assert_eq!(
        *program.parameter(parameter).unwrap().value_type(),
        ValueType::Domain(customer)
    );
    assert_ne!(ValueType::Domain(customer), ValueType::Domain(order));
    assert_ne!(
        ValueType::Domain(customer),
        ValueType::Intrinsic(IntrinsicType::Int64)
    );

    let function_id = function;
    let parameter_id = parameter;
    let revision = program.revision_id();
    let mut tx = program.begin_transaction();
    tx.set_domain_type_representation(customer, IntrinsicType::Text)
        .unwrap();
    assert_eq!(
        tx.expression_type(construct),
        Some(Ok(ValueType::Domain(customer)))
    );
    assert_eq!(
        tx.expression_type(project),
        Some(Ok(IntrinsicType::Text.into()))
    );
    tx.set_domain_type_preferred_name(customer, Some("Renamed".into()))
        .unwrap();
    tx.commit().unwrap();
    assert_ne!(program.revision_id(), revision);
    assert_eq!(program.domain_type(customer).unwrap().id(), customer);
    assert_eq!(program.function(function_id).unwrap().id(), function_id);
    assert_eq!(program.parameter(parameter_id).unwrap().id(), parameter_id);
}

// AR-DOMAIN-020 through -024, -027, -029 through -034, and -038/-068;
// MNIR-DOMAIN-035 through -046 and -055 through -064.
#[test]
fn generalized_inspection_preserves_semantic_invalidity_and_pure_dependencies() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let number = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let function = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(function).unwrap();
    let int = tx.add_int64_literal(block, 1).unwrap();
    let boolean = tx.add_bool_literal(block, true).unwrap();
    let good_construct = tx.add_domain_construct(block, number, int).unwrap();
    let bad_construct = tx.add_domain_construct(block, number, boolean).unwrap();
    let project = tx.add_domain_project(block, good_construct).unwrap();
    let bad_project = tx.add_domain_project(block, int).unwrap();
    let domain_add = tx
        .add_add_expression(block, good_construct, good_construct)
        .unwrap();
    let domain_equal = tx
        .add_equal_expression(block, good_construct, good_construct)
        .unwrap();
    let projected_add = tx.add_add_expression(block, project, project).unwrap();
    let text = tx.add_text_literal(block, "x").unwrap();
    let bytes = tx.add_bytes_literal(block, vec![120]).unwrap();
    let text_bytes = tx.add_equal_expression(block, text, bytes).unwrap();
    tx.set_return(block, bad_construct).unwrap();

    assert_eq!(
        tx.expression_type(bad_construct),
        Some(Ok(ValueType::Domain(number)))
    );
    assert!(matches!(
        tx.expression_type(bad_project),
        Some(Err(ExpressionTypeError::DomainProjectSourceNotDomain {
            actual_type: ValueType::Intrinsic(IntrinsicType::Int64),
            ..
        }))
    ));
    assert!(matches!(
        tx.expression_type(domain_add),
        Some(Err(ExpressionTypeError::UnsupportedOperandType {
            operand_type: ValueType::Domain(actual),
            ..
        })) if actual == number
    ));
    assert!(matches!(
        tx.expression_type(domain_equal),
        Some(Err(ExpressionTypeError::UnsupportedOperandType {
            operand_type: ValueType::Domain(actual),
            ..
        })) if actual == number
    ));
    assert_eq!(
        tx.expression_type(projected_add),
        Some(Ok(IntrinsicType::Int64.into()))
    );
    assert!(matches!(
        tx.expression_type(text_bytes),
        Some(Err(ExpressionTypeError::OperandTypeMismatch {
            left_type: ValueType::Intrinsic(IntrinsicType::Text),
            right_type: ValueType::Intrinsic(IntrinsicType::Bytes),
            ..
        }))
    ));
    assert_eq!(tx.state(), TransactionState::Active);
    assert!(tx.block(block).unwrap().effect_sequence().is_empty());
    tx.commit().unwrap();

    for pure in [text, bytes, good_construct, project] {
        let mut rejected = program.begin_transaction();
        assert_eq!(
            rejected.set_effect_sequence(block, vec![pure]),
            Err(MutationError::EffectSequenceEntryNotCall(pure))
        );
        assert_eq!(rejected.state(), TransactionState::Failed);
        rejected.discard().unwrap();
    }
}

// AR-DOMAIN-024/-038/-041 and MNIR-DOMAIN-040/-042/-043. Removing a
// referenced Domain Type may leave repairable working state; public type
// inspection reports the precise unavailable categories without poisoning.
#[test]
fn dangling_domain_inspection_is_typed_and_read_only() {
    let mut program = MnirProgram::new().unwrap();
    let mut setup = program.begin_transaction();
    let module = setup.add_module().unwrap();
    let domain = setup.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let function = setup.add_function(module, IntrinsicType::Unit).unwrap();
    let parameter = setup
        .add_parameter(function, ValueType::Domain(domain))
        .unwrap();
    let block = setup.create_function_body(function).unwrap();
    let parameter_reference = setup.add_parameter_reference(block, parameter).unwrap();
    let source = setup.add_int64_literal(block, 1).unwrap();
    let construct = setup.add_domain_construct(block, domain, source).unwrap();
    let project = setup.add_domain_project(block, construct).unwrap();
    let unit = setup.add_unit_literal(block).unwrap();
    setup.set_return(block, unit).unwrap();
    setup.commit().unwrap();

    let mut repairable = program.begin_transaction();
    repairable.remove_domain_type(domain).unwrap();
    assert!(matches!(
        repairable.expression_type(parameter_reference),
        Some(Err(ExpressionTypeError::UnresolvedDomainType {
            expression_id,
            type_id,
        })) if expression_id == parameter_reference && type_id == domain
    ));
    assert!(matches!(
        repairable.expression_type(construct),
        Some(Err(ExpressionTypeError::UnresolvedDomainType {
            expression_id,
            type_id,
        })) if expression_id == construct && type_id == domain
    ));
    assert!(matches!(
        repairable.expression_type(project),
        Some(Err(ExpressionTypeError::DomainProjectSourceTypeUnavailable {
            expression_id,
            source_expression_id,
        })) if expression_id == project && source_expression_id == construct
    ));
    assert_eq!(repairable.state(), TransactionState::Active);
    repairable.discard().unwrap();
}

// AR-DOMAIN-011 through -014, -040 through -046, -056, -059, -063 through
// -065; MNIR-DOMAIN-015 through -024 and -065 through -081.
#[test]
fn allocation_removal_snapshot_and_fork_semantics_are_preserved() {
    let mut program = MnirProgram::new().unwrap();
    let namespace = program.allocation_namespace_id();
    let before = program.allocation_counter_state();
    let unknown_module = {
        let mut tx = program.begin_transaction();
        let id = tx.add_module().unwrap();
        tx.discard().unwrap();
        id
    };
    let after_gap = program.allocation_counter_state();
    let mut failed = program.begin_transaction();
    assert!(matches!(
        failed.add_domain_type(unknown_module, IntrinsicType::Text),
        Err(MutationError::UnknownModule(_))
    ));
    assert_eq!(failed.allocation_counter_state(), after_gap);
    failed.discard().unwrap();
    assert_ne!(before, after_gap);

    let mut tx = program.begin_transaction();
    let module_a = tx.add_module().unwrap();
    let module_b = tx.add_module().unwrap();
    let type_id = tx.add_domain_type(module_a, IntrinsicType::Int64).unwrap();
    let function = tx
        .add_function(module_b, ValueType::Domain(type_id))
        .unwrap();
    let block = tx.create_function_body(function).unwrap();
    let source = tx.add_int64_literal(block, 1).unwrap();
    let construct = tx.add_domain_construct(block, type_id, source).unwrap();
    tx.set_return(block, construct).unwrap();
    tx.commit().unwrap();

    let s1 = program.snapshot();
    let source_authority = (
        program.allocation_namespace_id(),
        program.allocation_counter_state(),
    );
    let mut fork = s1.fork().unwrap();
    assert_eq!(
        source_authority,
        (
            program.allocation_namespace_id(),
            program.allocation_counter_state()
        )
    );
    assert_ne!(fork.program_id(), program.program_id());
    assert_ne!(fork.allocation_namespace_id(), namespace);
    assert_eq!(fork.domain_type(type_id), s1.domain_type(type_id));
    assert_eq!(fork.expression(construct), s1.expression(construct));

    let fork_before = fork.allocation_counter_state();
    let mut source_tx = program.begin_transaction();
    let source_new = source_tx
        .add_domain_type(module_a, IntrinsicType::Text)
        .unwrap();
    source_tx.commit().unwrap();
    assert_eq!(source_new.namespace_id(), namespace);
    assert_eq!(fork.allocation_counter_state(), fork_before);
    let source_after = program.allocation_counter_state();
    let mut fork_tx = fork.begin_transaction();
    let fork_new = fork_tx
        .add_domain_type(module_a, IntrinsicType::Bytes)
        .unwrap();
    fork_tx.commit().unwrap();
    assert_eq!(fork_new.namespace_id(), fork.allocation_namespace_id());
    assert_eq!(program.allocation_counter_state(), source_after);

    let mut module_removal = program.begin_transaction();
    module_removal.remove_module(module_a).unwrap();
    assert!(module_removal.module(module_b).is_some());
    assert!(module_removal.function(function).is_some());
    assert!(matches!(
        module_removal.commit(),
        Err(MutationError::StructuralViolation(
            StructuralError::DanglingValueType(actual)
        )) if actual == type_id
    ));
    module_removal.discard().unwrap();

    let revision = program.revision_id();
    let mut dangling = program.begin_transaction();
    dangling.remove_domain_type(type_id).unwrap();
    assert_eq!(dangling.state(), TransactionState::Active);
    assert!(matches!(
        dangling.commit(),
        Err(MutationError::StructuralViolation(
            StructuralError::DanglingValueType(actual)
        )) if actual == type_id
    ));
    dangling.discard().unwrap();
    assert_eq!(program.revision_id(), revision);

    let mut repair = program.begin_transaction();
    repair.remove_domain_type(type_id).unwrap();
    repair.remove_function(function).unwrap();
    repair.commit().unwrap();
    assert!(program.domain_type(type_id).is_none());
    let mut next = program.begin_transaction();
    let later = next
        .add_domain_type(module_a, IntrinsicType::Int64)
        .unwrap();
    assert_ne!(later, type_id);
}

// AR-DOMAIN-029 and -076; MNIR-DOMAIN-044/-045 and inherited sequencing.
#[test]
fn domain_dependencies_participate_transitively_in_effect_order() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let domain = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let producer = tx.add_function(module, IntrinsicType::Int64).unwrap();
    let consumer = tx.add_function(module, IntrinsicType::Unit).unwrap();
    tx.add_parameter(consumer, IntrinsicType::Int64).unwrap();
    let caller = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(caller).unwrap();
    let first = tx.add_call_expression(block, producer, vec![]).unwrap();
    let construct = tx.add_domain_construct(block, domain, first).unwrap();
    let project = tx.add_domain_project(block, construct).unwrap();
    let second = tx
        .add_call_expression(block, consumer, vec![project])
        .unwrap();
    tx.set_effect_sequence(block, vec![first, second]).unwrap();
    let unit = tx.add_unit_literal(block).unwrap();
    tx.set_return(block, unit).unwrap();
    tx.commit().unwrap();

    let mut reversed = program.begin_transaction();
    reversed
        .set_effect_sequence(block, vec![second, first])
        .unwrap();
    assert!(matches!(
        reversed.commit(),
        Err(MutationError::StructuralViolation(
            StructuralError::EffectSequenceOrderConflict { .. }
        ))
    ));
}

// AR-DOMAIN-006/-007/-014/-015/-016/-040/-047/-061/-062/-064 and
// MNIR-DOMAIN-003 through -027, -065, -070 through -072.
#[test]
fn shared_identity_mutation_ownership_and_no_op_contracts_are_explicit() {
    let mut program = MnirProgram::new().unwrap();
    let namespace = program.allocation_namespace_id();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let domain = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let function = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let parameter = tx.add_parameter(function, IntrinsicType::Int64).unwrap();
    let block = tx.create_function_body(function).unwrap();
    let expression = tx.add_unit_literal(block).unwrap();
    tx.set_return(block, expression).unwrap();

    let counters = [
        module.counter(),
        domain.counter(),
        function.counter(),
        parameter.counter(),
        block.counter(),
        expression.counter(),
    ];
    assert!(counters.windows(2).all(|pair| pair[1] == pair[0] + 1));
    assert!(
        counters
            .into_iter()
            .all(|counter| counter <= expression.counter())
    );
    assert_eq!(domain.namespace_id(), namespace);

    tx.set_parameter_type(parameter, ValueType::Domain(domain))
        .unwrap();
    tx.set_function_return_type(function, ValueType::Domain(domain))
        .unwrap();
    assert_eq!(
        tx.function(function).unwrap().parameters()[0].id(),
        parameter
    );
    assert_eq!(tx.parameter(parameter).unwrap().id(), parameter);
    assert_eq!(tx.function(function).unwrap().id(), function);
    assert_eq!(
        *tx.parameter(parameter).unwrap().value_type(),
        ValueType::Domain(domain)
    );
    assert_eq!(
        *tx.function(function).unwrap().return_type(),
        ValueType::Domain(domain)
    );
    tx.commit().unwrap();

    let revision = program.revision_id();
    let semantic_count = program.module(module).unwrap().domain_type_count();
    let mut no_op = program.begin_transaction();
    let retired = no_op.add_domain_type(module, IntrinsicType::Bytes).unwrap();
    no_op.remove_domain_type(retired).unwrap();
    assert!(no_op.is_semantic_no_op());
    no_op.commit().unwrap();
    assert_ne!(program.revision_id(), revision);
    assert_eq!(
        program.module(module).unwrap().domain_type_count(),
        semantic_count
    );

    let mut remove = program.begin_transaction();
    let unreferenced = remove.add_domain_type(module, IntrinsicType::Text).unwrap();
    remove.commit().unwrap();
    let mut remove = program.begin_transaction();
    remove.remove_domain_type(unreferenced).unwrap();
    remove.commit().unwrap();
    assert!(program.domain_type(unreferenced).is_none());

    for operation in 0..3 {
        let counter = program.allocation_counter_state();
        let mut failed = program.begin_transaction();
        let result = match operation {
            0 => failed.set_domain_type_representation(retired, IntrinsicType::Bool),
            1 => failed.set_domain_type_preferred_name(retired, Some("unknown".into())),
            2 => failed.remove_domain_type(retired),
            _ => unreachable!(),
        };
        assert_eq!(result, Err(MutationError::UnknownType(retired)));
        assert_eq!(failed.state(), TransactionState::Failed);
        assert_eq!(failed.allocation_counter_state(), counter);
        failed.discard().unwrap();
    }

    let mut later = program.begin_transaction();
    let fresh = later.add_domain_type(module, IntrinsicType::Bytes).unwrap();
    assert!(fresh.counter() > retired.counter());
    assert!(fresh.counter() > unreferenced.counter());
}

// AR-DOMAIN-027/-029: both Domain dependency constructors validate ownership
// before reserving an ExpressionId and never sequence pure Expressions.
#[test]
fn domain_dependency_preconditions_do_not_reserve_expression_ids() {
    let mut program = MnirProgram::new().unwrap();
    let mut setup = program.begin_transaction();
    let module = setup.add_module().unwrap();
    let domain = setup.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let first_function = setup.add_function(module, IntrinsicType::Unit).unwrap();
    let first_block = setup.create_function_body(first_function).unwrap();
    let first_value = setup.add_int64_literal(first_block, 1).unwrap();
    let first_unit = setup.add_unit_literal(first_block).unwrap();
    setup.set_return(first_block, first_unit).unwrap();
    let second_function = setup.add_function(module, IntrinsicType::Unit).unwrap();
    let second_block = setup.create_function_body(second_function).unwrap();
    let second_unit = setup.add_unit_literal(second_block).unwrap();
    setup.set_return(second_block, second_unit).unwrap();
    setup.commit().unwrap();

    for construct in [true, false] {
        let before = program.allocation_counter_state();
        let mut failed = program.begin_transaction();
        let result = if construct {
            failed.add_domain_construct(second_block, domain, first_value)
        } else {
            failed.add_domain_project(second_block, first_value)
        };
        assert_eq!(
            result,
            Err(MutationError::ExpressionNotOwnedByBlock {
                expression_id: first_value,
                block_id: second_block,
            })
        );
        assert_eq!(failed.allocation_counter_state(), before);
        failed.discard().unwrap();
    }

    let unknown = first_value;
    let mut remove = program.begin_transaction();
    remove.remove_function(first_function).unwrap();
    remove.commit().unwrap();
    for construct in [true, false] {
        let before = program.allocation_counter_state();
        let mut failed = program.begin_transaction();
        let result = if construct {
            failed.add_domain_construct(second_block, domain, unknown)
        } else {
            failed.add_domain_project(second_block, unknown)
        };
        assert_eq!(result, Err(MutationError::UnknownExpression(unknown)));
        assert_eq!(failed.allocation_counter_state(), before);
        failed.discard().unwrap();
    }
}

// AR-DOMAIN-031 through -034 and -047. Names are presentation only; Text and
// Bytes equality is exact while ordering and nominal Domain comparison remain
// unsupported unless values are explicitly projected.
#[test]
fn comparison_support_is_explicit_and_names_add_no_semantics() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let mut named = Vec::new();
    for name in ["EmailAddress", "PasswordHash", "Secret", "Validated"] {
        let id = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
        tx.set_domain_type_preferred_name(id, Some(name.into()))
            .unwrap();
        named.push(id);
    }
    let function = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(function).unwrap();
    let text_left = tx.add_text_literal(block, "é").unwrap();
    let text_right = tx.add_text_literal(block, "é").unwrap();
    let bytes_left = tx.add_bytes_literal(block, vec![0, 255]).unwrap();
    let bytes_right = tx.add_bytes_literal(block, vec![0, 255]).unwrap();
    assert_ne!(text_left, text_right);
    assert_ne!(bytes_left, bytes_right);
    let text_equal = tx
        .add_equal_expression(block, text_left, text_right)
        .unwrap();
    let text_not_equal = tx
        .add_not_equal_expression(block, text_left, text_right)
        .unwrap();
    let bytes_equal = tx
        .add_equal_expression(block, bytes_left, bytes_right)
        .unwrap();
    let bytes_not_equal = tx
        .add_not_equal_expression(block, bytes_left, bytes_right)
        .unwrap();
    let text_order = tx
        .add_less_than_expression(block, text_left, text_right)
        .unwrap();
    let bytes_order = tx
        .add_less_than_expression(block, bytes_left, bytes_right)
        .unwrap();
    let source = tx.add_int64_literal(block, 1).unwrap();
    let domain_left = tx.add_domain_construct(block, named[0], source).unwrap();
    let domain_right = tx.add_domain_construct(block, named[0], source).unwrap();
    let domain_order = tx
        .add_less_than_expression(block, domain_left, domain_right)
        .unwrap();
    let projected_left = tx.add_domain_project(block, domain_left).unwrap();
    let projected_right = tx.add_domain_project(block, domain_right).unwrap();
    let projected_compare = tx
        .add_greater_than_or_equal_expression(block, projected_left, projected_right)
        .unwrap();
    let unit = tx.add_unit_literal(block).unwrap();
    tx.set_return(block, unit).unwrap();

    assert_eq!(
        tx.expression_type(text_equal),
        Some(Ok(IntrinsicType::Bool.into()))
    );
    assert_eq!(
        tx.expression_type(text_not_equal),
        Some(Ok(ValueType::Intrinsic(IntrinsicType::Bool)))
    );
    assert_eq!(
        tx.expression_type(bytes_equal),
        Some(Ok(ValueType::Intrinsic(IntrinsicType::Bool)))
    );
    assert_eq!(
        tx.expression_type(bytes_not_equal),
        Some(Ok(ValueType::Intrinsic(IntrinsicType::Bool)))
    );
    assert!(matches!(
        tx.expression_type(text_order),
        Some(Err(ExpressionTypeError::UnsupportedOperandType {
            operand_type: ValueType::Intrinsic(IntrinsicType::Text),
            ..
        }))
    ));
    assert!(matches!(
        tx.expression_type(bytes_order),
        Some(Err(ExpressionTypeError::UnsupportedOperandType {
            operand_type: ValueType::Intrinsic(IntrinsicType::Bytes),
            ..
        }))
    ));
    assert!(matches!(
        tx.expression_type(domain_order),
        Some(Err(ExpressionTypeError::UnsupportedOperandType {
            operand_type: ValueType::Domain(actual),
            ..
        })) if actual == named[0]
    ));
    assert_eq!(
        tx.expression_type(projected_compare),
        Some(Ok(IntrinsicType::Bool.into()))
    );
    for pair in named.windows(2) {
        assert_ne!(ValueType::Domain(pair[0]), ValueType::Domain(pair[1]));
    }
}

// AR-DOMAIN-043/-065: enclosing ownership cascades are the only removal
// mechanism needed for the new Expression kinds and retired IDs stay retired.
#[test]
fn domain_expression_cascades_remove_all_new_kinds_without_id_reuse() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let domain = tx.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let function = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let block = tx.create_function_body(function).unwrap();
    let removed_expressions = add_all_domain_expression_kinds(&mut tx, block, domain);
    let unit = tx.add_unit_literal(block).unwrap();
    tx.set_return(block, unit).unwrap();
    let unrelated_function = tx.add_function(module, IntrinsicType::Unit).unwrap();
    let unrelated_block = tx.create_function_body(unrelated_function).unwrap();
    let unrelated_expression = tx.add_unit_literal(unrelated_block).unwrap();
    tx.set_return(unrelated_block, unrelated_expression)
        .unwrap();
    tx.commit().unwrap();

    let mut remove = program.begin_transaction();
    remove.remove_domain_type(domain).unwrap();
    remove.remove_function(function).unwrap();
    remove.commit().unwrap();
    for id in removed_expressions {
        assert!(program.expression(id).is_none());
    }
    assert!(program.function(unrelated_function).is_some());
    assert!(program.expression(unrelated_expression).is_some());
    let mut later = program.begin_transaction();
    let later_function = later.add_function(module, IntrinsicType::Unit).unwrap();
    let later_block = later.create_function_body(later_function).unwrap();
    let later_expression = later.add_text_literal(later_block, "x").unwrap();
    for retired in removed_expressions {
        assert_ne!(later_expression, retired);
        assert!(later_expression.counter() > retired.counter());
    }
}

// AR-DOMAIN-043/-065: a dangling DomainConstruct can be repaired by removing
// its non-entry Block and repairing every incoming CFG edge.
#[test]
fn non_entry_block_cascade_removes_domain_expressions_with_cfg_repair() {
    let mut program = MnirProgram::new().unwrap();
    let mut setup = program.begin_transaction();
    let module = setup.add_module().unwrap();
    let domain = setup.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let function = setup.add_function(module, IntrinsicType::Unit).unwrap();
    let entry = setup.create_function_body(function).unwrap();
    let removed_block = setup.add_block(function).unwrap();
    let surviving_block = setup.add_block(function).unwrap();
    let condition = setup.add_bool_literal(entry, true).unwrap();
    setup
        .set_branch(entry, condition, removed_block, surviving_block)
        .unwrap();
    let removed_expressions = add_all_domain_expression_kinds(&mut setup, removed_block, domain);
    let removed_return = setup.add_unit_literal(removed_block).unwrap();
    setup.set_return(removed_block, removed_return).unwrap();
    let surviving_expression = setup.add_unit_literal(surviving_block).unwrap();
    setup
        .set_return(surviving_block, surviving_expression)
        .unwrap();
    setup.commit().unwrap();

    let mut repair = program.begin_transaction();
    repair.remove_domain_type(domain).unwrap();
    repair.remove_block(removed_block).unwrap();
    repair
        .set_branch(entry, condition, surviving_block, surviving_block)
        .unwrap();
    repair.commit().unwrap();

    assert!(program.block(removed_block).is_none());
    for expression in removed_expressions {
        assert!(program.expression(expression).is_none());
    }
    assert!(program.block(entry).is_some());
    assert!(program.block(surviving_block).is_some());
    assert!(program.expression(surviving_expression).is_some());
    assert!(program.validate_structure().is_ok());

    let mut later = program.begin_transaction();
    let fresh = later.add_text_literal(surviving_block, "later").unwrap();
    for retired in removed_expressions {
        assert_ne!(fresh, retired);
        assert!(fresh.counter() > retired.counter());
    }
    later.commit().unwrap();
}

// AR-DOMAIN-043/-065: removing a Function body cascades all new Expression
// kinds while leaving the Function and unrelated entities intact.
#[test]
fn function_body_cascade_removes_domain_expressions_without_reuse() {
    let mut program = MnirProgram::new().unwrap();
    let mut setup = program.begin_transaction();
    let module = setup.add_module().unwrap();
    let domain = setup.add_domain_type(module, IntrinsicType::Int64).unwrap();
    let function = setup.add_function(module, IntrinsicType::Unit).unwrap();
    let block = setup.create_function_body(function).unwrap();
    let removed_expressions = add_all_domain_expression_kinds(&mut setup, block, domain);
    let returned = setup.add_unit_literal(block).unwrap();
    setup.set_return(block, returned).unwrap();
    let unrelated_function = setup.add_function(module, IntrinsicType::Unit).unwrap();
    let unrelated_block = setup.create_function_body(unrelated_function).unwrap();
    let unrelated_expression = setup.add_unit_literal(unrelated_block).unwrap();
    setup
        .set_return(unrelated_block, unrelated_expression)
        .unwrap();
    setup.commit().unwrap();

    let mut repair = program.begin_transaction();
    repair.remove_domain_type(domain).unwrap();
    repair.remove_function_body(function).unwrap();
    repair.commit().unwrap();

    assert!(!program.function(function).unwrap().has_body());
    for expression in removed_expressions {
        assert!(program.expression(expression).is_none());
    }
    assert!(program.function(unrelated_function).is_some());
    assert!(program.expression(unrelated_expression).is_some());
    assert!(program.validate_structure().is_ok());

    let mut later = program.begin_transaction();
    let fresh_block = later.create_function_body(function).unwrap();
    let fresh = later.add_text_literal(fresh_block, "later").unwrap();
    let fresh_return = later.add_unit_literal(fresh_block).unwrap();
    later.set_return(fresh_block, fresh_return).unwrap();
    for retired in removed_expressions {
        assert_ne!(fresh, retired);
        assert!(fresh.counter() > retired.counter());
    }
    later.commit().unwrap();
}

// AR-DOMAIN-065: Module removal owns the complete Domain Expression cascade
// and does not disturb entities in an unrelated Module.
#[test]
fn module_cascade_removes_domain_expressions_without_reuse() {
    let mut program = MnirProgram::new().unwrap();
    let mut setup = program.begin_transaction();
    let removed_module = setup.add_module().unwrap();
    let domain = setup
        .add_domain_type(removed_module, IntrinsicType::Int64)
        .unwrap();
    let removed_function = setup
        .add_function(removed_module, IntrinsicType::Unit)
        .unwrap();
    let removed_block = setup.create_function_body(removed_function).unwrap();
    let removed_expressions = add_all_domain_expression_kinds(&mut setup, removed_block, domain);
    let removed_return = setup.add_unit_literal(removed_block).unwrap();
    setup.set_return(removed_block, removed_return).unwrap();

    let surviving_module = setup.add_module().unwrap();
    let surviving_function = setup
        .add_function(surviving_module, IntrinsicType::Unit)
        .unwrap();
    let surviving_block = setup.create_function_body(surviving_function).unwrap();
    let surviving_expression = setup.add_unit_literal(surviving_block).unwrap();
    setup
        .set_return(surviving_block, surviving_expression)
        .unwrap();
    setup.commit().unwrap();

    let mut remove = program.begin_transaction();
    remove.remove_module(removed_module).unwrap();
    remove.commit().unwrap();

    assert!(program.module(removed_module).is_none());
    for expression in removed_expressions {
        assert!(program.expression(expression).is_none());
    }
    assert!(program.module(surviving_module).is_some());
    assert!(program.function(surviving_function).is_some());
    assert!(program.expression(surviving_expression).is_some());
    assert!(program.validate_structure().is_ok());

    let mut later = program.begin_transaction();
    let fresh = later.add_text_literal(surviving_block, "later").unwrap();
    for retired in removed_expressions {
        assert_ne!(fresh, retired);
        assert!(fresh.counter() > retired.counter());
    }
    later.commit().unwrap();
}

// AR-DOMAIN-045/-046/-063 and MNIR-DOMAIN-079/-080/-112: immutable
// snapshots and normal forks preserve the complete Domain-rich graph without
// rewriting identity or references.
#[test]
fn snapshots_and_forks_preserve_complete_domain_graphs_exactly() {
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let types_module = tx.add_module().unwrap();
    let code_module = tx.add_module().unwrap();
    let domain = tx
        .add_domain_type(types_module, IntrinsicType::Int64)
        .unwrap();
    let code_local_domain = tx
        .add_domain_type(code_module, IntrinsicType::Int64)
        .unwrap();
    tx.set_domain_type_preferred_name(domain, Some("CustomerId".into()))
        .unwrap();
    tx.set_domain_type_documentation(domain, Some("stable identity".into()))
        .unwrap();
    let target = tx
        .add_function(code_module, ValueType::Domain(domain))
        .unwrap();
    tx.add_parameter(target, ValueType::Domain(domain)).unwrap();
    let caller = tx
        .add_function(code_module, ValueType::Domain(domain))
        .unwrap();
    let entry = tx.create_function_body(caller).unwrap();
    let exit = tx.add_block(caller).unwrap();
    let text = tx.add_text_literal(entry, "å").unwrap();
    let bytes = tx.add_bytes_literal(entry, vec![0, 255]).unwrap();
    let source = tx.add_int64_literal(entry, 1).unwrap();
    let construct = tx.add_domain_construct(entry, domain, source).unwrap();
    let project = tx.add_domain_project(entry, construct).unwrap();
    let comparison = tx.add_equal_expression(entry, project, source).unwrap();
    let call = tx
        .add_call_expression(entry, target, vec![construct])
        .unwrap();
    tx.set_effect_sequence(entry, vec![call]).unwrap();
    tx.set_branch(entry, comparison, exit, exit).unwrap();
    let exit_source = tx.add_int64_literal(exit, 2).unwrap();
    let returned = tx.add_domain_construct(exit, domain, exit_source).unwrap();
    tx.set_return(exit, returned).unwrap();
    let snapshot = tx.commit().unwrap();
    let snapshot_authority = (
        snapshot.allocation_namespace_id(),
        snapshot.allocation_counter_state(),
    );

    let fork = snapshot.fork().unwrap();
    assert_ne!(fork.program_id(), snapshot.program_id());
    assert_eq!(fork.domain_type(domain), snapshot.domain_type(domain));
    assert_eq!(
        fork.domain_type(code_local_domain),
        snapshot.domain_type(code_local_domain)
    );
    assert!(
        snapshot
            .module(types_module)
            .unwrap()
            .domain_type(domain)
            .is_some()
    );
    assert!(
        snapshot
            .module(code_module)
            .unwrap()
            .domain_type(code_local_domain)
            .is_some()
    );
    assert_eq!(fork.function(target), snapshot.function(target));
    assert_eq!(fork.function(caller), snapshot.function(caller));
    for expression in [text, bytes, construct, project, comparison, call, returned] {
        assert_eq!(fork.expression(expression), snapshot.expression(expression));
    }
    assert_eq!(fork.block(entry), snapshot.block(entry));
    assert_eq!(fork.block(exit), snapshot.block(exit));
    assert_eq!(fork.block(entry).unwrap().effect_sequence(), &[call]);
    assert!(matches!(
        fork.block(entry).unwrap().terminator(),
        Some(mnir_core::Terminator::Branch {
            condition,
            true_block,
            false_block,
        }) if *condition == comparison && *true_block == exit && *false_block == exit
    ));

    let mut mutation = program.begin_transaction();
    mutation
        .set_domain_type_representation(domain, IntrinsicType::Text)
        .unwrap();
    mutation
        .set_domain_type_preferred_name(domain, Some("Renamed".into()))
        .unwrap();
    mutation.commit().unwrap();
    assert_eq!(
        snapshot.domain_type(domain).unwrap().representation(),
        IntrinsicType::Int64
    );
    assert_eq!(
        snapshot
            .domain_type(domain)
            .unwrap()
            .presentation()
            .preferred_name(),
        Some("CustomerId")
    );
    assert_eq!(
        (
            snapshot.allocation_namespace_id(),
            snapshot.allocation_counter_state()
        ),
        snapshot_authority
    );
    assert!(matches!(
        snapshot.expression(text).unwrap().kind(),
        ExpressionKind::TextLiteral(value) if value == "å"
    ));
    assert!(matches!(
        snapshot.expression(bytes).unwrap().kind(),
        ExpressionKind::BytesLiteral(value) if value == &[0, 255]
    ));
}

// AR-DOMAIN-062: the representation carrier is the same closed six-member
// IntrinsicType enum; no Domain, recursive, missing, or open representation
// can be supplied through the safe API.
#[test]
fn every_and_only_intrinsic_type_can_represent_a_domain_type() {
    let representations = [
        IntrinsicType::Int32,
        IntrinsicType::Int64,
        IntrinsicType::Bool,
        IntrinsicType::Unit,
        IntrinsicType::Text,
        IntrinsicType::Bytes,
    ];
    let mut program = MnirProgram::new().unwrap();
    let mut tx = program.begin_transaction();
    let module = tx.add_module().unwrap();
    let ids: Vec<_> = representations
        .into_iter()
        .map(|representation| tx.add_domain_type(module, representation).unwrap())
        .collect();
    tx.commit().unwrap();

    let observed: HashSet<_> = ids
        .into_iter()
        .map(|id| program.domain_type(id).unwrap().representation())
        .collect();
    assert_eq!(observed, HashSet::from(representations));
}
