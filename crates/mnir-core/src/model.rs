use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ids::{IdentifierCategory, ModuleId, ProgramId, RevisionId};

static NEXT_PROGRAM_ID: AtomicU64 = AtomicU64::new(1);

/// Optional human-oriented data associated with a Module.
///
/// These values never determine Module identity (`MNIR-CORE-022` through
/// `MNIR-CORE-025`).
#[derive(Debug, Default, Eq, PartialEq)]
pub struct PresentationMetadata {
    preferred_name: Option<String>,
    documentation: Option<String>,
}

impl PresentationMetadata {
    #[must_use]
    pub fn preferred_name(&self) -> Option<&str> {
        self.preferred_name.as_deref()
    }

    #[must_use]
    pub fn documentation(&self) -> Option<&str> {
        self.documentation.as_deref()
    }

    fn copied(&self) -> Self {
        Self {
            preferred_name: self.preferred_name.clone(),
            documentation: self.documentation.clone(),
        }
    }
}

/// A Module in Program Model 0.1.
///
/// Modules contain only typed identity and presentation metadata in this
/// specification increment.
#[derive(Debug, Eq, PartialEq)]
pub struct Module {
    id: ModuleId,
    presentation: PresentationMetadata,
}

impl Module {
    fn new(id: ModuleId) -> Self {
        Self {
            id,
            presentation: PresentationMetadata::default(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> ModuleId {
        self.id
    }

    #[must_use]
    pub const fn presentation(&self) -> &PresentationMetadata {
        &self.presentation
    }

    fn copied(&self) -> Self {
        Self {
            id: self.id,
            presentation: self.presentation.copied(),
        }
    }
}

#[derive(Debug)]
struct RevisionState {
    revision_id: RevisionId,
    modules: HashMap<ModuleId, Module>,
    committed_module_ids: HashSet<ModuleId>,
}

impl RevisionState {
    fn copied_modules(&self) -> HashMap<ModuleId, Module> {
        self.modules
            .iter()
            .map(|(&id, module)| (id, module.copied()))
            .collect()
    }
}

/// Structural failures defined by `MNIR-CORE-040` and `MNIR-CORE-041`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuralError {
    ModuleIdentityMismatch {
        collection_id: ModuleId,
        module_id: ModuleId,
    },
    ModuleIdentityNotCommitted(ModuleId),
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

/// Immutable observation of one committed Program revision.
///
/// A snapshot deliberately exposes no mutation entry point. Continuing from
/// a historical snapshot requires [`ProgramSnapshot::fork`]
/// (`MNIR-CORE-059`, `AR-CORE-015`).
///
/// ```compile_fail
/// use mnir_core::MnirProgram;
///
/// let program = MnirProgram::new().unwrap();
/// let mut snapshot = program.snapshot();
/// let _transaction = snapshot.begin_transaction();
/// ```
#[derive(Clone, Debug)]
pub struct ProgramSnapshot {
    program_id: ProgramId,
    state: Arc<RevisionState>,
}

impl ProgramSnapshot {
    #[must_use]
    pub const fn program_id(&self) -> ProgramId {
        self.program_id
    }

    #[must_use]
    pub fn revision_id(&self) -> RevisionId {
        self.state.revision_id
    }

    #[must_use]
    pub fn module_count(&self) -> usize {
        self.state.modules.len()
    }

    #[must_use]
    pub fn module(&self, id: ModuleId) -> Option<&Module> {
        self.state.modules.get(&id)
    }

    pub fn validate_structure(&self) -> Result<(), StructuralError> {
        validate_revision_state(&self.state)
    }

    /// Creates a distinct Program lineage from this snapshot.
    ///
    /// This implementation preserves the raw Module ID values while giving
    /// the fork a new Program ID (`MNIR-CORE-010`, `MNIR-CORE-017`).
    pub fn fork(&self) -> Result<MnirProgram, MutationError> {
        let program_id = allocate_program_id()?;
        let modules = self.state.copied_modules();
        let committed_module_ids = modules.keys().copied().collect();

        Ok(MnirProgram {
            program_id,
            head: Arc::new(RevisionState {
                revision_id: RevisionId(1),
                modules,
                committed_module_ids,
            }),
            next_revision_raw: Some(2),
            next_module_raw: Some(1),
        })
    }
}

/// Mutable, linear Program lineage and its current head revision.
#[derive(Debug)]
pub struct MnirProgram {
    program_id: ProgramId,
    head: Arc<RevisionState>,
    next_revision_raw: Option<u64>,
    next_module_raw: Option<u64>,
}

impl MnirProgram {
    /// Constructs an empty, structurally valid Program and its initial
    /// committed revision (`AR-CORE-001`).
    pub fn new() -> Result<Self, MutationError> {
        Ok(Self {
            program_id: allocate_program_id()?,
            head: Arc::new(RevisionState {
                revision_id: RevisionId(1),
                modules: HashMap::new(),
                committed_module_ids: HashSet::new(),
            }),
            next_revision_raw: Some(2),
            next_module_raw: Some(1),
        })
    }

    #[must_use]
    pub const fn program_id(&self) -> ProgramId {
        self.program_id
    }

    #[must_use]
    pub fn revision_id(&self) -> RevisionId {
        self.head.revision_id
    }

    #[must_use]
    pub fn module_count(&self) -> usize {
        self.head.modules.len()
    }

    #[must_use]
    pub fn module(&self, id: ModuleId) -> Option<&Module> {
        self.head.modules.get(&id)
    }

    #[must_use]
    pub fn committed_module_id_count(&self) -> usize {
        self.head.committed_module_ids.len()
    }

    #[must_use]
    pub fn is_module_id_committed(&self, id: ModuleId) -> bool {
        self.head.committed_module_ids.contains(&id)
    }

    #[must_use]
    pub fn snapshot(&self) -> ProgramSnapshot {
        ProgramSnapshot {
            program_id: self.program_id,
            state: Arc::clone(&self.head),
        }
    }

    pub fn validate_structure(&self) -> Result<(), StructuralError> {
        validate_revision_state(&self.head)
    }

    #[must_use]
    pub fn begin_transaction(&mut self) -> MutationTransaction<'_> {
        MutationTransaction {
            source_revision_id: self.head.revision_id,
            working_modules: self.head.copied_modules(),
            provisional_module_ids: HashSet::new(),
            next_module_raw: self.next_module_raw,
            lineage: self,
            state: TransactionState::Active,
        }
    }
}

/// A controlled set of sequential changes against one Program head revision.
pub struct MutationTransaction<'program> {
    lineage: &'program mut MnirProgram,
    source_revision_id: RevisionId,
    working_modules: HashMap<ModuleId, Module>,
    provisional_module_ids: HashSet<ModuleId>,
    next_module_raw: Option<u64>,
    state: TransactionState,
}

impl fmt::Debug for MutationTransaction<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MutationTransaction")
            .field("program_id", &self.lineage.program_id)
            .field("source_revision_id", &self.source_revision_id)
            .field("state", &self.state)
            .field("module_count", &self.working_modules.len())
            .finish_non_exhaustive()
    }
}

