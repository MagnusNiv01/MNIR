# MNIR Specification — Semantic Verification and Diagnostics

**Document:** `06-semantic-verification-and-diagnostics.md`
**Specification status:** Draft
**Specification version:** 0.1
**Normative:** Yes

---

## 1. Purpose

This document introduces the first semantic verification layer for MNIR.

Previous MNIR specifications intentionally permit structurally valid Program revisions that contain semantic type errors.

Semantic Verification and Diagnostics 0.1 defines how such a committed Program revision is checked and how machine-readable diagnostics are produced.

This specification introduces:

* semantic verification of an immutable Program revision;
* `VerifiedProgram`;
* revision-bound verification;
* arithmetic type verification;
* Function Return type verification;
* machine-readable diagnostics;
* stable diagnostic codes;
* deterministic diagnostic semantics;
* explicit separation between structural validation and semantic verification.

The target capability is:

```text
MnirProgram
    ↓ snapshot
ProgramSnapshot
    ↓ semantic verification
    ├── success → VerifiedProgram
    └── failure → Diagnostics
```

A Function conceptually equivalent to:

```text
add(a: Int32, b: Int32) -> Int32 {
    return a + b
}
```

must verify successfully.

A Function conceptually equivalent to:

```text
bad(a: Int32, b: Bool) -> Int32 {
    return a + b
}
```

must remain representable as structurally valid MNIR but fail semantic verification with machine-readable diagnostics.

This specification intentionally does not define:

* security verification;
* effects;
* contracts;
* policies;
* architectural patterns;
* warnings;
* informational diagnostics;
* diagnostic suppression;
* diagnostic localization;
* source-code locations;
* EasyH diagnostics;
* arithmetic fault detection;
* constant evaluation;
* execution;
* runtime behavior;
* backend verification.

---

# 2. Normative language

Every declarative statement inside a numbered `MNIR-VERIFY-*` rule is normative unless explicitly stated otherwise.

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** retain their normative meanings when used inside numbered rules.

Text outside numbered `MNIR-VERIFY-*` rules is informative unless explicitly stated otherwise.

Acceptance requirements use identifiers of the form:

```text
AR-VERIFY-NNN
```

All applicable `AR-VERIFY-*` requirements are mandatory project-level acceptance requirements for the implementation increment defined by this document.

Acceptance requirements demonstrate normative behavior but do not independently introduce MNIR semantics.

---

# 3. Dependencies

This specification builds on:

* Program Model 0.1;
* Type System Foundations 0.1;
* Functions and Parameters 0.1;
* Expressions and Basic Function Bodies 0.1;
* Arithmetic Expressions 0.1.

All structural rules from those specifications remain in force.

Semantic verification defined here operates only after structural validity has been established.

---

# 4. Terminology

## 4.1 Semantic verification

**Semantic verification** is the process of checking a structurally valid, committed MNIR Program revision against the semantic rules defined by a specific verification specification.

Semantic verification:

* does not mutate the Program;
* does not create a new Program revision;
* does not repair errors;
* does not execute Function bodies.

---

## 4.2 Verification rule set

A **verification rule set** is the exact set of normative semantic checks applied during verification.

Semantic Verification and Diagnostics 0.1 defines one fixed rule set.

Version 0.1 does not define configurable verification profiles or selective rule disabling.

---

## 4.3 VerifiedProgram

A **VerifiedProgram** is an immutable verification artifact establishing that one specific committed Program revision successfully satisfied all semantic verification rules applicable under this specification version.

`VerifiedProgram` is not a new Program revision.

---

## 4.4 Diagnostic

A **Diagnostic** is a machine-readable description of one semantic verification failure.

Diagnostics identify:

* a stable diagnostic code;
* the semantic entity primarily associated with the failure;
* and diagnostic-specific semantic data.

Human-readable messages may additionally be provided but are not the normative identity of a diagnostic.

---

## 4.5 Primary semantic subject

The **primary semantic subject** of a Diagnostic is the semantic identity used together with its diagnostic code for duplicate-diagnostic prevention.

The primary subject does not replace other required diagnostic payload identifiers.

---

## MNIR-VERIFY-086 — Primary subjects

The primary semantic subject for each version 0.1 diagnostic code MUST be:

```text
MNIR-DIAG-001 → affected arithmetic ExpressionId
MNIR-DIAG-002 → affected arithmetic ExpressionId
MNIR-DIAG-003 → affected arithmetic ExpressionId
MNIR-DIAG-004 → affected FunctionId
```

For `MNIR-DIAG-004`, the Return `ExpressionId` remains required diagnostic payload according to `MNIR-VERIFY-025` and `MNIR-VERIFY-040`, but it is not the primary subject used for duplicate prevention.

---

# 5. Crate ownership

## MNIR-VERIFY-001 — Verifier ownership

Semantic verification defined by this specification MUST be implemented in:

```text
mnir-verify
```

rather than `mnir-core`.

---

## MNIR-VERIFY-002 — Dependency direction

`mnir-verify` MAY depend on `mnir-core`.

