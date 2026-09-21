use mnir_core::{
    AllocationCounterState, ExpressionKind, IntrinsicType, MnirProgram, PersistenceBlock,
    PersistenceBody, PersistenceDomainType, PersistenceEntityId, PersistenceExpression,
    PersistenceExpressionKind, PersistenceFunction, PersistenceModule, PersistencePresentation,
    PersistenceProgram, PersistenceTerminator, PersistenceValueType, RevisionCounterState,
    Terminator, ValueType,
};
use mnir_format::{DecodeError, decode, encode};
use mnir_verify::{DiagnosticCode, VerificationError, VerificationRuleSet, verify_with_rule_set};

fn encode_test_id(encoder: &mut minicbor::Encoder<Vec<u8>>, namespace: [u8; 16], counter: u64) {
    encoder.array(2).unwrap();
    encoder.bytes(&namespace).unwrap();
    encoder.u64(counter).unwrap();
}

fn encode_absent_presentation(encoder: &mut minicbor::Encoder<Vec<u8>>) {
    encoder.array(2).unwrap();
    encoder.array(1).unwrap().u8(0).unwrap();
    encoder.array(1).unwrap().u8(0).unwrap();
}

fn manual_file(
    allocation_next: u64,
    encode_modules: impl FnOnce(&mut minicbor::Encoder<Vec<u8>>),
) -> Vec<u8> {
    let mut encoder = minicbor::Encoder::new(Vec::new());
    encoder.array(5).unwrap();
    encoder.bytes(&[0x31; 16]).unwrap();
    encoder.u8(1).unwrap();
    encoder.array(2).unwrap().u8(0).unwrap().u8(2).unwrap();
    encoder.array(2).unwrap().bytes(&[0x32; 16]).unwrap();
    encoder
        .array(2)
        .unwrap()
        .u8(0)
        .unwrap()
        .u64(allocation_next)
        .unwrap();
    encode_modules(&mut encoder);
    file(&encoder.into_writer())
}

#[derive(Clone, Copy)]
enum WireExpression {
    Unit,
    Bool,
    Add { left: u64, right: u64 },
    Call { target: u64, argument: Option<u64> },
}

#[derive(Clone, Copy)]
enum WireTerminator {
    Return(u64),
    Branch {
        condition: u64,
        true_block: u64,
        false_block: u64,
    },
}

struct WireBlock {
    id: u64,
    expressions: Vec<(u64, WireExpression)>,
    effect_sequence: Vec<u64>,
    terminator: WireTerminator,
}

struct WireFunction {
    id: u64,
    domain_return: Option<u64>,
    entry: u64,
    blocks: Vec<WireBlock>,
}

fn encode_wire_expression(encoder: &mut minicbor::Encoder<Vec<u8>>, expression: WireExpression) {
    match expression {
        WireExpression::Unit => {
            encoder.array(1).unwrap().u8(3).unwrap();
        }
        WireExpression::Bool => {
            encoder.array(2).unwrap().u8(2).unwrap().bool(true).unwrap();
        }
        WireExpression::Add { left, right } => {
            encoder.array(3).unwrap().u8(7).unwrap();
            encode_test_id(encoder, [0x32; 16], left);
            encode_test_id(encoder, [0x32; 16], right);
        }
        WireExpression::Call { target, argument } => {
            encoder.array(3).unwrap().u8(18).unwrap();
            encode_test_id(encoder, [0x32; 16], target);
            encoder.array(u64::from(argument.is_some())).unwrap();
            if let Some(argument) = argument {
                encode_test_id(encoder, [0x32; 16], argument);
            }
        }
    }
}

fn encode_wire_terminator(encoder: &mut minicbor::Encoder<Vec<u8>>, terminator: WireTerminator) {
    match terminator {
        WireTerminator::Return(expression) => {
            encoder.array(2).unwrap().u8(0).unwrap();
            encode_test_id(encoder, [0x32; 16], expression);
        }
        WireTerminator::Branch {
            condition,
            true_block,
            false_block,
        } => {
            encoder.array(4).unwrap().u8(1).unwrap();
            encode_test_id(encoder, [0x32; 16], condition);
            encode_test_id(encoder, [0x32; 16], true_block);
            encode_test_id(encoder, [0x32; 16], false_block);
        }
    }
}

fn structural_file(functions: Vec<WireFunction>) -> Vec<u8> {
    manual_file(200, |encoder| {
        encoder.array(1).unwrap();
        encoder.array(4).unwrap();
        encode_test_id(encoder, [0x32; 16], 1);
        encode_absent_presentation(encoder);
        encoder.array(0).unwrap();
        encoder.array(functions.len() as u64).unwrap();
        for function in functions {
            encoder.array(5).unwrap();
            encode_test_id(encoder, [0x32; 16], function.id);
            encoder.array(2).unwrap();
            if let Some(type_id) = function.domain_return {
                encoder.u8(1).unwrap();
                encode_test_id(encoder, [0x32; 16], type_id);
            } else {
                encoder.u8(0).unwrap().u8(3).unwrap();
            }
            encoder.array(0).unwrap();
            encoder.array(2).unwrap().u8(1).unwrap();
            encoder.array(2).unwrap();
            encode_test_id(encoder, [0x32; 16], function.entry);
            encoder.array(function.blocks.len() as u64).unwrap();
            for block in function.blocks {
                encoder.array(4).unwrap();
                encode_test_id(encoder, [0x32; 16], block.id);
                encoder.array(block.expressions.len() as u64).unwrap();
                for (id, expression) in block.expressions {
                    encoder.array(2).unwrap();
                    encode_test_id(encoder, [0x32; 16], id);
                    encode_wire_expression(encoder, expression);
                }
                encoder.array(block.effect_sequence.len() as u64).unwrap();
                for expression in block.effect_sequence {
                    encode_test_id(encoder, [0x32; 16], expression);
                }
                encode_wire_terminator(encoder, block.terminator);
            }
            encode_absent_presentation(encoder);
        }
    })
}

fn return_block(id: u64, expression: u64) -> WireBlock {
    WireBlock {
        id,
        expressions: vec![(expression, WireExpression::Unit)],
        effect_sequence: Vec::new(),
        terminator: WireTerminator::Return(expression),
    }
}

