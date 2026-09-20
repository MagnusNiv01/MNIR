use std::error::Error;
use std::fmt;

use crate::ValueType;
use crate::ids::{
    BlockId, ExpressionId, FunctionId, IdentifierCategory, ModuleId, ParameterId, RevisionId,
    TypeId,
};

/// Structural failures defined by `MNIR-CORE-040` and `MNIR-CORE-041`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuralError {
    ModuleIdentityMismatch {
        collection_id: ModuleId,
        module_id: ModuleId,
    },
    FunctionIdentityMismatch {
        collection_id: FunctionId,
        function_id: FunctionId,
    },
    DuplicateFunctionIdentity(FunctionId),
    DomainTypeIdentityMismatch {
        collection_id: TypeId,
        type_id: TypeId,
    },
    DuplicateTypeIdentity(TypeId),
    DuplicateParameterIdentity(ParameterId),
    FunctionBodyHasNoBlocks(FunctionId),
    EntryBlockNotInBody {
        function_id: FunctionId,
        entry_block_id: BlockId,
    },
    BlockIdentityMismatch {
        collection_id: BlockId,
        block_id: BlockId,
    },
    DuplicateBlockIdentity(BlockId),
    ExpressionIdentityMismatch {
        collection_id: ExpressionId,
        expression_id: ExpressionId,
    },
    DuplicateExpressionIdentity(ExpressionId),
    UnterminatedBlock(BlockId),
    ReturnExpressionNotInBlock {
        block_id: BlockId,
        expression_id: ExpressionId,
    },
    BranchConditionNotInBlock {
        block_id: BlockId,
        expression_id: ExpressionId,
    },
    BranchTargetNotInBody {
        block_id: BlockId,
        target_block_id: BlockId,
    },
    UnreachableBlock {
        function_id: FunctionId,
        block_id: BlockId,
    },
    CyclicControlFlow(FunctionId),
    DanglingParameterReference {
        expression_id: ExpressionId,
        parameter_id: ParameterId,
        function_id: FunctionId,
    },
    DanglingValueType(TypeId),
    DanglingDomainConstructTarget {
        expression_id: ExpressionId,
        type_id: TypeId,
    },
    DomainExpressionSourceNotInBlock {
        expression_id: ExpressionId,
        source_id: ExpressionId,
        block_id: BlockId,
    },
    ArithmeticOperandNotInBlock {
        expression_id: ExpressionId,
        operand_id: ExpressionId,
        block_id: BlockId,
    },
    ComparisonOperandNotInBlock {
        expression_id: ExpressionId,
        operand_id: ExpressionId,
        block_id: BlockId,
    },
    DanglingCallTarget {
        expression_id: ExpressionId,
        function_id: FunctionId,
    },
    CallArgumentNotInBlock {
        expression_id: ExpressionId,
        argument_id: ExpressionId,
        block_id: BlockId,
    },
    EffectSequenceEntryNotInBlock {
        block_id: BlockId,
        expression_id: ExpressionId,
    },
    EffectSequenceEntryNotCall {
        block_id: BlockId,
        expression_id: ExpressionId,
    },
    DuplicateEffectSequenceEntry {
        block_id: BlockId,
        expression_id: ExpressionId,
    },
    CallMissingFromEffectSequence {
        block_id: BlockId,
        expression_id: ExpressionId,
    },
    EffectSequenceOrderConflict {
        block_id: BlockId,
        dependency_id: ExpressionId,
        dependent_id: ExpressionId,
    },
    CyclicExpressionDependency(BlockId),
}