Implementing this specification MUST NOT cause `mnir-core` to depend on `mnir-verify`.

---

## MNIR-VERIFY-003 — Core semantics remain independent

`mnir-core` MUST remain capable of representing structurally valid but semantically invalid MNIR without requiring the verifier.

---

# 6. Verification input

## MNIR-VERIFY-004 — Committed revision input

Semantic verification MUST operate on one specific committed Program revision.

The input MUST be immutable for the duration of verification.

---

## MNIR-VERIFY-005 — Snapshot-oriented verification

The public verification API MUST accept `ProgramSnapshot` or another API-level representation that unambiguously identifies and preserves one immutable committed Program revision.

The exact Rust API signature is implementation-defined.

---

## MNIR-VERIFY-087 — Complete read-only traversal

The verification input API MUST provide sufficient read-only access for `mnir-verify` to discover every semantic entity to which Semantic Verification and Diagnostics 0.1 applies.

For version 0.1 this includes traversal from the Program revision through:

```text
ProgramSnapshot
    ↓
Modules
    ↓
Functions
    ↓
Function bodies
    ↓
Blocks
    ↓
Expressions
```

The verifier MUST NOT require knowledge of private `mnir-core` storage representation to perform complete verification.

---

## MNIR-VERIFY-088 — ProgramSnapshot Module enumeration

The public `mnir-core` read-only API MUST permit enumeration of all Modules contained in a `ProgramSnapshot`.

Enumeration order is non-semantic.

The exact Rust API is implementation-defined.

Conceptually, an API equivalent to:

```text
snapshot.modules()
```

is sufficient.

This requirement introduces no semantic Module ordering.

---

## MNIR-VERIFY-089 — Traversal must preserve encapsulation

Read-only traversal introduced to support verification MUST NOT expose unrestricted mutation access to committed Program state.

The traversal API MUST preserve the encapsulation and mutation guarantees established by Program Model 0.1.

---

## MNIR-VERIFY-006 — No mutable Program verification state

Successful verification MUST NOT mark `MnirProgram` itself with mutable global state equivalent to:

```text
verified = true
```

Verification state belongs to the verified revision artifact.

---

## MNIR-VERIFY-007 — Structural prerequisite

A Program revision MUST satisfy all applicable structural validity requirements before it can produce `VerifiedProgram`.

Semantic verification MUST NOT convert structurally invalid MNIR into a successful verification result.

---

## MNIR-VERIFY-008 — Structural failure is not a semantic diagnostic

If structural invalidity is encountered before semantic verification can safely proceed, it MUST NOT be represented as one of the semantic diagnostics defined by this specification.

The exact verification-input error representation is implementation-defined.

---

# 7. Verification immutability

## MNIR-VERIFY-009 — Verification is read-only

Semantic verification MUST NOT mutate:

* Program contents;
* presentation metadata;
* identifier allocation history;
* revision identity;
* Function bodies;
* Expressions;
* Parameters.

---

## MNIR-VERIFY-010 — Verification creates no revision

Running semantic verification MUST NOT allocate a new `RevisionId`.

---

## MNIR-VERIFY-011 — Repeated verification

Running verification multiple times against the same Program revision under Semantic Verification and Diagnostics 0.1 MUST produce the same semantic success/failure result.

When verification fails, repeated runs MUST produce equivalent:

* diagnostic codes;
* primary semantic subjects;
* and normative diagnostic payload contents.

Human-readable messages, non-normative nested cause information, internal allocation identity, and diagnostic collection iteration order are not required to be identical.

This rule is consistent with the diagnostic determinism defined by `MNIR-VERIFY-053` and `MNIR-VERIFY-054`.

---

# 8. VerifiedProgram identity

## MNIR-VERIFY-012 — Revision binding

Every `VerifiedProgram` MUST be bound to exactly:

```text
ProgramId
RevisionId
```

of the Program revision that was verified.

---

## MNIR-VERIFY-013 — Verification rule-set binding

`VerifiedProgram` MUST also retain or expose sufficient information to establish that verification was performed under Semantic Verification and Diagnostics 0.1.

The exact Rust representation of verification-version identity is implementation-defined.

---

## MNIR-VERIFY-014 — No cross-revision implication

Verification of:

```text
Revision R1
```

MUST NOT imply verification of:

```text
Revision R2
```

even when both revisions belong to the same `ProgramId`.

---

## MNIR-VERIFY-015 — Historical validity

A `VerifiedProgram` MUST remain valid as evidence that its bound immutable Program revision successfully passed Semantic Verification and Diagnostics 0.1.

Advancing the mutable Program lineage to a newer revision MUST NOT invalidate the verification evidence for the older immutable revision.

The historical `VerifiedProgram` MUST NOT imply that any newer revision has been verified.

---

## MNIR-VERIFY-016 — Snapshot preservation

`VerifiedProgram` MUST retain or provide access to the exact verified Program snapshot or equivalent immutable revision contents.

---

# 9. Verification result

## MNIR-VERIFY-017 — Successful verification