fn minimal_golden() -> Vec<u8> {
    let hex = include_str!("fixtures/minimal-v1.hex").trim();
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(text, 16).unwrap()
        })
        .collect()
}

// AR-SER-001, AR-SER-005, AR-SER-037: the normative minimal fixture is
// decoded field-for-field and re-encoded byte-identically.
#[test]
fn minimal_normative_golden_round_trips_exactly() {
    let bytes = minimal_golden();
    assert_eq!(bytes.len(), 56);
    let program = decode(&bytes).unwrap().activate().unwrap();
    let checkpoint = program.persistence_snapshot();

    assert_eq!(
        checkpoint.program_id().persistent_bytes(),
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    );
    assert_eq!(checkpoint.revision_id().persistent_value(), 1);
    assert_eq!(
        checkpoint.revision_counter_state(),
        RevisionCounterState::Available(2)
    );
    assert_eq!(
        checkpoint.allocation_namespace_id().persistent_bytes(),
        [
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
            0x1e, 0x1f,
        ]
    );
    assert_eq!(
        checkpoint.allocation_counter_state(),
        AllocationCounterState::Available(1)
    );
    assert_eq!(checkpoint.modules().count(), 0);
    program.validate_structure().unwrap();
    assert_eq!(encode(&checkpoint).unwrap(), bytes);
}

fn file(payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x89, b'M', b'N', b'I', b'R', 0x0d, 0x0a, 0x1a, 0, 1, 0, 0];
    bytes.extend_from_slice(payload);
    bytes
}

// AR-SER-002: envelope syntax precedes version and payload validation.
#[test]
fn envelope_and_version_failures_are_classified() {
    assert_eq!(decode(&[]).unwrap_err(), DecodeError::MalformedEncoding);
    assert_eq!(
        decode(&[0x89, b'M', b'N']).unwrap_err(),
        DecodeError::MalformedEncoding
    );
    let mut wrong_magic = minimal_golden();
    wrong_magic[1] = b'X';
    assert_eq!(
        decode(&wrong_magic).unwrap_err(),
        DecodeError::MalformedEncoding
    );
    let mut wrong_major = minimal_golden();
    wrong_major[9] = 2;
    assert_eq!(
        decode(&wrong_major).unwrap_err(),
        DecodeError::UnsupportedFormatVersion
    );
    let mut wrong_minor = minimal_golden();
    wrong_minor[11] = 1;
    assert_eq!(
        decode(&wrong_minor).unwrap_err(),
        DecodeError::UnsupportedFormatVersion
    );
    assert_eq!(
        decode(&minimal_golden()[..12]).unwrap_err(),
        DecodeError::MalformedEncoding
    );
    let mut trailing = minimal_golden();
    trailing.push(0);
    assert_eq!(
        decode(&trailing).unwrap_err(),
        DecodeError::MalformedEncoding
    );
}

// AR-SER-003, AR-SER-004: generic CBOR outside the restricted profile is not
// accepted, while well-formed profile data with the wrong schema is distinct.
#[test]
fn canonical_profile_and_wire_schema_are_distinct() {
    for payload in [
        &[0x18, 0x00][..], // non-shortest integer
        &[0x38, 0x00][..], // non-shortest negative integer
        &[0x9f, 0xff][..], // indefinite array
        &[0x5f, 0xff][..], // indefinite byte string
        &[0x7f, 0xff][..], // indefinite text string
        &[0xa0][..],       // map
        &[0xc0, 0x00][..], // semantic tag
        &[0xf9, 0, 0][..], // float
        &[0xf6][..],       // null
        &[0xf7][..],       // undefined
        &[0xf0][..],       // disallowed simple value
    ] {
        assert_eq!(
            decode(&file(payload)).unwrap_err(),
            DecodeError::NonCanonicalEncoding
        );
    }
    assert_eq!(
        decode(&file(&[0x63, 0xff, 0xff, 0xff])).unwrap_err(),
        DecodeError::MalformedEncoding
    );
    assert_eq!(
        decode(&file(&[0x85])).unwrap_err(),
        DecodeError::MalformedEncoding
    );
    assert_eq!(
        decode(&file(&[0x80])).unwrap_err(),
        DecodeError::InvalidWireSchema
    );
}

