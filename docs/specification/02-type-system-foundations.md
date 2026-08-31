# MNIR Specification — Type System Foundations

**Document:** `02-type-system-foundations.md`
**Specification status:** Draft
**Specification version:** 0.1
**Normative:** Yes

---

## 1. Purpose

This document defines the first foundational type concepts in MNIR.

Specification version 0.1 introduces only four intrinsic types:

```text
Int32
Int64
Bool
Unit
```

The purpose of this increment is to establish:

* intrinsic type semantics,
* intrinsic type identity,
* type equality,
* target-independent integer width,
* separation between intrinsic types and future program-defined types,
* and architectural constraints for future domain and security types.

This document intentionally does not define:

* values or literals,
* variables,
* parameters,
* expressions,
* arithmetic operations,
* integer overflow behavior during operations,
* numeric conversions,
* assignment,
* subtyping,
* user-defined types,
* `TypeId`,
* `Text`,
* `Bytes`,
* collections,
* generics,
* nullable types,
* `Option`,
* `Result`,
* domain types,
* security classifications,
* trust classifications,
* `Password`,
* or EasyH type syntax.

Those concepts are introduced by later specification increments.

---

# 2. Normative language

Every declarative statement inside a numbered `MNIR-TYPE-*` rule is normative unless the rule explicitly states otherwise.

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** retain their normal normative meanings when used inside numbered rules.

Text outside numbered `MNIR-TYPE-*` rules is informative unless explicitly stated otherwise.

Acceptance requirements use identifiers of the form:

```text
AR-TYPE-NNN
```

All applicable `AR-TYPE-*` requirements are mandatory project-level acceptance requirements for the initial implementation increment defined by this document.

Acceptance requirements demonstrate normative behavior but do not independently introduce MNIR semantics.

---

# 3. Terminology

## 3.1 Type

A **type** describes the semantic category of a value.

This specification defines only intrinsic types.

A general MNIR type representation capable of referring to both intrinsic and future program-defined types is intentionally deferred.

---

## 3.2 Intrinsic type

An **intrinsic type** is a type whose identity and semantics are defined directly by the MNIR specification.

Intrinsic types:

* are not declared by a Program,
* are not owned by a Module,
* do not have Program-local identity,
* and exist independently of any specific Program lineage.

---

## 3.3 Program-defined type

A **program-defined type** is a future type whose identity is introduced by an MNIR Program.

Program-defined types are not defined by this specification.

A future specification is expected to introduce `TypeId` when such identity is required.

---

## 3.4 Abstract value domain

An **abstract value domain** defines the mathematical or logical set of values associated with a type.

Defining an abstract value domain does not require this specification increment to define:

* runtime value objects,
* literals,
* constructors,
* storage representation,
* serialization,
* expressions,
* arithmetic,
* or public APIs for individual values.

Type System Foundations 0.1 defines abstract value domains only where required to establish the semantic meaning of an intrinsic type.

---

# 4. Initial intrinsic type set

## MNIR-TYPE-001 — Defined intrinsic types

MNIR Type System Foundations 0.1 MUST define exactly the following intrinsic types:

```text
Int32
Int64
Bool
Unit
```

The implementation MUST NOT introduce additional normative intrinsic types as part of this specification increment.

---

## MNIR-TYPE-002 — Closed intrinsic type representation

The public intrinsic type representation introduced by this specification increment MUST represent the four types defined by `MNIR-TYPE-001` as distinct alternatives.

The representation MUST NOT permit additional unspecified intrinsic type alternatives through normal safe public APIs.

The exact Rust type name and variant names are implementation-defined.

An implementation MAY use an enum such as:

```rust
enum IntrinsicType {
    Int32,
    Int64,
    Bool,
    Unit,
}
```

but these Rust identifiers are not normative API names.

---

# 5. Intrinsic type identity

## MNIR-TYPE-003 — Global intrinsic identity

Intrinsic type identity is defined by the MNIR specification rather than by a Program.

For example:

```text
Int32
```

in one Program represents the same MNIR type as:

```text
Int32
```

in another Program.

---

## MNIR-TYPE-004 — No intrinsic TypeId