Semantic verification succeeds if and only if no Error diagnostic defined by the active verification rule set applies to the Program revision.

Successful verification MUST permit creation of `VerifiedProgram`.

---

## MNIR-VERIFY-018 — Failed verification

If one or more Error diagnostics apply, semantic verification MUST fail.

A failed semantic verification MUST NOT produce `VerifiedProgram` for that revision under this rule set.

---

## MNIR-VERIFY-019 — Multiple diagnostics

Verification MUST NOT stop after the first semantic error when independent additional semantic errors can safely be discovered from the structurally valid Program revision.

The verifier MUST collect all diagnostics required by the rules in this specification.

---

# 10. Diagnostic severity

## MNIR-VERIFY-020 — Error severity

All diagnostics defined by Semantic Verification and Diagnostics 0.1 have severity:

```text
Error
```

---

## MNIR-VERIFY-021 — No warning semantics

Version 0.1 does not define:

```text
Warning
Info
Hint
```

or other diagnostic severities.

An implementation MUST NOT establish such categories as normative 0.1 verification semantics.

---

# 11. Stable diagnostic codes

Semantic Verification and Diagnostics 0.1 defines exactly these diagnostic codes:

```text
MNIR-DIAG-001  ArithmeticOperandTypeUnavailable
MNIR-DIAG-002  ArithmeticOperandTypeMismatch
MNIR-DIAG-003  ArithmeticUnsupportedOperandType
MNIR-DIAG-004  ReturnTypeMismatch
```

---

## MNIR-VERIFY-022 — Diagnostic code stability

A diagnostic code identifies one semantic diagnostic category defined by this specification.

Implementations MUST NOT assign different semantic meanings to the codes listed above.

---

## MNIR-VERIFY-023 — Human message is not diagnostic identity

A human-readable diagnostic message MAY vary in wording.

Consumers MUST use the diagnostic code and structured semantic data rather than exact human-readable message text when programmatic behavior depends on diagnostic identity.

---

# 12. Diagnostic subjects

## MNIR-VERIFY-024 — Arithmetic diagnostic subject

Diagnostics:

```text
MNIR-DIAG-001
MNIR-DIAG-002
MNIR-DIAG-003
```

MUST identify the affected arithmetic `ExpressionId`.

---

## MNIR-VERIFY-025 — Return diagnostic subject

`MNIR-DIAG-004` MUST identify:

* the affected `FunctionId`;
* and the Return `ExpressionId`.

---

## MNIR-VERIFY-026 — No generic NodeId requirement

This specification does not require a generic diagnostic `NodeId`.

Implementations MAY represent diagnostic subjects using concrete typed identifiers or an implementation-defined diagnostic subject enum.

---

# 13. Arithmetic verification scope

## MNIR-VERIFY-027 — Every committed arithmetic Expression is verified

Every arithmetic Expression present in every committed Function body MUST be semantically type-checked.

This applies regardless of whether the Expression is directly returned.

Version 0.1 defines no dead-expression or reachability exemption.

---

## MNIR-VERIFY-028 — Valid arithmetic Expression

An arithmetic Expression produces no arithmetic-type diagnostic when its type inspection yields a valid intrinsic type according to Arithmetic Expressions 0.1.

---

# 14. Arithmetic operand unavailable diagnostic

## MNIR-VERIFY-029 — ArithmeticOperandTypeUnavailable

When arithmetic type inspection for an Arithmetic Expression produces:

```text
OperandTypeUnavailable
```

the verifier MUST emit exactly one:

```text
MNIR-DIAG-001
```

for that arithmetic `ExpressionId`.

---

## MNIR-VERIFY-030 — Unavailable diagnostic payload

`MNIR-DIAG-001` MUST expose at minimum:

```text
expression_id: ExpressionId
```

The implementation MAY additionally expose nested causes or related operand identities.

Such additional cause representation is implementation-defined.

---

# 15. Arithmetic operand mismatch diagnostic

## MNIR-VERIFY-031 — ArithmeticOperandTypeMismatch

When arithmetic type inspection produces:

```text
OperandTypeMismatch
```

the verifier MUST emit exactly one:

```text
MNIR-DIAG-002
```

for that arithmetic Expression.

---

## MNIR-VERIFY-032 — Mismatch diagnostic payload

`MNIR-DIAG-002` MUST expose at minimum:

```text
expression_id
left_type
right_type
```

where:

```text
left_type  : IntrinsicType
right_type : IntrinsicType
```

correspond to the successfully derived operand types.

---

# 16. Unsupported arithmetic type diagnostic

## MNIR-VERIFY-033 — ArithmeticUnsupportedOperandType

When arithmetic type inspection produces:

```text
UnsupportedOperandType
```

the verifier MUST emit exactly one:

```text
MNIR-DIAG-003
```

for that arithmetic Expression.

---

## MNIR-VERIFY-034 — Unsupported diagnostic payload

`MNIR-DIAG-003` MUST expose at minimum:

```text
expression_id
operand_type
```

