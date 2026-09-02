use std::collections::HashMap;

use crate::ids::{BlockId, ExpressionId, ParameterId};

/// The closed set of Expression alternatives currently defined by MNIR.
///
/// Arithmetic operators extend the existing Expression model and retain
/// operand position directly in their semantic data (`MNIR-ARITH-001` through
/// `MNIR-ARITH-005`, `MNIR-ARITH-084`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpressionKind {
    Int32Literal(i32),
    Int64Literal(i64),
    BoolLiteral(bool),
    UnitLiteral,
    ParameterReference(ParameterId),
    Add {
        left: ExpressionId,
        right: ExpressionId,
    },
    Subtract {
        left: ExpressionId,
        right: ExpressionId,
    },
    Multiply {
        left: ExpressionId,
        right: ExpressionId,
    },
    Divide {
        left: ExpressionId,
        right: ExpressionId,
    },
    Remainder {
        left: ExpressionId,
        right: ExpressionId,
    },
}

impl ExpressionKind {
    pub(super) const fn arithmetic_operands(&self) -> Option<(ExpressionId, ExpressionId)> {
        match self {
            Self::Add { left, right }
            | Self::Subtract { left, right }
            | Self::Multiply { left, right }
            | Self::Divide { left, right }
            | Self::Remainder { left, right } => Some((*left, *right)),
            Self::Int32Literal(_)
            | Self::Int64Literal(_)
            | Self::BoolLiteral(_)
            | Self::UnitLiteral
            | Self::ParameterReference(_) => None,
        }
    }
}

/// An independently addressable Expression owned by one Block.
#[derive(Debug, Eq, PartialEq)]
pub struct Expression {
    pub(super) id: ExpressionId,
    pub(super) kind: ExpressionKind,
}

impl Expression {
    pub(super) const fn new(id: ExpressionId, kind: ExpressionKind) -> Self {
        Self { id, kind }
    }

    #[must_use]
    pub const fn id(&self) -> ExpressionId {
        self.id
    }

    /// Exposes all semantic data carried by this Expression kind
    /// (`MNIR-EXPR-068`, `MNIR-EXPR-100`).
    #[must_use]
    pub const fn kind(&self) -> &ExpressionKind {
        &self.kind
    }

    pub(super) const fn copied(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind,
        }
    }
}

/// The single Block owned by a version 0.1 Function body.
#[derive(Debug, Eq, PartialEq)]
pub struct Block {
    pub(super) id: BlockId,
    pub(super) expressions: HashMap<ExpressionId, Expression>,
    // Return is a terminator represented directly by its Expression reference,
    // not an Expression or a general Statement (`MNIR-EXPR-039`, -101).
    pub(super) return_expression_id: Option<ExpressionId>,
}

impl Block {
    pub(super) fn new(id: BlockId) -> Self {
        Self {
            id,
            expressions: HashMap::new(),
            return_expression_id: None,
        }
    }

    #[must_use]
    pub const fn id(&self) -> BlockId {
        self.id
    }

    #[must_use]
    pub fn expression_count(&self) -> usize {
        self.expressions.len()
    }

    #[must_use]
    pub fn expression(&self, id: ExpressionId) -> Option<&Expression> {
        self.expressions.get(&id)
    }

    /// Iteration order has no MNIR semantic meaning (`MNIR-EXPR-018`).
    pub fn expressions(&self) -> impl Iterator<Item = &Expression> {
        self.expressions.values()
    }

    #[must_use]
    pub const fn return_expression_id(&self) -> Option<ExpressionId> {
        self.return_expression_id
    }

    pub(super) fn copied(&self) -> Self {
        Self {
            id: self.id,
            expressions: self
                .expressions
                .iter()
                .map(|(&id, expression)| (id, expression.copied()))
                .collect(),
            return_expression_id: self.return_expression_id,
        }
    }
}

/// The optional executable-body structure owned by a Function.
///
/// Version 0.1 contains exactly one Block and defines no execution backend.
#[derive(Debug, Eq, PartialEq)]
pub struct FunctionBody {
    pub(super) block: Block,
}

impl FunctionBody {
    pub(super) fn new(block_id: BlockId) -> Self {
        Self {
            block: Block::new(block_id),
        }
    }

    #[must_use]
    pub const fn block_id(&self) -> BlockId {
        self.block.id
    }

    #[must_use]
    pub const fn block(&self) -> &Block {
        &self.block
    }

    pub(super) fn copied(&self) -> Self {
        Self {
            block: self.block.copied(),
        }
    }
}