// AR-SER-006 through AR-SER-019, AR-SER-033, AR-SER-037: one representative
// checkpoint covers every current value/type/expression family plus semantic
// sequence and Branch-role preservation. Exact re-encoding is the persisted
// state equality oracle.
#[test]
fn comprehensive_checkpoint_round_trips_byte_identically() {
    let mut program = MnirProgram::new().unwrap();
    let (module_id, function_id, entry, true_block, false_block, call_two, call_one) = {
        let mut tx = program.begin_transaction();
        let module_id = tx.add_module().unwrap();
        tx.set_module_preferred_name(module_id, Some("modulé".into()))
            .unwrap();
        tx.set_module_documentation(module_id, Some(String::new()))
            .unwrap();

        let domain = tx.add_domain_type(module_id, IntrinsicType::Int32).unwrap();
        tx.set_domain_type_preferred_name(domain, Some("Meters".into()))
            .unwrap();
        for representation in [
            IntrinsicType::Int64,
            IntrinsicType::Bool,
            IntrinsicType::Unit,
            IntrinsicType::Text,
            IntrinsicType::Bytes,
        ] {
            tx.add_domain_type(module_id, representation).unwrap();
        }

        for value_type in [
            IntrinsicType::Int32,
            IntrinsicType::Int64,
            IntrinsicType::Bool,
            IntrinsicType::Unit,
            IntrinsicType::Text,
            IntrinsicType::Bytes,
        ] {
            let typed = tx.add_function(module_id, value_type).unwrap();
            tx.add_parameter(typed, value_type).unwrap();
        }

        let other_module = tx.add_module().unwrap();
        let cross_module = tx
            .add_function(other_module, ValueType::Domain(domain))
            .unwrap();
        tx.add_parameter(cross_module, ValueType::Domain(domain))
            .unwrap();

        let callee = tx.add_function(module_id, IntrinsicType::Unit).unwrap();
        let single_block = tx.add_function(module_id, IntrinsicType::Unit).unwrap();
        let single_entry = tx.create_function_body(single_block).unwrap();
        let single_value = tx.add_unit_literal(single_entry).unwrap();
        tx.set_return(single_entry, single_value).unwrap();
        let function_id = tx
            .add_function(module_id, ValueType::Domain(domain))
            .unwrap();
        tx.set_function_preferred_name(function_id, Some("f".into()))
            .unwrap();
        let parameter = tx.add_parameter(function_id, IntrinsicType::Int32).unwrap();
        tx.set_parameter_preferred_name(parameter, Some("x".into()))
            .unwrap();
        let second_parameter = tx
            .add_parameter(function_id, ValueType::Domain(domain))
            .unwrap();
        tx.set_parameter_preferred_name(second_parameter, Some("x".into()))
            .unwrap();

        let entry = tx.create_function_body(function_id).unwrap();
        let false_block = tx.add_block(function_id).unwrap();
        let true_block = tx.add_block(function_id).unwrap();

        let i32_a = tx.add_int32_literal(entry, i32::MIN).unwrap();
        let i32_b = tx.add_int32_literal(entry, i32::MAX).unwrap();
        tx.add_int64_literal(entry, i64::MIN).unwrap();
        tx.add_int64_literal(entry, i64::MAX).unwrap();
        tx.add_int64_literal(entry, -1).unwrap();
        tx.add_int64_literal(entry, 0).unwrap();
        let condition = tx.add_bool_literal(entry, true).unwrap();
        tx.add_bool_literal(entry, false).unwrap();
        tx.add_unit_literal(entry).unwrap();
        tx.add_text_literal(entry, "Aé").unwrap();
        tx.add_text_literal(entry, "e\u{301}").unwrap();
        tx.add_text_literal(entry, "").unwrap();
        tx.add_bytes_literal(entry, vec![0, 0xff]).unwrap();
        tx.add_bytes_literal(entry, Vec::new()).unwrap();
        tx.add_parameter_reference(entry, parameter).unwrap();
        let constructed = tx.add_domain_construct(entry, domain, i32_a).unwrap();
        tx.add_domain_project(entry, constructed).unwrap();
        tx.add_add_expression(entry, i32_a, i32_b).unwrap();
        tx.add_subtract_expression(entry, i32_a, i32_b).unwrap();
        tx.add_multiply_expression(entry, i32_a, i32_b).unwrap();
        tx.add_divide_expression(entry, i32_a, i32_b).unwrap();
        tx.add_remainder_expression(entry, i32_a, i32_b).unwrap();
        tx.add_equal_expression(entry, i32_a, i32_b).unwrap();
        tx.add_not_equal_expression(entry, i32_a, i32_b).unwrap();
        tx.add_less_than_expression(entry, i32_a, i32_b).unwrap();
        tx.add_less_than_or_equal_expression(entry, i32_a, i32_b)
            .unwrap();
        tx.add_greater_than_expression(entry, i32_a, i32_b).unwrap();
        tx.add_greater_than_or_equal_expression(entry, i32_a, i32_b)
            .unwrap();
        let call_one = tx.add_call_expression(entry, callee, vec![]).unwrap();
        let call_two = tx
            .add_call_expression(entry, callee, vec![i32_b, i32_a, i32_b])
            .unwrap();
        tx.set_effect_sequence(entry, vec![call_two, call_one])
            .unwrap();
        tx.set_branch(entry, condition, true_block, false_block)
            .unwrap();

        let true_literal = tx.add_int32_literal(true_block, -1).unwrap();
        let true_value = tx
            .add_domain_construct(true_block, domain, true_literal)
            .unwrap();
        tx.set_return(true_block, true_value).unwrap();
        let false_value = tx.add_int32_literal(false_block, 0).unwrap();
        let false_domain = tx
            .add_domain_construct(false_block, domain, false_value)
            .unwrap();
        tx.set_return(false_block, false_domain).unwrap();
        tx.commit().unwrap();
        (
            module_id,
            function_id,
            entry,
            true_block,
            false_block,
            call_two,
            call_one,
        )
    };

    let bytes = encode(&program.persistence_snapshot()).unwrap();
    let source_program_id = program.program_id();
    let source_revision_id = program.revision_id();
    drop(program);
    let restored = decode(&bytes).unwrap().activate().unwrap();
    assert_eq!(encode(&restored.persistence_snapshot()).unwrap(), bytes);
    assert_eq!(restored.program_id(), source_program_id);
    assert_eq!(restored.revision_id(), source_revision_id);
    let module = restored.module(module_id).unwrap();
    assert_eq!(module.presentation().preferred_name(), Some("modulé"));
    let body = restored.function(function_id).unwrap().body().unwrap();
    assert_eq!(body.entry_block_id(), entry);
    assert_eq!(body.block_count(), 3);
    assert!(false_block.counter() < true_block.counter());
    assert_eq!(
        body.block_by_id(entry).unwrap().effect_sequence(),
        &[call_two, call_one]
    );
    assert_eq!(
        body.block_by_id(entry).unwrap().terminator(),
        Some(&Terminator::Branch {
            condition: match body.block_by_id(entry).unwrap().terminator().unwrap() {
                Terminator::Branch { condition, .. } => *condition,
                _ => unreachable!(),
            },
            true_block,
            false_block
        })
    );
    assert!(
        body.blocks()
            .flat_map(|block| block.expressions())
            .any(|expression| matches!(expression.kind(), ExpressionKind::DomainProject { .. }))
    );
    restored.validate_structure().unwrap();

    let mut fork = restored.snapshot().fork().unwrap();
    let inherited_namespace = module_id.namespace_id();
    let fork_namespace = fork.allocation_namespace_id();
    let fork_minted = {
        let mut tx = fork.begin_transaction();
        let id = tx.add_module().unwrap();
        tx.commit().unwrap();
        id
    };
    assert_eq!(
        fork.module(module_id).unwrap().id().namespace_id(),
        inherited_namespace
    );
    assert_eq!(fork_minted.namespace_id(), fork_namespace);
    let fork_bytes = encode(&fork.persistence_snapshot()).unwrap();
    drop(fork);
    let restored_fork = decode(&fork_bytes).unwrap().activate().unwrap();
    assert_eq!(
        encode(&restored_fork.persistence_snapshot()).unwrap(),
        fork_bytes
    );
    assert_eq!(
        restored_fork
            .function(function_id)
            .unwrap()
            .body()
            .unwrap()
            .entry_block_id(),
        entry
    );
}

