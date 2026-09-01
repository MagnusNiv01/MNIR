use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::IntrinsicType;
use crate::ids::{FunctionId, IdentifierCategory, ModuleId, ParameterId, ProgramId, RevisionId};

static NEXT_PROGRAM_ID: AtomicU64 = AtomicU64::new(1);

/// Optional human-oriented data associated with a Module, Function, or Parameter.
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

/// One typed entry in a Function's ordered Parameter sequence
/// (`MNIR-FUNC-020`, `MNIR-FUNC-025`).
#[derive(Debug, Eq, PartialEq)]
pub struct Parameter {
    id: ParameterId,
    intrinsic_type: IntrinsicType,
    presentation: PresentationMetadata,
}

impl Parameter {
    fn new(id: ParameterId, intrinsic_type: IntrinsicType) -> Self {
        Self {
            id,
            intrinsic_type,
            presentation: PresentationMetadata::default(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> ParameterId {
        self.id
    }

    #[must_use]
    pub const fn intrinsic_type(&self) -> &IntrinsicType {
        &self.intrinsic_type
    }

    #[must_use]
    pub const fn presentation(&self) -> &PresentationMetadata {
        &self.presentation
    }

    fn copied(&self) -> Self {
        Self {
            id: self.id,
            intrinsic_type: copy_intrinsic_type(&self.intrinsic_type),
            presentation: self.presentation.copied(),
        }
    }
}

/// A Function signature owned by exactly one Module (`MNIR-FUNC-006`).
#[derive(Debug, Eq, PartialEq)]
pub struct Function {
    id: FunctionId,
    return_type: IntrinsicType,
    parameters: Vec<Parameter>,
    presentation: PresentationMetadata,
}

impl Function {
    fn new(id: FunctionId, return_type: IntrinsicType) -> Self {
        Self {
            id,
            return_type,
            parameters: Vec::new(),
            presentation: PresentationMetadata::default(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> FunctionId {
        self.id
    }

    #[must_use]
    pub const fn return_type(&self) -> &IntrinsicType {
        &self.return_type
    }

    #[must_use]
    pub const fn presentation(&self) -> &PresentationMetadata {
        &self.presentation
    }

    #[must_use]
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    #[must_use]
    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    #[must_use]
    pub fn parameter(&self, id: ParameterId) -> Option<&Parameter> {
        self.parameters.iter().find(|parameter| parameter.id == id)
    }

    fn copied(&self) -> Self {
        Self {
            id: self.id,
            return_type: copy_intrinsic_type(&self.return_type),
            parameters: self.parameters.iter().map(Parameter::copied).collect(),
            presentation: self.presentation.copied(),
        }
    }
}

/// A Module in the canonical MNIR Program representation.
#[derive(Debug, Eq, PartialEq)]
pub struct Module {
    id: ModuleId,
    functions: HashMap<FunctionId, Function>,
    presentation: PresentationMetadata,
}

impl Module {
    fn new(id: ModuleId) -> Self {
        Self {
            id,
            functions: HashMap::new(),
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

    #[must_use]
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    #[must_use]
    pub fn function(&self, id: FunctionId) -> Option<&Function> {
        self.functions.get(&id)
    }

    /// Iterates Functions without assigning semantic meaning to iteration order
    /// (`MNIR-FUNC-075`, `MNIR-FUNC-076`).
    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.functions.values()
    }

    fn copied(&self) -> Self {
        Self {
            id: self.id,
            functions: self
                .functions
                .iter()
                .map(|(&id, function)| (id, function.copied()))
                .collect(),
            presentation: self.presentation.copied(),
        }
    }
}

#[derive(Debug)]
struct RevisionState {
    revision_id: RevisionId,
    modules: HashMap<ModuleId, Module>,
    committed_module_ids: HashSet<ModuleId>,
    committed_function_ids: HashSet<FunctionId>,
    committed_parameter_ids: HashSet<ParameterId>,
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

    #[must_use]
    pub fn function(&self, id: FunctionId) -> Option<&Function> {
        find_function(&self.state.modules, id)
    }

    #[must_use]
    pub fn parameter(&self, id: ParameterId) -> Option<&Parameter> {
        find_parameter(&self.state.modules, id)
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
        let committed_function_ids = modules
            .values()
            .flat_map(|module| module.functions.keys().copied())
            .collect();
        let committed_parameter_ids = modules
            .values()
            .flat_map(|module| module.functions.values())
            .flat_map(|function| function.parameters.iter().map(Parameter::id))
            .collect();

        Ok(MnirProgram {
            program_id,
            head: Arc::new(RevisionState {
                revision_id: RevisionId(1),
                modules,
                committed_module_ids,
                committed_function_ids,
                committed_parameter_ids,
            }),
            next_revision_raw: Some(2),
            next_module_raw: Some(1),
            next_function_raw: Some(1),
            next_parameter_raw: Some(1),
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
    next_function_raw: Option<u64>,
    next_parameter_raw: Option<u64>,
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
                committed_function_ids: HashSet::new(),
                committed_parameter_ids: HashSet::new(),
            }),
            next_revision_raw: Some(2),
            next_module_raw: Some(1),
            next_function_raw: Some(1),
            next_parameter_raw: Some(1),
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
    pub fn function(&self, id: FunctionId) -> Option<&Function> {
        find_function(&self.head.modules, id)
    }

    #[must_use]
    pub fn parameter(&self, id: ParameterId) -> Option<&Parameter> {
        find_parameter(&self.head.modules, id)
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
    pub fn committed_function_id_count(&self) -> usize {
        self.head.committed_function_ids.len()
    }

    #[must_use]
    pub fn is_function_id_committed(&self, id: FunctionId) -> bool {
        self.head.committed_function_ids.contains(&id)
    }

    #[must_use]
    pub fn committed_parameter_id_count(&self) -> usize {
        self.head.committed_parameter_ids.len()
    }

    #[must_use]
    pub fn is_parameter_id_committed(&self, id: ParameterId) -> bool {
        self.head.committed_parameter_ids.contains(&id)
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
            provisional_function_ids: HashSet::new(),
            provisional_parameter_ids: HashSet::new(),
            next_module_raw: self.next_module_raw,
            next_function_raw: self.next_function_raw,
            next_parameter_raw: self.next_parameter_raw,
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
    provisional_function_ids: HashSet<FunctionId>,
    provisional_parameter_ids: HashSet<ParameterId>,
    next_module_raw: Option<u64>,
    next_function_raw: Option<u64>,
    next_parameter_raw: Option<u64>,
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
    pub fn function(&self, id: FunctionId) -> Option<&Function> {
        find_function(&self.working_modules, id)
    }

    #[must_use]
    pub fn parameter(&self, id: ParameterId) -> Option<&Parameter> {
        find_parameter(&self.working_modules, id)
    }

    #[must_use]
    pub fn is_module_id_provisional(&self, id: ModuleId) -> bool {
        self.state == TransactionState::Active && self.provisional_module_ids.contains(&id)
    }

    #[must_use]
    pub fn is_function_id_provisional(&self, id: FunctionId) -> bool {
        self.state == TransactionState::Active && self.provisional_function_ids.contains(&id)
    }

    #[must_use]
    pub fn is_parameter_id_provisional(&self, id: ParameterId) -> bool {
        self.state == TransactionState::Active && self.provisional_parameter_ids.contains(&id)
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

    /// Adds a signature-only Function to a Module (`MNIR-FUNC-001`).
    pub fn add_function(
        &mut self,
        module_id: ModuleId,
        return_type: IntrinsicType,
    ) -> Result<FunctionId, MutationError> {
        self.require_active()?;

        if !self.working_modules.contains_key(&module_id) {
            return Err(self.fail(MutationError::UnknownModule(module_id)));
        }

        let id = match self.allocate_function_id() {
            Ok(id) => id,
            Err(error) => return Err(self.fail(error)),
        };
        let Some(module) = self.working_modules.get_mut(&module_id) else {
            return Err(self.fail(MutationError::UnknownModule(module_id)));
        };
        module.functions.insert(id, Function::new(id, return_type));
        self.provisional_function_ids.insert(id);
        Ok(id)
    }

    pub fn remove_function(&mut self, id: FunctionId) -> Result<(), MutationError> {
        self.require_active()?;

        for module in self.working_modules.values_mut() {
            if module.functions.remove(&id).is_some() {
                return Ok(());
            }
        }

        Err(self.fail(MutationError::UnknownFunction(id)))
    }

    pub fn set_function_preferred_name(
        &mut self,
        id: FunctionId,
        preferred_name: Option<String>,
    ) -> Result<(), MutationError> {
        self.require_active()?;
        let Some(function) = find_function_mut(&mut self.working_modules, id) else {
            return Err(self.fail(MutationError::UnknownFunction(id)));
        };
        function.presentation.preferred_name = preferred_name;
        Ok(())
    }

    pub fn set_function_documentation(
        &mut self,
        id: FunctionId,
        documentation: Option<String>,
    ) -> Result<(), MutationError> {
        self.require_active()?;
        let Some(function) = find_function_mut(&mut self.working_modules, id) else {
            return Err(self.fail(MutationError::UnknownFunction(id)));
        };
        function.presentation.documentation = documentation;
        Ok(())
    }

    pub fn set_function_return_type(
        &mut self,
        id: FunctionId,
        return_type: IntrinsicType,
    ) -> Result<(), MutationError> {
        self.require_active()?;
        let Some(function) = find_function_mut(&mut self.working_modules, id) else {
            return Err(self.fail(MutationError::UnknownFunction(id)));
        };
        function.return_type = return_type;
        Ok(())
    }

    pub fn add_parameter(
        &mut self,
        function_id: FunctionId,
        intrinsic_type: IntrinsicType,
    ) -> Result<ParameterId, MutationError> {
        self.require_active()?;

        if find_function(&self.working_modules, function_id).is_none() {
            return Err(self.fail(MutationError::UnknownFunction(function_id)));
        }

        let id = match self.allocate_parameter_id() {
            Ok(id) => id,
            Err(error) => return Err(self.fail(error)),
        };
        let Some(function) = find_function_mut(&mut self.working_modules, function_id) else {
            return Err(self.fail(MutationError::UnknownFunction(function_id)));
        };
        function.parameters.push(Parameter::new(id, intrinsic_type));
        self.provisional_parameter_ids.insert(id);
        Ok(id)
    }

    pub fn remove_parameter(&mut self, id: ParameterId) -> Result<(), MutationError> {
        self.require_active()?;

        for function in self
            .working_modules
            .values_mut()
            .flat_map(|module| module.functions.values_mut())
        {
            if let Some(index) = function
                .parameters
                .iter()
                .position(|parameter| parameter.id == id)
            {
                function.parameters.remove(index);
                return Ok(());
            }
        }

        Err(self.fail(MutationError::UnknownParameter(id)))
    }

    pub fn set_parameter_preferred_name(
        &mut self,
        id: ParameterId,
        preferred_name: Option<String>,
    ) -> Result<(), MutationError> {
        self.require_active()?;
        let Some(parameter) = find_parameter_mut(&mut self.working_modules, id) else {
            return Err(self.fail(MutationError::UnknownParameter(id)));
        };
        parameter.presentation.preferred_name = preferred_name;
        Ok(())
    }

    pub fn set_parameter_documentation(
        &mut self,
        id: ParameterId,
        documentation: Option<String>,
    ) -> Result<(), MutationError> {
        self.require_active()?;
        let Some(parameter) = find_parameter_mut(&mut self.working_modules, id) else {
            return Err(self.fail(MutationError::UnknownParameter(id)));
        };
        parameter.presentation.documentation = documentation;
        Ok(())
    }

    pub fn set_parameter_type(
        &mut self,
        id: ParameterId,
        intrinsic_type: IntrinsicType,
    ) -> Result<(), MutationError> {
        self.require_active()?;
        let Some(parameter) = find_parameter_mut(&mut self.working_modules, id) else {
            return Err(self.fail(MutationError::UnknownParameter(id)));
        };
        parameter.intrinsic_type = intrinsic_type;
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
        let mut committed_function_ids = self.lineage.head.committed_function_ids.clone();
        committed_function_ids.extend(self.provisional_function_ids.iter().copied());
        let mut committed_parameter_ids = self.lineage.head.committed_parameter_ids.clone();
        committed_parameter_ids.extend(self.provisional_parameter_ids.iter().copied());

        if let Err(error) = validate_modules(
            &self.working_modules,
            &committed_module_ids,
            &committed_function_ids,
            &committed_parameter_ids,
        ) {
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
            committed_function_ids,
            committed_parameter_ids,
        });
        self.lineage.next_module_raw = self.next_module_raw;
        self.lineage.next_function_raw = self.next_function_raw;
        self.lineage.next_parameter_raw = self.next_parameter_raw;
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

    fn allocate_function_id(&mut self) -> Result<FunctionId, MutationError> {
        loop {
            let raw =
                allocate_from_cursor(&mut self.next_function_raw, IdentifierCategory::Function)?;
            let candidate = FunctionId(raw);

            if !self
                .lineage
                .head
                .committed_function_ids
                .contains(&candidate)
                && !self.provisional_function_ids.contains(&candidate)
            {
                return Ok(candidate);
            }
        }
    }

    fn allocate_parameter_id(&mut self) -> Result<ParameterId, MutationError> {
        loop {
            let raw =
                allocate_from_cursor(&mut self.next_parameter_raw, IdentifierCategory::Parameter)?;
            let candidate = ParameterId(raw);

            if !self
                .lineage
                .head
                .committed_parameter_ids
                .contains(&candidate)
                && !self.provisional_parameter_ids.contains(&candidate)
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
    validate_modules(
        &state.modules,
        &state.committed_module_ids,
        &state.committed_function_ids,
        &state.committed_parameter_ids,
    )
}

fn validate_modules(
    modules: &HashMap<ModuleId, Module>,
    committed_module_ids: &HashSet<ModuleId>,
    committed_function_ids: &HashSet<FunctionId>,
    committed_parameter_ids: &HashSet<ParameterId>,
) -> Result<(), StructuralError> {
    let mut seen_function_ids = HashSet::new();
    let mut seen_parameter_ids = HashSet::new();

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

        for (&function_collection_id, function) in &module.functions {
            if function_collection_id != function.id {
                return Err(StructuralError::FunctionIdentityMismatch {
                    collection_id: function_collection_id,
                    function_id: function.id,
                });
            }
            if !committed_function_ids.contains(&function.id) {
                return Err(StructuralError::FunctionIdentityNotCommitted(function.id));
            }
            if !seen_function_ids.insert(function.id) {
                return Err(StructuralError::DuplicateFunctionIdentity(function.id));
            }

            for parameter in &function.parameters {
                if !committed_parameter_ids.contains(&parameter.id) {
                    return Err(StructuralError::ParameterIdentityNotCommitted(parameter.id));
                }
                if !seen_parameter_ids.insert(parameter.id) {
                    return Err(StructuralError::DuplicateParameterIdentity(parameter.id));
                }
            }
        }
    }

    Ok(())
}

fn find_function(modules: &HashMap<ModuleId, Module>, id: FunctionId) -> Option<&Function> {
    modules
        .values()
        .find_map(|module| module.functions.get(&id))
}

fn find_function_mut(
    modules: &mut HashMap<ModuleId, Module>,
    id: FunctionId,
) -> Option<&mut Function> {
    modules
        .values_mut()
        .find_map(|module| module.functions.get_mut(&id))
}

fn find_parameter(modules: &HashMap<ModuleId, Module>, id: ParameterId) -> Option<&Parameter> {
    modules
        .values()
        .flat_map(|module| module.functions.values())
        .find_map(|function| function.parameter(id))
}

fn find_parameter_mut(
    modules: &mut HashMap<ModuleId, Module>,
    id: ParameterId,
) -> Option<&mut Parameter> {
    modules
        .values_mut()
        .flat_map(|module| module.functions.values_mut())
        .find_map(|function| {
            function
                .parameters
                .iter_mut()
                .find(|parameter| parameter.id == id)
        })
}

fn copy_intrinsic_type(intrinsic_type: &IntrinsicType) -> IntrinsicType {
    match intrinsic_type {
        IntrinsicType::Int32 => IntrinsicType::Int32,
        IntrinsicType::Int64 => IntrinsicType::Int64,
        IntrinsicType::Bool => IntrinsicType::Bool,
        IntrinsicType::Unit => IntrinsicType::Unit,
    }
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

    // AR-FUNC-034 and MNIR-FUNC-041 through MNIR-FUNC-043.
    #[test]
    fn structurally_invalid_function_candidate_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        transaction.commit().unwrap();
        let source_revision_id = program.revision_id();

        let mut transaction = program.begin_transaction();
        let function_id = transaction
            .add_function(module_id, IntrinsicType::Unit)
            .unwrap();
        let function = transaction
            .working_modules
            .get_mut(&module_id)
            .unwrap()
            .functions
            .remove(&function_id)
            .unwrap();
        let mismatched_collection_id = FunctionId(function_id.0 + 1);
        transaction
            .working_modules
            .get_mut(&module_id)
            .unwrap()
            .functions
            .insert(mismatched_collection_id, function);

        assert!(matches!(
            transaction.commit(),
            Err(MutationError::StructuralViolation(
                StructuralError::FunctionIdentityMismatch { .. }
            ))
        ));
        assert_eq!(transaction.state(), TransactionState::Failed);
        drop(transaction);

        assert_eq!(program.revision_id(), source_revision_id);
        assert_eq!(program.module(module_id).unwrap().function_count(), 0);
        assert!(program.validate_structure().is_ok());
    }

    // MNIR-FUNC-016, MNIR-FUNC-017, MNIR-FUNC-053, and MNIR-FUNC-054.
    #[test]
    fn duplicate_parameter_ownership_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let module_id = transaction.add_module().unwrap();
        let first_function = transaction
            .add_function(module_id, IntrinsicType::Unit)
            .unwrap();
        transaction
            .add_parameter(first_function, IntrinsicType::Bool)
            .unwrap();
        transaction.commit().unwrap();
        let source_revision_id = program.revision_id();

        let mut transaction = program.begin_transaction();
        let second_function = transaction
            .add_function(module_id, IntrinsicType::Unit)
            .unwrap();
        let duplicated_parameter = transaction
            .working_modules
            .get(&module_id)
            .unwrap()
            .functions
            .get(&first_function)
            .unwrap()
            .parameters[0]
            .copied();
        transaction
            .working_modules
            .get_mut(&module_id)
            .unwrap()
            .functions
            .get_mut(&second_function)
            .unwrap()
            .parameters
            .push(duplicated_parameter);

        assert!(matches!(
            transaction.commit(),
            Err(MutationError::StructuralViolation(
                StructuralError::DuplicateParameterIdentity(_)
            ))
        ));
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision_id);
        assert!(program.function(second_function).is_none());
        assert!(program.validate_structure().is_ok());
    }

    // MNIR-FUNC-002, MNIR-FUNC-006, MNIR-FUNC-007, and MNIR-FUNC-054.
    #[test]
    fn duplicate_function_ownership_cannot_commit() {
        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let first_module = transaction.add_module().unwrap();
        let second_module = transaction.add_module().unwrap();
        let function_id = transaction
            .add_function(first_module, IntrinsicType::Unit)
            .unwrap();
        transaction.commit().unwrap();
        let source_revision_id = program.revision_id();

        let mut transaction = program.begin_transaction();
        let duplicated_function = transaction
            .working_modules
            .get(&first_module)
            .unwrap()
            .functions
            .get(&function_id)
            .unwrap()
            .copied();
        transaction
            .working_modules
            .get_mut(&second_module)
            .unwrap()
            .functions
            .insert(function_id, duplicated_function);

        assert!(matches!(
            transaction.commit(),
            Err(MutationError::StructuralViolation(
                StructuralError::DuplicateFunctionIdentity(_)
            ))
        ));
        drop(transaction);
        assert_eq!(program.revision_id(), source_revision_id);
        assert_eq!(program.module(second_module).unwrap().function_count(), 0);
        assert!(program.validate_structure().is_ok());
    }
}