where `operand_type` is the equal but unsupported intrinsic operand type.

---

# 17. Arithmetic diagnostic exclusivity

## MNIR-VERIFY-035 — One arithmetic type diagnostic per Expression

One arithmetic Expression MUST NOT receive more than one of:

```text
MNIR-DIAG-001
MNIR-DIAG-002
MNIR-DIAG-003
```

during one verification run.

The diagnostic category follows the deterministic arithmetic type-inspection outcome defined by Arithmetic Expressions 0.1.

---

## MNIR-VERIFY-036 — Nested arithmetic diagnostics

If an inner arithmetic Expression has a semantic type error and an outer arithmetic Expression consequently produces `OperandTypeUnavailable`, both Expressions independently satisfy their respective diagnostic rules.

For example:

```text
E1 : Int32
E2 : Bool
E3 = Add(E1, E2)
E4 = Multiply(E3, E1)
```

produces:

```text
E3 → MNIR-DIAG-002
E4 → MNIR-DIAG-001
```

because `E3` has successfully derived but different operand types, while `E4` depends on an Expression whose valid intrinsic type cannot be derived.

Version 0.1 does not define cascading-diagnostic suppression.

---

# 18. Return verification

## MNIR-VERIFY-037 — Return verification scope

Every Function that owns a committed Function body MUST have its Return type relationship semantically verified.

Bodyless Functions require no Return diagnostic under this specification.

---

## MNIR-VERIFY-038 — Matching Return type

If the Return Expression derives a valid intrinsic type equal to the Function's declared return type, no Return type diagnostic applies.

---

## MNIR-VERIFY-039 — Return type mismatch

If the Return Expression derives a valid intrinsic type different from the Function's declared return type, the verifier MUST emit exactly one:

```text
MNIR-DIAG-004
```

for that Function body.

---

## MNIR-VERIFY-040 — Return mismatch payload

`MNIR-DIAG-004` MUST expose at minimum:

```text
function_id
return_expression_id
expected_type
actual_type
```

where:

```text
expected_type
```

is the Function's declared return type and:

```text
actual_type
```

is the successfully derived Return Expression type.

---

## MNIR-VERIFY-041 — Unavailable Return Expression type

If the Return Expression does not have a successfully derivable intrinsic type, the verifier MUST NOT emit `MNIR-DIAG-004`.

This applies whether the failure originates:

* directly in the Return Expression;
* from arithmetic `OperandTypeMismatch`;
* from arithmetic `UnsupportedOperandType`;
* from arithmetic `OperandTypeUnavailable`;
* or transitively from an underlying Expression dependency whose valid intrinsic type cannot be derived.

The semantic failure remains represented by the applicable Expression diagnostics.

Version 0.1 does not define a separate:

```text
ReturnTypeUnavailable
```

diagnostic.

---

# 19. Return diagnostic exclusivity

## MNIR-VERIFY-042 — At most one Return mismatch per Function body

One Function body MUST receive at most one `MNIR-DIAG-004` during one verification run.

---

# 20. Literals and Parameter references

## MNIR-VERIFY-043 — Literal Expressions require no standalone semantic diagnostic

Structurally valid:

```text
Int32Literal
Int64Literal
BoolLiteral
UnitLiteral
```

Expressions produce no standalone semantic diagnostics under version 0.1.

---

## MNIR-VERIFY-044 — ParameterReference requires no standalone semantic diagnostic

A structurally valid `ParameterReference` produces no standalone semantic diagnostic under version 0.1.

Dangling Parameter references are structural invalidity and therefore cannot appear in a structurally valid verification input.

---

# 21. Arithmetic fault semantics and verification

## MNIR-VERIFY-045 — Runtime arithmetic faults are not verification errors

The possibility that concrete runtime values may produce:

```text
Overflow
DivisionByZero
```

does not make a type-correct Arithmetic Expression semantically invalid under Semantic Verification 0.1.

---

## MNIR-VERIFY-046 — Statically guaranteed arithmetic fault does not cause verification failure

A statically type-correct Arithmetic Expression MUST NOT cause Semantic Verification 0.1 to fail merely because its concrete literal operands make an arithmetic fault statically guaranteed if evaluated.

This includes Expressions conceptually equivalent to:

```text
2147483647 + 1
```

and:

```text
1 / 0
```

using `Int32` literals.

These Expressions remain subject to all applicable static type rules.

Semantic Verification 0.1 performs no arithmetic fault analysis.

---

## MNIR-VERIFY-047 — No fault diagnostic

Semantic Verification 0.1 MUST NOT introduce diagnostics for statically possible or statically guaranteed arithmetic faults.

Such analysis requires a future specification.

---

# 22. Presentation independence

## MNIR-VERIFY-048 — Presentation metadata does not affect semantic result

Verification success or failure MUST NOT depend on:

* preferred names;
* documentation;
* presentation metadata.

---

## MNIR-VERIFY-049 — Presentation-only revision

A presentation-only Program revision is still a distinct `RevisionId` and therefore requires a distinct `VerifiedProgram` artifact if that new revision is to be represented as verified.

