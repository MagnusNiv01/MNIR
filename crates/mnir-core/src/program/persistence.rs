use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::{
    AllocationCounterState, AllocationNamespaceId, BlockId, ExpressionId, ExpressionKind,
    FunctionId, IntrinsicType, ModuleId, ParameterId, PresentationMetadata, ProgramId, RevisionId,
    Terminator, TypeId, ValueType,
};

use super::authority::{AuthorityClaimError, AuthorityLease};
use super::body::{Block, Expression, FunctionBody};
use super::model::{
    AllocationAuthorityState, DomainType, Function, MnirProgram, Module, Parameter, RevisionState,
};
use super::validation::validate_revision_state;

/// Immutable observation of a Program head suitable for durable persistence.
///
/// Unlike [`crate::ProgramSnapshot`], this checkpoint also captures the
/// revision-allocation cursor required to resume the same lineage exactly.
#[derive(Clone, Debug)]
pub struct PersistenceSnapshot {
    pub(super) program_id: ProgramId,
    pub(super) state: Arc<RevisionState>,
    pub(super) next_revision_raw: Option<u64>,
    pub(super) allocation_authority: AllocationAuthorityState,
}

impl PersistenceSnapshot {
    #[must_use]
    pub const fn program_id(&self) -> ProgramId {
        self.program_id
    }

    #[must_use]
    pub fn revision_id(&self) -> RevisionId {
        self.state.revision_id
    }

    #[must_use]
    pub const fn revision_counter_state(&self) -> RevisionCounterState {
        match self.next_revision_raw {
            Some(next) => RevisionCounterState::Available(next),
            None => RevisionCounterState::Exhausted,
        }
    }

    #[must_use]
    pub const fn allocation_namespace_id(&self) -> AllocationNamespaceId {
        self.allocation_authority.namespace_id()
    }

    #[must_use]
    pub const fn allocation_counter_state(&self) -> AllocationCounterState {
        self.allocation_authority.counter_state()
    }

    pub fn modules(&self) -> impl Iterator<Item = &Module> {
        self.state.modules.values()
    }
}

/// Persistent state of the lineage-local revision allocator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevisionCounterState {
    Available(u64),
    Exhausted,
}

/// Raw wire components for one typed persistent entity reference.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistenceEntityId {
    namespace: [u8; 16],
    counter: u64,
}

impl PersistenceEntityId {
    #[must_use]
    pub const fn new(namespace: [u8; 16], counter: u64) -> Self {
        Self { namespace, counter }
    }

    #[must_use]
    pub const fn namespace_bytes(self) -> [u8; 16] {
        self.namespace
    }

    #[must_use]
    pub const fn counter(self) -> u64 {
        self.counter
    }

    const fn namespace_id(self) -> AllocationNamespaceId {
        AllocationNamespaceId(self.namespace)
    }

    const fn module_id(self) -> ModuleId {
        ModuleId::new(self.namespace_id(), self.counter)
    }

    const fn type_id(self) -> TypeId {
        TypeId::new(self.namespace_id(), self.counter)
    }

    const fn function_id(self) -> FunctionId {
        FunctionId::new(self.namespace_id(), self.counter)
    }

    const fn parameter_id(self) -> ParameterId {
        ParameterId::new(self.namespace_id(), self.counter)
    }

    const fn block_id(self) -> BlockId {
        BlockId::new(self.namespace_id(), self.counter)
    }

