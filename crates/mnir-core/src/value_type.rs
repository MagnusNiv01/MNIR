use crate::{IntrinsicType, TypeId};

/// The complete semantic type of an MNIR value (`MNIR-DOMAIN-010` through
/// `MNIR-DOMAIN-012`).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ValueType {
    Intrinsic(IntrinsicType),
    Domain(TypeId),
}

impl From<IntrinsicType> for ValueType {
    fn from(value: IntrinsicType) -> Self {
        Self::Intrinsic(value)
    }
}
