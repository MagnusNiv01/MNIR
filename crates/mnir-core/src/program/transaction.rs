use std::collections::{HashMap, HashSet};
use std::fmt;
use std::mem;
use std::sync::Arc;

use crate::IntrinsicType;
use crate::ids::{FunctionId, IdentifierCategory, ModuleId, ParameterId, RevisionId};

use super::error::{MutationError, TransactionState};
use super::model::{
    Function, MnirProgram, Module, Parameter, ProgramSnapshot, RevisionState, find_function,
    find_function_mut, find_parameter, find_parameter_mut,
};
use super::validation::validate_modules;

impl MnirProgram {
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
    #[cfg(test)]
    pub(super) fn working_modules_mut(&mut self) -> &mut HashMap<ModuleId, Module> {
        &mut self.working_modules
    }

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

        module.presentation.set_preferred_name(preferred_name);
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

        module.presentation.set_documentation(documentation);
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
        function.presentation.set_preferred_name(preferred_name);
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
        function.presentation.set_documentation(documentation);
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
        parameter.presentation.set_preferred_name(preferred_name);
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
        parameter.presentation.set_documentation(documentation);
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

fn allocate_from_cursor(
    cursor: &mut Option<u64>,
    category: IdentifierCategory,
) -> Result<u64, MutationError> {
    let raw = cursor.ok_or(MutationError::IdentifierExhausted(category))?;
    *cursor = raw.checked_add(1);
    Ok(raw)
}