The semantic diagnostic contents MAY otherwise be equivalent to the previous revision.

---

# 23. Diagnostic collection semantics

## MNIR-VERIFY-050 — Diagnostics form a semantic collection

The set of diagnostics produced by verification is semantic.

Collection position or iteration order MUST NOT affect diagnostic meaning.

---

## MNIR-VERIFY-051 — Diagnostic order is non-semantic

Version 0.1 does not define semantic ordering between diagnostics.

Consumers MUST NOT infer severity, causality, priority, or source order from diagnostic collection position.

---

## MNIR-VERIFY-052 — Duplicate category prevention

During one verification run, the verifier MUST NOT emit more than one Diagnostic having the same:

```text
diagnostic code
primary semantic subject
```

where primary semantic subject is defined by `MNIR-VERIFY-086`.

Diagnostics with different codes for the same semantic entity are not duplicates under this rule.

---

# 24. Diagnostic determinism

## MNIR-VERIFY-053 — Diagnostic determinism

For the same immutable Program revision and verification rule set, verification MUST produce equivalent diagnostic codes, subjects, and normative payload contents.

---

## MNIR-VERIFY-054 — Implementation ordering may vary

Equivalent diagnostic results remain equivalent even if the implementation exposes them in a different iteration order.

Future presentation or serialization specifications MAY define canonical ordering.

---

# 25. Verification traversal independence

## MNIR-VERIFY-055 — Verification must not depend on collection iteration order

Semantic results MUST NOT depend on:

* Module collection iteration order;
* Function collection iteration order;
* Expression collection iteration order;
* `HashMap` order;
* memory layout.

---

# 26. VerifiedProgram API

## MNIR-VERIFY-056 — Distinct verified artifact

The public API MUST expose an API-level distinction between an ordinary `ProgramSnapshot` and successfully verified Program contents.

Conceptually:

```text
ProgramSnapshot
VerifiedProgram
```

MUST NOT be freely interchangeable as proof of verification.

---

## MNIR-VERIFY-057 — VerifiedProgram construction restriction

Normal callers MUST NOT be able to construct a valid `VerifiedProgram` for an arbitrary Program revision without successful semantic verification.

The exact Rust visibility and construction mechanism are implementation-defined.

---

## MNIR-VERIFY-058 — VerifiedProgram inspection

The public API MUST permit inspection of at least:

```text
ProgramId
RevisionId
```

associated with `VerifiedProgram`.

---

## MNIR-VERIFY-059 — Verified snapshot access

The public API MUST permit read-only access to the verified Program contents, either by exposing the underlying `ProgramSnapshot` or an equivalent immutable view.

---

# 27. Verification failure API

## MNIR-VERIFY-060 — Failure exposes diagnostics

When semantic verification fails, the public API MUST provide access to all diagnostics produced by that verification run.

---

## MNIR-VERIFY-061 — No partial VerifiedProgram

A failed verification MUST NOT expose a `VerifiedProgram` representing partially verified success.

---

## MNIR-VERIFY-062 — Rust result shape implementation-defined

The Rust API MAY represent verification using:

```text
Result
enum
report object
```

or another explicit type.

The exact API shape is implementation-defined provided success and failure semantics remain unambiguous.

---

# 28. Diagnostic representation

## MNIR-VERIFY-063 — Machine-readable diagnostic code

Every Diagnostic MUST expose its stable diagnostic code programmatically.

---

## MNIR-VERIFY-064 — Machine-readable severity

Every Diagnostic MUST expose severity programmatically.

For version 0.1 this is always:

```text
Error
```

---

## MNIR-VERIFY-065 — Machine-readable structured payload

Diagnostic-specific normative payload data MUST be accessible programmatically.

Consumers MUST NOT need to parse a human-readable string to obtain:

* Expression identifiers;
* Function identifiers;
* expected or actual intrinsic types.

---

## MNIR-VERIFY-066 — Human-readable message optional

A Diagnostic MAY expose a human-readable message.

Exact message wording is implementation-defined.

---

# 29. Semantic verification coverage

## MNIR-VERIFY-067 — Complete 0.1 verification coverage

A verification run MUST traverse the complete verification input and apply every semantic rule defined by this specification to every applicable entity in the Program revision.

The verifier MUST NOT omit a Module, Function, Function body, or Arithmetic Expression merely because the underlying collection implementation has no semantic ordering.

---

## MNIR-VERIFY-068 — No selective rule disabling

Version 0.1 MUST NOT expose normative semantics for selectively disabling:

```text
MNIR-DIAG-001
MNIR-DIAG-002
MNIR-DIAG-003
MNIR-DIAG-004
```

during verification.

Verification profiles belong to a future specification.

---

# 30. No mutation or repair

## MNIR-VERIFY-069 — Verifier does not repair Programs

The verifier MUST NOT automatically:

* change operand types;
* convert integer widths;
* delete Expressions;
* change Return types;
* change Function signatures;
* retarget references;
* modify arithmetic operators.

---