Intrinsic types MUST NOT require a Program-local `TypeId`.

The implementation MUST NOT allocate `TypeId` values for:

```text
Int32
Int64
Bool
Unit
```

as part of this specification increment.

---

## MNIR-TYPE-005 — Independence from Program identity

Intrinsic type equality MUST NOT depend on:

```text
ProgramId
ModuleId
RevisionId
```

or any future Program-local semantic identifier.

---

# 6. Type equality

## MNIR-TYPE-006 — Intrinsic type equality

Two intrinsic types MUST compare as semantically equal if and only if they represent the same intrinsic type defined by this specification.

Therefore:

```text
Int32 == Int32
Int64 == Int64
Bool  == Bool
Unit  == Unit
```

and every comparison between two different intrinsic types is unequal.

---

## MNIR-TYPE-007 — Equality is semantic

Intrinsic type equality MUST be based on intrinsic type identity.

It MUST NOT depend on:

* memory address,
* allocation order,
* Program identity,
* textual formatting,
* human-readable aliases,
* or target platform representation.

---

# 7. Int32

## MNIR-TYPE-008 — Int32 semantics

`Int32` MUST represent the abstract domain of signed integer values with a semantic width of exactly 32 bits.

Its mathematical value domain is:

```text
-2^31 .. 2^31 - 1
```

equivalent to:

```text
-2147483648 .. 2147483647
```

This rule defines abstract type semantics only and does not require an executable value API in this specification increment.

---

## MNIR-TYPE-009 — Int32 target independence

The semantic width and value domain of `Int32` MUST NOT change according to:

* CPU architecture,
* operating system,
* native machine word size,
* execution backend,
* or compiler implementation.

---

# 8. Int64

## MNIR-TYPE-010 — Int64 semantics

`Int64` MUST represent the abstract domain of signed integer values with a semantic width of exactly 64 bits.

Its mathematical value domain is:

```text
-2^63 .. 2^63 - 1
```

equivalent to:

```text
-9223372036854775808
..
9223372036854775807
```

This rule defines abstract type semantics only and does not require an executable value API in this specification increment.

---

## MNIR-TYPE-011 — Int64 target independence

The semantic width and value domain of `Int64` MUST NOT change according to:

* CPU architecture,
* operating system,
* native machine word size,
* execution backend,
* or compiler implementation.

---

# 9. Bool

## MNIR-TYPE-012 — Bool semantics

`Bool` MUST have an abstract value domain containing exactly two logical values:

```text
true
false
```

No third logical state is part of `Bool`.

This rule does not require Boolean value objects or literal syntax in this specification increment.

---

## MNIR-TYPE-013 — Bool is not an integer type

`Bool` MUST be semantically distinct from:

```text
Int32
Int64
```

The implementation MUST NOT define `Bool` as an alias for an integer intrinsic type.

The concrete runtime representation of `Bool` is outside the scope of this specification.

---

# 10. Unit

## MNIR-TYPE-014 — Unit semantics

`Unit` MUST have an abstract value domain containing exactly one semantic value.

That value carries no domain information beyond the fact that a computation produced a value of type `Unit`.

The syntax, value object, runtime representation, and storage representation of the Unit value are outside the scope of this specification increment.

---

## MNIR-TYPE-015 — Unit distinction

`Unit` MUST be semantically distinct from:

* absence of a value caused by missing data,
* failure,
* nullability,
* `Bool`,
* and integer types.

Future specifications MUST NOT interpret `Unit` itself as a nullable or error state.

---

# 11. Integer type distinction

## MNIR-TYPE-016 — Int32 and Int64 are distinct types

`Int32` and `Int64` MUST be treated as distinct types.

The fact that every `Int32` mathematical value can be represented within the `Int64` mathematical range does not make the two types equal.

---

## MNIR-TYPE-017 — No implicit type relation

Specification version 0.1 defines no implicit conversion, subtype, widening, narrowing, or coercion relationship between `Int32` and `Int64`.

A future specification MAY define conversion operations.

An implementation MUST NOT establish an implicit conversion relationship between these types as MNIR semantics under this specification increment.

---

# 12. No target-native integer type

## MNIR-TYPE-018 — No ambiguous machine integer

