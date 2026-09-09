use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::IntrinsicType;
use crate::ids::{
    AllocationCounterState, AllocationNamespaceId, BlockId, ExpressionId, FunctionId,
    IdentifierCategory, ModuleId, ParameterId, ProgramId, RevisionId,
};
use crate::presentation::PresentationMetadata;

use super::body::{Block, Expression, ExpressionKind, FunctionBody};
use super::error::{ExpressionTypeError, MutationError, StructuralError};
use super::validation::validate_revision_state;

const IDENTITY_GENERATION_ATTEMPTS: usize = 32;

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
}

impl RevisionState {
    pub(super) fn copied_modules(&self) -> HashMap<ModuleId, Module> {
        self.modules
            .iter()
            .map(|(&id, module)| (id, module.copied()))
            .collect()
    }
}

/// The one active allocation authority owned by a mutable Program lineage.
///
/// This state is deliberately outside [`RevisionState`]: successful issuance
/// advances it even when semantic transaction state is later discarded
/// (`MNIR-PSI-022` through `MNIR-PSI-024`, `MNIR-PSI-046` through
/// `MNIR-PSI-051`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AllocationAuthorityState {
    namespace_id: AllocationNamespaceId,
    counter_state: AllocationCounterState,
}

impl AllocationAuthorityState {
    const fn new(namespace_id: AllocationNamespaceId) -> Self {
        Self {
            namespace_id,
            counter_state: AllocationCounterState::Available(1),
        }
    }

    pub(super) const fn namespace_id(self) -> AllocationNamespaceId {
        self.namespace_id
    }

    pub(super) const fn counter_state(self) -> AllocationCounterState {
        self.counter_state
    }

    /// Atomically reserves the current counter and advances authoritative
    /// in-memory allocator state (`MNIR-PSI-032` through `MNIR-PSI-037`).
    pub(super) fn reserve(
        &mut self,
        category: IdentifierCategory,
    ) -> Result<(AllocationNamespaceId, u64), MutationError> {
        let AllocationCounterState::Available(counter) = self.counter_state else {
            return Err(MutationError::IdentifierExhausted(category));
        };

        self.counter_state = match counter.checked_add(1) {
            Some(next_counter) => AllocationCounterState::Available(next_counter),
            None => AllocationCounterState::Exhausted,
        };
        Ok((self.namespace_id, counter))
    }