## MNIR-VERIFY-070 — Verification failure preserves input

Failed verification MUST leave the input Program revision unchanged.

---

# 31. No semantic inference from names

## MNIR-VERIFY-071 — No name-derived semantics

The verifier MUST NOT infer semantic type corrections from:

* Function preferred names;
* Parameter preferred names;
* documentation;
* naming conventions.

For example, a Parameter named:

```text
count
```

does not acquire integer semantics from its name.

---

# 32. No future verification domains

## MNIR-VERIFY-072 — Security verification excluded

Version 0.1 MUST NOT introduce security classification verification.

---

## MNIR-VERIFY-073 — Effects excluded

Version 0.1 MUST NOT introduce effect verification.

---

## MNIR-VERIFY-074 — Contracts excluded

Version 0.1 MUST NOT introduce contract verification.

---

## MNIR-VERIFY-075 — Policies and patterns excluded

Version 0.1 MUST NOT introduce policy or architectural-pattern verification.

---

# 33. No execution

## MNIR-VERIFY-076 — Verifier is not an evaluator

Semantic verification MUST NOT require execution of Function bodies.

---

## MNIR-VERIFY-077 — No arithmetic evaluation

Semantic verification MUST NOT execute arithmetic operators merely to determine type validity.

---

## MNIR-VERIFY-078 — No constant folding

Version 0.1 MUST NOT require or establish constant-folding semantics.

---

# 34. Specification gaps

## MNIR-VERIFY-079 — Undefined verification semantics

If implementation requires externally observable verification or diagnostic behavior not defined by this specification, the implementation MUST NOT establish that behavior as normative MNIR semantics.

The missing behavior MUST be reported as a specification gap.

---

# 35. Explicitly unresolved topics

The following topics are intentionally unresolved:

* diagnostic source locations;
* EasyH source mapping;
* diagnostic rendering;
* warning diagnostics;
* informational diagnostics;
* hints;
* diagnostic suppression;
* diagnostic prioritization;
* diagnostic canonical ordering;
* nested diagnostic cause trees;
* fix suggestions;
* automated repairs;
* verification profiles;
* optional verification rules;
* security verification;
* effects;
* contracts;
* policies;
* patterns;
* arithmetic fault analysis;
* constant propagation;
* constant evaluation;
* reachability;
* dead Expressions;
* unreachable code;
* backend verification;
* execution verification;
* caching verification across revisions;
* incremental verification;
* parallel verification;
* persisted VerifiedProgram format;
* serialized diagnostic format.

---

## MNIR-VERIFY-080 — Unresolved topics are not 0.1 semantics

An implementation MUST NOT establish unresolved topics listed by this document as normative Semantic Verification and Diagnostics 0.1 semantics.

---

# 36. Acceptance requirements

## AR-VERIFY-001 — Valid empty Program

Verify an empty structurally valid Program revision.

Verify that:

* semantic verification succeeds;
* zero diagnostics are produced;
* a `VerifiedProgram` is created.

---

## AR-VERIFY-002 — Bodyless Function verifies

Create a structurally valid bodyless Function.

Verify semantic verification succeeds.

---

## AR-VERIFY-003 — Valid identity Function

Construct conceptually:

```text
identity(value: Int32) -> Int32 {
    return value
}
```

Verify:

* zero diagnostics;
* successful `VerifiedProgram`.

---

## AR-VERIFY-004 — Valid arithmetic Function

Construct conceptually:

```text
add(a: Int32, b: Int32) -> Int32 {
    return a + b
}
```

Verify semantic success.

---

## AR-VERIFY-005 — Arithmetic operand mismatch

Create structurally valid:

```text
Int32 + Int64
```

Verify failure with exactly one `MNIR-DIAG-002` for that arithmetic Expression.

Verify its payload contains the affected `ExpressionId`, `Int32`, and `Int64`.

---

## AR-VERIFY-006 — Arithmetic unsupported Bool

Create:

```text
Bool + Bool
```

Verify exactly one `MNIR-DIAG-003` for the arithmetic Expression with `Bool` as unsupported operand type.

---

## AR-VERIFY-007 — Arithmetic unsupported Unit

Demonstrate equivalent `MNIR-DIAG-003` behavior for `Unit`.

---

## AR-VERIFY-008 — Nested arithmetic unavailable

Create an invalid inner arithmetic Expression and an outer arithmetic Expression depending on it.

Verify:

* the inner Expression receives its own mismatch or unsupported diagnostic;
* the outer Expression receives exactly one `MNIR-DIAG-001`.

---

## AR-VERIFY-009 — Return type mismatch

Construct a Function declared:

```text
-> Int32
```

whose Return Expression derives:

```text
Bool
```

Verify exactly one `MNIR-DIAG-004`.

Verify payload includes:

* `FunctionId`;
* Return `ExpressionId`;
* expected `Int32`;
* actual `Bool`.

---

## AR-VERIFY-010 — Return mismatch not emitted for unavailable Return type

Construct a Function whose Return Expression is arithmetic with an unavailable type because of an inner semantic arithmetic error.