Type System Foundations 0.1 MUST NOT define an intrinsic integer type whose semantic width or value domain varies according to the execution target.

Concepts equivalent to:

```text
native int
machine int
pointer-sized signed int
```

are not part of this specification increment.

A future specification MAY introduce platform-related types only if their semantics, including any target dependence, are explicitly defined by that specification.

---

# 13. No Program registration

## MNIR-TYPE-019 — Intrinsic types are not Program entities

A Program MUST NOT need to register, declare, allocate, import, or own the intrinsic types defined by this specification.

Creating an empty `MnirProgram` does not create new identities for:

```text
Int32
Int64
Bool
Unit
```

---

## MNIR-TYPE-020 — Revision independence

Changing a Program revision MUST NOT create a new identity for an intrinsic type.

Intrinsic type identity exists independently of Program revision history.

---

# 14. Domain semantics

Intrinsic types describe fundamental computational value categories.

They do not, by themselves, express application-domain meaning.

For example, future values such as:

```text
CustomerId
InvoiceId
Percentage
PasswordInput
```

must not gain those semantics merely because their underlying representation eventually uses an intrinsic type.

---

## MNIR-TYPE-021 — No inferred domain semantics

The implementation MUST NOT infer domain type semantics from:

* variable names,
* field names,
* parameter names,
* preferred names,
* documentation text,
* or naming conventions.

For example, a future value named:

```text
password
```

does not become security-sensitive merely because of that name.

---

## MNIR-TYPE-022 — Intrinsic types carry no implicit domain identity

`Int32`, `Int64`, `Bool`, and `Unit` MUST NOT implicitly represent application-specific domain identities.

Future domain types may use intrinsic types as part of their representation, but domain identity must be expressed explicitly by later MNIR semantics.

---

# 15. Security semantics

Security classification is outside the scope of Type System Foundations 0.1.

Future specifications are expected to introduce concepts capable of expressing semantics such as:

```text
Secret<T>
Untrusted<T>
Validated<T>
Credential<T>
PersonalData<T>
PasswordInput
PasswordHash
```

The exact model is intentionally unresolved.

---

## MNIR-TYPE-023 — No inferred security semantics

Intrinsic type identity MUST NOT imply security classification.

Security-sensitive semantics MUST NOT be inferred from human-readable names or presentation metadata.

A future security specification must define security classification explicitly.

---

# 16. Future type composition

This section is informative.

The long-term type model is expected to allow MNIR to distinguish concepts similar to:

```text
Intrinsic computational type
Domain identity
Security classification
Trust state
Constraints
```

For example, a future password input might conceptually express:

```text
Domain:
    PasswordInput

Representation:
    Text

Security:
    Secret

Trust:
    Untrusted
```

This document does not define how those dimensions are represented.

No implementation abstraction for them is required by Type System Foundations 0.1.

---

# 17. Arithmetic design direction

This section is informative and does not yet define arithmetic operations.

The current non-normative design intention for a future arithmetic specification is:

```text
checked arithmetic by default
```

The current design intention is that implicit wrapping arithmetic will not become the default behavior.

The future arithmetic specification remains authoritative when those semantics are formally defined.

Future arithmetic specifications are expected to define explicit semantics for concepts such as:

```text
checked arithmetic
wrapping arithmetic
saturating arithmetic
```

No arithmetic implementation is required by this document.

---

# 18. Numeric conversion design direction

This section is informative.

The current non-normative design intention is to avoid implicit numeric conversions that can hide semantic changes.

No future conversion behavior is normatively established by this section.

In particular:

```text
Int64 -> Int32
```

is not intended to occur implicitly.

Whether:

```text
Int32 -> Int64
```

will ever be allowed implicitly remains a future specification decision.

Type System Foundations 0.1 defines no conversion operation.

---

# 19. Structural representation

## MNIR-TYPE-024 — Intrinsic representation validity

Every value of the implementation's public intrinsic type representation MUST correspond to exactly one intrinsic type defined by `MNIR-TYPE-001`.

The public representation MUST NOT permit construction of an unknown or unspecified intrinsic type through normal safe APIs.

---