impl fmt::Display for StructuralError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModuleIdentityMismatch {
                collection_id,
                module_id,
            } => write!(
                formatter,
                "module collection identity {collection_id:?} does not match contained identity {module_id:?}"
            ),
            Self::FunctionIdentityMismatch {
                collection_id,
                function_id,
            } => write!(
                formatter,
                "function collection identity {collection_id:?} does not match contained identity {function_id:?}"
            ),
            Self::DuplicateFunctionIdentity(id) => {
                write!(formatter, "function identity {id:?} occurs more than once")
            }
            Self::DomainTypeIdentityMismatch {
                collection_id,
                type_id,
            } => write!(
                formatter,
                "Domain Type collection identity {collection_id:?} does not match contained identity {type_id:?}"
            ),
            Self::DuplicateTypeIdentity(id) => {
                write!(
                    formatter,
                    "Domain Type identity {id:?} occurs more than once"
                )
            }
            Self::DuplicateParameterIdentity(id) => {
                write!(formatter, "parameter identity {id:?} occurs more than once")
            }
            Self::FunctionBodyHasNoBlocks(id) => {
                write!(formatter, "function {id:?} has a body with no Blocks")
            }
            Self::EntryBlockNotInBody {
                function_id,
                entry_block_id,
            } => write!(
                formatter,
                "entry block {entry_block_id:?} is not owned by function {function_id:?}"
            ),
            Self::BlockIdentityMismatch {
                collection_id,
                block_id,
            } => write!(
                formatter,
                "block collection identity {collection_id:?} does not match contained identity {block_id:?}"
            ),
            Self::DuplicateBlockIdentity(id) => {
                write!(formatter, "block identity {id:?} occurs more than once")
            }
            Self::ExpressionIdentityMismatch {
                collection_id,
                expression_id,
            } => write!(
                formatter,
                "expression collection identity {collection_id:?} does not match contained identity {expression_id:?}"
            ),
            Self::DuplicateExpressionIdentity(id) => {
                write!(
                    formatter,
                    "expression identity {id:?} occurs more than once"
                )
            }
            Self::UnterminatedBlock(id) => {
                write!(formatter, "block {id:?} has no Return terminator")
            }
            Self::ReturnExpressionNotInBlock {
                block_id,
                expression_id,
            } => write!(
                formatter,
                "Return expression {expression_id:?} is not owned by block {block_id:?}"
            ),
            Self::BranchConditionNotInBlock {
                block_id,
                expression_id,
            } => write!(
                formatter,
                "Branch condition {expression_id:?} is not owned by block {block_id:?}"
            ),
            Self::BranchTargetNotInBody {
                block_id,
                target_block_id,
            } => write!(
                formatter,
                "Branch target {target_block_id:?} is not in the Function body containing block {block_id:?}"
            ),
            Self::UnreachableBlock {
                function_id,
                block_id,
            } => write!(
                formatter,
                "block {block_id:?} is unreachable from the entry of function {function_id:?}"
            ),
            Self::CyclicControlFlow(function_id) => {
                write!(formatter, "function {function_id:?} contains a cyclic CFG")
            }
            Self::DanglingParameterReference {
                expression_id,
                parameter_id,
                function_id,
            } => write!(
                formatter,
                "expression {expression_id:?} refers to parameter {parameter_id:?} not owned by function {function_id:?}"
            ),
            Self::DanglingValueType(type_id) => {
                write!(
                    formatter,
                    "ValueType refers to missing Domain Type {type_id:?}"
                )
            }
            Self::DanglingDomainConstructTarget {
                expression_id,
                type_id,
            } => write!(
                formatter,
                "DomainConstruct expression {expression_id:?} targets missing Domain Type {type_id:?}"
            ),
            Self::DomainExpressionSourceNotInBlock {
                expression_id,
                source_id,
                block_id,
            } => write!(
                formatter,
                "Domain expression {expression_id:?} refers to source {source_id:?} not owned by block {block_id:?}"
            ),
            Self::ArithmeticOperandNotInBlock {
                expression_id,
                operand_id,
                block_id,
            } => write!(
                formatter,
                "arithmetic expression {expression_id:?} refers to operand {operand_id:?} not owned by block {block_id:?}"
            ),
            Self::ComparisonOperandNotInBlock {
                expression_id,
                operand_id,
                block_id,
            } => write!(
                formatter,
                "comparison expression {expression_id:?} refers to operand {operand_id:?} not owned by block {block_id:?}"
            ),
            Self::DanglingCallTarget {
                expression_id,
                function_id,
            } => write!(
                formatter,
                "Call expression {expression_id:?} targets missing function {function_id:?}"
            ),
            Self::CallArgumentNotInBlock {
                expression_id,
                argument_id,
                block_id,
            } => write!(
                formatter,
                "Call expression {expression_id:?} refers to argument {argument_id:?} not owned by block {block_id:?}"
            ),
            Self::EffectSequenceEntryNotInBlock {
                block_id,
                expression_id,
            } => write!(
                formatter,
                "effect sequence in block {block_id:?} refers to missing expression {expression_id:?}"
            ),
            Self::EffectSequenceEntryNotCall {
                block_id,
                expression_id,
            } => write!(
                formatter,
                "effect sequence in block {block_id:?} refers to non-Call expression {expression_id:?}"
            ),
            Self::DuplicateEffectSequenceEntry {
                block_id,
                expression_id,
            } => write!(
                formatter,
                "effect sequence in block {block_id:?} contains Call {expression_id:?} more than once"
            ),
            Self::CallMissingFromEffectSequence {
                block_id,
                expression_id,
            } => write!(
                formatter,
                "Call expression {expression_id:?} is absent from block {block_id:?}'s effect sequence"
            ),
            Self::EffectSequenceOrderConflict {
                block_id,
                dependency_id,
                dependent_id,
            } => write!(
                formatter,
                "effect sequence in block {block_id:?} orders dependent Call {dependent_id:?} before Call dependency {dependency_id:?}"
            ),
            Self::CyclicExpressionDependency(block_id) => write!(
                formatter,
                "block {block_id:?} contains a cyclic Expression dependency"
            ),
        }
    }
}

