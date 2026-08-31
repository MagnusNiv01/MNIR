use mnir_core::{IntrinsicType, MnirProgram};

const ALL_INTRINSIC_TYPES: [IntrinsicType; 4] = [
    IntrinsicType::Int32,
    IntrinsicType::Int64,
    IntrinsicType::Bool,
    IntrinsicType::Unit,
];

// An exhaustive match is compile-time evidence that the closed public
// representation has exactly the alternatives specified by MNIR-TYPE-001,
// MNIR-TYPE-002, and MNIR-TYPE-024.
fn semantic_name(intrinsic: &IntrinsicType) -> &'static str {
    match intrinsic {
        IntrinsicType::Int32 => "Int32",
        IntrinsicType::Int64 => "Int64",
        IntrinsicType::Bool => "Bool",
        IntrinsicType::Unit => "Unit",
    }
}

fn new_program() -> MnirProgram {
    MnirProgram::new().expect("the process-local ProgramId allocator should have capacity")
}

// AR-TYPE-001; MNIR-TYPE-001, MNIR-TYPE-002.
#[test]
fn ar_type_001_exposes_exactly_the_four_intrinsic_types() {
    let names = ALL_INTRINSIC_TYPES.map(|intrinsic| semantic_name(&intrinsic));

    assert_eq!(names, ["Int32", "Int64", "Bool", "Unit"]);
}

// AR-TYPE-002; MNIR-TYPE-003, MNIR-TYPE-006, MNIR-TYPE-007.
#[test]
fn ar_type_002_intrinsic_equality_is_reflexive_and_distinguishes_alternatives() {
    for (left_index, left) in ALL_INTRINSIC_TYPES.iter().enumerate() {
        for (right_index, right) in ALL_INTRINSIC_TYPES.iter().enumerate() {
            assert_eq!(left == right, left_index == right_index);
        }
    }
}

// AR-TYPE-003; MNIR-TYPE-003, MNIR-TYPE-005, MNIR-TYPE-019,
// MNIR-TYPE-020. Intrinsic types are constructed independently; no Program
// registration API is involved.
#[test]
fn ar_type_003_intrinsic_identity_is_independent_of_programs_and_revisions() {
    let mut first_program = new_program();
    let second_program = new_program();
    assert_ne!(first_program.program_id(), second_program.program_id());

    let before_revision_change = IntrinsicType::Int32;
    let source_revision = first_program.revision_id();
    first_program.begin_transaction().commit().unwrap();
    assert_ne!(first_program.revision_id(), source_revision);
    let after_revision_change = IntrinsicType::Int32;

    assert!(before_revision_change == after_revision_change);
}

// AR-TYPE-004; MNIR-TYPE-004. Construction requires neither a Program nor an
// MNIR TypeId, and this increment introduces no TypeId API.
#[test]
fn ar_type_004_intrinsic_types_require_no_type_id() {
    let intrinsic = IntrinsicType::Bool;

    assert!(intrinsic == IntrinsicType::Bool);
}

// AR-TYPE-005; MNIR-TYPE-016, MNIR-TYPE-017.
#[test]
fn ar_type_005_int32_and_int64_are_distinct() {
    assert!(IntrinsicType::Int32 != IntrinsicType::Int64);
}

// AR-TYPE-006; MNIR-TYPE-013.
#[test]
fn ar_type_006_bool_is_distinct_from_integers_and_unit() {
    assert!(IntrinsicType::Bool != IntrinsicType::Int32);
    assert!(IntrinsicType::Bool != IntrinsicType::Int64);
    assert!(IntrinsicType::Bool != IntrinsicType::Unit);
}

// AR-TYPE-007; MNIR-TYPE-014, MNIR-TYPE-015.
#[test]
fn ar_type_007_unit_is_a_distinct_intrinsic_type() {
    assert!(IntrinsicType::Unit != IntrinsicType::Int32);
    assert!(IntrinsicType::Unit != IntrinsicType::Int64);
    assert!(IntrinsicType::Unit != IntrinsicType::Bool);
}

// AR-TYPE-008; MNIR-TYPE-002, MNIR-TYPE-024. `semantic_name` is exhaustive,
// and the compile-fail doctest on IntrinsicType rejects an unknown alternative.
#[test]
fn ar_type_008_public_representation_is_closed_and_exhaustive() {
    for intrinsic in &ALL_INTRINSIC_TYPES {
        assert!(!semantic_name(intrinsic).is_empty());
    }
}

// AR-TYPE-014; MNIR-TYPE-027. The ownership of IntrinsicType is demonstrated
// by importing it from mnir_core above; the manifest proves dependency
// direction without introducing a runtime API.
#[test]
fn ar_type_014_mnir_core_has_no_forbidden_dependencies() {
    const MANIFEST: &str = include_str!("../Cargo.toml");

    assert!(!MANIFEST.contains("mnir-verify"));
    assert!(!MANIFEST.contains("easyh-render"));
    assert!(!MANIFEST.contains("mnir-cli"));
}