Verify:

* appropriate arithmetic diagnostics are produced;
* no `MNIR-DIAG-004` is produced for that Function.

---

## AR-VERIFY-011 — Multiple independent diagnostics

Create a Program containing multiple independent semantic errors in different Functions.

Verify all required diagnostics are returned in one verification run.

The test MUST NOT assume diagnostic iteration order.

---

## AR-VERIFY-012 — One arithmetic diagnostic per Expression

Demonstrate that an arithmetic Expression receives at most one of:

```text
MNIR-DIAG-001
MNIR-DIAG-002
MNIR-DIAG-003
```

during one run.

---

## AR-VERIFY-013 — No duplicate diagnostics

Verify that no duplicate diagnostic with equal code and primary semantic subject is emitted.

---

## AR-VERIFY-014 — Revision binding

Verify a Program revision successfully.

Then commit a new Program revision.

Verify that the original `VerifiedProgram` still reports the original:

```text
ProgramId
RevisionId
```

and does not represent the new revision as verified.

---

## AR-VERIFY-015 — New revision requires new verification

Verify Revision R1.

Create Revision R2.

Verify that representing R2 as `VerifiedProgram` requires a separate successful verification run.

---

## AR-VERIFY-016 — Verification does not mutate Program

Capture the Program snapshot contents and revision before verification.

Run verification.

Verify Program contents and `RevisionId` are unchanged.

---

## AR-VERIFY-017 — Repeated verification determinism

Verify the same snapshot more than once.

Verify equivalent:

* success/failure;
* diagnostic codes;
* diagnostic subjects;
* normative payload data.

Do not require collection iteration order to match.

---

## AR-VERIFY-018 — Presentation independence

Create two revisions differing only in presentation metadata.

Verify that semantic verification results are equivalent except for revision binding.

---

## AR-VERIFY-019 — Diagnostic codes are machine-readable

Demonstrate programmatic inspection of all four defined diagnostic codes.

---

## AR-VERIFY-020 — Diagnostic severity

Demonstrate every version 0.1 diagnostic exposes severity `Error`.

---

## AR-VERIFY-021 — Diagnostic structured payload

Demonstrate that consumers can retrieve diagnostic identifiers and intrinsic types without parsing human-readable messages.

---

## AR-VERIFY-022 — VerifiedProgram cannot be arbitrarily constructed

Demonstrate through compile-time evidence, visibility inspection, or equivalent means that normal callers cannot create a valid `VerifiedProgram` for an arbitrary snapshot without successful verification.

---

## AR-VERIFY-023 — Verified snapshot inspection

Verify that a `VerifiedProgram` exposes:

```text
ProgramId
RevisionId
```

and read-only access to the verified Program contents.

---

## AR-VERIFY-024 — Failed verification exposes no VerifiedProgram

Create semantically invalid MNIR.

Verify failure returns diagnostics but no `VerifiedProgram`.

---

## AR-VERIFY-025 — Guaranteed arithmetic overflow still verifies

Construct an otherwise semantically valid Function containing:

```text
2147483647 + 1
```

using `Int32` literals.

Verify:

* Semantic Verification 0.1 succeeds;
* no arithmetic-fault diagnostic is emitted;
* and a `VerifiedProgram` is produced.

---

## AR-VERIFY-026 — Division by zero still verifies statically

Construct an otherwise semantically valid Function containing:

```text
1 / 0
```

using `Int32` literals.

Verify:

* Semantic Verification 0.1 succeeds;
* no divide-by-zero diagnostic is emitted;
* and a `VerifiedProgram` is produced.

---

## AR-VERIFY-027 — Dead/unreturned invalid arithmetic is still verified

Create a Function body containing:

* one invalid arithmetic Expression;
* a separate valid Return Expression.

Verify the invalid arithmetic Expression still produces its required diagnostic.

Version 0.1 has no reachability exemption.

---

## AR-VERIFY-028 — No name-based correction

Give an invalid Bool Parameter a preferred name suggesting numeric meaning.

Verify the semantic result remains based only on actual MNIR types.

---

## AR-VERIFY-029 — No evaluator or constant folder

Demonstrate by conformance inspection that `mnir-verify` introduces no:

* Function interpreter;
* arithmetic evaluator;
* constant folder;
* execution backend.

---

## AR-VERIFY-030 — No future verification systems

Demonstrate by conformance inspection that version 0.1 introduces no:

* security verifier;
* effects verifier;
* contract verifier;
* policy verifier;
* pattern verifier;
* warning system;
* diagnostic suppression profile.

---

## AR-VERIFY-031 — Crate dependency direction

Demonstrate:

```text
mnir-verify → mnir-core
```

is permitted and used as needed, while:

```text
mnir-core → mnir-verify
```

does not exist.

---

## AR-VERIFY-032 — Existing conformance preserved

All acceptance suites from specifications 01 through 05 MUST continue to pass without semantic regression.

---

## AR-VERIFY-033 — Complete snapshot traversal

