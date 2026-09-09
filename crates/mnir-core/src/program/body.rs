use std::collections::HashMap;

use crate::ids::{BlockId, ExpressionId, FunctionId, ParameterId};

/// The closed set of Block terminators defined by Conditional Control Flow 0.1.
///
/// Terminators deliberately have no independent identity and are not
/// Expressions (`MNIR-CFG-016` through `MNIR-CFG-023`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Terminator {
    Return {
        expression: ExpressionId,
    },
    Branch {
        condition: ExpressionId,
        true_block: BlockId,
        false_block: BlockId,
    },
}

/// The closed set of Expression alternatives currently defined by MNIR.
///
/// Arithmetic, comparison, and Call alternatives extend the existing
/// Expression model and retain operand/argument position directly in their semantic data
/// (`MNIR-ARITH-001` through `MNIR-ARITH-005`, `MNIR-CMP-005` through
/// `MNIR-CMP-009`, `MNIR-CALL-006` through `MNIR-CALL-008`).
#[derive(Clone, Debug, Eq, PartialEq)]
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
    Equal {
        left: ExpressionId,
        right: ExpressionId,
    },
    NotEqual {
        left: ExpressionId,
        right: ExpressionId,
    },
    LessThan {
        left: ExpressionId,
        right: ExpressionId,
    },
    LessThanOrEqual {
        left: ExpressionId,
        right: ExpressionId,
    },
    GreaterThan {
        left: ExpressionId,
        right: ExpressionId,
    },
    GreaterThanOrEqual {
        left: ExpressionId,
        right: ExpressionId,
    },
    Call {
        target: FunctionId,
        arguments: Vec<ExpressionId>,
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
            Self::Equal { .. }
            | Self::NotEqual { .. }
            | Self::LessThan { .. }
            | Self::LessThanOrEqual { .. }
            | Self::GreaterThan { .. }
            | Self::GreaterThanOrEqual { .. }
            | Self::Int32Literal(_)
            | Self::Int64Literal(_)
            | Self::BoolLiteral(_)
            | Self::UnitLiteral
            | Self::ParameterReference(_)
            | Self::Call { .. } => None,
        }
    }

    pub(super) const fn comparison_operands(&self) -> Option<(ExpressionId, ExpressionId)> {
        match self {
            Self::Equal { left, right }
            | Self::NotEqual { left, right }
            | Self::LessThan { left, right }
            | Self::LessThanOrEqual { left, right }
            | Self::GreaterThan { left, right }
            | Self::GreaterThanOrEqual { left, right } => Some((*left, *right)),
            Self::Int32Literal(_)
            | Self::Int64Literal(_)
            | Self::BoolLiteral(_)
            | Self::UnitLiteral
            | Self::ParameterReference(_)
            | Self::Add { .. }
            | Self::Subtract { .. }
            | Self::Multiply { .. }
            | Self::Divide { .. }
            | Self::Remainder { .. }
            | Self::Call { .. } => None,
        }
    }

    pub(super) fn dependencies(&self) -> Vec<ExpressionId> {
        if let Some((left, right)) = self.arithmetic_operands() {
            vec![left, right]
        } else if let Some((left, right)) = self.comparison_operands() {
            vec![left, right]
        } else if let Self::Call { arguments, .. } = self {
            arguments.clone()
        } else {
            Vec::new()
        }
    }

    pub(super) const fn is_call(&self) -> bool {
        matches!(self, Self::Call { .. })
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

    pub(super) fn copied(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
        }
    }
}

/// One independently addressable Block owned by a Function body.
#[derive(Debug, Eq, PartialEq)]
pub struct Block {
    pub(super) id: BlockId,
    pub(super) expressions: HashMap<ExpressionId, Expression>,
    pub(super) effect_sequence: Vec<ExpressionId>,
    pub(super) terminator: Option<Terminator>,
}

impl Block {
    pub(super) fn new(id: BlockId) -> Self {
        Self {
            id,
            expressions: HashMap::new(),
            effect_sequence: Vec::new(),
            terminator: None,
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

    /// Returns effectful Call Expressions in semantic execution order
    /// (`MNIR-CALL-002` through `MNIR-CALL-004`, `MNIR-CALL-043`,
    /// `MNIR-CALL-044`).
    #[must_use]
    pub fn effect_sequence(&self) -> &[ExpressionId] {
        &self.effect_sequence
    }

    #[must_use]
    pub const fn return_expression_id(&self) -> Option<ExpressionId> {
        match self.terminator {
            Some(Terminator::Return { expression }) => Some(expression),
            Some(Terminator::Branch { .. }) | None => None,
        }
    }

    /// Inspects this Block's terminator, if one has been installed.
    #[must_use]
    pub const fn terminator(&self) -> Option<&Terminator> {
        self.terminator.as_ref()
    }

    pub(super) fn copied(&self) -> Self {
        Self {
            id: self.id,
            expressions: self
                .expressions
                .iter()
                .map(|(&id, expression)| (id, expression.copied()))
                .collect(),
            effect_sequence: self.effect_sequence.clone(),
            terminator: self.terminator,
        }
    }
}

/// The optional executable-body structure owned by a Function.
///
/// The identity-based Block collection owned by one Function.
#[derive(Debug, Eq, PartialEq)]
pub struct FunctionBody {
    pub(super) entry_block_id: BlockId,
    pub(super) blocks: HashMap<BlockId, Block>,
}

impl FunctionBody {
    pub(super) fn new(block_id: BlockId) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(block_id, Block::new(block_id));
        Self {
            entry_block_id: block_id,
            blocks,
        }
    }

    #[must_use]
    pub const fn block_id(&self) -> BlockId {
        self.entry_block_id
    }

    #[must_use]
    pub fn block(&self) -> &Block {
        self.blocks
            .get(&self.entry_block_id)
            .expect("validated FunctionBody entry Block")
    }

    /// Returns the distinguished entry Block identity (`MNIR-CFG-058`).
    #[must_use]
    pub const fn entry_block_id(&self) -> BlockId {
        self.entry_block_id
    }

    /// Looks up any Block owned by this body by identity (`MNIR-CFG-059`).
    #[must_use]
    pub fn block_by_id(&self, id: BlockId) -> Option<&Block> {
        self.blocks.get(&id)
    }

    /// Enumerates all owned Blocks in non-semantic order (`MNIR-CFG-060`).
    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.values()
    }

    #[must_use]
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub(super) fn copied(&self) -> Self {
        Self {
            entry_block_id: self.entry_block_id,
            blocks: self
                .blocks
                .iter()
                .map(|(&id, block)| (id, block.copied()))
                .collect(),
        }
    }
}