// AR-SER-020, AR-SER-023, AR-SER-024, AR-SER-035: restored authorities issue
// exactly their persisted next values, and observations are immutable.
#[test]
fn allocator_and_revision_continuations_resume_without_remapping() {
    let old = snapshot_from_candidate(detached_program_with_tag(
        0x84,
        41,
        RevisionCounterState::Available(42),
        AllocationCounterState::Available(7),
        Vec::new(),
    ));
    let old_bytes = encode(&old).unwrap();
    let expected_counter = old.allocation_counter_state();
    let expected_revision = old.revision_counter_state();

    let mut restored = decode(&old_bytes).unwrap().activate().unwrap();
    let mut tx = restored.begin_transaction();
    let module_id = tx.add_module().unwrap();
    assert_eq!(module_id.namespace_id(), old.allocation_namespace_id());
    assert_eq!(
        Some(module_id.counter()),
        match expected_counter {
            AllocationCounterState::Available(value) => Some(value),
            AllocationCounterState::Exhausted => None,
        }
    );
    let committed = tx.commit().unwrap();
    assert_eq!(
        Some(committed.revision_id().persistent_value()),
        match expected_revision {
            RevisionCounterState::Available(value) => Some(value),
            RevisionCounterState::Exhausted => None,
        }
    );
    assert_eq!(encode(&old).unwrap(), old_bytes);
    assert_eq!(restored.revision_id().persistent_value(), 42);
    assert_eq!(
        restored.persistence_snapshot().revision_counter_state(),
        RevisionCounterState::Available(43)
    );
}

fn absent_presentation() -> PersistencePresentation {
    PersistencePresentation::new(None, None)
}

fn wire_id(namespace: [u8; 16], counter: u64) -> PersistenceEntityId {
    PersistenceEntityId::new(namespace, counter)
}

fn detached_program(
    revision: u64,
    revision_counter_state: RevisionCounterState,
    allocation_counter_state: AllocationCounterState,
    modules: Vec<PersistenceModule>,
) -> PersistenceProgram {
    detached_program_with_tag(
        0x41,
        revision,
        revision_counter_state,
        allocation_counter_state,
        modules,
    )
}

fn detached_program_with_tag(
    program_tag: u8,
    revision: u64,
    revision_counter_state: RevisionCounterState,
    allocation_counter_state: AllocationCounterState,
    modules: Vec<PersistenceModule>,
) -> PersistenceProgram {
    PersistenceProgram::new(
        [program_tag; 16],
        revision,
        revision_counter_state,
        [program_tag.wrapping_add(1); 16],
        allocation_counter_state,
        modules,
    )
}

fn snapshot_from_candidate(candidate: PersistenceProgram) -> mnir_core::PersistenceSnapshot {
    MnirProgram::validate_persistence(candidate)
        .unwrap()
        .persistence_snapshot()
}

fn activate_candidate(candidate: PersistenceProgram) -> MnirProgram {
    MnirProgram::activate_persistence(MnirProgram::validate_persistence(candidate).unwrap())
        .unwrap()
}

// AR-SER-005, AR-SER-006, AR-SER-018: numeric boundary encodings and bytes
// are independent of HashMap insertion history after controlled reconstruction.
#[test]
fn canonical_collections_ignore_reconstruction_insertion_order() {
    let namespace = [0x20; 16];
    let counters = [0, 1, 23, 24, 255, 256, u64::from(u32::MAX) + 1, u64::MAX];
    let make_modules = |reverse: bool| {
        let iterator: Box<dyn Iterator<Item = u64>> = if reverse {
            Box::new(counters.into_iter().rev())
        } else {
            Box::new(counters.into_iter())
        };
        iterator
            .map(|counter| {
                PersistenceModule::new(
                    wire_id(namespace, counter),
                    absent_presentation(),
                    Vec::new(),
                    Vec::new(),
                )
            })
            .collect()
    };
    let first = snapshot_from_candidate(detached_program(
        1,
        RevisionCounterState::Available(2),
        AllocationCounterState::Exhausted,
        make_modules(false),
    ));
    let second = snapshot_from_candidate(detached_program(
        1,
        RevisionCounterState::Available(2),
        AllocationCounterState::Exhausted,
        make_modules(true),
    ));
    let first_bytes = encode(&first).unwrap();
    assert_eq!(first_bytes, encode(&second).unwrap());
    assert_eq!(
        encode(&decode(&first_bytes).unwrap().persistence_snapshot()).unwrap(),
        first_bytes
    );
}

fn fully_permuted_candidate(reverse: bool) -> PersistenceProgram {
    let namespace = [0x70; 16];
    let type_a = wire_id(namespace, 2);
    let entry = wire_id(namespace, 6);
    let exit = wire_id(namespace, 7);
    let condition = wire_id(namespace, 8);
    let unused = wire_id(namespace, 9);
    let result = wire_id(namespace, 10);

    let mut domain_types = vec![
        PersistenceDomainType::new(type_a, IntrinsicType::Int32, absent_presentation()),
        PersistenceDomainType::new(
            wire_id(namespace, 3),
            IntrinsicType::Text,
            absent_presentation(),
        ),
    ];
    let mut entry_expressions = vec![
        PersistenceExpression::new(condition, PersistenceExpressionKind::BoolLiteral(true)),
        PersistenceExpression::new(unused, PersistenceExpressionKind::UnitLiteral),
    ];
    if reverse {
        domain_types.reverse();
        entry_expressions.reverse();
    }
    let mut blocks = vec![
        PersistenceBlock::new(
            entry,
            entry_expressions,
            Vec::new(),
            PersistenceTerminator::Branch {
                condition,
                true_block: exit,
                false_block: exit,
            },
        ),
        PersistenceBlock::new(
            exit,
            vec![PersistenceExpression::new(
                result,
                PersistenceExpressionKind::UnitLiteral,
            )],
            Vec::new(),
            PersistenceTerminator::Return { expression: result },
        ),
    ];
    if reverse {
        blocks.reverse();
    }
    let mut functions = vec![
        PersistenceFunction::new(
            wire_id(namespace, 4),
            PersistenceValueType::Intrinsic(IntrinsicType::Unit),
            Vec::new(),
            Some(PersistenceBody::new(entry, blocks)),
            absent_presentation(),
        ),
        PersistenceFunction::new(
            wire_id(namespace, 5),
            PersistenceValueType::Domain(type_a),
            Vec::new(),
            None,
            absent_presentation(),
        ),
    ];
    if reverse {
        functions.reverse();
    }
    let first_module = PersistenceModule::new(
        wire_id(namespace, 1),
        absent_presentation(),
        domain_types,
        functions,
    );
    let second_module = PersistenceModule::new(
        wire_id(namespace, 11),
        absent_presentation(),
        Vec::new(),
        Vec::new(),
    );
    let modules = if reverse {
        vec![second_module, first_module]
    } else {
        vec![first_module, second_module]
    };
    PersistenceProgram::new(
        [0x71; 16],
        1,
        RevisionCounterState::Available(2),
        namespace,
        AllocationCounterState::Available(12),
        modules,
    )
}