## MNIR-TYPE-025 — Safe Rust implementation

The implementation MUST NOT use or require `unsafe` Rust to satisfy Type System Foundations 0.1.

---

## MNIR-TYPE-031 — Abstract value domains do not require value APIs

The abstract value domains defined for `Int32`, `Int64`, `Bool`, and `Unit` are normative type semantics.

Implementation of Type System Foundations 0.1 MUST NOT require public MNIR value objects, literal representations, constructors, arithmetic operations, or runtime storage representations for those domains.

The implementation MAY expose descriptive type metadata if useful, but such API is not required for conformance unless specified by an acceptance requirement.

---

# 20. Specification gaps

## MNIR-TYPE-026 — Undefined type semantics

If implementation requires externally observable type behavior that is not defined by this specification, the implementation MUST NOT establish that behavior as normative MNIR semantics.

The missing behavior MUST be reported as a specification gap according to the repository's established specification-gap process.

---

# 21. Explicitly unresolved topics

The following topics are intentionally unresolved:

* `TypeId`
* program-defined types
* type declarations
* type references
* type aliases
* structural types
* nominal types
* values
* literals
* variables
* parameters
* function signatures
* expressions
* arithmetic
* overflow execution behavior
* integer conversion operations
* implicit widening
* explicit narrowing
* comparisons
* equality expressions
* assignment
* subtyping
* generics
* `Text`
* `Bytes`
* floating-point types
* decimal types
* unsigned integers
* additional signed integer widths
* collections
* nullable types
* `Option`
* `Result`
* domain types
* refined types
* constrained types
* security types
* trust classifications
* security qualifiers
* serialization encoding
* runtime representation
* ABI representation
* EasyH syntax
* backend lowering

---

## MNIR-TYPE-032 — Unresolved topics are not 0.1 semantics

An implementation MUST NOT establish any topic listed as explicitly unresolved in this document as normative MNIR Type System Foundations 0.1 semantics.

Experimental internal implementation details are permitted only when they do not create externally observable MNIR semantics or conflict with another normative rule.

---

# 22. Acceptance requirements

## AR-TYPE-001 — Complete intrinsic type set

Demonstrate that the implementation exposes exactly:

```text
Int32
Int64
Bool
Unit
```

as the intrinsic types defined by this specification increment.

---

## AR-TYPE-002 — Intrinsic equality

Demonstrate that each intrinsic type compares equal to itself and unequal to each of the other intrinsic types.

---

## AR-TYPE-003 — Program independence

Demonstrate that intrinsic type identity does not depend on a particular `MnirProgram`, `ProgramId`, or Program revision.

The test must not require registering intrinsic types with a Program.

---

## AR-TYPE-004 — No TypeId

Demonstrate through the public API that use of the intrinsic types does not require allocation of a `TypeId`.

The implementation must not introduce `TypeId` merely to satisfy this increment.

---

## AR-TYPE-005 — Integer distinction

Demonstrate that:

```text
Int32 != Int64
```

and that the implementation does not treat the two as aliases.

---

## AR-TYPE-006 — Bool distinction

Demonstrate that `Bool` is distinct from both signed integer intrinsic types and `Unit`.

---

## AR-TYPE-007 — Unit distinction

Demonstrate that `Unit` is a distinct intrinsic type.

No null, optional, or error semantics are required or permitted by this acceptance requirement.

---

## AR-TYPE-008 — Closed safe representation

Demonstrate that normal safe public APIs cannot construct an unspecified intrinsic type.

---

## AR-TYPE-009 — Existing Program Model compatibility

Demonstrate that all Program Model 0.1 tests continue to pass unchanged in semantic behavior after introduction of intrinsic types.

---

## AR-TYPE-010 — No future type-system implementation

Demonstrate by conformance review that this increment introduces no normative implementation of:

* `TypeId`,
* values,
* variables,
* expressions,
* conversions,
* domain types,
* or security types.

---

## AR-TYPE-011 — Fixed integer semantics

Demonstrate through automated tests or explicit compile-time constants/metadata tests, if such metadata is exposed, that the implementation's intrinsic type definitions correspond to:

```text
Int32 -> 32-bit signed semantic type
Int64 -> 64-bit signed semantic type
```

If the implementation intentionally exposes no bit-width API, conformance with `MNIR-TYPE-008` through `MNIR-TYPE-011` MUST instead be documented in the implementation conformance review.

No MNIR value implementation is required.

---

## AR-TYPE-012 — Target-independent integer identity

Demonstrate by implementation inspection and conformance review that `Int32` and `Int64` identity and semantics do not depend on Rust target pointer width, operating system, or native integer width.

The implementation MUST NOT use a target-native integer category as an intrinsic MNIR type.

---

## AR-TYPE-013 — Safe Rust

Demonstrate that the implementation of this specification contains no `unsafe` Rust.

Existing repository validation and source inspection may be used for this requirement.

---

## AR-TYPE-014 — Crate ownership

Demonstrate that the intrinsic type representation is owned by `mnir-core` and does not introduce dependencies from `mnir-core` to:

```text
mnir-verify
easyh-render
mnir-cli
```

---

## AR-TYPE-015 — No inferred domain or security semantics

Demonstrate through implementation inspection that the intrinsic type implementation contains no behavior that derives domain identity or security classification from:

* names,
* documentation,
* presentation metadata,
* or naming conventions.

No domain or security type implementation is required.

---

## Acceptance verification categories

Acceptance requirements may be demonstrated using one of three mechanisms:

```text
Automated runtime test
Compile-time/type-system evidence
Documented conformance inspection
```

Runtime tests should be preferred when observable behavior exists.

Compile-time evidence should be preferred for Rust type-safety properties.

Conformance inspection is appropriate for negative architectural requirements such as:

* absence of `unsafe`,
* absence of forbidden dependencies,
* absence of speculative future abstractions,
* and target-independence where no runtime API exists solely for testing purposes.

The implementation SHOULD NOT add unnecessary public APIs merely to make a normative semantic statement runtime-testable.

---

# 23. Implementation constraints

## MNIR-TYPE-027 — Core ownership

The intrinsic type representation defined by this specification MUST belong to:

```text
mnir-core
```

It MUST NOT depend on `mnir-verify`, EasyH, or an execution backend.

---

## MNIR-TYPE-028 — No speculative general Type abstraction

This specification does not require a general `Type`, `TypeRef`, `TypeId`, type registry, or type arena abstraction.

The implementation MUST NOT introduce one as a normative MNIR concept solely in anticipation of future specifications.

An internal abstraction that does not establish externally visible MNIR semantics is permitted only when independently justified.

---

## MNIR-TYPE-029 — No speculative values

Implementation of this specification MUST NOT introduce integer values, Boolean values, Unit values, literals, or expressions merely to demonstrate the intrinsic type definitions.

Those concepts belong to later specification increments.

---

## MNIR-TYPE-030 — No security or domain inference

Implementation of this specification MUST NOT introduce name-based domain or security classification.

---

# 24. Implementation scope summary

The expected implementation scope is intentionally small.

Conceptually:

```text
IntrinsicType
├── Int32
├── Int64
├── Bool
└── Unit
```

The exact Rust names are implementation-defined.

No public APIs for integer bounds, bit widths, values, or literals are required solely by this specification increment.

An implementation MAY expose such metadata only when independently useful and consistent with the specification.

No Program-owned type registry is required.

No `TypeId` is required.

No values or operations are required.

No verifier changes are required beyond what is necessary to keep the existing workspace compiling.

---

# 25. Foundational invariants

The following is an informative summary:

```text
Int32, Int64, Bool, and Unit are intrinsic MNIR types.

Intrinsic types have specification-defined global identity.

Intrinsic types do not have Program-local TypeIds.

Int32 and Int64 have fixed target-independent widths.

Int32 and Int64 are distinct types.

Bool is not an integer type.

Unit is a distinct single-value type.

No implicit numeric conversion relation exists yet.

Intrinsic types do not carry application-domain meaning.

Security semantics are never inferred from names.

Program-defined types and TypeId are deferred.

Values, variables, expressions, and arithmetic are deferred.

Future integer arithmetic is intended to be checked by default,
but arithmetic is not part of this specification increment.
```