impl Error for StructuralError {}

/// The logical lifecycle states required by `MNIR-CORE-060`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionState {
    Active,
    Failed,
    Committed,
    Discarded,
}

/// Typed failures from Program Model construction and mutation APIs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MutationError {
    IdentifierExhausted(IdentifierCategory),
    IdentityGenerationFailed(IdentifierCategory),
    IdentityCollision(IdentifierCategory),
    TransactionNotActive(TransactionState),
    UnknownModule(ModuleId),
    UnknownType(TypeId),
    UnknownFunction(FunctionId),
    UnknownParameter(ParameterId),
    UnknownBlock(BlockId),
    UnknownExpression(ExpressionId),
    FunctionBodyAlreadyExists(FunctionId),
    FunctionBodyAbsent(FunctionId),
    EntryBlockCannotBeRemoved(BlockId),
    ParameterNotOwnedByFunction {
        parameter_id: ParameterId,
        function_id: FunctionId,
    },
    ExpressionNotOwnedByBlock {
        expression_id: ExpressionId,
        block_id: BlockId,
    },
    BlockNotOwnedByFunctionBody {
        block_id: BlockId,
        function_id: FunctionId,
    },
    EffectSequenceEntryNotCall(ExpressionId),
    DuplicateEffectSequenceEntry(ExpressionId),
    SourceRevisionChanged {
        expected: RevisionId,
        actual: RevisionId,
    },
    StructuralViolation(StructuralError),
}

impl fmt::Display for MutationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentifierExhausted(category) => {
                write!(formatter, "{category:?} identifier space is exhausted")
            }
            Self::IdentityGenerationFailed(category) => {
                write!(formatter, "failed to generate {category:?} identity")
            }
            Self::IdentityCollision(category) => {
                write!(
                    formatter,
                    "could not avoid detected {category:?} identity collision"
                )
            }
            Self::TransactionNotActive(state) => {
                write!(formatter, "transaction is not active: {state:?}")
            }
            Self::UnknownModule(id) => write!(formatter, "unknown module: {id:?}"),
            Self::UnknownType(id) => write!(formatter, "unknown Domain Type: {id:?}"),
            Self::UnknownFunction(id) => write!(formatter, "unknown function: {id:?}"),
            Self::UnknownParameter(id) => write!(formatter, "unknown parameter: {id:?}"),
            Self::UnknownBlock(id) => write!(formatter, "unknown block: {id:?}"),
            Self::UnknownExpression(id) => write!(formatter, "unknown expression: {id:?}"),
            Self::FunctionBodyAlreadyExists(id) => {
                write!(formatter, "function already has a body: {id:?}")
            }
            Self::FunctionBodyAbsent(id) => {
                write!(formatter, "function has no body: {id:?}")
            }
            Self::EntryBlockCannotBeRemoved(id) => {
                write!(
                    formatter,
                    "entry block cannot be removed individually: {id:?}"
                )
            }
            Self::ParameterNotOwnedByFunction {
                parameter_id,
                function_id,
            } => write!(
                formatter,
                "parameter {parameter_id:?} is not owned by function {function_id:?}"
            ),
            Self::ExpressionNotOwnedByBlock {
                expression_id,
                block_id,
            } => write!(
                formatter,
                "expression {expression_id:?} is not owned by block {block_id:?}"
            ),
            Self::BlockNotOwnedByFunctionBody {
                block_id,
                function_id,
            } => write!(
                formatter,
                "block {block_id:?} is not owned by function {function_id:?}"
            ),
            Self::EffectSequenceEntryNotCall(expression_id) => write!(
                formatter,
                "expression {expression_id:?} is not a Call and cannot be effect-sequenced"
            ),
            Self::DuplicateEffectSequenceEntry(expression_id) => write!(
                formatter,
                "Call expression {expression_id:?} occurs more than once in the effect sequence"
            ),
            Self::SourceRevisionChanged { expected, actual } => write!(
                formatter,
                "transaction source revision changed from {expected:?} to {actual:?}"
            ),
            Self::StructuralViolation(error) => error.fmt(formatter),
        }
    }
}