// AR-SER-018: every unordered collection level is permuted independently;
// canonical bytes remain identical while semantic sequences are untouched.
#[test]
fn every_unordered_collection_is_insertion_history_independent() {
    let forward = snapshot_from_candidate(fully_permuted_candidate(false));
    let reverse = snapshot_from_candidate(fully_permuted_candidate(true));
    assert_eq!(encode(&forward).unwrap(), encode(&reverse).unwrap());
}

// AR-SER-019: presentation is persisted independently of semantic identity.
#[test]
fn presentation_only_state_changes_bytes_without_changing_identity() {
    let namespace = [0x22; 16];
    let module = |name| {
        PersistenceModule::new(
            wire_id(namespace, 7),
            PersistencePresentation::new(name, None),
            Vec::new(),
            Vec::new(),
        )
    };
    let without = snapshot_from_candidate(detached_program(
        1,
        RevisionCounterState::Available(2),
        AllocationCounterState::Exhausted,
        vec![module(None)],
    ));
    let with = snapshot_from_candidate(detached_program(
        1,
        RevisionCounterState::Available(2),
        AllocationCounterState::Exhausted,
        vec![module(Some("same identity".into()))],
    ));
    assert_eq!(
        without.modules().next().unwrap().id(),
        with.modules().next().unwrap().id()
    );
    assert_ne!(encode(&without).unwrap(), encode(&with).unwrap());
}

// AR-SER-020, AR-SER-022, AR-SER-023: all allocator state alternatives are
// restored exactly, including last-value issuance and both exhaustion modes.
#[test]
fn available_and_exhausted_allocator_boundaries_are_exact() {
    for next in [0, 1, 24, u64::MAX] {
        let source = snapshot_from_candidate(detached_program_with_tag(
            0x81,
            41,
            RevisionCounterState::Available(42),
            AllocationCounterState::Available(next),
            Vec::new(),
        ));
        let bytes = encode(&source).unwrap();
        let mut restored = decode(&bytes).unwrap().activate().unwrap();
        assert_eq!(
            restored.allocation_counter_state(),
            AllocationCounterState::Available(next)
        );
        let namespace = restored.allocation_namespace_id();
        let mut tx = restored.begin_transaction();
        let id = tx.add_module().unwrap();
        assert_eq!((id.namespace_id(), id.counter()), (namespace, next));
        let expected = next.checked_add(1).map_or(
            AllocationCounterState::Exhausted,
            AllocationCounterState::Available,
        );
        assert_eq!(tx.allocation_counter_state(), expected);
        tx.discard().unwrap();
    }

    let exhausted = snapshot_from_candidate(detached_program_with_tag(
        0x81,
        u64::MAX,
        RevisionCounterState::Exhausted,
        AllocationCounterState::Exhausted,
        Vec::new(),
    ));
    let bytes = encode(&exhausted).unwrap();
    let mut restored = decode(&bytes).unwrap().activate().unwrap();
    assert_eq!(
        restored.allocation_counter_state(),
        AllocationCounterState::Exhausted
    );
    let namespace = restored.allocation_namespace_id();
    let mut tx = restored.begin_transaction();
    assert!(tx.add_module().is_err());
    assert_eq!(tx.allocation_namespace_id(), namespace);

    drop(restored);
    let mut revision_exhausted = activate_candidate(detached_program_with_tag(
        0x82,
        u64::MAX,
        RevisionCounterState::Exhausted,
        AllocationCounterState::Available(0),
        Vec::new(),
    ));
    let mut tx = revision_exhausted.begin_transaction();
    assert!(tx.commit().is_err());
}

// AR-SER-021: successful reservations survive discard in the persisted
// authority while semantic contents and RevisionId remain unchanged.
#[test]
fn allocator_only_advancement_changes_checkpoint_without_semantic_commit() {
    let mut program = activate_candidate(detached_program_with_tag(
        0x83,
        9,
        RevisionCounterState::Available(10),
        AllocationCounterState::Available(100),
        Vec::new(),
    ));
    let before = program.persistence_snapshot();
    let before_bytes = encode(&before).unwrap();
    let mut tx = program.begin_transaction();
    for expected in 100..=103 {
        assert_eq!(tx.add_module().unwrap().counter(), expected);
    }
    tx.discard().unwrap();
    drop(tx);
    let after = program.persistence_snapshot();
    let after_bytes = encode(&after).unwrap();
    assert_eq!(before.revision_id(), after.revision_id());
    assert_eq!(before.modules().count(), after.modules().count());
    assert_ne!(before_bytes, after_bytes);
    assert_eq!(
        after.allocation_counter_state(),
        AllocationCounterState::Available(104)
    );
    assert_eq!(
        decode(&after_bytes).unwrap().allocation_counter_state(),
        AllocationCounterState::Available(104)
    );
    assert_eq!(encode(&before).unwrap(), before_bytes);

    drop(program);
    let mut restored = decode(&after_bytes).unwrap().activate().unwrap();
    let mut next = restored.begin_transaction();
    let issued = next.add_module().unwrap();
    assert_eq!(issued.counter(), 104);
    assert!(!(100..=103).contains(&issued.counter()));
    next.discard().unwrap();
}

