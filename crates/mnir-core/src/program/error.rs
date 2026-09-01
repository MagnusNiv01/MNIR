use std::error::Error;
use std::fmt;

use crate::ids::{FunctionId, IdentifierCategory, ModuleId, ParameterId, RevisionId};

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
            Self::SourceRevisionChanged { expected, actual } => write!(
                formatter,
                "transaction source revision changed from {expected:?} to {actual:?}"
            ),
            Self::StructuralViolation(error) => error.fmt(formatter),
        }
    }
}

impl Error for MutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::StructuralViolation(error) => Some(error),
            _ => None,
        }
    }
}