/// A read-only failure to derive an Expression's intrinsic type.
///
/// Unlike [`MutationError`], this error never poisons a transaction
/// (`MNIR-EXPR-090`, `MNIR-EXPR-091`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpressionTypeError {
    UnresolvedParameter {
        expression_id: ExpressionId,
        parameter_id: ParameterId,
    },
    UnresolvedFunction {
        expression_id: ExpressionId,
        function_id: FunctionId,
    },
    UnresolvedDomainType {
        expression_id: ExpressionId,
        type_id: TypeId,
    },
    OperandTypeUnavailable {
        expression_id: ExpressionId,
    },
    OperandTypeMismatch {
        expression_id: ExpressionId,
        left_type: ValueType,
        right_type: ValueType,
    },
    UnsupportedOperandType {
        expression_id: ExpressionId,
        operand_type: ValueType,
    },
    DomainProjectSourceTypeUnavailable {
        expression_id: ExpressionId,
        source_expression_id: ExpressionId,
    },
    DomainProjectSourceNotDomain {
        expression_id: ExpressionId,
        source_expression_id: ExpressionId,
        actual_type: ValueType,
    },
}

impl fmt::Display for ExpressionTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnresolvedParameter {
                expression_id,
                parameter_id,
            } => write!(
                formatter,
                "cannot derive type of expression {expression_id:?}: parameter {parameter_id:?} does not resolve"
            ),
            Self::UnresolvedFunction {
                expression_id,
                function_id,
            } => write!(
                formatter,
                "cannot derive type of expression {expression_id:?}: target function {function_id:?} does not resolve"
            ),
            Self::UnresolvedDomainType {
                expression_id,
                type_id,
            } => write!(
                formatter,
                "cannot derive type of expression {expression_id:?}: Domain Type {type_id:?} does not resolve"
            ),
            Self::OperandTypeUnavailable { expression_id } => write!(
                formatter,
                "cannot derive type of expression {expression_id:?}: an operand type is unavailable"
            ),
            Self::OperandTypeMismatch {
                expression_id,
                left_type,
                right_type,
            } => write!(
                formatter,
                "cannot derive type of expression {expression_id:?}: operand types {left_type:?} and {right_type:?} differ"
            ),
            Self::UnsupportedOperandType {
                expression_id,
                operand_type,
            } => write!(
                formatter,
                "cannot derive type of expression {expression_id:?}: operand type {operand_type:?} is unsupported"
            ),
            Self::DomainProjectSourceTypeUnavailable {
                expression_id,
                source_expression_id,
            } => write!(
                formatter,
                "cannot derive type of DomainProject {expression_id:?}: source {source_expression_id:?} has no available type"
            ),
            Self::DomainProjectSourceNotDomain {
                expression_id,
                source_expression_id,
                actual_type,
            } => write!(
                formatter,
                "cannot derive type of DomainProject {expression_id:?}: source {source_expression_id:?} has non-Domain type {actual_type:?}"
            ),
        }
    }
}

impl Error for ExpressionTypeError {}

impl Error for MutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::StructuralViolation(error) => Some(error),
            _ => None,
        }
    }
}