// AR-SER-025, AR-SER-041: a decoded continuation preserves its lineage; an
// independent evolution is made by the normal fork operation and allocates in
// a fresh authority without remapping inherited IDs.
#[test]
fn independent_restored_evolution_uses_normal_fork() {
    let mut source = MnirProgram::new().unwrap();
    let inherited = {
        let mut tx = source.begin_transaction();
        let id = tx.add_module().unwrap();
        tx.commit().unwrap();
        id
    };
    let bytes = encode(&source.persistence_snapshot()).unwrap();
    drop(source);
    let mut continuation = decode(&bytes).unwrap().activate().unwrap();
    let snapshot = continuation.snapshot();
    let mut independent = snapshot.fork().unwrap();
    assert_ne!(continuation.program_id(), independent.program_id());
    assert_eq!(continuation.module(inherited).unwrap().id(), inherited);
    assert_eq!(independent.module(inherited).unwrap().id(), inherited);
    assert_ne!(
        continuation.allocation_namespace_id(),
        independent.allocation_namespace_id()
    );
    let continuation_namespace = continuation.allocation_namespace_id();
    let independent_namespace = independent.allocation_namespace_id();
    let continuation_new = {
        let mut transaction = continuation.begin_transaction();
        let id = transaction.add_module().unwrap();
        transaction.commit().unwrap();
        id
    };
    let independent_new = {
        let mut transaction = independent.begin_transaction();
        let id = transaction.add_module().unwrap();
        transaction.commit().unwrap();
        id
    };
    assert_eq!(continuation_new.namespace_id(), continuation_namespace);
    assert_eq!(independent_new.namespace_id(), independent_namespace);
    assert!(continuation.module(continuation_new).is_some());
    assert!(independent.module(independent_new).is_some());
}

// AR-SER-030, AR-SER-038: structurally valid semantic errors survive exact
// persistence and remain V0_5 diagnostics rather than decode failures.
#[test]
fn semantic_invalidity_is_loadable_and_verified_after_decode() {
    let mut program = MnirProgram::new().unwrap();
    let snapshot = {
        let mut tx = program.begin_transaction();
        let module = tx.add_module().unwrap();
        let domain = tx.add_domain_type(module, IntrinsicType::Int32).unwrap();
        let callee = tx.add_function(module, IntrinsicType::Unit).unwrap();
        tx.add_parameter(callee, IntrinsicType::Int32).unwrap();
        let function = tx.add_function(module, IntrinsicType::Int32).unwrap();
        let block = tx.create_function_body(function).unwrap();
        let left = tx.add_int32_literal(block, 1).unwrap();
        let right = tx.add_bool_literal(block, true).unwrap();
        tx.add_add_expression(block, left, right).unwrap();
        tx.add_domain_construct(block, domain, right).unwrap();
        let call = tx.add_call_expression(block, callee, vec![right]).unwrap();
        tx.set_effect_sequence(block, vec![call]).unwrap();
        tx.set_return(block, right).unwrap();
        tx.commit().unwrap()
    };
    let codes = |error: VerificationError| {
        error
            .diagnostics()
            .unwrap()
            .iter()
            .map(|diagnostic| diagnostic.code())
            .collect::<std::collections::HashSet<_>>()
    };
    let before = codes(
        verify_with_rule_set(
            &snapshot,
            VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5,
        )
        .unwrap_err(),
    );
    for expected in [
        DiagnosticCode::ArithmeticOperandTypeMismatch,
        DiagnosticCode::ReturnTypeMismatch,
        DiagnosticCode::DomainConstructRepresentationMismatch,
        DiagnosticCode::CallArgumentTypeMismatch,
    ] {
        assert!(before.contains(&expected));
    }
    let bytes = encode(&program.persistence_snapshot()).unwrap();
    drop(program);
    let restored = decode(&bytes).unwrap().activate().unwrap();
    assert_eq!(encode(&restored.persistence_snapshot()).unwrap(), bytes);
    let after = codes(
        verify_with_rule_set(
            &restored.snapshot(),
            VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5,
        )
        .unwrap_err(),
    );
    assert_eq!(after, before);
}

// AR-SER-033: semantic dependency depth does not become CBOR nesting depth;
// the wire representation remains a flat collection of ID references.
#[test]
fn deep_expression_dag_round_trips_as_flat_wire_data() {
    let mut program = MnirProgram::new().unwrap();
    {
        let mut tx = program.begin_transaction();
        let module = tx.add_module().unwrap();
        let function = tx.add_function(module, IntrinsicType::Int32).unwrap();
        let block = tx.create_function_body(function).unwrap();
        let base = tx.add_int32_literal(block, 1).unwrap();
        let mut current = base;
        for _ in 0..512 {
            current = tx.add_add_expression(block, current, base).unwrap();
        }
        tx.set_return(block, current).unwrap();
        tx.commit().unwrap();
    }
    let bytes = encode(&program.persistence_snapshot()).unwrap();
    drop(program);
    let restored = decode(&bytes).unwrap().activate().unwrap();
    assert_eq!(encode(&restored.persistence_snapshot()).unwrap(), bytes);
}

// AR-SER-032: hostile declared sizes/depth fail before payload-sized
// allocation. These compact fixtures exercise checked limit handling.
#[test]
fn hostile_declared_resources_are_rejected_before_allocation() {
    assert_eq!(
        decode(&file(&[0x5a, 0x10, 0x00, 0x00, 0x01])).unwrap_err(),
        DecodeError::ResourceLimitExceeded
    );
    assert_eq!(
        decode(&file(&[0x9a, 0x01, 0x00, 0x00, 0x01])).unwrap_err(),
        DecodeError::ResourceLimitExceeded
    );
    let mut depth = vec![0x81; 16];
    depth.push(0);
    assert_eq!(
        decode(&file(&depth)).unwrap_err(),
        DecodeError::ResourceLimitExceeded
    );
    assert_eq!(
        decode(&file(&[
            0x5b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ]))
        .unwrap_err(),
        DecodeError::ResourceLimitExceeded
    );
}

