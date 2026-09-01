use std::error::Error;
use std::fmt;

use crate::ids::{
    BlockId, ExpressionId, FunctionId, IdentifierCategory, ModuleId, ParameterId, RevisionId,
};

/// Structural failures defined by `MNIR-CORE-040` and `MNIR-CORE-041`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuralError {
    ModuleIdentityMismatch {
        collection_id: ModuleId,
        module_id: ModuleId,
    },
    ModuleIdentityNotCommitted(ModuleId),
    FunctionIdentityMismatch {
        collection_id: FunctionId,
        function_id: FunctionId,
    },
    FunctionIdentityNotCommitted(FunctionId),
    DuplicateFunctionIdentity(FunctionId),
    ParameterIdentityNotCommitted(ParameterId),
    DuplicateParameterIdentity(ParameterId),
    BlockIdentityNotCommitted(BlockId),
    DuplicateBlockIdentity(BlockId),
    ExpressionIdentityMismatch {
        collection_id: ExpressionId,
        expression_id: ExpressionId,
    },
    ExpressionIdentityNotCommitted(ExpressionId),
    DuplicateExpressionIdentity(ExpressionId),
    UnterminatedBlock(BlockId),
    ReturnExpressionNotInBlock {
        block_id: BlockId,
        expression_id: ExpressionId,
    },
    DanglingParameterReference {
        expression_id: ExpressionId,
        parameter_id: ParameterId,
        function_id: FunctionId,
    },
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
            Self::ModuleIdentityNotCommitted(id) => {
                write!(
                    formatter,
                    "module identity {id:?} is not in committed history"
                )
            }
            Self::FunctionIdentityMismatch {
                collection_id,
                function_id,
            } => write!(
                formatter,
                "function collection identity {collection_id:?} does not match contained identity {function_id:?}"
            ),
            Self::FunctionIdentityNotCommitted(id) => {
                write!(
                    formatter,
                    "function identity {id:?} is not in committed history"
                )
            }
            Self::DuplicateFunctionIdentity(id) => {
                write!(formatter, "function identity {id:?} occurs more than once")
            }
            Self::ParameterIdentityNotCommitted(id) => {
                write!(
                    formatter,
                    "parameter identity {id:?} is not in committed history"
                )
            }
            Self::DuplicateParameterIdentity(id) => {
                write!(formatter, "parameter identity {id:?} occurs more than once")
            }
            Self::BlockIdentityNotCommitted(id) => {
                write!(
                    formatter,
                    "block identity {id:?} is not in committed history"
                )
            }
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
            Self::ExpressionIdentityNotCommitted(id) => write!(
                formatter,
                "expression identity {id:?} is not in committed history"
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
            Self::DanglingParameterReference {
                expression_id,
                parameter_id,
                function_id,
            } => write!(
                formatter,
                "expression {expression_id:?} refers to parameter {parameter_id:?} not owned by function {function_id:?}"
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
    TransactionNotActive(TransactionState),
    UnknownModule(ModuleId),
    UnknownFunction(FunctionId),
    UnknownParameter(ParameterId),
    UnknownBlock(BlockId),
    UnknownExpression(ExpressionId),
    FunctionBodyAlreadyExists(FunctionId),
    FunctionBodyAbsent(FunctionId),
    ParameterNotOwnedByFunction {
        parameter_id: ParameterId,
        function_id: FunctionId,
    },
    ExpressionNotOwnedByBlock {
        expression_id: ExpressionId,
        block_id: BlockId,
    },
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
            Self::TransactionNotActive(state) => {
                write!(formatter, "transaction is not active: {state:?}")
            }
            Self::UnknownModule(id) => write!(formatter, "unknown module: {id:?}"),
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