impl MutationTransaction<'_> {
    #[must_use]
    pub const fn state(&self) -> TransactionState {
        self.state
    }

    #[must_use]
    pub const fn source_revision_id(&self) -> RevisionId {
        self.source_revision_id
    }

    #[must_use]
    pub fn module_count(&self) -> usize {
        self.working_modules.len()
    }

    #[must_use]
    pub fn module(&self, id: ModuleId) -> Option<&Module> {
        self.working_modules.get(&id)
    }

    #[must_use]
    pub fn is_module_id_provisional(&self, id: ModuleId) -> bool {
        self.state == TransactionState::Active && self.provisional_module_ids.contains(&id)
    }

    pub fn add_module(&mut self) -> Result<ModuleId, MutationError> {
        self.require_active()?;

        let id = match self.allocate_module_id() {
            Ok(id) => id,
            Err(error) => return Err(self.fail(error)),
        };

        self.working_modules.insert(id, Module::new(id));
        self.provisional_module_ids.insert(id);
        Ok(id)
    }

    pub fn remove_module(&mut self, id: ModuleId) -> Result<(), MutationError> {
        self.require_active()?;

        if self.working_modules.remove(&id).is_none() {
            return Err(self.fail(MutationError::UnknownModule(id)));
        }

        Ok(())
    }

    pub fn set_module_preferred_name(
        &mut self,
        id: ModuleId,
        preferred_name: Option<String>,
    ) -> Result<(), MutationError> {
        self.require_active()?;

        let Some(module) = self.working_modules.get_mut(&id) else {
            return Err(self.fail(MutationError::UnknownModule(id)));
        };

        module.presentation.preferred_name = preferred_name;
        Ok(())
    }

    pub fn set_module_documentation(
        &mut self,
        id: ModuleId,
        documentation: Option<String>,
    ) -> Result<(), MutationError> {
        self.require_active()?;

        let Some(module) = self.working_modules.get_mut(&id) else {
            return Err(self.fail(MutationError::UnknownModule(id)));
        };

        module.presentation.documentation = documentation;
        Ok(())
    }

    pub fn commit(&mut self) -> Result<ProgramSnapshot, MutationError> {
        self.require_active()?;

        if self.lineage.head.revision_id != self.source_revision_id {
            let error = MutationError::SourceRevisionChanged {
                expected: self.source_revision_id,
                actual: self.lineage.head.revision_id,
            };
            return Err(self.fail(error));
        }

        let mut committed_module_ids = self.lineage.head.committed_module_ids.clone();
        committed_module_ids.extend(self.provisional_module_ids.iter().copied());

        if let Err(error) = validate_modules(&self.working_modules, &committed_module_ids) {
            return Err(self.fail(MutationError::StructuralViolation(error)));
        }

        let revision_id = match allocate_from_cursor(
            &mut self.lineage.next_revision_raw,
            IdentifierCategory::Revision,
        ) {
            Ok(raw) => RevisionId(raw),
            Err(error) => return Err(self.fail(error)),
        };

        let modules = mem::take(&mut self.working_modules);
        self.lineage.head = Arc::new(RevisionState {
            revision_id,
            modules,
            committed_module_ids,
        });
        self.lineage.next_module_raw = self.next_module_raw;
        self.state = TransactionState::Committed;

        Ok(self.lineage.snapshot())
    }

    pub fn discard(&mut self) -> Result<(), MutationError> {
        match self.state {
            TransactionState::Active | TransactionState::Failed => {
                self.state = TransactionState::Discarded;
                Ok(())
            }
            state => Err(MutationError::TransactionNotActive(state)),
        }
    }

    fn require_active(&self) -> Result<(), MutationError> {
        if self.state == TransactionState::Active {
            Ok(())
        } else {
            Err(MutationError::TransactionNotActive(self.state))
        }
    }

    fn fail(&mut self, error: MutationError) -> MutationError {
        self.state = TransactionState::Failed;
        error
    }

    fn allocate_module_id(&mut self) -> Result<ModuleId, MutationError> {
        loop {
            let raw = allocate_from_cursor(&mut self.next_module_raw, IdentifierCategory::Module)?;
            let candidate = ModuleId(raw);

            if !self.lineage.head.committed_module_ids.contains(&candidate)
                && !self.provisional_module_ids.contains(&candidate)
            {
                return Ok(candidate);
            }
        }
    }
}