// AR-SER-004, AR-SER-026, AR-SER-029: schema failures and canonical structural
// failures remain typed and cannot expose a partial Program.
#[test]
fn unknown_tags_and_structurally_invalid_identity_are_rejected_atomically() {
    let mut unknown_counter_tag = minimal_golden();
    let position = unknown_counter_tag
        .windows(3)
        .position(|window| window == [0x82, 0x00, 0x02])
        .unwrap();
    unknown_counter_tag[position + 1] = 2;
    assert_eq!(
        decode(&unknown_counter_tag).unwrap_err(),
        DecodeError::InvalidWireSchema
    );

    let allocator_behind = manual_file(1, |encoder| {
        encoder.array(1).unwrap();
        encoder.array(4).unwrap();
        encode_test_id(encoder, [0x32; 16], 1);
        encode_absent_presentation(encoder);
        encoder.array(0).unwrap().array(0).unwrap();
    });
    assert_structural_bytes(&allocator_behind);

    let duplicate_across_categories = manual_file(2, |encoder| {
        encoder.array(1).unwrap();
        encoder.array(4).unwrap();
        encode_test_id(encoder, [0x32; 16], 1);
        encode_absent_presentation(encoder);
        encoder.array(1).unwrap();
        encoder.array(3).unwrap();
        encode_test_id(encoder, [0x32; 16], 1);
        encoder.u8(0).unwrap();
        encode_absent_presentation(encoder);
        encoder.array(0).unwrap();
    });
    assert_structural_bytes(&duplicate_across_categories);
}

// MNIR-SER-067: unsorted identity collections are non-canonical while duplicate
// persistent identity is a structural defect regardless of adjacency.
#[test]
fn collection_order_and_duplicate_identity_have_distinct_categories() {
    let modules = |counters: [u64; 2]| {
        manual_file(3, |encoder| {
            encoder.array(2).unwrap();
            for counter in counters {
                encoder.array(4).unwrap();
                encode_test_id(encoder, [0x32; 16], counter);
                encode_absent_presentation(encoder);
                encoder.array(0).unwrap().array(0).unwrap();
            }
        })
    };
    assert_eq!(
        decode(&modules([2, 1])).unwrap_err(),
        DecodeError::NonCanonicalEncoding
    );
    assert_eq!(
        decode(&modules([1, 1])).unwrap_err(),
        DecodeError::StructurallyInvalidProgram
    );
}

fn assert_structural_bytes(bytes: &[u8]) {
    static AUTHORITY_TEST_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> =
        std::sync::OnceLock::new();
    let _guard = AUTHORITY_TEST_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(
        decode(bytes).unwrap_err(),
        DecodeError::StructurallyInvalidProgram
    );

    // AR-SER-029: each failed public decode is followed by successful exact
    // activation of the same lineage/authority coordinates. A partial claim,
    // allocation, or revision advance would make this control activation fail.
    let control = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 3,
        blocks: vec![return_block(3, 4)],
    }]);
    let restored = decode(&control).unwrap().activate().unwrap();
    assert_eq!(restored.revision_id().persistent_value(), 1);
    assert_eq!(
        restored.allocation_counter_state(),
        AllocationCounterState::Available(200)
    );
    drop(restored);
}

// AR-SER-027 through AR-SER-029: every required reference/graph family is a
// canonical byte fixture exercising the public decode boundary. The core
// reconstruction tests separately exercise the same validator directly.
#[test]
fn structural_rejection_matrix_is_exercised_through_canonical_bytes() {
    let unresolved_call = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 3,
        blocks: vec![WireBlock {
            id: 3,
            expressions: vec![(
                4,
                WireExpression::Call {
                    target: 90,
                    argument: None,
                },
            )],
            effect_sequence: vec![4],
            terminator: WireTerminator::Return(4),
        }],
    }]);
    assert_structural_bytes(&unresolved_call);

    let unresolved_type = structural_file(vec![WireFunction {
        id: 2,
        domain_return: Some(90),
        entry: 3,
        blocks: vec![return_block(3, 4)],
    }]);
    assert_structural_bytes(&unresolved_type);

    let invalid_branch = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 3,
        blocks: vec![WireBlock {
            id: 3,
            expressions: vec![(4, WireExpression::Bool)],
            effect_sequence: Vec::new(),
            terminator: WireTerminator::Branch {
                condition: 4,
                true_block: 90,
                false_block: 90,
            },
        }],
    }]);
    assert_structural_bytes(&invalid_branch);

    let invalid_entry = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 90,
        blocks: vec![return_block(3, 4)],
    }]);
    assert_structural_bytes(&invalid_entry);

    let foreign_expression = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 3,
        blocks: vec![
            WireBlock {
                id: 3,
                expressions: vec![(4, WireExpression::Bool)],
                effect_sequence: Vec::new(),
                terminator: WireTerminator::Branch {
                    condition: 4,
                    true_block: 5,
                    false_block: 5,
                },
            },
            WireBlock {
                id: 5,
                expressions: vec![(6, WireExpression::Add { left: 4, right: 4 })],
                effect_sequence: Vec::new(),
                terminator: WireTerminator::Return(6),
            },
        ],
    }]);
    assert_structural_bytes(&foreign_expression);

    let expression_cycle = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 3,
        blocks: vec![WireBlock {
            id: 3,
            expressions: vec![
                (4, WireExpression::Add { left: 5, right: 5 }),
                (5, WireExpression::Add { left: 4, right: 4 }),
            ],
            effect_sequence: Vec::new(),
            terminator: WireTerminator::Return(4),
        }],
    }]);
    assert_structural_bytes(&expression_cycle);

    let cfg_cycle = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 3,
        blocks: vec![
            WireBlock {
                id: 3,
                expressions: vec![(4, WireExpression::Bool)],
                effect_sequence: Vec::new(),
                terminator: WireTerminator::Branch {
                    condition: 4,
                    true_block: 5,
                    false_block: 5,
                },
            },
            WireBlock {
                id: 5,
                expressions: vec![(6, WireExpression::Bool)],
                effect_sequence: Vec::new(),
                terminator: WireTerminator::Branch {
                    condition: 6,
                    true_block: 3,
                    false_block: 3,
                },
            },
        ],
    }]);
    assert_structural_bytes(&cfg_cycle);

    let unreachable_cfg = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 3,
        blocks: vec![return_block(3, 4), return_block(5, 6)],
    }]);
    assert_structural_bytes(&unreachable_cfg);

    let missing_call = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 3,
        blocks: vec![WireBlock {
            id: 3,
            expressions: vec![(
                4,
                WireExpression::Call {
                    target: 2,
                    argument: None,
                },
            )],
            effect_sequence: Vec::new(),
            terminator: WireTerminator::Return(4),
        }],
    }]);
    assert_structural_bytes(&missing_call);

    let duplicate_sequence = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 3,
        blocks: vec![WireBlock {
            id: 3,
            expressions: vec![(
                4,
                WireExpression::Call {
                    target: 2,
                    argument: None,
                },
            )],
            effect_sequence: vec![4, 4],
            terminator: WireTerminator::Return(4),
        }],
    }]);
    assert_structural_bytes(&duplicate_sequence);

    let foreign_sequence = structural_file(vec![
        WireFunction {
            id: 2,
            domain_return: None,
            entry: 3,
            blocks: vec![WireBlock {
                id: 3,
                expressions: vec![(
                    4,
                    WireExpression::Call {
                        target: 2,
                        argument: None,
                    },
                )],
                effect_sequence: vec![4, 12],
                terminator: WireTerminator::Return(4),
            }],
        },
        WireFunction {
            id: 10,
            domain_return: None,
            entry: 11,
            blocks: vec![WireBlock {
                id: 11,
                expressions: vec![(
                    12,
                    WireExpression::Call {
                        target: 10,
                        argument: None,
                    },
                )],
                effect_sequence: vec![12],
                terminator: WireTerminator::Return(12),
            }],
        },
    ]);
    assert_structural_bytes(&foreign_sequence);

    let ordering_conflict = structural_file(vec![WireFunction {
        id: 2,
        domain_return: None,
        entry: 3,
        blocks: vec![WireBlock {
            id: 3,
            expressions: vec![
                (
                    4,
                    WireExpression::Call {
                        target: 2,
                        argument: None,
                    },
                ),
                (
                    5,
                    WireExpression::Call {
                        target: 2,
                        argument: Some(4),
                    },
                ),
            ],
            effect_sequence: vec![5, 4],
            terminator: WireTerminator::Return(5),
        }],
    }]);
    assert_structural_bytes(&ordering_conflict);
}

