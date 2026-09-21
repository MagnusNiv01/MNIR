use mnir_core::{
    AllocationCounterState, IntrinsicType, MnirProgram, PersistenceBlock, PersistenceBody,
    PersistenceEntityId, PersistenceExpression, PersistenceExpressionKind, PersistenceFunction,
    PersistenceModule, PersistencePresentation, PersistenceProgram, PersistenceRestoreError,
    PersistenceTerminator, PersistenceValueType, RevisionCounterState,
};

fn id(counter: u64) -> PersistenceEntityId {
    PersistenceEntityId::new([0x51; 16], counter)
}

fn expression(counter: u64, kind: PersistenceExpressionKind) -> PersistenceExpression {
    PersistenceExpression::new(id(counter), kind)
}

fn block(
    counter: u64,
    expressions: Vec<PersistenceExpression>,
    effect_sequence: Vec<PersistenceEntityId>,
    terminator: PersistenceTerminator,
) -> PersistenceBlock {
    PersistenceBlock::new(id(counter), expressions, effect_sequence, terminator)
}

fn body(entry_counter: u64, blocks: Vec<PersistenceBlock>) -> PersistenceBody {
    PersistenceBody::new(id(entry_counter), blocks)
}

fn absent_presentation() -> PersistencePresentation {
    PersistencePresentation::new(None, None)
}

fn candidate(body: PersistenceBody) -> PersistenceProgram {
    candidate_with_return(body, PersistenceValueType::Intrinsic(IntrinsicType::Unit))
}

fn candidate_with_return(
    body: PersistenceBody,
    return_type: PersistenceValueType,
) -> PersistenceProgram {
    PersistenceProgram::new(
        [0x50; 16],
        1,
        RevisionCounterState::Available(2),
        [0x51; 16],
        AllocationCounterState::Available(100),
        vec![PersistenceModule::new(
            id(1),
            absent_presentation(),
            Vec::new(),
            vec![PersistenceFunction::new(
                id(2),
                return_type,
                Vec::new(),
                Some(body),
                absent_presentation(),
            )],
        )],
    )
}

fn assert_structural(candidate: PersistenceProgram) {
    assert_eq!(
        MnirProgram::validate_persistence(candidate).unwrap_err(),
        PersistenceRestoreError::Structural
    );
}

// AR-SER-027: controlled reconstruction delegates every unresolved ownership
// and reference relation to the complete mnir-core structural validator.
#[test]
fn reconstruction_rejects_invalid_entry_branch_call_type_and_foreign_expression() {
    assert_structural(candidate(PersistenceBody::new(
        id(99),
        vec![block(
            3,
            vec![expression(4, PersistenceExpressionKind::UnitLiteral)],
            Vec::new(),
            PersistenceTerminator::Return { expression: id(4) },
        )],
    )));

    assert_structural(candidate(body(
        3,
        vec![block(
            3,
            vec![expression(4, PersistenceExpressionKind::BoolLiteral(true))],
            Vec::new(),
            PersistenceTerminator::Branch {
                condition: id(4),
                true_block: id(90),
                false_block: id(91),
            },
        )],
    )));

    assert_structural(candidate(body(
        3,
        vec![block(
            3,
            vec![expression(
                4,
                PersistenceExpressionKind::Call {
                    target: id(90),
                    arguments: Vec::new(),
                },
            )],
            vec![id(4)],
            PersistenceTerminator::Return { expression: id(4) },
        )],
    )));

    let dangling_type = candidate_with_return(
        body(
            3,
            vec![block(
                3,
                vec![expression(4, PersistenceExpressionKind::UnitLiteral)],
                Vec::new(),
                PersistenceTerminator::Return { expression: id(4) },
            )],
        ),
        PersistenceValueType::Domain(id(90)),
    );
    assert_structural(dangling_type);

    assert_structural(candidate(body(
        3,
        vec![
            block(
                3,
                vec![expression(4, PersistenceExpressionKind::UnitLiteral)],
                Vec::new(),
                PersistenceTerminator::Return { expression: id(4) },
            ),
            block(
                5,
                vec![expression(
                    6,
                    PersistenceExpressionKind::Add {
                        left: id(4),
                        right: id(4),
                    },
                )],
                Vec::new(),
                PersistenceTerminator::Return { expression: id(6) },
            ),
        ],
    )));
}

// AR-SER-028: graph and EffectSequence invalidity is rejected after staged
// reconstruction rather than repaired or semantically verified.
#[test]
fn reconstruction_rejects_expression_cfg_and_effect_graph_defects() {
    assert_structural(candidate(body(
        3,
        vec![block(
            3,
            vec![expression(
                4,
                PersistenceExpressionKind::Add {
                    left: id(4),
                    right: id(4),
                },
            )],
            Vec::new(),
            PersistenceTerminator::Return { expression: id(4) },
        )],
    )));

    let cyclic_block = |counter: u64, condition: u64, target: u64| {
        block(
            counter,
            vec![expression(
                condition,
                PersistenceExpressionKind::BoolLiteral(true),
            )],
            Vec::new(),
            PersistenceTerminator::Branch {
                condition: id(condition),
                true_block: id(target),
                false_block: id(target),
            },
        )
    };
    assert_structural(candidate(body(
        3,
        vec![cyclic_block(3, 4, 5), cyclic_block(5, 6, 3)],
    )));

    assert_structural(candidate(body(
        3,
        vec![
            block(
                3,
                vec![expression(4, PersistenceExpressionKind::UnitLiteral)],
                Vec::new(),
                PersistenceTerminator::Return { expression: id(4) },
            ),
            block(
                5,
                vec![expression(6, PersistenceExpressionKind::UnitLiteral)],
                Vec::new(),
                PersistenceTerminator::Return { expression: id(6) },
            ),
        ],
    )));

    assert_structural(candidate(body(
        3,
        vec![block(
            3,
            vec![expression(4, PersistenceExpressionKind::UnitLiteral)],
            vec![id(90)],
            PersistenceTerminator::Return { expression: id(4) },
        )],
    )));

    let call = expression(
        4,
        PersistenceExpressionKind::Call {
            target: id(2),
            arguments: Vec::new(),
        },
    );
    assert_structural(candidate(body(
        3,
        vec![block(
            3,
            vec![call],
            Vec::new(),
            PersistenceTerminator::Return { expression: id(4) },
        )],
    )));

    let call = expression(
        4,
        PersistenceExpressionKind::Call {
            target: id(2),
            arguments: Vec::new(),
        },
    );
    assert_structural(candidate(body(
        3,
        vec![block(
            3,
            vec![call],
            vec![id(4), id(4)],
            PersistenceTerminator::Return { expression: id(4) },
        )],
    )));

    let first = expression(
        4,
        PersistenceExpressionKind::Call {
            target: id(2),
            arguments: Vec::new(),
        },
    );
    let second = expression(
        5,
        PersistenceExpressionKind::Call {
            target: id(2),
            arguments: vec![id(4)],
        },
    );
    assert_structural(candidate(body(
        3,
        vec![block(
            3,
            vec![first, second],
            vec![id(5), id(4)],
            PersistenceTerminator::Return { expression: id(5) },
        )],
    )));
}