fn allocate_program_id() -> Result<ProgramId, MutationError> {
    NEXT_PROGRAM_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map(ProgramId)
        .map_err(|_| MutationError::IdentifierExhausted(IdentifierCategory::Program))
}

fn allocate_from_cursor(
    cursor: &mut Option<u64>,
    category: IdentifierCategory,
) -> Result<u64, MutationError> {
    let raw = cursor.ok_or(MutationError::IdentifierExhausted(category))?;
    *cursor = raw.checked_add(1);
    Ok(raw)
}

fn validate_revision_state(state: &RevisionState) -> Result<(), StructuralError> {
    validate_modules(&state.modules, &state.committed_module_ids)
}

fn validate_modules(
    modules: &HashMap<ModuleId, Module>,
    committed_module_ids: &HashSet<ModuleId>,
) -> Result<(), StructuralError> {
    for (&collection_id, module) in modules {
        if collection_id != module.id {
            return Err(StructuralError::ModuleIdentityMismatch {
                collection_id,
                module_id: module.id,
            });
        }

        if !committed_module_ids.contains(&collection_id) {
            return Err(StructuralError::ModuleIdentityNotCommitted(collection_id));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // MNIR-CORE-035 through MNIR-CORE-037 and MNIR-CORE-042.
    #[test]
    fn structurally_invalid_internal_candidate_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let source_program_id = program.program_id();
        let source_revision_id = program.revision_id();
        let source_module_count = program.module_count();

        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        let module = transaction.working_modules.remove(&module_id).unwrap();
        let mismatched_collection_id = ModuleId(module_id.0 + 1);
        transaction
            .working_modules
            .insert(mismatched_collection_id, module);

        assert!(matches!(
            transaction.commit(),
            Err(MutationError::StructuralViolation(
                StructuralError::ModuleIdentityMismatch { .. }
            ))
        ));
        assert_eq!(transaction.state(), TransactionState::Failed);
        drop(transaction);

        assert_eq!(program.program_id(), source_program_id);
        assert_eq!(program.revision_id(), source_revision_id);
        assert_eq!(program.module_count(), source_module_count);
        assert!(program.validate_structure().is_ok());
    }
}
