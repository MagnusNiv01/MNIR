use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::IntrinsicType;
use crate::ids::{
    BlockId, ExpressionId, FunctionId, IdentifierCategory, ModuleId, ParameterId, ProgramId,
    RevisionId,
};
use crate::presentation::PresentationMetadata;

use super::body::{Block, Expression, ExpressionKind, FunctionBody};
use super::error::{ExpressionTypeError, MutationError, StructuralError};
use super::validation::validate_revision_state;

static NEXT_PROGRAM_ID: AtomicU64 = AtomicU64::new(1);

/// One typed entry in a Function's ordered Parameter sequence
/// (`MNIR-FUNC-020`, `MNIR-FUNC-025`).
#[derive(Debug, Eq, PartialEq)]
pub struct Parameter {
    pub(super) id: ParameterId,
    pub(super) intrinsic_type: IntrinsicType,
    pub(super) presentation: PresentationMetadata,
}

impl Parameter {
    pub(super) fn new(id: ParameterId, intrinsic_type: IntrinsicType) -> Self {
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

    pub(super) fn copied(&self) -> Self {
        Self {
            id: self.id,
            intrinsic_type: self.intrinsic_type.copied(),
            presentation: self.presentation.copied(),
        }
    }
}

/// A Function signature owned by exactly one Module (`MNIR-FUNC-006`).
#[derive(Debug, Eq, PartialEq)]
pub struct Function {
    pub(super) id: FunctionId,
    pub(super) return_type: IntrinsicType,
    pub(super) parameters: Vec<Parameter>,
    pub(super) body: Option<FunctionBody>,
    pub(super) presentation: PresentationMetadata,
}

impl Function {
    pub(super) fn new(id: FunctionId, return_type: IntrinsicType) -> Self {
        Self {
            id,
            return_type,
            parameters: Vec::new(),
            body: None,
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

    #[must_use]
    pub const fn has_body(&self) -> bool {
        self.body.is_some()
    }

    #[must_use]
    pub const fn body(&self) -> Option<&FunctionBody> {
        self.body.as_ref()
    }

    pub(super) fn copied(&self) -> Self {
        Self {
            id: self.id,
            return_type: self.return_type.copied(),
            parameters: self.parameters.iter().map(Parameter::copied).collect(),
            body: self.body.as_ref().map(FunctionBody::copied),
            presentation: self.presentation.copied(),
        }
    }
}

/// A Module in the canonical MNIR Program representation.
#[derive(Debug, Eq, PartialEq)]
pub struct Module {
    pub(super) id: ModuleId,
    pub(super) functions: HashMap<FunctionId, Function>,
    pub(super) presentation: PresentationMetadata,
}

impl Module {
    pub(super) fn new(id: ModuleId) -> Self {
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

    pub(super) fn copied(&self) -> Self {
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
pub(super) struct RevisionState {
    pub(super) revision_id: RevisionId,
    pub(super) modules: HashMap<ModuleId, Module>,
    pub(super) committed_module_ids: HashSet<ModuleId>,
    pub(super) committed_function_ids: HashSet<FunctionId>,
    pub(super) committed_parameter_ids: HashSet<ParameterId>,
    pub(super) committed_block_ids: HashSet<BlockId>,
    pub(super) committed_expression_ids: HashSet<ExpressionId>,
}

impl RevisionState {
    pub(super) fn copied_modules(&self) -> HashMap<ModuleId, Module> {
        self.modules
            .iter()
            .map(|(&id, module)| (id, module.copied()))
            .collect()
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
    pub(super) program_id: ProgramId,
    pub(super) state: Arc<RevisionState>,
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

    #[must_use]
    pub fn block(&self, id: BlockId) -> Option<&Block> {
        find_block(&self.state.modules, id)
    }

    #[must_use]
    pub fn expression(&self, id: ExpressionId) -> Option<&Expression> {
        find_expression(&self.state.modules, id)
    }

    #[must_use]
    pub fn expression_type(
        &self,
        id: ExpressionId,
    ) -> Option<Result<IntrinsicType, ExpressionTypeError>> {
        derive_expression_type(&self.state.modules, id)
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
        let committed_block_ids = modules
            .values()
            .flat_map(|module| module.functions.values())
            .filter_map(|function| function.body().map(FunctionBody::block_id))
            .collect();
        let committed_expression_ids = modules
            .values()
            .flat_map(|module| module.functions.values())
            .filter_map(Function::body)
            .flat_map(|body| body.block().expressions().map(Expression::id))
            .collect();

        Ok(MnirProgram {
            program_id,
            head: Arc::new(RevisionState {
                revision_id: RevisionId(1),
                modules,
                committed_module_ids,
                committed_function_ids,
                committed_parameter_ids,
                committed_block_ids,
                committed_expression_ids,
            }),
            next_revision_raw: Some(2),
            next_module_raw: Some(1),
            next_function_raw: Some(1),
            next_parameter_raw: Some(1),
            next_block_raw: Some(1),
            next_expression_raw: Some(1),
        })
    }
}

/// Mutable, linear Program lineage and its current head revision.
#[derive(Debug)]
pub struct MnirProgram {
    pub(super) program_id: ProgramId,
    pub(super) head: Arc<RevisionState>,
    pub(super) next_revision_raw: Option<u64>,
    pub(super) next_module_raw: Option<u64>,
    pub(super) next_function_raw: Option<u64>,
    pub(super) next_parameter_raw: Option<u64>,
    pub(super) next_block_raw: Option<u64>,
    pub(super) next_expression_raw: Option<u64>,
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
                committed_block_ids: HashSet::new(),
                committed_expression_ids: HashSet::new(),
            }),
            next_revision_raw: Some(2),
            next_module_raw: Some(1),
            next_function_raw: Some(1),
            next_parameter_raw: Some(1),
            next_block_raw: Some(1),
            next_expression_raw: Some(1),
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
    pub fn block(&self, id: BlockId) -> Option<&Block> {
        find_block(&self.head.modules, id)
    }

    #[must_use]
    pub fn expression(&self, id: ExpressionId) -> Option<&Expression> {
        find_expression(&self.head.modules, id)
    }

    #[must_use]
    pub fn expression_type(
        &self,
        id: ExpressionId,
    ) -> Option<Result<IntrinsicType, ExpressionTypeError>> {
        derive_expression_type(&self.head.modules, id)
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
    pub fn committed_block_id_count(&self) -> usize {
        self.head.committed_block_ids.len()
    }

    #[must_use]
    pub fn is_block_id_committed(&self, id: BlockId) -> bool {
        self.head.committed_block_ids.contains(&id)
    }

    #[must_use]
    pub fn committed_expression_id_count(&self) -> usize {
        self.head.committed_expression_ids.len()
    }

    #[must_use]
    pub fn is_expression_id_committed(&self, id: ExpressionId) -> bool {
        self.head.committed_expression_ids.contains(&id)
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
}

pub(super) fn find_function(
    modules: &HashMap<ModuleId, Module>,
    id: FunctionId,
) -> Option<&Function> {
    modules
        .values()
        .find_map(|module| module.functions.get(&id))
}

pub(super) fn find_function_mut(
    modules: &mut HashMap<ModuleId, Module>,
    id: FunctionId,
) -> Option<&mut Function> {
    modules
        .values_mut()
        .find_map(|module| module.functions.get_mut(&id))
}

pub(super) fn find_parameter(
    modules: &HashMap<ModuleId, Module>,
    id: ParameterId,
) -> Option<&Parameter> {
    modules
        .values()
        .flat_map(|module| module.functions.values())
        .find_map(|function| function.parameter(id))
}

pub(super) fn find_parameter_mut(
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

pub(super) fn find_block(modules: &HashMap<ModuleId, Module>, id: BlockId) -> Option<&Block> {
    modules
        .values()
        .flat_map(|module| module.functions.values())
        .filter_map(Function::body)
        .map(FunctionBody::block)
        .find(|block| block.id == id)
}

pub(super) fn find_block_mut(
    modules: &mut HashMap<ModuleId, Module>,
    id: BlockId,
) -> Option<&mut Block> {
    modules
        .values_mut()
        .flat_map(|module| module.functions.values_mut())
        .filter_map(|function| function.body.as_mut())
        .map(|body| &mut body.block)
        .find(|block| block.id == id)
}

pub(super) fn find_block_owner(
    modules: &HashMap<ModuleId, Module>,
    id: BlockId,
) -> Option<&Function> {
    modules
        .values()
        .flat_map(|module| module.functions.values())
        .find(|function| function.body().is_some_and(|body| body.block_id() == id))
}

pub(super) fn find_expression(
    modules: &HashMap<ModuleId, Module>,
    id: ExpressionId,
) -> Option<&Expression> {
    modules
        .values()
        .flat_map(|module| module.functions.values())
        .filter_map(Function::body)
        .find_map(|body| body.block().expression(id))
}

pub(super) fn derive_expression_type(
    modules: &HashMap<ModuleId, Module>,
    id: ExpressionId,
) -> Option<Result<IntrinsicType, ExpressionTypeError>> {
    let (function, expression) = modules
        .values()
        .flat_map(|module| module.functions.values())
        .find_map(|function| {
            function
                .body()
                .and_then(|body| body.block().expression(id))
                .map(|expression| (function, expression))
        })?;

    Some(match expression.kind {
        ExpressionKind::Int32Literal(_) => Ok(IntrinsicType::Int32),
        ExpressionKind::Int64Literal(_) => Ok(IntrinsicType::Int64),
        ExpressionKind::BoolLiteral(_) => Ok(IntrinsicType::Bool),
        ExpressionKind::UnitLiteral => Ok(IntrinsicType::Unit),
        ExpressionKind::ParameterReference(parameter_id) => function
            .parameter(parameter_id)
            .map(|parameter| parameter.intrinsic_type.copied())
            .ok_or(ExpressionTypeError::UnresolvedParameter {
                expression_id: id,
                parameter_id,
            }),
    })
}

fn allocate_program_id() -> Result<ProgramId, MutationError> {
    NEXT_PROGRAM_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map(ProgramId)
        .map_err(|_| MutationError::IdentifierExhausted(IdentifierCategory::Program))
}