    #[cfg(test)]
    pub(super) fn set_counter_state_for_test(&mut self, counter_state: AllocationCounterState) {
        self.counter_state = counter_state;
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
    pub(super) allocation_authority: AllocationAuthorityState,
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

    /// Returns the namespace observed when this immutable snapshot was made.
    #[must_use]
    pub const fn allocation_namespace_id(&self) -> AllocationNamespaceId {
        self.allocation_authority.namespace_id()
    }

    /// Returns the immutable counter-state observation paired with this
    /// snapshot (`MNIR-PSI-056`).
    #[must_use]
    pub const fn allocation_counter_state(&self) -> AllocationCounterState {
        self.allocation_authority.counter_state()
    }

    #[must_use]
    pub fn module_count(&self) -> usize {
        self.state.modules.len()
    }

    #[must_use]
    pub fn module(&self, id: ModuleId) -> Option<&Module> {
        self.state.modules.get(&id)
    }

    /// Iterates every Module in this immutable revision.
    ///
    /// Iteration order has no MNIR semantic meaning. This is the minimal
    /// read-only traversal entry point required by `MNIR-VERIFY-087` through
    /// `MNIR-VERIFY-089`.
    pub fn modules(&self) -> impl Iterator<Item = &Module> {
        self.state.modules.values()
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
    /// Inherited persistent entity identities and references are preserved
    /// exactly. The fork receives a fresh Program ID and allocation namespace
    /// (`MNIR-PSI-061` through `MNIR-PSI-070`).
    pub fn fork(&self) -> Result<MnirProgram, MutationError> {
        let program_id = allocate_program_id(&HashSet::from([self.program_id.0]))?;
        let modules = self.state.copied_modules();
        let known_namespaces = collect_present_namespaces(&modules, self.allocation_namespace_id());
        let namespace_id = allocate_namespace_id(&known_namespaces)?;

        Ok(MnirProgram {
            program_id,
            head: Arc::new(RevisionState {
                revision_id: RevisionId(1),
                modules,
            }),
            next_revision_raw: Some(2),
            allocation_authority: AllocationAuthorityState::new(namespace_id),
        })
    }
}

/// Mutable, linear Program lineage and its current head revision.
#[derive(Debug)]
pub struct MnirProgram {
    pub(super) program_id: ProgramId,
    pub(super) head: Arc<RevisionState>,
    pub(super) next_revision_raw: Option<u64>,
    pub(super) allocation_authority: AllocationAuthorityState,
}

impl MnirProgram {
    /// Constructs an empty, structurally valid Program and its initial
    /// committed revision (`AR-CORE-001`).
    pub fn new() -> Result<Self, MutationError> {
        let program_id = allocate_program_id(&HashSet::new())?;
        let namespace_id = allocate_namespace_id(&HashSet::new())?;
        Ok(Self {
            program_id,
            head: Arc::new(RevisionState {
                revision_id: RevisionId(1),
                modules: HashMap::new(),
            }),
            next_revision_raw: Some(2),
            allocation_authority: AllocationAuthorityState::new(namespace_id),
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
    pub const fn allocation_namespace_id(&self) -> AllocationNamespaceId {
        self.allocation_authority.namespace_id()
    }

    #[must_use]
    pub const fn allocation_counter_state(&self) -> AllocationCounterState {
        self.allocation_authority.counter_state()
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
    pub fn snapshot(&self) -> ProgramSnapshot {
        ProgramSnapshot {
            program_id: self.program_id,
            state: Arc::clone(&self.head),
            allocation_authority: self.allocation_authority,
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
        .flat_map(FunctionBody::blocks)
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
        .flat_map(|body| body.blocks.values_mut())
        .find(|block| block.id == id)
}

pub(super) fn find_block_owner(
    modules: &HashMap<ModuleId, Module>,
    id: BlockId,
) -> Option<&Function> {
    modules
        .values()
        .flat_map(|module| module.functions.values())
        .find(|function| {
            function
                .body()
                .is_some_and(|body| body.block_by_id(id).is_some())
        })
}

pub(super) fn find_expression(
    modules: &HashMap<ModuleId, Module>,
    id: ExpressionId,
) -> Option<&Expression> {
    modules
        .values()
        .flat_map(|module| module.functions.values())
        .filter_map(Function::body)
        .flat_map(FunctionBody::blocks)
        .find_map(|block| block.expression(id))
}

pub(super) fn derive_expression_type(
    modules: &HashMap<ModuleId, Module>,
    id: ExpressionId,
) -> Option<Result<IntrinsicType, ExpressionTypeError>> {
    let (function, block) = modules
        .values()
        .flat_map(|module| module.functions.values())
        .find_map(|function| {
            let block = function
                .body()?
                .blocks()
                .find(|block| block.expression(id).is_some())?;
            Some((function, block))
        })?;

    Some(derive_expression_type_in_block(
        modules,
        function,
        block,
        id,
        &mut HashMap::new(),
        &mut HashSet::new(),
    ))
}

fn derive_expression_type_in_block(
    modules: &HashMap<ModuleId, Module>,
    function: &Function,
    block: &Block,
    id: ExpressionId,
    memo: &mut HashMap<ExpressionId, Result<IntrinsicType, ExpressionTypeError>>,
    visiting: &mut HashSet<ExpressionId>,
) -> Result<IntrinsicType, ExpressionTypeError> {
    if let Some(result) = memo.get(&id) {
        return copy_expression_type_result(result);
    }
    if !visiting.insert(id) {
        return Err(ExpressionTypeError::OperandTypeUnavailable { expression_id: id });
    }

    let result = match block.expression(id).map(Expression::kind) {
        None => Err(ExpressionTypeError::OperandTypeUnavailable { expression_id: id }),
        Some(ExpressionKind::Int32Literal(_)) => Ok(IntrinsicType::Int32),
        Some(ExpressionKind::Int64Literal(_)) => Ok(IntrinsicType::Int64),
        Some(ExpressionKind::BoolLiteral(_)) => Ok(IntrinsicType::Bool),
        Some(ExpressionKind::UnitLiteral) => Ok(IntrinsicType::Unit),
        Some(ExpressionKind::ParameterReference(parameter_id)) => function
            .parameter(*parameter_id)
            .map(|parameter| parameter.intrinsic_type.copied())
            .ok_or(ExpressionTypeError::UnresolvedParameter {
                expression_id: id,
                parameter_id: *parameter_id,
            }),
        Some(ExpressionKind::Call { target, .. }) => find_function(modules, *target)
            .map(|target_function| target_function.return_type.copied())
            .ok_or(ExpressionTypeError::UnresolvedFunction {
                expression_id: id,
                function_id: *target,
            }),
        Some(
            ExpressionKind::Add { left, right }
            | ExpressionKind::Subtract { left, right }
            | ExpressionKind::Multiply { left, right }
            | ExpressionKind::Divide { left, right }
            | ExpressionKind::Remainder { left, right },
        ) => {
            let left_type =
                derive_expression_type_in_block(modules, function, block, *left, memo, visiting);
            let right_type =
                derive_expression_type_in_block(modules, function, block, *right, memo, visiting);
            derive_arithmetic_type(id, left_type, right_type)
        }
        Some(ExpressionKind::Equal { left, right } | ExpressionKind::NotEqual { left, right }) => {
            let left_type =
                derive_expression_type_in_block(modules, function, block, *left, memo, visiting);
            let right_type =
                derive_expression_type_in_block(modules, function, block, *right, memo, visiting);
            derive_comparison_type(id, left_type, right_type, false)
        }
        Some(
            ExpressionKind::LessThan { left, right }
            | ExpressionKind::LessThanOrEqual { left, right }
            | ExpressionKind::GreaterThan { left, right }
            | ExpressionKind::GreaterThanOrEqual { left, right },
        ) => {
            let left_type =
                derive_expression_type_in_block(modules, function, block, *left, memo, visiting);
            let right_type =
                derive_expression_type_in_block(modules, function, block, *right, memo, visiting);
            derive_comparison_type(id, left_type, right_type, true)
        }
    };

    visiting.remove(&id);
    memo.insert(id, copy_expression_type_result(&result));
    result
}

fn derive_arithmetic_type(
    expression_id: ExpressionId,
    left: Result<IntrinsicType, ExpressionTypeError>,
    right: Result<IntrinsicType, ExpressionTypeError>,
) -> Result<IntrinsicType, ExpressionTypeError> {
    // The outcome order is semantic and is independent of which operand was
    // inspected first (`MNIR-ARITH-090` through `MNIR-ARITH-095`).
    let (Ok(left), Ok(right)) = (left, right) else {
        return Err(ExpressionTypeError::OperandTypeUnavailable { expression_id });
    };

    if left != right {
        return Err(ExpressionTypeError::OperandTypeMismatch { expression_id });
    }

    match left {
        IntrinsicType::Int32 => Ok(IntrinsicType::Int32),
        IntrinsicType::Int64 => Ok(IntrinsicType::Int64),
        IntrinsicType::Bool | IntrinsicType::Unit => {
            Err(ExpressionTypeError::UnsupportedOperandType { expression_id })
        }
    }
}

fn derive_comparison_type(
    expression_id: ExpressionId,
    left: Result<IntrinsicType, ExpressionTypeError>,
    right: Result<IntrinsicType, ExpressionTypeError>,
    ordering: bool,
) -> Result<IntrinsicType, ExpressionTypeError> {
    // Comparison outcomes use the same deterministic precedence as arithmetic,
    // but every supported comparison derives Bool (`MNIR-CMP-034` through
    // `MNIR-CMP-041`).
    let (Ok(left), Ok(right)) = (left, right) else {
        return Err(ExpressionTypeError::OperandTypeUnavailable { expression_id });
    };

    if left != right {
        return Err(ExpressionTypeError::OperandTypeMismatch { expression_id });
    }

    if ordering && matches!(left, IntrinsicType::Bool | IntrinsicType::Unit) {
        return Err(ExpressionTypeError::UnsupportedOperandType { expression_id });
    }

    Ok(IntrinsicType::Bool)
}

fn copy_expression_type_result(
    result: &Result<IntrinsicType, ExpressionTypeError>,
) -> Result<IntrinsicType, ExpressionTypeError> {
    match result {
        Ok(intrinsic_type) => Ok(intrinsic_type.copied()),
        Err(error) => Err(*error),
    }
}

fn allocate_program_id(known: &HashSet<[u8; 16]>) -> Result<ProgramId, MutationError> {
    allocate_random_identity(IdentifierCategory::Program, known).map(ProgramId)
}

fn allocate_namespace_id(
    known: &HashSet<[u8; 16]>,
) -> Result<AllocationNamespaceId, MutationError> {
    allocate_random_identity(IdentifierCategory::AllocationNamespace, known)
        .map(AllocationNamespaceId)
}

fn allocate_random_identity(
    category: IdentifierCategory,
    known: &HashSet<[u8; 16]>,
) -> Result<[u8; 16], MutationError> {
    allocate_identity_with(category, known, || {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes)
            .map_err(|_| MutationError::IdentityGenerationFailed(category))?;
        Ok(bytes)
    })
}

fn allocate_identity_with(
    category: IdentifierCategory,
    known: &HashSet<[u8; 16]>,
    mut generate: impl FnMut() -> Result<[u8; 16], MutationError>,
) -> Result<[u8; 16], MutationError> {
    for _ in 0..IDENTITY_GENERATION_ATTEMPTS {
        let candidate = generate()?;
        if !known.contains(&candidate) {
            return Ok(candidate);
        }
    }
    Err(MutationError::IdentityCollision(category))
}

fn collect_present_namespaces(
    modules: &HashMap<ModuleId, Module>,
    active_namespace: AllocationNamespaceId,
) -> HashSet<[u8; 16]> {
    let mut namespaces = HashSet::from([active_namespace.0]);
    for module in modules.values() {
        namespaces.insert(module.id.namespace_id().0);
        for function in module.functions.values() {
            namespaces.insert(function.id.namespace_id().0);
            for parameter in &function.parameters {
                namespaces.insert(parameter.id.namespace_id().0);
            }
            if let Some(body) = function.body() {
                for block in body.blocks() {
                    namespaces.insert(block.id.namespace_id().0);
                    for expression in block.expressions() {
                        namespaces.insert(expression.id.namespace_id().0);
                    }
                }
            }
        }
    }
    namespaces
}

#[cfg(test)]
mod persistent_identity_tests {
    use std::collections::{HashSet, VecDeque};

    use crate::{IdentifierCategory, MutationError};

    use super::{IDENTITY_GENERATION_ATTEMPTS, allocate_identity_with};

    // AR-PSI-006; MNIR-PSI-013/-021 require detected collisions to be
    // handled rather than accepted as shared identity or authority.
    #[test]
    fn detected_random_identity_collision_is_retried() {
        let collision = [7_u8; 16];
        let distinct = [8_u8; 16];
        let known = HashSet::from([collision]);
        let mut candidates = VecDeque::from([collision, distinct]);

        let allocated =
            allocate_identity_with(IdentifierCategory::AllocationNamespace, &known, || {
                Ok(candidates.pop_front().unwrap())
            })
            .unwrap();

        assert_eq!(allocated, distinct);
    }

    // AR-PSI-006; bounded retry exhaustion is a typed construction failure.
    #[test]
    fn repeated_detected_identity_collision_is_rejected() {
        let collision = [7_u8; 16];
        let known = HashSet::from([collision]);
        let mut attempts = 0;

        let error = allocate_identity_with(IdentifierCategory::Program, &known, || {
            attempts += 1;
            Ok(collision)
        })
        .unwrap_err();

        assert_eq!(attempts, IDENTITY_GENERATION_ATTEMPTS);
        assert_eq!(
            error,
            MutationError::IdentityCollision(IdentifierCategory::Program)
        );
    }
}