Create a Program containing multiple Modules, with Functions and Function bodies distributed across those Modules.

Verify through the public read-only API that `mnir-verify` can discover every Module and every applicable descendant semantic entity without private `mnir-core` access.

The test MUST NOT rely on collection iteration order.

---

## AR-VERIFY-034 — Semantic errors in every Module are discovered

Create at least two Modules containing independent semantic errors.

Run verification once.

Verify that required diagnostics from both Modules are returned.

This demonstrates that verification coverage is complete across the Program revision.

---

## AR-VERIFY-035 — Historical VerifiedProgram remains valid evidence

Successfully verify Revision R1 and retain the resulting `VerifiedProgram`.

Advance the same Program lineage to Revision R2.

Verify that the original `VerifiedProgram` still:

* exposes Revision R1;
* exposes the original immutable verified contents;
* remains valid verification evidence for R1;
* and does not claim verification of R2.

---

## AR-VERIFY-036 — Diagnostic primary subjects

Demonstrate that primary semantic subjects are:

```text
MNIR-DIAG-001 → ExpressionId
MNIR-DIAG-002 → ExpressionId
MNIR-DIAG-003 → ExpressionId
MNIR-DIAG-004 → FunctionId
```

For `MNIR-DIAG-004`, also verify that the Return `ExpressionId` remains accessible as required structured payload.

---

# 37. Implementation constraints

## MNIR-VERIFY-081 — No core semantic-verification state

Implementation MUST NOT add mutable semantic-verification state to `mnir-core` Program entities.

---

## MNIR-VERIFY-082 — Safe Rust

The implementation MUST NOT use or require `unsafe` Rust to satisfy this specification.

---

## MNIR-VERIFY-083 — No generic diagnostic node requirement

This specification does not require a generic:

```text
NodeId
SourceLocation
DiagnosticSpan
```

abstraction.

The implementation MUST NOT introduce such abstractions as normative MNIR semantics solely in anticipation of future source languages.

---

## MNIR-VERIFY-084 — No speculative diagnostic framework

The implementation SHOULD remain sufficient for the four diagnostics defined in version 0.1.

It SHOULD NOT introduce a large extensible plugin or rule framework solely in anticipation of future verification domains.

---

## MNIR-VERIFY-085 — Explicit verification ownership

Only successful semantic verification under the applicable rule set may create the API artifact recognized as `VerifiedProgram`.

---

# 38. Expected implementation shape

Conceptually:

```text
mnir-core
    │
    └── ProgramSnapshot
             │
             ▼
        mnir-verify
             │
             ├── verify arithmetic Expressions
             ├── verify Function Return types
             │
             ▼
      VerificationResult
        │             │
        │ success     │ failure
        ▼             ▼
 VerifiedProgram   Diagnostics
```

A possible Rust organization is:

```text
mnir-verify/src/
├── lib.rs
├── diagnostic.rs
├── verified.rs
└── verifier.rs
```

This structure is informative.

Exact Rust organization and type names are implementation-defined.

Semantic Verification 0.1 may require a small additive `mnir-core` API change to expose complete read-only `ProgramSnapshot` traversal.

Such a change is permitted when it:

* exposes no new MNIR semantics;
* preserves Program Model encapsulation;
* introduces no mutation path;
* and exists solely to expose semantic entities already present in the committed Program model.

---

# 39. Target successful example

Conceptually:

```text
add(a: Int32, b: Int32) -> Int32 {
    return a + b
}
```

produces:

```text
verification:
    success

diagnostics:
    []

VerifiedProgram:
    ProgramId(P1)
    RevisionId(R7)
```

---

# 40. Target failure example

Conceptually:

```text
bad(a: Int32, b: Bool) -> Int32 {
    return a + b
}
```

may contain:

```text
E1 = ParameterReference(a)  -> Int32
E2 = ParameterReference(b)  -> Bool
E3 = Add(E1, E2)
Return(E3)
```

Semantic verification produces an arithmetic type diagnostic for `E3`.

Because `E3` has no valid derived type, version 0.1 does not additionally emit `ReturnTypeMismatch`.

No `VerifiedProgram` is produced.

---

# 41. Foundational invariants

The following is an informative summary:

```text
Structural validity and semantic validity are separate.

mnir-core stores structurally valid MNIR.

mnir-verify performs semantic verification.

Verification operates on one immutable committed revision.

Verification never mutates the Program.

VerifiedProgram is bound to ProgramId + RevisionId + rule set.

A newer revision is never implicitly verified.

Semantic failure produces machine-readable diagnostics.

Diagnostics have stable codes and structured payloads.

Arithmetic type errors are verified.

Return type mismatch is verified.

Nested arithmetic type failures may produce cascading diagnostics.

Return mismatch is suppressed when Return type cannot be derived.

All committed arithmetic Expressions are checked, even if unreturned.

Runtime Overflow and DivisionByZero are not verification errors in 0.1.

Verification does not execute the Program.

Verification does not repair the Program.

No security, effects, contracts, policies, or patterns are verified yet.
```
