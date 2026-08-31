use std::fmt;

/// The identifier categories defined by Program Model 0.1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierCategory {
    Program,
    Revision,
    Module,
}

/// Opaque identity of one Program lineage.
///
/// Consumers may compare this value for equality but must not infer semantic
/// meaning from its internal representation (`MNIR-CORE-006`).
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ProgramId(pub(crate) u64);

/// Opaque identity of one committed revision within a Program lineage.
///
/// Consumers must not infer chronological ordering from its internal
/// representation (`MNIR-CORE-029`).
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct RevisionId(pub(crate) u64);

/// Opaque identity of one Module within a Program lineage.
///
/// The Rust type is intentionally distinct from [`ProgramId`] and
/// [`RevisionId`] (`MNIR-CORE-005`).
///
/// ```compile_fail
/// use mnir_core::{MnirProgram, ModuleId};
///
/// let program = MnirProgram::new().unwrap();
/// let _: ModuleId = program.program_id();
/// ```
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ModuleId(pub(crate) u64);

macro_rules! impl_opaque_debug {
    ($type:ident) => {
        impl fmt::Debug for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_tuple(stringify!($type))
                    .field(&self.0)
                    .finish()
            }
        }
    };
}

impl_opaque_debug!(ProgramId);
impl_opaque_debug!(RevisionId);
impl_opaque_debug!(ModuleId);
