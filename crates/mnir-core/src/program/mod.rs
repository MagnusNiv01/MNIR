mod authority;
mod body;
mod error;
mod model;
mod persistence;
mod transaction;
mod validation;

pub use body::{Block, Expression, ExpressionKind, FunctionBody, Terminator};
pub use error::{ExpressionTypeError, MutationError, StructuralError, TransactionState};
pub use model::{DomainType, Function, MnirProgram, Module, Parameter, ProgramSnapshot};
pub use persistence::{
    PersistenceBlock, PersistenceBody, PersistenceDomainType, PersistenceEntityId,
    PersistenceExpression, PersistenceExpressionKind, PersistenceFunction, PersistenceModule,
    PersistenceParameter, PersistencePresentation, PersistenceProgram, PersistenceRestoreError,
    PersistenceSnapshot, PersistenceTerminator, PersistenceValueType, RevisionCounterState,
    ValidatedPersistenceState,
};
pub use transaction::MutationTransaction;