    const fn expression_id(self) -> ExpressionId {
        ExpressionId::new(self.namespace_id(), self.counter)
    }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceValueType {
    Intrinsic(IntrinsicType),
    Domain(PersistenceEntityId),
}

impl PersistenceValueType {
    fn into_value_type(self) -> ValueType {
        match self {
            Self::Intrinsic(intrinsic) => ValueType::Intrinsic(intrinsic),
            Self::Domain(id) => ValueType::Domain(id.type_id()),
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PersistenceExpressionKind {
    Int32Literal(i32),
    Int64Literal(i64),
    BoolLiteral(bool),
    UnitLiteral,
    TextLiteral(String),
    BytesLiteral(Vec<u8>),
    ParameterReference(PersistenceEntityId),
    Add {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    Subtract {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    Multiply {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    Divide {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    Remainder {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    Equal {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    NotEqual {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    LessThan {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    LessThanOrEqual {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    GreaterThan {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    GreaterThanOrEqual {
        left: PersistenceEntityId,
        right: PersistenceEntityId,
    },
    Call {
        target: PersistenceEntityId,
        arguments: Vec<PersistenceEntityId>,
    },
    DomainConstruct {
        type_id: PersistenceEntityId,
        value: PersistenceEntityId,
    },
    DomainProject {
        value: PersistenceEntityId,
    },
}

impl PersistenceExpressionKind {
    fn into_expression_kind(self) -> ExpressionKind {
        macro_rules! binary {
            ($variant:ident, $left:ident, $right:ident) => {
                ExpressionKind::$variant {
                    left: $left.expression_id(),
                    right: $right.expression_id(),
                }
            };
        }
        match self {
            Self::Int32Literal(value) => ExpressionKind::Int32Literal(value),
            Self::Int64Literal(value) => ExpressionKind::Int64Literal(value),
            Self::BoolLiteral(value) => ExpressionKind::BoolLiteral(value),
            Self::UnitLiteral => ExpressionKind::UnitLiteral,
            Self::TextLiteral(value) => ExpressionKind::TextLiteral(value),
            Self::BytesLiteral(value) => ExpressionKind::BytesLiteral(value),
            Self::ParameterReference(id) => ExpressionKind::ParameterReference(id.parameter_id()),
            Self::Add { left, right } => binary!(Add, left, right),
            Self::Subtract { left, right } => binary!(Subtract, left, right),
            Self::Multiply { left, right } => binary!(Multiply, left, right),
            Self::Divide { left, right } => binary!(Divide, left, right),
            Self::Remainder { left, right } => binary!(Remainder, left, right),
            Self::Equal { left, right } => binary!(Equal, left, right),
            Self::NotEqual { left, right } => binary!(NotEqual, left, right),
            Self::LessThan { left, right } => binary!(LessThan, left, right),
            Self::LessThanOrEqual { left, right } => binary!(LessThanOrEqual, left, right),
            Self::GreaterThan { left, right } => binary!(GreaterThan, left, right),
            Self::GreaterThanOrEqual { left, right } => binary!(GreaterThanOrEqual, left, right),
            Self::Call { target, arguments } => ExpressionKind::Call {
                target: target.function_id(),
                arguments: arguments
                    .into_iter()
                    .map(PersistenceEntityId::expression_id)
                    .collect(),
            },
            Self::DomainConstruct { type_id, value } => ExpressionKind::DomainConstruct {
                type_id: type_id.type_id(),
                value: value.expression_id(),
            },
            Self::DomainProject { value } => ExpressionKind::DomainProject {
                value: value.expression_id(),
            },
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceTerminator {
    Return {
        expression: PersistenceEntityId,
    },
    Branch {
        condition: PersistenceEntityId,
        true_block: PersistenceEntityId,
        false_block: PersistenceEntityId,
    },
}

impl PersistenceTerminator {
    fn into_terminator(self) -> Terminator {
        match self {
            Self::Return { expression } => Terminator::Return {
                expression: expression.expression_id(),
            },
            Self::Branch {
                condition,
                true_block,
                false_block,
            } => Terminator::Branch {
                condition: condition.expression_id(),
                true_block: true_block.block_id(),
                false_block: false_block.block_id(),
            },
        }
    }
}

/// Detached reconstruction candidate accepted only by validated restore.
///
/// This type does not provide allocation or mutation authority. It is public
/// solely so persistence crates can stage a complete decoded document before
/// handing it to [`MnirProgram::validate_persistence`].
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct PersistenceProgram {
    program_id: ProgramId,
    revision_id: RevisionId,
    revision_counter_state: RevisionCounterState,
    allocation_namespace_id: AllocationNamespaceId,
    allocation_counter_state: AllocationCounterState,
    modules: Vec<PersistenceModule>,
}

impl PersistenceProgram {
    #[must_use]
    pub fn new(
        program_id: [u8; 16],
        revision_id: u64,
        revision_counter_state: RevisionCounterState,
        allocation_namespace_id: [u8; 16],
        allocation_counter_state: AllocationCounterState,
        modules: Vec<PersistenceModule>,
    ) -> Self {
        Self {
            program_id: ProgramId(program_id),
            revision_id: RevisionId(revision_id),
            revision_counter_state,
            allocation_namespace_id: AllocationNamespaceId(allocation_namespace_id),
            allocation_counter_state,
            modules,
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct PersistencePresentation {
    preferred_name: Option<String>,
    documentation: Option<String>,
}

impl PersistencePresentation {
    #[must_use]
    pub const fn new(preferred_name: Option<String>, documentation: Option<String>) -> Self {
        Self {
            preferred_name,
            documentation,
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct PersistenceModule {
    id: ModuleId,
    presentation: PersistencePresentation,
    domain_types: Vec<PersistenceDomainType>,
    functions: Vec<PersistenceFunction>,
}

impl PersistenceModule {
    #[must_use]
    pub fn new(
        id: PersistenceEntityId,
        presentation: PersistencePresentation,
        domain_types: Vec<PersistenceDomainType>,
        functions: Vec<PersistenceFunction>,
    ) -> Self {
        Self {
            id: id.module_id(),
            presentation,
            domain_types,
            functions,
        }
    }

    #[must_use]
    pub const fn persistent_id(&self) -> PersistenceEntityId {
        PersistenceEntityId::new(self.id.namespace_id().0, self.id.counter())
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct PersistenceDomainType {
    id: TypeId,
    representation: IntrinsicType,
    presentation: PersistencePresentation,
}

impl PersistenceDomainType {
    #[must_use]
    pub fn new(
        id: PersistenceEntityId,
        representation: IntrinsicType,
        presentation: PersistencePresentation,
    ) -> Self {
        Self {
            id: id.type_id(),
            representation,
            presentation,
        }
    }

    #[must_use]
    pub const fn persistent_id(&self) -> PersistenceEntityId {
        PersistenceEntityId::new(self.id.namespace_id().0, self.id.counter())
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct PersistenceFunction {
    id: FunctionId,
    return_type: ValueType,
    parameters: Vec<PersistenceParameter>,
    body: Option<PersistenceBody>,
    presentation: PersistencePresentation,
}

impl PersistenceFunction {
    #[must_use]
    pub fn new(
        id: PersistenceEntityId,
        return_type: PersistenceValueType,
        parameters: Vec<PersistenceParameter>,
        body: Option<PersistenceBody>,
        presentation: PersistencePresentation,
    ) -> Self {
        Self {
            id: id.function_id(),
            return_type: return_type.into_value_type(),
            parameters,
            body,
            presentation,
        }
    }

    #[must_use]
    pub const fn persistent_id(&self) -> PersistenceEntityId {
        PersistenceEntityId::new(self.id.namespace_id().0, self.id.counter())
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct PersistenceParameter {
    id: ParameterId,
    value_type: ValueType,
    presentation: PersistencePresentation,
}

impl PersistenceParameter {
    #[must_use]
    pub fn new(
        id: PersistenceEntityId,
        value_type: PersistenceValueType,
        presentation: PersistencePresentation,
    ) -> Self {
        Self {
            id: id.parameter_id(),
            value_type: value_type.into_value_type(),
            presentation,
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct PersistenceBody {
    entry_block_id: BlockId,
    blocks: Vec<PersistenceBlock>,
}

impl PersistenceBody {
    #[must_use]
    pub fn new(entry_block_id: PersistenceEntityId, blocks: Vec<PersistenceBlock>) -> Self {
        Self {
            entry_block_id: entry_block_id.block_id(),
            blocks,
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct PersistenceBlock {
    id: BlockId,
    expressions: Vec<PersistenceExpression>,
    effect_sequence: Vec<ExpressionId>,
    terminator: Terminator,
}

impl PersistenceBlock {
    #[must_use]
    pub fn new(
        id: PersistenceEntityId,
        expressions: Vec<PersistenceExpression>,
        effect_sequence: Vec<PersistenceEntityId>,
        terminator: PersistenceTerminator,
    ) -> Self {
        Self {
            id: id.block_id(),
            expressions,
            effect_sequence: effect_sequence
                .into_iter()
                .map(PersistenceEntityId::expression_id)
                .collect(),
            terminator: terminator.into_terminator(),
        }
    }

    #[must_use]
    pub const fn persistent_id(&self) -> PersistenceEntityId {
        PersistenceEntityId::new(self.id.namespace_id().0, self.id.counter())
    }
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct PersistenceExpression {
    id: ExpressionId,
    kind: ExpressionKind,
}

impl PersistenceExpression {
    #[must_use]
    pub fn new(id: PersistenceEntityId, kind: PersistenceExpressionKind) -> Self {
        Self {
            id: id.expression_id(),
            kind: kind.into_expression_kind(),
        }
    }

    #[must_use]
    pub const fn persistent_id(&self) -> PersistenceEntityId {
        PersistenceEntityId::new(self.id.namespace_id().0, self.id.counter())
    }
}

/// Failure to reconstruct a live lineage from detached persistent state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PersistenceRestoreError {
    InvalidRevisionAllocator,
    DuplicateEntityIdentity,
    EntityAllocatorBehind,
    Structural,
    AuthorityAlreadyActive,
    StaleAuthority,
    AuthorityIdentityCollision,
}

impl fmt::Display for PersistenceRestoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidRevisionAllocator => "invalid revision allocator state",
            Self::DuplicateEntityIdentity => "duplicate persistent entity identity",
            Self::EntityAllocatorBehind => "entity allocator is behind reconstructed identities",
            Self::Structural => "reconstructed Program is structurally invalid",
            Self::AuthorityAlreadyActive => "persistent allocation authority is already active",
            Self::StaleAuthority => "persistent allocation authority is stale",
            Self::AuthorityIdentityCollision => "persistent allocation authority identity collides",
        };
        formatter.write_str(message)
    }
}

impl Error for PersistenceRestoreError {}

/// Structurally validated immutable persistence state.
///
/// Validation does not claim mutable allocation authority. Activation consumes
/// this value and atomically claims process-local authority.
#[derive(Debug)]
pub struct ValidatedPersistenceState {
    program_id: ProgramId,
    state: Arc<RevisionState>,
    next_revision_raw: Option<u64>,
    allocation_authority: AllocationAuthorityState,
}

impl ValidatedPersistenceState {
    #[must_use]
    pub const fn program_id(&self) -> ProgramId {
        self.program_id
    }

    #[must_use]
    pub fn revision_id(&self) -> RevisionId {
        self.state.revision_id
    }

    #[must_use]
    pub const fn revision_counter_state(&self) -> RevisionCounterState {
        match self.next_revision_raw {
            Some(next) => RevisionCounterState::Available(next),
            None => RevisionCounterState::Exhausted,
        }
    }

    #[must_use]
    pub const fn allocation_namespace_id(&self) -> AllocationNamespaceId {
        self.allocation_authority.namespace_id()
    }

    #[must_use]
    pub const fn allocation_counter_state(&self) -> AllocationCounterState {
        self.allocation_authority.counter_state()
    }

    pub fn modules(&self) -> impl Iterator<Item = &Module> {
        self.state.modules.values()
    }

    /// Returns an immutable encodable observation without granting mutation
    /// authority.
    #[must_use]
    pub fn persistence_snapshot(&self) -> PersistenceSnapshot {
        PersistenceSnapshot {
            program_id: self.program_id,
            state: Arc::clone(&self.state),
            next_revision_raw: self.next_revision_raw,
            allocation_authority: self.allocation_authority,
        }
    }
}

impl MnirProgram {
    /// Captures one immutable, internally consistent persistence checkpoint.
    #[must_use]
    pub fn persistence_snapshot(&self) -> PersistenceSnapshot {
        PersistenceSnapshot {
            program_id: self.program_id,
            state: Arc::clone(&self.head),
            next_revision_raw: self.next_revision_raw,
            allocation_authority: self.allocation_authority,
        }
    }

    /// Validates detached persistent state without claiming mutable authority.
    #[doc(hidden)]
    pub fn validate_persistence(
        candidate: PersistenceProgram,
    ) -> Result<ValidatedPersistenceState, PersistenceRestoreError> {
        let expected_next = candidate.revision_id.persistent_value().checked_add(1);
        match candidate.revision_counter_state {
            RevisionCounterState::Available(next) if Some(next) == expected_next && next != 0 => {}
            RevisionCounterState::Exhausted if expected_next.is_none() => {}
            RevisionCounterState::Available(_) | RevisionCounterState::Exhausted => {
                return Err(PersistenceRestoreError::InvalidRevisionAllocator);
            }
        }

        let mut seen = HashSet::new();
        let mut modules = HashMap::with_capacity(candidate.modules.len().min(4096));
        for persisted_module in candidate.modules {
            register_identity(
                &mut seen,
                persisted_module.id.namespace_id(),
                persisted_module.id.counter(),
                candidate.allocation_namespace_id,
                candidate.allocation_counter_state,
            )?;

            let mut module = Module::new(persisted_module.id);
            module.presentation = restore_presentation(persisted_module.presentation);

            for persisted_type in persisted_module.domain_types {
                register_identity(
                    &mut seen,
                    persisted_type.id.namespace_id(),
                    persisted_type.id.counter(),
                    candidate.allocation_namespace_id,
                    candidate.allocation_counter_state,
                )?;
                let mut domain_type =
                    DomainType::new(persisted_type.id, persisted_type.representation);
                domain_type.presentation = restore_presentation(persisted_type.presentation);
                if module
                    .domain_types
                    .insert(persisted_type.id, domain_type)
                    .is_some()
                {
                    return Err(PersistenceRestoreError::DuplicateEntityIdentity);
                }
            }

            for persisted_function in persisted_module.functions {
                register_identity(
                    &mut seen,
                    persisted_function.id.namespace_id(),
                    persisted_function.id.counter(),
                    candidate.allocation_namespace_id,
                    candidate.allocation_counter_state,
                )?;
                let mut function =
                    Function::new(persisted_function.id, persisted_function.return_type);
                function.presentation = restore_presentation(persisted_function.presentation);

                for persisted_parameter in persisted_function.parameters {
                    register_identity(
                        &mut seen,
                        persisted_parameter.id.namespace_id(),
                        persisted_parameter.id.counter(),
                        candidate.allocation_namespace_id,
                        candidate.allocation_counter_state,
                    )?;
                    let mut parameter =
                        Parameter::new(persisted_parameter.id, persisted_parameter.value_type);
                    parameter.presentation = restore_presentation(persisted_parameter.presentation);
                    function.parameters.push(parameter);
                }

                if let Some(persisted_body) = persisted_function.body {
                    let mut blocks = HashMap::with_capacity(persisted_body.blocks.len().min(4096));
                    for persisted_block in persisted_body.blocks {
                        register_identity(
                            &mut seen,
                            persisted_block.id.namespace_id(),
                            persisted_block.id.counter(),
                            candidate.allocation_namespace_id,
                            candidate.allocation_counter_state,
                        )?;
                        let mut expressions =
                            HashMap::with_capacity(persisted_block.expressions.len().min(4096));
                        for persisted_expression in persisted_block.expressions {
                            register_identity(
                                &mut seen,
                                persisted_expression.id.namespace_id(),
                                persisted_expression.id.counter(),
                                candidate.allocation_namespace_id,
                                candidate.allocation_counter_state,
                            )?;
                            let expression =
                                Expression::new(persisted_expression.id, persisted_expression.kind);
                            if expressions
                                .insert(persisted_expression.id, expression)
                                .is_some()
                            {
                                return Err(PersistenceRestoreError::DuplicateEntityIdentity);
                            }
                        }
                        let block = Block {
                            id: persisted_block.id,
                            expressions,
                            effect_sequence: persisted_block.effect_sequence,
                            terminator: Some(persisted_block.terminator),
                        };
                        if blocks.insert(persisted_block.id, block).is_some() {
                            return Err(PersistenceRestoreError::DuplicateEntityIdentity);
                        }
                    }
                    function.body = Some(FunctionBody {
                        entry_block_id: persisted_body.entry_block_id,
                        blocks,
                    });
                }

                if module
                    .functions
                    .insert(persisted_function.id, function)
                    .is_some()
                {
                    return Err(PersistenceRestoreError::DuplicateEntityIdentity);
                }
            }

            if modules.insert(persisted_module.id, module).is_some() {
                return Err(PersistenceRestoreError::DuplicateEntityIdentity);
            }
        }

        let state = Arc::new(RevisionState {
            revision_id: candidate.revision_id,
            modules,
        });
        validate_revision_state(&state).map_err(|_| PersistenceRestoreError::Structural)?;

        Ok(ValidatedPersistenceState {
            program_id: candidate.program_id,
            state,
            next_revision_raw: match candidate.revision_counter_state {
                RevisionCounterState::Available(next) => Some(next),
                RevisionCounterState::Exhausted => None,
            },
            allocation_authority: AllocationAuthorityState {
                namespace_id: candidate.allocation_namespace_id,
                counter_state: candidate.allocation_counter_state,
            },
        })
    }

    /// Activates one validated checkpoint as the unique process-local mutable
    /// continuation of its Program lineage.
    pub fn activate_persistence(
        validated: ValidatedPersistenceState,
    ) -> Result<Self, PersistenceRestoreError> {
        let authority_lease = AuthorityLease::claim_restored(
            validated.program_id,
            validated.allocation_authority.namespace_id(),
            validated.state.revision_id,
            validated.allocation_authority.counter_state(),
        )
        .map_err(|error| match error {
            AuthorityClaimError::AlreadyActive => PersistenceRestoreError::AuthorityAlreadyActive,
            AuthorityClaimError::Stale => PersistenceRestoreError::StaleAuthority,
            AuthorityClaimError::IdentityCollision(_) => {
                PersistenceRestoreError::AuthorityIdentityCollision
            }
        })?;

        Ok(Self {
            program_id: validated.program_id,
            head: validated.state,
            next_revision_raw: validated.next_revision_raw,
            allocation_authority: validated.allocation_authority,
            authority_lease,
        })
    }
}

fn restore_presentation(persisted: PersistencePresentation) -> PresentationMetadata {
    let mut presentation = PresentationMetadata::default();
    presentation.set_preferred_name(persisted.preferred_name);
    presentation.set_documentation(persisted.documentation);
    presentation
}

fn register_identity(
    seen: &mut HashSet<(AllocationNamespaceId, u64)>,
    namespace: AllocationNamespaceId,
    counter: u64,
    active_namespace: AllocationNamespaceId,
    active_state: AllocationCounterState,
) -> Result<(), PersistenceRestoreError> {
    if !seen.insert((namespace, counter)) {
        return Err(PersistenceRestoreError::DuplicateEntityIdentity);
    }
    if namespace == active_namespace
        && let AllocationCounterState::Available(next) = active_state
        && counter >= next
    {
        return Err(PersistenceRestoreError::EntityAllocatorBehind);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::model::identity_generation_calls;

    fn empty_candidate(program: u8, namespace: u8) -> PersistenceProgram {
        PersistenceProgram::new(
            [program; 16],
            41,
            RevisionCounterState::Available(42),
            [namespace; 16],
            AllocationCounterState::Available(104),
            Vec::new(),
        )
    }

    // AR-SER-035: validation and activation restore exact persisted authority
    // without invoking the entropy-backed normal identity generator.
    #[test]
    fn persistence_activation_is_entropy_free_and_identity_exact() {
        let before = identity_generation_calls();
        let validated = MnirProgram::validate_persistence(empty_candidate(0xe8, 0xe9)).unwrap();
        assert_eq!(identity_generation_calls(), before);
        let program_id = validated.program_id();
        let namespace = validated.allocation_namespace_id();

        let restored = MnirProgram::activate_persistence(validated).unwrap();
        assert_eq!(identity_generation_calls(), before);
        assert_eq!(restored.program_id(), program_id);
        assert_eq!(restored.allocation_namespace_id(), namespace);
        assert_eq!(restored.revision_id().persistent_value(), 41);
        assert_eq!(
            restored.allocation_counter_state(),
            AllocationCounterState::Available(104)
        );
    }

    // AR-SER-029 and AR-SER-035: validation failure claims no authority and
    // therefore cannot leave a phantom active lineage.
    #[test]
    fn failed_validation_leaves_no_authority_claim() {
        let mut invalid = empty_candidate(0xea, 0xeb);
        invalid.revision_counter_state = RevisionCounterState::Available(99);
        assert_eq!(
            MnirProgram::validate_persistence(invalid).unwrap_err(),
            PersistenceRestoreError::InvalidRevisionAllocator
        );
        let validated = MnirProgram::validate_persistence(empty_candidate(0xea, 0xeb)).unwrap();
        assert!(MnirProgram::activate_persistence(validated).is_ok());
    }
}
