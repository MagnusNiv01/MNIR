/// The identifier categories currently defined by MNIR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierCategory {
    Program,
    Revision,
    AllocationNamespace,
    Module,
    Function,
    Parameter,
    Block,
    Expression,
}

/// Opaque identity of one Program lineage.
///
/// Program identity is collision-resistant and independent of every contained
/// semantic entity identity (`MNIR-PSI-018` through `MNIR-PSI-021`).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProgramId(pub(crate) [u8; 16]);

/// Opaque identity of one committed revision within a Program lineage.
///
/// Consumers must not infer chronological ordering from its internal
/// representation (`MNIR-CORE-029`).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RevisionId(pub(crate) u64);

/// Collision-resistant minting domain for persistent semantic entity IDs.
///
/// This identity is distinct from Program lineage identity and does not imply
/// trust, provenance, or allocation authority (`MNIR-PSI-005`,
/// `MNIR-PSI-012` through `MNIR-PSI-017`).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AllocationNamespaceId(pub(crate) [u8; 16]);

/// Read-only state of a lineage's single active entity counter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationCounterState {
    Available(u64),
    Exhausted,
}

macro_rules! persistent_entity_id {
    ($(#[$meta:meta])* $type:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $type {
            namespace_id: AllocationNamespaceId,
            counter: u64,
        }

        impl $type {
            pub(crate) const fn new(
                namespace_id: AllocationNamespaceId,
                counter: u64,
            ) -> Self {
                Self {
                    namespace_id,
                    counter,
                }
            }

            /// Returns the persistent allocation namespace component.
            #[must_use]
            pub const fn namespace_id(self) -> AllocationNamespaceId {
                self.namespace_id
            }

            /// Returns the persistent monotonic counter component.
            #[must_use]
            pub const fn counter(self) -> u64 {
                self.counter
            }
        }
    };
}

persistent_entity_id!(
    /// Persistent typed identity of one Module.
    ///
    /// ```compile_fail
    /// use mnir_core::{MnirProgram, ModuleId};
    ///
    /// let program = MnirProgram::new().unwrap();
    /// let _: ModuleId = program.program_id();
    /// ```
    ModuleId
);

persistent_entity_id!(
    /// Persistent typed identity of one Function.
    ///
    /// ```compile_fail
    /// use mnir_core::{FunctionId, MnirProgram, ParameterId};
    ///
    /// fn require_parameter(_: ParameterId) {}
    ///
    /// let mut program = MnirProgram::new().unwrap();
    /// let mut transaction = program.begin_transaction();
    /// let module_id = transaction.add_module().unwrap();
    /// let function_id: FunctionId = transaction
    ///     .add_function(module_id, mnir_core::IntrinsicType::Unit)
    ///     .unwrap();
    /// require_parameter(function_id);
    /// ```
    FunctionId
);

persistent_entity_id!(
    /// Persistent typed identity of one Parameter.
    ParameterId
);

persistent_entity_id!(
    /// Persistent typed identity of one Block.
    ///
    /// ```compile_fail
    /// use mnir_core::{BlockId, ExpressionId};
    ///
    /// fn require_expression(_: ExpressionId) {}
    /// fn demonstrate(block_id: BlockId) {
    ///     require_expression(block_id);
    /// }
    /// ```
    BlockId
);

persistent_entity_id!(
    /// Persistent typed identity of one Expression.
    ExpressionId
);
