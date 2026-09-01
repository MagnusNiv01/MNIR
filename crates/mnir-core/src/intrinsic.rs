/// An intrinsic type whose identity and semantics are defined by MNIR.
///
/// Type System Foundations 0.1 defines exactly these four alternatives. The
/// enum has no Program-owned identifier or registration state, so equality is
/// based only on the intrinsic alternative (`MNIR-TYPE-001` through
/// `MNIR-TYPE-007`). Rust discriminants and debug formatting are not MNIR
/// semantics.
///
/// Normal safe Rust cannot construct an unspecified intrinsic type
/// (`MNIR-TYPE-024`):
///
/// ```compile_fail
/// use mnir_core::IntrinsicType;
///
/// let _ = IntrinsicType::Unknown;
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum IntrinsicType {
    /// The target-independent signed 32-bit intrinsic type.
    Int32,
    /// The target-independent signed 64-bit intrinsic type.
    Int64,
    /// The two-valued logical intrinsic type.
    Bool,
    /// The single-valued Unit intrinsic type.
    Unit,
}