// AR-SER-038: a compatible Program remains accepted by every historical
// verification rule set, and persistence restores no verification evidence.
#[test]
fn all_verifier_rule_sets_run_normally_after_decode() {
    let mut program = MnirProgram::new().unwrap();
    {
        let mut tx = program.begin_transaction();
        let module = tx.add_module().unwrap();
        let function = tx.add_function(module, IntrinsicType::Int32).unwrap();
        let block = tx.create_function_body(function).unwrap();
        let value = tx.add_int32_literal(block, 1).unwrap();
        tx.set_return(block, value).unwrap();
        tx.commit().unwrap();
    }
    let bytes = encode(&program.persistence_snapshot()).unwrap();
    drop(program);
    let restored = decode(&bytes).unwrap().activate().unwrap();
    for rule_set in [
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_1,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_2,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_3,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_4,
        VerificationRuleSet::SemanticVerificationAndDiagnosticsV0_5,
    ] {
        let evidence = verify_with_rule_set(&restored.snapshot(), rule_set).unwrap();
        assert_eq!(evidence.program_id(), restored.program_id());
        assert_eq!(evidence.revision_id(), restored.revision_id());
        assert_eq!(evidence.rule_set(), rule_set);
    }
}

// AR-SER-040 and AR-SER-041: immutable decode is always safe, while mutable
// activation is process-wide exclusive and permanently remembers freshness.
#[test]
fn known_stale_checkpoint_authority_is_rejected() {
    let mut program = MnirProgram::new().unwrap();
    let old_bytes = encode(&program.persistence_snapshot()).unwrap();
    let mut tx = program.begin_transaction();
    for _ in 0..4 {
        tx.add_module().unwrap();
    }
    tx.discard().unwrap();
    drop(tx);
    let current_bytes = encode(&program.persistence_snapshot()).unwrap();

    let historical = decode(&old_bytes).unwrap();
    assert_eq!(historical.program_id(), program.program_id());
    assert_ne!(
        historical.allocation_counter_state(),
        program.allocation_counter_state()
    );
    assert_eq!(
        decode(&old_bytes).unwrap().activate().unwrap_err(),
        DecodeError::StructurallyInvalidProgram
    );
    assert_eq!(
        decode(&current_bytes).unwrap().activate().unwrap_err(),
        DecodeError::StructurallyInvalidProgram
    );

    drop(program);
    assert_eq!(
        decode(&old_bytes).unwrap().activate().unwrap_err(),
        DecodeError::StructurallyInvalidProgram
    );
    let current = decode(&current_bytes).unwrap().activate().unwrap();
    assert_eq!(
        encode(&historical.persistence_snapshot()).unwrap(),
        old_bytes
    );
    assert_eq!(
        historical.allocation_counter_state(),
        AllocationCounterState::Available(1)
    );
    assert_eq!(
        decode(&current_bytes).unwrap().activate().unwrap_err(),
        DecodeError::StructurallyInvalidProgram
    );
    drop(current);
    assert!(decode(&current_bytes).unwrap().activate().is_ok());
}

// AR-SER-034, AR-SER-036, AR-SER-039: the public boundary is pure bytes,
// crate dependencies are one-way, and excluded filesystem/EasyH/verifier
// mechanisms are absent from the production format crate.
#[test]
fn crate_direction_and_scope_remain_bounded() {
    let manifest = include_str!("../Cargo.toml");
    let source = include_str!("../src/lib.rs");
    let core_manifest = include_str!("../../mnir-core/Cargo.toml");
    assert!(manifest.contains("minicbor.workspace = true"));
    assert!(manifest.contains("mnir-core.workspace = true"));
    assert!(!manifest.contains("easyh-render"));
    assert!(!core_manifest.contains("mnir-format"));
    for excluded in [
        "std::fs",
        "serde",
        "minicbor::Encode",
        "minicbor::Decode",
        "compression",
        "signature",
        "checksum",
    ] {
        assert!(!source.contains(excluded), "unexpected scope: {excluded}");
    }
}
