# MNIR Specification — Domain Type Foundations

**Document:** `11-domain-type-foundations.md`

**Specification status:** Draft

**Specification version:** 0.1

**Normative:** Yes

---

## 1. Purpose

This document defines the first nominal, Program-defined type model in MNIR.
It introduces:

- `Text` and `Bytes` as specification-defined intrinsic types;
- `ValueType` as the common semantic type of signatures and Expressions;
- persistent typed `TypeId` identity;
- Module-owned Domain Types with one intrinsic representation;
- `TextLiteral` and `BytesLiteral` Expressions;
- explicit `DomainConstruct` and `DomainProject` Expressions;
- Domain-aware Function, Call, Return, Branch, arithmetic, and comparison typing;
- Domain Type structural validity, mutation, removal, snapshot, and fork behavior;
- Semantic Verification and Diagnostics rule set `V0_5`; and
- machine-readable Domain typing diagnostics.

This specification extends, and where stated supersedes, specifications `01`
through `10`. It does not define a general type-composition system, validation
types, security qualifiers, serialization, or execution.

The target distinction is nominal:

```text
CustomerId : Int64
OrderId    : Int64
```

`CustomerId` and `OrderId` are different types when their `TypeId` values
differ, even though both use `Int64` as representation.

---

# 2. Normative language

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are
normative only when they occur inside a numbered MNIR rule.

Normative requirements in this document use identifiers of the form:

```text
MNIR-DOMAIN-NNN
```

Acceptance requirements use identifiers of the form:

```text
AR-DOMAIN-NNN
```

Acceptance requirements demonstrate normative behavior but do not
independently introduce MNIR semantics.

---

# 3. Authority and compatibility

## MNIR-DOMAIN-001 — Supersession authority

Domain Type Foundations 0.1 supersedes or extends earlier rules only as
explicitly listed in section 31.

When a listed earlier rule conflicts with this document, this document MUST
take precedence for a Program claiming conformance with Domain Type
Foundations 0.1.

Unlisted portions of earlier rules remain normative.

---

## MNIR-DOMAIN-002 — Existing behavior remains applicable

Except where section 31 explicitly states otherwise, semantics and structural
behavior defined by specifications `01` through `10` MUST remain unchanged.

Introducing `ValueType`, `TypeId`, and Domain Expressions MUST NOT silently
change identity, transaction atomicity, CFG, EffectSequence, dependency,
snapshot, fork, or verifier semantics.

---

# 4. Terminology

## 4.1 Intrinsic type

An **intrinsic type** is a globally specification-defined type that has no
Program-owned `TypeId`.

## 4.2 Value type

A **value type** is the complete semantic type of a Function Parameter,
Function result, or typed Expression under this specification.

## 4.3 Domain Type

A **Domain Type** is a nominal, Module-owned Program entity identified by a
`TypeId` and represented by exactly one `IntrinsicType`.

## 4.4 Representation

A Domain Type's **representation** is its current intrinsic value domain. It
does not define the Domain Type's identity and does not grant operations to the
Domain Type.

---

# 5. Intrinsic type extension

## MNIR-DOMAIN-003 — Complete intrinsic type set

For Programs claiming conformance with Domain Type Foundations 0.1,
`IntrinsicType` MUST contain exactly these alternatives:

```text
Int32
Int64
Bool
Unit
Text
Bytes
```

No other intrinsic type is introduced by this specification.

---

## MNIR-DOMAIN-004 — Intrinsic identity remains global

All six intrinsic types MUST retain globally specification-defined identity.
They MUST NOT receive a `TypeId`, require Program registration, or depend on a
`ProgramId`, `RevisionId`, `ModuleId`, or allocation namespace.

Each intrinsic type is semantically equal only to the same intrinsic type.
In particular, `Text` and `Bytes` are distinct.

---

## MNIR-DOMAIN-005 — Closed intrinsic representation

The safe public intrinsic-type representation MUST represent exactly the six
alternatives in `MNIR-DOMAIN-003` and MUST NOT permit an unknown intrinsic type
through normal safe APIs.

---

# 6. Text and Bytes value semantics

## MNIR-DOMAIN-006 — Text abstract value domain

`Text` MUST represent a finite sequence of Unicode scalar values.

A Unicode scalar value is a Unicode code point in either range:

```text
U+0000..U+D7FF
U+E000..U+10FFFF
```

Surrogate code points `U+D800..U+DFFF` are not Unicode scalar values and MUST
NOT occur in an MNIR `Text` value.

`Text` is defined in terms of scalar values, not UTF-8, UTF-16, code units,
bytes, locale, or platform-native characters.

---

## MNIR-DOMAIN-007 — No Unicode normalization

Domain Type Foundations 0.1 MUST NOT normalize `Text`.

Two `Text` values are equal if and only if they contain the same number of
Unicode scalar values and equal scalar values at every position. Canonically
equivalent but scalar-distinct Unicode sequences remain distinct values.

---

## MNIR-DOMAIN-008 — Bytes abstract value domain

`Bytes` MUST represent a finite sequence of octets. Each octet has the
mathematical range `0..255`.

Two `Bytes` values are equal if and only if they have equal length and equal
octets at every position.

---

## MNIR-DOMAIN-009 — Text and Bytes have no implicit relation

No implicit conversion, coercion, encoding, decoding, subtype relation, or
compatibility relation exists between `Text` and `Bytes`.

This specification defines no text encoding or decoding operation.

---

# 7. ValueType

## MNIR-DOMAIN-010 — ValueType alternatives

The complete semantic type carrier introduced by this specification MUST be:

```text
ValueType
    Intrinsic(IntrinsicType)
    Domain(TypeId)
```

The two alternatives MUST remain distinguishable. A Domain Type MUST NOT be
collapsed into its intrinsic representation.

The exact Rust type and variant names are implementation-defined.

---

## MNIR-DOMAIN-011 — ValueType equality

Two `ValueType` values are equal if and only if either:

1. both are `Intrinsic` and contain equal `IntrinsicType` values; or
2. both are `Domain` and contain the same `TypeId`.

`ValueType::Domain(T)` is never equal to
`ValueType::Intrinsic(R)`, including when Domain Type `T` currently has
representation `R`.

---

## MNIR-DOMAIN-012 — General semantic type carrier

`ValueType` MUST be the semantic type of:

- every Function Parameter;
- every Function return;
- every Expression for which a valid derived type is available; and
- every V0_5 diagnostic field whose normative domain can include a Domain
  Type.

This rule does not require unrelated intrinsically defined fields to become
`ValueType`.

---

# 8. TypeId and persistent semantic identity

## MNIR-DOMAIN-013 — TypeId category

Every Domain Type MUST have exactly one `TypeId`.

`TypeId` MUST be a distinct typed persistent semantic entity identifier
category. It MUST NOT compare equal to a `ModuleId`, `FunctionId`,
`ParameterId`, `BlockId`, or `ExpressionId`, even if an internal or test
representation gives them equal namespace and counter components.

---

## MNIR-DOMAIN-014 — TypeId components

A `TypeId` MUST consist semantically of:

```text
AllocationNamespaceId + MonotonicCounter
```

No `ProgramId` component participates in `TypeId` equality.

---

## MNIR-DOMAIN-015 — Shared namespace-wide counter

`TypeId` issuance MUST use the same active namespace-wide allocation counter
as Module, Function, Parameter, Block, and Expression identity issuance.

The implementation MUST NOT introduce a Type-specific counter, a new
namespace, or fresh entropy for ordinary Domain Type creation.

---

## MNIR-DOMAIN-016 — Validate before reserve

All operation preconditions that can be checked before allocating a new
`TypeId` MUST be checked before reservation.

At minimum, `add_domain_type` MUST validate that its target Module exists and
that its representation is a valid `IntrinsicType` before reserving a
`TypeId`. A pre-reservation failure MUST consume no identity.

---

## MNIR-DOMAIN-017 — TypeId issuance and non-reuse

Once a `TypeId` namespace/counter pair is successfully reserved, it is issued
and MUST never be reused, including after:

- later operation failure;
- transaction poisoning;
- transaction discard;
- failed commit;
- create-then-remove in one transaction; or
- committed Domain Type removal.

Allocation gaps are valid.

---

## MNIR-DOMAIN-018 — Exhaustion

`TypeId` allocation MUST use the exhaustion behavior of Persistent Semantic
Identity 0.1.

`Available(u64::MAX)` issues that counter exactly once and transitions the
existing allocation authority to `Exhausted`. Allocation from `Exhausted`
fails before issuance and MUST NOT rotate or create a namespace.

---

# 9. Domain Type ownership and collection

## MNIR-DOMAIN-019 — Module ownership

Every committed Domain Type MUST be owned by exactly one existing Module. A
Domain Type MUST NOT simultaneously belong to more than one Module.

Ownership is semantic Program structure and MUST NOT be inferred from names,
documentation, or collection position.

---

## MNIR-DOMAIN-020 — Module collection

A Module MAY own zero or more Domain Types in an identity-based, semantically
unordered collection.

Collection position and iteration order MUST NOT contribute to Domain Type
identity or semantics. Lookup MUST be possible by `TypeId` and MUST NOT depend
on presentation names.

---

## MNIR-DOMAIN-021 — No movement

Domain Type Foundations 0.1 does not define moving an existing Domain Type
between Modules while preserving its `TypeId`.

An implementation MUST NOT expose Domain Type movement as normative 0.1
behavior.

---

# 10. Nominal identity and presentation

## MNIR-DOMAIN-022 — Nominal identity

Domain Type identity MUST be determined solely by `TypeId`.

Preferred name, documentation, representation, owning Module, memory address,
collection position, and revision MUST NOT substitute for `TypeId` equality.

Two Domain Types with different `TypeId` values are distinct even when all
other observable fields are equal.

---

## MNIR-DOMAIN-023 — Presentation metadata

A Domain Type MAY carry optional `preferred_name` and `documentation`
metadata consistent with existing entity presentation metadata.

Presentation metadata MUST NOT determine `TypeId`, `ValueType` equality,
representation, or verification outcome. Updating it MUST preserve `TypeId`.

The existing Program revision rule still applies to a successful
presentation-only mutation; calling presentation non-semantic does not make
that mutation revision-neutral.

---

## MNIR-DOMAIN-024 — Duplicate preferred names

Multiple Domain Types, including Domain Types in the same Module, MAY have the
same preferred name. Domain Type Foundations 0.1 defines no name resolution or
name uniqueness.

---

# 11. Domain representation

## MNIR-DOMAIN-025 — Exactly one intrinsic representation

Every Domain Type MUST have exactly one current representation, and that
representation MUST be exactly one `IntrinsicType` from
`MNIR-DOMAIN-003`.

A Domain Type representation MUST NOT be another Domain Type. This
specification defines no representation graph or recursive representation.

---

## MNIR-DOMAIN-026 — Representation does not grant operations

A Domain Type does not automatically inherit arithmetic, comparison, Branch,
conversion, validation, security, or other capabilities from its
representation.

Only an explicit `DomainProject` produces a value having the intrinsic
representation type.

---

## MNIR-DOMAIN-027 — Representation mutation preserves identity

Changing a Domain Type's representation MUST preserve its `TypeId`, ownership,
and presentation metadata.

The change is a semantic Program-state mutation and MUST participate in normal
revision behavior. It MUST NOT be modeled as removal and recreation.

---

## MNIR-DOMAIN-028 — Live representation semantics

All semantic checks and derived types that consult a Domain Type
representation MUST use the representation in the inspected committed
revision or Active transaction working state.

They MUST NOT treat a former representation as authoritative. Consequently, a
representation change MAY make existing construction Expressions
semantically invalid and MUST immediately change the successfully derived type
of corresponding projection Expressions.

---

# 12. Text and Bytes literals

## MNIR-DOMAIN-029 — TextLiteral

`TextLiteral` MUST be a pure Expression whose semantic data is exactly one
finite sequence of Unicode scalar values and whose derived type is:

```text
ValueType::Intrinsic(Text)
```

Its semantic value is the exact scalar sequence. No normalization or storage
encoding is part of the literal's MNIR meaning.

---

## MNIR-DOMAIN-030 — BytesLiteral

`BytesLiteral` MUST be a pure Expression whose semantic data is exactly one
finite octet sequence and whose derived type is:

```text
ValueType::Intrinsic(Bytes)
```

The exact octet sequence, including length and order, is semantic.

---

## MNIR-DOMAIN-031 — Literal identity

`TextLiteral` and `BytesLiteral` MUST use existing `ExpressionId` identity.
Equal literal contents in two Expressions MUST NOT make the Expressions
identical and MUST NOT permit identity deduplication.

Literal contents participate in Program state independently of collection
iteration order.

---

## MNIR-DOMAIN-032 — Literal construction

Controlled mutation MUST support operations equivalent to:

```text
add_text_literal(block_id, value)
add_bytes_literal(block_id, value)
```

The target Block MUST exist. A textual input representation that can represent
non-scalar code points MUST reject such input before `ExpressionId`
reservation. Invalid structural targets are operation failures governed by
existing transaction poisoning rules.

The exact Rust signatures and in-memory storage are implementation-defined.

---

# 13. Domain construction

## MNIR-DOMAIN-033 — DomainConstruct data

`DomainConstruct` MUST be a pure Expression with semantic data:

```text
type_id: TypeId
value: ExpressionId
```

It uses existing `ExpressionId` identity and introduces no constructor-specific
identity.

---

## MNIR-DOMAIN-034 — DomainConstruct structural references

At construction time:

- the target Block MUST exist;
- `type_id` MUST resolve to an existing Domain Type in the transaction working
  state; and
- `value` MUST identify an existing Expression in that same Block.

These checks MUST occur before reserving the new `ExpressionId`. Failure is an
operation failure and MUST poison the transaction according to existing rules.

---

## MNIR-DOMAIN-035 — DomainConstruct derived type

When its `type_id` resolves, a `DomainConstruct` derived type MUST be:

```text
ValueType::Domain(type_id)
```

This type remains derivable even when the source Expression type is
unavailable or differs from the Domain Type representation. Source validity
is a semantic-verification concern, analogous to a Call result remaining typed
despite invalid arguments.

---

## MNIR-DOMAIN-036 — DomainConstruct semantic validity

A `DomainConstruct` is semantically valid if and only if its source Expression
has the successfully derived type:

```text
ValueType::Intrinsic(current representation of type_id)
```

No implicit intrinsic-to-Domain conversion exists. A source having another
Domain Type is not valid merely because its representation matches.

When valid, the construct denotes a value of the target Domain Type carrying
the source's intrinsic representation value without validation,
normalization, conversion, or other transformation. This abstract meaning does
not require an evaluator.

---

# 14. Domain projection

## MNIR-DOMAIN-037 — DomainProject data

`DomainProject` MUST be a pure Expression with semantic data:

```text
value: ExpressionId
```

It stores no redundant `TypeId`; its source's current semantic type determines
the projected Domain Type.

---

## MNIR-DOMAIN-038 — DomainProject structural references

At construction time, the target Block and source Expression MUST exist and
the source Expression MUST belong to that same Block.

Construction MUST NOT require the source to have a valid Domain type. A
projection of an intrinsic or currently type-invalid Expression is
structurally representable and is rejected semantically by V0_5.

Structural preconditions MUST be checked before new `ExpressionId`
reservation.

---

## MNIR-DOMAIN-039 — DomainProject valid derived type

If the source Expression derives `ValueType::Domain(T)`, and `T` resolves to
an existing Domain Type whose current representation is `R`, the
`DomainProject` derived type MUST be:

```text
ValueType::Intrinsic(R)
```

No implicit Domain-to-intrinsic conversion exists outside this explicit
Expression.

When valid, the projection denotes the source Domain value's contained
intrinsic representation value without validation, normalization, conversion,
or other transformation. This abstract meaning does not require an evaluator.

---

## MNIR-DOMAIN-040 — DomainProject unavailable source

If the source Expression has no successfully derived type, type inspection of
the `DomainProject` MUST report `DomainProjectSourceTypeUnavailable` or an
equally specific typed failure.

A nested source failure MAY be retained as non-normative cause information,
but the normative project outcome MUST NOT depend on inspection order.

---

## MNIR-DOMAIN-041 — DomainProject non-Domain source

If the source Expression successfully derives
`ValueType::Intrinsic(I)`, type inspection of the `DomainProject` MUST report
`DomainProjectSourceNotDomain` carrying the actual `ValueType` or equivalent
machine-readable information.

The implementation MUST NOT infer a Domain Type from the intrinsic type or
from presentation names.

---

## MNIR-DOMAIN-042 — Unresolved Domain Type during inspection

If type inspection directly encounters a `ValueType::Domain(T)` or
`DomainConstruct` target whose `TypeId` does not resolve in an Active
transaction working state, it MUST report a typed `UnresolvedDomainType`
failure containing `T`.

If that failure is encountered through a `DomainProject` source, the project
outcome is `DomainProjectSourceTypeUnavailable` according to
`MNIR-DOMAIN-040`.

No former representation may be returned as authoritative.

---

## MNIR-DOMAIN-043 — Read-only inspection does not poison

Any expression-type inspection failure defined by this specification is
read-only and MUST NOT by itself poison an Active mutation transaction.

---

# 15. Shared Expression dependency graph

## MNIR-DOMAIN-044 — Pure Expression classification

`TextLiteral`, `BytesLiteral`, `DomainConstruct`, and `DomainProject` are pure
Expressions.

They MUST NOT appear in `EffectSequence` and MUST NOT acquire semantic
execution order from Expression collection position.

---

## MNIR-DOMAIN-045 — Dependency participation

The `value` references of `DomainConstruct` and `DomainProject` MUST be normal
Expression dependencies in the shared Block-local Expression dependency graph.

They MUST participate in existing same-Block integrity, snapshot, fork,
cascade, and acyclicity rules together with arithmetic, comparison, and Call
dependencies.

---

## MNIR-DOMAIN-046 — Acyclicity

A committed Program MUST NOT contain a dependency cycle involving any
combination of existing Expressions, `DomainConstruct`, and `DomainProject`.

Safe construction APIs MUST prevent creation of such cycles. This
specification does not introduce operand retargeting or individual Expression
mutation.

---

# 16. Function signatures and Calls

## MNIR-DOMAIN-047 — Parameter ValueType

Every Parameter MUST have exactly one `ValueType`.

Changing that `ValueType` MUST preserve `ParameterId` and Parameter position.
A `ValueType::Domain(T)` Parameter is structurally valid only while `T`
resolves to an existing Domain Type in the same Program revision.

---

## MNIR-DOMAIN-048 — Function return ValueType

Every Function MUST have exactly one return `ValueType`.

Changing that `ValueType` MUST preserve `FunctionId`. A
`ValueType::Domain(T)` return is structurally valid only while `T` resolves to
an existing Domain Type in the same Program revision.

---

## MNIR-DOMAIN-049 — Signature shape

A Function's signature shape MUST comprise the ordered sequence of Parameter
`ValueType` values and the Function return `ValueType`.

Presentation metadata and entity identities remain excluded from signature
shape. Domain Types in signature shape compare by exact `TypeId`.

---

## MNIR-DOMAIN-050 — ParameterReference type

A `ParameterReference` derived type MUST be the referenced Parameter's current
`ValueType`.

If the Parameter has `ValueType::Domain(T)` and `T` does not resolve in an
Active transaction working state, type inspection MUST report
`UnresolvedDomainType(T)` rather than returning a stale or invented type.

---

## MNIR-DOMAIN-051 — Call result type

A Call Expression's derived type MUST be the target Function's current return
`ValueType`.

Argument invalidity MUST NOT make the Call result type unavailable when the
target and its declared return type are structurally resolvable.

---

## MNIR-DOMAIN-052 — Call argument validity

Each Call argument MUST be checked against its corresponding Parameter using
exact `ValueType` equality.

There is no implicit conversion. In particular, an intrinsic value of a Domain
Type's representation does not satisfy a Parameter of that Domain Type, and a
value of one Domain Type does not satisfy a different Domain Type with the
same representation.

Call effectfulness and `EffectSequence` semantics remain unchanged.

---

# 17. Return and Branch typing

## MNIR-DOMAIN-053 — Return ValueType equality

A Return is semantically valid with respect to its Function declaration only
when the Return Expression's successfully derived `ValueType` equals the
Function's declared return `ValueType`.

An unavailable Return Expression type suppresses the Return mismatch
diagnostic exactly as under earlier verifier rule sets.

---

## MNIR-DOMAIN-054 — Branch requires intrinsic Bool

A Branch condition is semantically valid only when its successfully derived
type is exactly:

```text
ValueType::Intrinsic(Bool)
```

`ValueType::Domain(T)` is not a valid Branch condition even when `T` is
represented by `Bool`. Explicit projection is required.

---

# 18. Arithmetic typing

## MNIR-DOMAIN-055 — Arithmetic support remains intrinsic-only

Arithmetic is supported only for equal operands of exactly:

```text
ValueType::Intrinsic(Int32)
ValueType::Intrinsic(Int64)
```

The valid result type is the equal operand `ValueType`.

`Text`, `Bytes`, `Bool`, `Unit`, and every Domain Type are unsupported
arithmetic operand types. A numeric representation does not grant arithmetic
to a Domain Type.

---

## MNIR-DOMAIN-056 — Arithmetic outcome precedence

Arithmetic type inspection under this specification MUST use:

```text
1. if either operand type cannot be derived:
       OperandTypeUnavailable

2. otherwise, if operand ValueTypes differ:
       OperandTypeMismatch(left ValueType, right ValueType)

3. otherwise, if the equal ValueType is not intrinsic Int32 or Int64:
       UnsupportedOperandType(ValueType)

4. otherwise:
       ValidType(the equal intrinsic ValueType)
```

The outcome MUST be deterministic and read-only.

---

## MNIR-DOMAIN-057 — Explicit projection enables intrinsic arithmetic

A valid `DomainProject` of an `Int32`- or `Int64`-represented Domain Type
produces the corresponding intrinsic `ValueType` and MAY therefore be used by
arithmetic according to existing arithmetic rules.

This does not grant arithmetic to the Domain Type itself.

---

# 19. Comparison typing

## MNIR-DOMAIN-058 — Equality-supported ValueTypes

Under V0_5, `Equal` and `NotEqual` are supported for equal operands of each
intrinsic type:

```text
Int32
Int64
Bool
Unit
Text
Bytes
```

`Text` equality uses exact scalar-sequence equality from `MNIR-DOMAIN-007`.
`Bytes` equality uses exact octet-sequence equality from
`MNIR-DOMAIN-008`.

No `ValueType::Domain(T)` is equality-supported in this specification, even
when both operands have the same `TypeId`.

---

## MNIR-DOMAIN-059 — Ordering-supported ValueTypes

`LessThan`, `LessThanOrEqual`, `GreaterThan`, and `GreaterThanOrEqual` remain
supported only for equal operands of exactly:

```text
ValueType::Intrinsic(Int32)
ValueType::Intrinsic(Int64)
```

Ordering is unsupported for `Bool`, `Unit`, `Text`, `Bytes`, and every Domain
Type. This specification defines no lexicographic Text or Bytes ordering.

---

## MNIR-DOMAIN-060 — Comparison outcome precedence

Comparison type inspection under this specification MUST use:

```text
1. if either operand type cannot be derived:
       OperandTypeUnavailable

2. otherwise, if operand ValueTypes differ:
       OperandTypeMismatch(left ValueType, right ValueType)

3. otherwise, if the comparison family does not support the equal ValueType:
       UnsupportedOperandType(ValueType)

4. otherwise:
       ValidType(ValueType::Intrinsic(Bool))
```

The outcome MUST be deterministic and read-only.

---

## MNIR-DOMAIN-061 — Explicit projection enables intrinsic comparison

A valid `DomainProject` produces an intrinsic value that MAY participate in
the comparison operations supported for that intrinsic type.

This is an explicit comparison of representations and MUST NOT be interpreted
as Domain Type equality or ordering semantics.

---

# 20. Complete Expression type inspection

## MNIR-DOMAIN-062 — Generalized result

The read-only Expression type-inspection API MUST return a successfully derived
`ValueType` or a typed inspection failure.

It MUST NOT return an `IntrinsicType` as though intrinsic types were the
complete type universe.

---

## MNIR-DOMAIN-063 — Expression-family derivation table

Type inspection MUST follow this table:

| Expression family | Successful derived `ValueType` |
| --- | --- |
| `Int32Literal` | `Intrinsic(Int32)` |
| `Int64Literal` | `Intrinsic(Int64)` |
| `BoolLiteral` | `Intrinsic(Bool)` |
| `UnitLiteral` | `Intrinsic(Unit)` |
| `TextLiteral` | `Intrinsic(Text)` |
| `BytesLiteral` | `Intrinsic(Bytes)` |
| `ParameterReference` | referenced Parameter's current `ValueType` |
| Arithmetic | equal supported intrinsic integer `ValueType` |
| Comparison | `Intrinsic(Bool)` when valid |
| Call | target Function's current return `ValueType` |
| `DomainConstruct(T, value)` | `Domain(T)` when `T` resolves |
| `DomainProject(value)` | source Domain Type's current intrinsic representation when valid |

The failure and precedence rules in earlier specifications remain applicable
after replacing intrinsic-only operand values with `ValueType` and adding the
Domain-specific failures in this document.

---

## MNIR-DOMAIN-064 — Type-inspection failure categories

The type-inspection error model MUST be able to distinguish at least:

```text
UnresolvedParameter
UnresolvedFunction
UnresolvedDomainType(TypeId)
OperandTypeUnavailable
OperandTypeMismatch(ValueType, ValueType)
UnsupportedOperandType(ValueType)
DomainProjectSourceTypeUnavailable
DomainProjectSourceNotDomain(ValueType)
```

Exact Rust variant names and additional non-normative cause data are
implementation-defined. Failure categories MUST NOT be replaced by panics.

---

# 21. Structural validity

## MNIR-DOMAIN-065 — Structurally valid Domain Type

A Domain Type is structurally valid when:

1. it has exactly one valid persistent `TypeId`;
2. it is owned by exactly one existing Module;
3. its `TypeId` is unique among Domain Types in the Program state;
4. it has exactly one valid intrinsic representation; and
5. its optional presentation metadata is representable.

---

## MNIR-DOMAIN-066 — Structurally valid Domain references

Every committed `ValueType::Domain(T)` in a Function or Parameter and every
committed `DomainConstruct` target `T` MUST resolve to exactly one existing
Domain Type in the same Program revision.

The referenced Domain Type MAY be owned by any Module in that Program.
Cross-Module Domain references are permitted; cross-Program references are not
defined.

---

## MNIR-DOMAIN-067 — DomainProject structural boundary

`DomainProject` stores only an Expression dependency and therefore has no
independent stored `TypeId` reference to validate.

Its same-Block dependency is structural. Whether its source is Domain-typed is
semantic. Structural validation MUST still reject any dangling Domain
reference stored by the source graph or a relevant signature.

---

## MNIR-DOMAIN-068 — Structural versus semantic Domain validity

The following are semantic invalidities and MUST remain structurally
representable when all references resolve:

- `DomainConstruct` source type unavailable;
- `DomainConstruct` representation mismatch;
- `DomainProject` source type unavailable;
- `DomainProject` source not Domain;
- Domain operands used directly by arithmetic or comparison;
- a Domain-backed Bool used directly as a Branch condition; and
- Call or Return `ValueType` mismatch.

Dangling `TypeId` references are structural invalidity, not semantic
diagnostics.

---

## MNIR-DOMAIN-069 — Structural commit rejection

A transaction MUST NOT commit a Program that violates
`MNIR-DOMAIN-065` through `MNIR-DOMAIN-067` or any inherited structural rule.

Structural rejection MUST remain atomic and MUST NOT be encoded as a V0_5
semantic diagnostic.

---

# 22. Controlled mutation

## MNIR-DOMAIN-070 — Domain Type operations

Controlled mutation MUST support operations equivalent to:

```text
add_domain_type(module_id, representation)
set_domain_type_representation(type_id, representation)
set_domain_type_preferred_name(type_id, optional_name)
set_domain_type_documentation(type_id, optional_documentation)
remove_domain_type(type_id)
```

The exact Rust signatures and optional presentation inputs are
implementation-defined.

Unknown Module or Type targets are operation failures and MUST use typed
errors, preserve operation atomicity, and poison the transaction under the
existing mutation model.

---

## MNIR-DOMAIN-071 — Domain Expression operations

Controlled mutation MUST support operations equivalent to:

```text
add_domain_construct(block_id, type_id, value)
add_domain_project(block_id, value)
```

Domain Expression semantic invalidity MUST NOT gate construction after the
structural preconditions in `MNIR-DOMAIN-034` and `MNIR-DOMAIN-038` pass.

---

## MNIR-DOMAIN-072 — ValueType mutation operations

Existing Parameter-type and Function-return-type mutation operations MUST
accept `ValueType`.

Setting `ValueType::Domain(T)` MUST require `T` to resolve in the current
working state before the operation succeeds. The operation does not allocate
a new `TypeId`.

---

# 23. Removal and repair

## MNIR-DOMAIN-073 — Domain Type removal

Removing a Domain Type MUST remove it from its owning Module and preserve its
issued `TypeId` as permanently unavailable for reuse.

Removing a Domain Type MUST NOT implicitly remove or rewrite references owned
outside that Domain Type. The operation MAY therefore leave temporarily
dangling `TypeId` references in an Active transaction and MUST NOT fail or
poison solely because such references exist.

---

## MNIR-DOMAIN-074 — Module removal cascade

Removing a Module MUST remove all Domain Types owned by that Module together
with its existing descendant entities.

This applies to committed and provisional Domain Types. Every TypeId already
issued for a removed Domain Type remains unavailable according to
`MNIR-DOMAIN-017`, regardless of whether the transaction later commits,
fails, or is discarded.

References from surviving Modules to removed Domain Types MAY temporarily
dangle and MUST be repaired before commit. Removal of a Module MUST NOT
silently cascade into unrelated Modules merely to repair those references.

---

## MNIR-DOMAIN-075 — Repairing signature references

A dangling Domain reference in a Function signature MAY be repaired using only
operations already available in this increment, including:

- change the Parameter or return `ValueType` to an intrinsic or existing
  Domain Type;
- remove the affected Parameter;
- remove the affected Function; or
- remove the affected Module.

No replacement Domain Type can recreate the removed `TypeId`.

---

## MNIR-DOMAIN-076 — Repairing DomainConstruct references

A dangling `DomainConstruct` target MAY be repaired by removing an enclosing
entity through existing cascade operations:

- remove its non-entry Block, while repairing all CFG references and
  preserving all other structural rules;
- remove the containing Function body;
- remove the containing Function; or
- remove the containing Module.

Domain Type Foundations 0.1 defines no individual Expression removal,
retargeting, or mutation operation. Implementations MUST NOT invent one as a
normative repair path.

---

## MNIR-DOMAIN-077 — Indirect DomainProject repair

Because `DomainProject` has no stored `TypeId`, removal repair applies to the
source construct or signature that stores the dangling reference. Existing
enclosing cascade operations MAY remove the projection together with that
source graph.

---

# 24. Program state, snapshots, and forks

## MNIR-DOMAIN-078 — Program semantic state

The following MUST participate in Program semantic state and semantic no-op
comparison:

- Domain Type membership, `TypeId`, ownership, and representation;
- Function and Parameter `ValueType` values;
- `TextLiteral` scalar sequences;
- `BytesLiteral` octet sequences;
- `DomainConstruct` target and source references; and
- `DomainProject` source references.

Presentation metadata remains excluded from type identity, type equality, and
semantic verification. Its storage and revision behavior remains governed by
existing Program presentation rules.

Allocation-authority metadata remains excluded from semantic no-op comparison
according to Persistent Semantic Identity 0.1.

---

## MNIR-DOMAIN-079 — Snapshot preservation

A Program snapshot MUST preserve exactly:

- each Domain Type's `TypeId`, owner, representation, and presentation
  metadata;
- all `ValueType::Domain` references;
- all new Expression identities, contents, and dependencies; and
- the allocation-authority observation after any TypeId issuance.

Historical snapshot observations MUST remain immutable after later mutations
or allocations.

---

## MNIR-DOMAIN-080 — Normal fork preservation

A normal fork MUST preserve every inherited `TypeId` and every inherited
TypeId reference exactly. No remapping is permitted.

The fork MUST preserve representations, ownership, presentation metadata,
signatures, Expression data, dependencies, CFG, and EffectSequence contents.

---

## MNIR-DOMAIN-081 — Post-fork allocation

A normal fork MUST receive fresh allocation authority as defined by
Persistent Semantic Identity 0.1. Domain Types created after the fork MUST use
the active namespace and shared counter of the lineage in which they are
created.

The source and fork MUST NOT share post-fork allocation authority. Neither may
allocate a new `TypeId` in the other's active namespace.

---

# 25. Security boundary

## MNIR-DOMAIN-082 — Names imply no security semantics

Domain Type names, documentation, Module names, Parameter names, and Function
names MUST NOT imply validation, trust, sanitization, authentication,
authorization, confidentiality, integrity protection, or any other security
property.

For example, a Domain Type named `EmailAddress` and represented by `Text` is
nominally distinct from `Text` but is not thereby validated as an email
address.

---

## MNIR-DOMAIN-083 — Security-looking names remain nominal only

Names such as:

```text
PasswordHash
Secret
Credential
Validated
Untrusted
```

MUST confer no special semantics under this specification. Security types,
qualifiers, validation proofs, and information-flow rules remain future work.

---

# 26. Verification rule-set applicability

## MNIR-DOMAIN-084 — Older rule sets remain immutable

`SemanticVerificationAndDiagnosticsV0_1` through `V0_4` MUST retain their
existing meanings. They MUST NOT be silently expanded to understand `Text`,
`Bytes`, Domain Types, `ValueType::Domain`, `TextLiteral`, `BytesLiteral`,
`DomainConstruct`, or `DomainProject`.

---

## MNIR-DOMAIN-085 — V0_1 through V0_4 inapplicability

V0_1 through V0_4 are not applicable to a Program revision containing any of:

- a Domain Type declaration;
- `ValueType::Domain` in any signature;
- `Intrinsic(Text)` or `Intrinsic(Bytes)` in any signature;
- a `TextLiteral` or `BytesLiteral`;
- a `DomainConstruct`; or
- a `DomainProject`.

Existing inapplicability rules for comparison, multi-Block CFG, and Calls
continue to apply independently.

---

## MNIR-DOMAIN-086 — Complete applicability scan

Each V0_1 through V0_4 verification request MUST scan the complete Program,
including every Module, Domain Type collection, Function signature, Block, and
Expression, before beginning semantic verification.

If any unsupported type form or construct is found, the request MUST fail as
rule-set inapplicable, produce no semantic Diagnostic, produce no partial
verification result, and produce no `VerifiedProgram`.

---

# 27. Semantic Verification and Diagnostics V0_5

## MNIR-DOMAIN-087 — Verification V0_5

This specification introduces:

```text
SemanticVerificationAndDiagnosticsV0_5
```

The public verification API MUST permit explicit V0_5 selection.

---

## MNIR-DOMAIN-088 — V0_5 coverage

V0_5 MUST understand:

- every construct understood by V0_4;
- all six intrinsic types;
- generalized `ValueType` signatures and Expression typing;
- Domain Type identity and current representation;
- `TextLiteral` and `BytesLiteral`;
- `DomainConstruct` and `DomainProject`; and
- the Domain-aware arithmetic, comparison, Return, Branch, and Call rules in
  this document.

V0_5 MUST preserve every applicable V0_4 semantic check.

---

## MNIR-DOMAIN-089 — V0_5 binding

A `VerifiedProgram` produced by successful V0_5 verification MUST report
`SemanticVerificationAndDiagnosticsV0_5` as its authoritative rule set and
remain bound to the verified `ProgramId`, `RevisionId`, and immutable snapshot.

Evidence for V0_1, V0_2, V0_3, or V0_4 MUST NOT satisfy a requirement for
V0_5 verification.

---

## MNIR-DOMAIN-090 — V0_5 success

A Program revision may produce a V0_5 `VerifiedProgram` if and only if:

- it is structurally valid;
- V0_5 is applicable; and
- no inherited or Domain Type Foundations Error diagnostic applies.

Verification MUST remain read-only, revision-neutral, deterministic in
semantic result, non-repairing, and non-evaluating.

---

## MNIR-DOMAIN-091 — Complete V0_5 traversal

V0_5 MUST inspect every applicable Function signature, Branch, Return,
arithmetic Expression, comparison Expression, Call, `DomainConstruct`, and
`DomainProject` in every Module.

Unreturned, unused, or presentation-unnamed Expressions MUST NOT be skipped.

---

# 28. Versioned diagnostic contracts

## MNIR-DOMAIN-092 — Historical diagnostic contracts are immutable

For verification rule sets V0_1 through V0_4, diagnostic codes
`MNIR-DIAG-001` through `MNIR-DIAG-013` MUST retain exactly their historical:

- applicability and trigger conditions;
- primary semantic subjects;
- severity;
- payload field names and field types;
- payload ordering requirements; and
- duplicate-prevention semantics.

In particular, every historically specified `IntrinsicType` payload field
MUST remain externally observable as `IntrinsicType`. It MUST NOT be exposed
as `ValueType::Intrinsic(...)` under V0_1 through V0_4.

An implementation MAY internally share code or use a generalized carrier,
but the rule-set-specific public and observable contract MUST preserve the
historical payload schema exactly. A Program verified with V0_1 through V0_4
MUST NOT expose a V0_5-only diagnostic.

---

## MNIR-DOMAIN-093 — V0_5 diagnostic versioning

V0_5 MAY reuse a historical diagnostic code only when the complete normative
payload can be represented exactly by that historical diagnostic's existing
payload schema.

If a Domain `ValueType` must appear in a diagnostic payload, V0_5 MUST use the
V0_5-specific diagnostic defined by this document. Historical diagnostic
payload schemas MUST NOT be widened.

Consequently:

```text
V0_1 through V0_4:
    historical triggers, codes, and payload schemas

V0_5:
    ValueType-based trigger semantics
    historical diagnostics only when their payload schemas remain exact
    V0_5-specific diagnostics when Domain ValueTypes must be represented
```

---

## MNIR-DOMAIN-094 — Historical payloads reused by V0_5

When V0_5 reuses `MNIR-DIAG-001` through `MNIR-DIAG-013`, every payload field
MUST retain its historical name and type. This includes:

- `IntrinsicType` for every type-valued field of `MNIR-DIAG-002`,
  `MNIR-DIAG-003`, `MNIR-DIAG-004`, `MNIR-DIAG-006`, `MNIR-DIAG-007`,
  `MNIR-DIAG-009`, `MNIR-DIAG-010`, and `MNIR-DIAG-013`;
- the identifier-only unavailable payloads of `MNIR-DIAG-001`,
  `MNIR-DIAG-005`, and `MNIR-DIAG-008`;
- the expected and actual counts of `MNIR-DIAG-011`;
- the strictly increasing sequence of zero-based unavailable argument indices
  of `MNIR-DIAG-012`; and
- every historical Expression, Function, Block, target Function, operator,
  and zero-based index field.

---

## MNIR-DOMAIN-095 — Diagnostic collection properties

All V0_5 diagnostics MUST have severity `Error`.

Global diagnostic collection order is non-semantic. For each diagnostic code
and normative primary subject, the verifier MUST emit at most one diagnostic.
Repeated verification of the same revision under V0_5 MUST produce equivalent
codes, primary subjects, and normative payloads.

---

# 29. V0_5 diagnostics

V0_5 adds the following stable codes:

```text
MNIR-DIAG-014  DomainConstructSourceTypeUnavailable
MNIR-DIAG-015  DomainConstructRepresentationMismatch
MNIR-DIAG-016  DomainProjectSourceTypeUnavailable
MNIR-DIAG-017  DomainProjectSourceNotDomain
MNIR-DIAG-018  ArithmeticOperandValueTypeMismatch
MNIR-DIAG-019  ArithmeticUnsupportedValueType
MNIR-DIAG-020  ComparisonOperandValueTypeMismatch
MNIR-DIAG-021  ComparisonUnsupportedValueType
MNIR-DIAG-022  ReturnValueTypeMismatch
MNIR-DIAG-023  BranchConditionValueTypeNotBool
MNIR-DIAG-024  ControlFlowReturnValueTypeMismatch
MNIR-DIAG-025  CallArgumentValueTypeMismatch
```

## MNIR-DOMAIN-096 — Domain diagnostic subjects and severity

Each `MNIR-DIAG-014` through `MNIR-DIAG-017` diagnostic MUST have severity
`Error` and primary subject equal to the affected `DomainConstruct` or
`DomainProject` `ExpressionId`.

---

## MNIR-DOMAIN-097 — DomainConstructSourceTypeUnavailable

V0_5 MUST emit `MNIR-DIAG-014` when a `DomainConstruct` source Expression has
no successfully derived `ValueType`.

Its normative payload MUST contain:

```text
expression_id: affected DomainConstruct ExpressionId
type_id: target TypeId
source_expression_id: source ExpressionId
```

---

## MNIR-DOMAIN-098 — DomainConstructRepresentationMismatch

V0_5 MUST emit `MNIR-DIAG-015` when a `DomainConstruct` source has a derived
type that is not exactly `ValueType::Intrinsic(R)`, where `R` is the target
Domain Type's current representation.

Its normative payload MUST contain:

```text
expression_id: affected DomainConstruct ExpressionId
type_id: target TypeId
source_expression_id: source ExpressionId
expected_representation: IntrinsicType
actual_type: ValueType
```

`expected_representation` remains `IntrinsicType` by definition; it MUST NOT be
generalized to `ValueType`.

---

## MNIR-DOMAIN-099 — DomainConstruct diagnostic exclusivity

For one `DomainConstruct`, V0_5 MUST emit at most one of `MNIR-DIAG-014` and
`MNIR-DIAG-015`.

Unavailable source type takes precedence over representation mismatch. The
construct's result type remains `ValueType::Domain(type_id)` for verification
of dependent Expressions and terminators.

---

## MNIR-DOMAIN-100 — DomainProjectSourceTypeUnavailable

V0_5 MUST emit `MNIR-DIAG-016` when a `DomainProject` source has no
successfully derived `ValueType`.

Its normative payload MUST contain:

```text
expression_id: affected DomainProject ExpressionId
source_expression_id: source ExpressionId
```

---

## MNIR-DOMAIN-101 — DomainProjectSourceNotDomain

V0_5 MUST emit `MNIR-DIAG-017` when a `DomainProject` source successfully
derives `ValueType::Intrinsic(I)`.

Its normative payload MUST contain:

```text
expression_id: affected DomainProject ExpressionId
source_expression_id: source ExpressionId
actual_type: ValueType
```

---

## MNIR-DOMAIN-102 — DomainProject diagnostic exclusivity and propagation

For one `DomainProject`, V0_5 MUST emit at most one of `MNIR-DIAG-016` and
`MNIR-DIAG-017`.

Unavailable source type takes precedence over non-Domain type. An invalid
projection has no valid derived type.

Each dependent construct MUST independently apply its own normative
unavailable-type rule. If that rule requires an unavailable diagnostic, the
diagnostic MUST be emitted. This applies to arithmetic, comparison, Call
arguments, `DomainConstruct`, and `DomainProject`; no cascading-diagnostic
suppression is permitted unless a rule explicitly defines it.

Return remains different: it has no separate unavailable-type diagnostic, so
Return mismatch suppression MUST continue when the returned Expression has no
successfully derived `ValueType`.

---

## MNIR-DOMAIN-103 — No structural dangling-reference diagnostic

V0_5 MUST NOT emit a semantic diagnostic for a dangling `TypeId` reference.
Such a Program is structurally invalid and cannot be a normal V0_5 semantic
verification input.

---

## MNIR-DOMAIN-104 — Literals require no standalone diagnostic

A structurally valid `TextLiteral` or `BytesLiteral` requires no standalone
semantic diagnostic. Its use by another Expression or terminator remains
subject to that consumer's semantic rules.

---

## MNIR-DOMAIN-105 — V0_5 arithmetic diagnostic selection

V0_5 MUST apply the precedence in `MNIR-DOMAIN-056` and select exactly one
arithmetic type diagnostic per affected arithmetic Expression as follows:

1. If either operand has no successfully derived `ValueType`, emit historical
   `MNIR-DIAG-001` with its historical payload.
2. If both types are available and differ:
   - when both are `ValueType::Intrinsic`, emit historical `MNIR-DIAG-002`
     with `left_type: IntrinsicType` and `right_type: IntrinsicType`;
   - when either is `ValueType::Domain`, emit `MNIR-DIAG-018`.
3. If both types are equal but unsupported:
   - for an intrinsic type other than `Int32` or `Int64`, emit historical
     `MNIR-DIAG-003` with `operand_type: IntrinsicType`;
   - for `ValueType::Domain`, emit `MNIR-DIAG-019`.

`MNIR-DIAG-018` MUST have the affected arithmetic `ExpressionId` as primary
subject and MUST expose:

```text
expression_id: ExpressionId
left_type: ValueType
right_type: ValueType
```

`MNIR-DIAG-019` MUST have the affected arithmetic `ExpressionId` as primary
subject and MUST expose:

```text
expression_id: ExpressionId
operand_type: ValueType
```

For `MNIR-DIAG-019`, `operand_type` MUST be the equal
`ValueType::Domain(type_id)` operand type.

---

## MNIR-DOMAIN-106 — V0_5 comparison diagnostic selection

V0_5 MUST apply the precedence in `MNIR-DOMAIN-060` and select exactly one
comparison type diagnostic per affected comparison Expression as follows:

1. If either operand has no successfully derived `ValueType`, emit historical
   `MNIR-DIAG-005` with its historical payload.
2. If both types are available and differ:
   - when both are `ValueType::Intrinsic`, emit historical `MNIR-DIAG-006`
     with `left_type: IntrinsicType` and `right_type: IntrinsicType`;
   - when either is `ValueType::Domain`, emit `MNIR-DIAG-020`.
3. If both types are equal but the comparison kind does not support that type:
   - for an intrinsic type, emit historical `MNIR-DIAG-007` with
     `operand_type: IntrinsicType`;
   - for `ValueType::Domain`, emit `MNIR-DIAG-021`.

Equal intrinsic `Int32`, `Int64`, `Bool`, `Unit`, `Text`, and `Bytes` support
equality and inequality. Only equal intrinsic `Int32` and `Int64` support
ordering. Equal Domain operands support neither family.

`MNIR-DIAG-020` MUST have the affected comparison `ExpressionId` as primary
subject and MUST expose:

```text
expression_id: ExpressionId
left_type: ValueType
right_type: ValueType
```

`MNIR-DIAG-021` MUST have the affected comparison `ExpressionId` as primary
subject and MUST expose:

```text
expression_id: ExpressionId
operand_type: ValueType
```

For `MNIR-DIAG-021`, `operand_type` MUST be the equal
`ValueType::Domain(type_id)` operand type.

---

## MNIR-DOMAIN-107 — V0_5 Return diagnostic selection

V0_5 MUST supersede the intrinsic-only trigger clauses in `MNIR-VERIFY-038`,
`MNIR-VERIFY-039`, `MNIR-VERIFY-041`, `MNIR-CFG-091`, and `MNIR-CFG-094`
only for V0_5.

If the Return Expression has no successfully derived `ValueType`, V0_5 MUST
emit no Return mismatch diagnostic. If expected and actual `ValueType` values
are equal, the Return is valid.

For unequal available types in a single-Block body:

- if both are intrinsic, emit historical `MNIR-DIAG-004` with its historical
  `IntrinsicType` payload;
- if either is Domain, emit `MNIR-DIAG-022`.

`MNIR-DIAG-022` MUST have the affected `FunctionId` as primary subject and
MUST expose:

```text
function_id: FunctionId
return_expression_id: ExpressionId
expected_type: ValueType
actual_type: ValueType
```

For unequal available types in a multi-Block body:

- if both are intrinsic, emit historical `MNIR-DIAG-010` with its historical
  `IntrinsicType` payload;
- if either is Domain, emit `MNIR-DIAG-024`.

`MNIR-DIAG-024` MUST have the Return Block's `BlockId` as primary subject and
MUST expose:

```text
function_id: FunctionId
block_id: BlockId
return_expression_id: ExpressionId
expected_type: ValueType
actual_type: ValueType
```

Exact `ValueType` inequality MUST make nominally distinct Domain Types
mismatch even when their representations are equal.

---

## MNIR-DOMAIN-108 — V0_5 Branch diagnostic selection

V0_5 MUST supersede the intrinsic-only trigger clauses in `MNIR-CFG-085` and
`MNIR-CFG-087` only for V0_5 and select exactly one result:

```text
no successfully derived ValueType  -> MNIR-DIAG-008
ValueType::Intrinsic(Bool)         -> valid Branch condition
ValueType::Intrinsic(non-Bool)     -> MNIR-DIAG-009
ValueType::Domain(type_id)         -> MNIR-DIAG-023
```

Historical codes `MNIR-DIAG-008` and `MNIR-DIAG-009` MUST retain their
historical payloads. `MNIR-DIAG-023` MUST have the Branch Block's `BlockId` as
primary subject and MUST expose:

```text
block_id: BlockId
condition_expression_id: ExpressionId
actual_type: ValueType
```

For `MNIR-DIAG-023`, `actual_type` MUST be
`ValueType::Domain(type_id)`. A Bool-represented Domain value therefore
receives `MNIR-DIAG-023`, not `MNIR-DIAG-008` or `MNIR-DIAG-009`.

---

## MNIR-DOMAIN-109 — V0_5 Call argument diagnostic selection

V0_5 MUST supersede the intrinsic-only trigger clause in `MNIR-CALL-100` and
the intrinsic-only mismatch clauses in `MNIR-CALL-102` and `MNIR-CALL-103`
only for V0_5.

For every supplied argument corresponding to a Parameter:

- no successfully derived argument `ValueType` contributes its zero-based
  index to historical `MNIR-DIAG-012`;
- equal available argument and Parameter `ValueType` values are valid,
  including `Domain(T)` equal to `Domain(T)`; and
- unequal available values form a mismatch entry.

V0_5 MUST emit exactly one mismatch diagnostic per Call that has one or more
mismatch entries. If every mismatch entry has intrinsic expected and actual
types, it MUST emit historical `MNIR-DIAG-013` with all entries and historical
`IntrinsicType` payloads. If any mismatch entry has a Domain expected or
actual type, it MUST emit `MNIR-DIAG-025` instead of `MNIR-DIAG-013` and MUST
include every mismatch entry for that Call, including simultaneously
intrinsic-only entries.

`MNIR-DIAG-025` MUST have the affected Call `ExpressionId` as primary subject
and MUST expose:

```text
expression_id: ExpressionId
function_id: FunctionId
mismatches: ordered sequence of entries
```

Each entry MUST contain:

```text
argument_index: non-negative zero-based index
expected_type: ValueType
actual_type: ValueType
```

Entries MUST appear in strictly increasing `argument_index` order. Count,
unavailable, and mismatch diagnostics MAY coexist according to
`MNIR-CALL-104` through `MNIR-CALL-107`, but one Call MUST NOT receive both
`MNIR-DIAG-013` and `MNIR-DIAG-025`.

---

## MNIR-DOMAIN-110 — Complete diagnostic contract audit

The complete V0_5 diagnostic contract MUST be:

| Code | V0_5 trigger | Primary subject | Normative payload and ordering |
| --- | --- | --- | --- |
| `001` | Arithmetic operand type unavailable | arithmetic `ExpressionId` | `expression_id: ExpressionId` |
| `002` | Different available intrinsic arithmetic operands | arithmetic `ExpressionId` | `expression_id: ExpressionId`, `left_type: IntrinsicType`, `right_type: IntrinsicType` |
| `003` | Equal unsupported intrinsic arithmetic operands | arithmetic `ExpressionId` | `expression_id: ExpressionId`, `operand_type: IntrinsicType` |
| `004` | Unequal available intrinsic Return types in a single-Block body | `FunctionId` | `function_id: FunctionId`, `return_expression_id: ExpressionId`, `expected_type: IntrinsicType`, `actual_type: IntrinsicType` |
| `005` | Comparison operand type unavailable | comparison `ExpressionId` | `expression_id: ExpressionId` |
| `006` | Different available intrinsic comparison operands | comparison `ExpressionId` | `expression_id: ExpressionId`, `left_type: IntrinsicType`, `right_type: IntrinsicType` |
| `007` | Equal unsupported intrinsic comparison operands | comparison `ExpressionId` | `expression_id: ExpressionId`, `operand_type: IntrinsicType` |
| `008` | Branch condition type unavailable | Branch `BlockId` | `block_id: BlockId`, `condition_expression_id: ExpressionId` |
| `009` | Available intrinsic non-Bool Branch condition | Branch `BlockId` | `block_id: BlockId`, `condition_expression_id: ExpressionId`, `actual_type: IntrinsicType` |
| `010` | Unequal available intrinsic Return types in a multi-Block body | Return `BlockId` | `function_id: FunctionId`, `block_id: BlockId`, `return_expression_id: ExpressionId`, `expected_type: IntrinsicType`, `actual_type: IntrinsicType` |
| `011` | Call argument count differs from Parameter count | Call `ExpressionId` | `expression_id: ExpressionId`, `function_id: FunctionId`, `expected_count: non-negative integer`, `actual_count: non-negative integer` |
| `012` | One or more corresponding Call argument types unavailable | Call `ExpressionId` | `expression_id: ExpressionId`, `function_id: FunctionId`, `argument_indices: strictly increasing sequence of non-negative zero-based indices` |
| `013` | All available Call mismatch entries are intrinsic | Call `ExpressionId` | `expression_id: ExpressionId`, `function_id: FunctionId`, strictly increasing entries with `argument_index: non-negative zero-based index`, `expected_type: IntrinsicType`, `actual_type: IntrinsicType` |
| `014` | DomainConstruct source type unavailable | construct `ExpressionId` | `expression_id: ExpressionId`, `type_id: TypeId`, `source_expression_id: ExpressionId` |
| `015` | DomainConstruct source does not equal current intrinsic representation | construct `ExpressionId` | `expression_id: ExpressionId`, `type_id: TypeId`, `source_expression_id: ExpressionId`, `expected_representation: IntrinsicType`, `actual_type: ValueType` |
| `016` | DomainProject source type unavailable | project `ExpressionId` | `expression_id: ExpressionId`, `source_expression_id: ExpressionId` |
| `017` | DomainProject source is intrinsic | project `ExpressionId` | `expression_id: ExpressionId`, `source_expression_id: ExpressionId`, `actual_type: ValueType` |
| `018` | Different arithmetic operands with either type Domain | arithmetic `ExpressionId` | `expression_id`, `left_type: ValueType`, `right_type: ValueType` |
| `019` | Equal Domain arithmetic operands | arithmetic `ExpressionId` | `expression_id`, `operand_type: ValueType` |
| `020` | Different comparison operands with either type Domain | comparison `ExpressionId` | `expression_id`, `left_type: ValueType`, `right_type: ValueType` |
| `021` | Equal Domain comparison operands | comparison `ExpressionId` | `expression_id`, `operand_type: ValueType` |
| `022` | Unequal Return types with either type Domain in a single-Block body | `FunctionId` | `function_id`, `return_expression_id`, `expected_type: ValueType`, `actual_type: ValueType` |
| `023` | Available Domain Branch condition | Branch `BlockId` | `block_id`, `condition_expression_id`, `actual_type: ValueType` |
| `024` | Unequal Return types with either type Domain in a multi-Block body | Return `BlockId` | `function_id`, `block_id`, `return_expression_id`, `expected_type: ValueType`, `actual_type: ValueType` |
| `025` | Any available Call mismatch entry involves Domain | Call `ExpressionId` | `expression_id`, `function_id`, strictly increasing entries with `argument_index`, `expected_type: ValueType`, `actual_type: ValueType` |

Every code in the table MUST have severity `Error`. Historical applicability
MUST remain:

```text
MNIR-DIAG-001 through 004: V0_1 and later historical rule sets where applicable
MNIR-DIAG-005 through 007: V0_2 through V0_4 where applicable
MNIR-DIAG-008 through 010: V0_3 and V0_4 where applicable
MNIR-DIAG-011 through 013: V0_4 where applicable
```

V0_5 reuses codes `001` through `013` only in the cases specified by
`MNIR-DOMAIN-105` through `109`; the table's trigger wording for those codes
describes that V0_5 selection. Codes `014` through `025` MUST be V0_5-only.

---

# 30. Persistent Semantic Identity integration

## MNIR-DOMAIN-111 — PSI extension

For a Program conforming to this specification, the persistent semantic entity
set in `MNIR-PSI-003` is extended to:

```text
Module
Domain Type
Function
Parameter
Block
Expression
```

All general persistent entity rules in `MNIR-PSI-004` through
`MNIR-PSI-017`, `MNIR-PSI-022` through `MNIR-PSI-077`, and
`MNIR-PSI-082` through `MNIR-PSI-104` apply to `TypeId` where their subject is
a covered entity ID or ordinary existing-lineage allocation.

---

## MNIR-DOMAIN-112 — PSI snapshots and forks

The covered IDs and exact references in `MNIR-PSI-055` and
`MNIR-PSI-063` through `MNIR-PSI-073` MUST include `TypeId` and every
`ValueType::Domain` or `DomainConstruct` TypeId reference.

Normal forks preserve inherited TypeIds exactly and issue later TypeIds only
through their respective lineage allocation authority.

---

## MNIR-DOMAIN-113 — No PSI namespace creation change

Creating a Domain Type in an existing lineage MUST NOT invoke the identity
generation backend, create an allocation namespace, rotate a namespace, or
alter `ProgramId`.

Only the existing active allocation authority is advanced.

---

# 31. Complete normative audit of specifications 01 through 10

This section is normative. “Supersedes” applies only to the identified clause.
“Extends” retains the earlier rule and adds the stated Domain Type Foundations
behavior. All rules and acceptance requirements not identified below were
reviewed and remain unchanged.

## 31.1 Program Model 0.1

| Earlier rule | Disposition under Domain Type Foundations 0.1 |
| --- | --- |
| `MNIR-CORE-002` | Extended: Module contents may now include Domain Types in addition to Functions introduced later. |
| `MNIR-CORE-004`, `MNIR-CORE-041` | Extended by `MNIR-DOMAIN-019`, `020`, and `065`: a Module owns its Domain Types and validates their ownership. The PSI fork exception for inherited entity identity remains. |
| `MNIR-CORE-005`, `MNIR-CORE-012` | Extended by `MNIR-DOMAIN-013` and `014` with the distinct typed `TypeId` category. |
| `MNIR-CORE-014` | Extended by `MNIR-DOMAIN-023` and `027`: presentation and representation updates preserve `TypeId`. |
| `MNIR-CORE-020`, `MNIR-CORE-021` | Applied independently to the identity-based Domain Type collection by `MNIR-DOMAIN-020`; the Program's Module collection itself is unchanged. |
| `MNIR-CORE-022` through `MNIR-CORE-025` | Extended to Domain Type presentation by `MNIR-DOMAIN-023` and `024`. Presentation remains non-identifying while successful mutations retain existing revision behavior. |
| `MNIR-CORE-030`, `MNIR-CORE-038`, `MNIR-CORE-039` | Extended by `MNIR-DOMAIN-027`, `070`, `072`, and `078`; Domain semantic mutations use normal commit and explicit no-op rules. Allocator metadata remains excluded under PSI. |
| `MNIR-CORE-034` | Extended with the controlled operations in `MNIR-DOMAIN-032`, `070`, `071`, and `072`. PSI validate-before-reserve and issuance rules govern new IDs. |
| `MNIR-CORE-040`, `MNIR-CORE-042` | Extended by `MNIR-DOMAIN-065` through `069`. |
| `MNIR-CORE-047` | Superseded only to the extent that Domain Type is now a specified entity; other speculative entities remain prohibited. |

`MNIR-CORE-001`, `003`, `006` through `010`, `016`, `018`, `019`,
`026` through `029`, `031` through `033`, `035` through `037`, `043`
through `046`, `048` through `053`, `058` through `063`, and every Program
Model clause already superseded by specification 10 were reviewed. They remain
unchanged under the PSI disposition recorded in specification 10.

Affected acceptance requirements: `AR-CORE-002`, `003`, `006`, `007`, `008`,
`009`, `011`, and `012` retain their scenarios and are extended to the new
collection, typed ID, mutation, and revision state by `AR-DOMAIN-005` through
`014`, `AR-DOMAIN-036`, and `AR-DOMAIN-040` through `045`. All other
`AR-CORE-*` requirements remain applicable under specification 10.

## 31.2 Type System Foundations 0.1

| Earlier rule | Disposition under Domain Type Foundations 0.1 |
| --- | --- |
| `MNIR-TYPE-001`, `MNIR-TYPE-002`, `MNIR-TYPE-024` | Superseded by `MNIR-DOMAIN-003` and `005`; the closed set now has six alternatives. |
| `MNIR-TYPE-003` through `MNIR-TYPE-007`, `MNIR-TYPE-019`, `MNIR-TYPE-020` | Extended to `Text` and `Bytes` by `MNIR-DOMAIN-003` through `005`. `TypeId` applies only to Domain Types. |
| `MNIR-TYPE-017` | Extended by `MNIR-DOMAIN-009`, `011`, `026`, `036`, `052`, and `054` through `061`; no new implicit conversion is introduced. |
| `MNIR-TYPE-021` through `MNIR-TYPE-023`, `MNIR-TYPE-030` | Retained and made concrete by nominal identity and the security boundary in `MNIR-DOMAIN-022`, `026`, `082`, and `083`. |
| `MNIR-TYPE-031` | Extended: this document defines abstract Text and Bytes values and literal data only where required; it still does not require a general runtime value API. |
| `MNIR-TYPE-028`, `MNIR-TYPE-032` and section 21's unresolved `TypeId`, program-defined type, nominal type, Text, Bytes, Function-signature, Expression, and literal entries | Superseded only for the constructs explicitly defined here. A general speculative type abstraction remains prohibited. |

`MNIR-TYPE-008` through `MNIR-TYPE-016`, `MNIR-TYPE-018`,
`MNIR-TYPE-025` through `MNIR-TYPE-027`, and `MNIR-TYPE-029` remain
unchanged.

`AR-TYPE-001` and `AR-TYPE-008` are superseded for the closed intrinsic set by
`AR-DOMAIN-001` through `003`. `AR-TYPE-004` remains valid for intrinsic types
and is complemented by Domain `TypeId` evidence in `AR-DOMAIN-005` through
`009`. `AR-TYPE-010` is superseded only for the type features defined here.
`AR-TYPE-002`, `003`, `005` through `009`, and `011` through `015` remain
applicable, with Text/Bytes and security-name evidence added by
`AR-DOMAIN-001` through `004` and `AR-DOMAIN-047`.

## 31.3 Functions and Parameters 0.1

| Earlier rule | Disposition under Domain Type Foundations 0.1 |
| --- | --- |
| `MNIR-FUNC-021`, `MNIR-FUNC-055` | Extended: signature shape uses ordered Parameter `ValueType` values and return `ValueType` under `MNIR-DOMAIN-049`. |
| `MNIR-FUNC-025`, `MNIR-FUNC-026` | Superseded by `MNIR-DOMAIN-047` and `048`; types are `ValueType`, not intrinsic-only. |
| `MNIR-FUNC-027` | Extended by exact `ValueType` equality and the no-conversion rules in `MNIR-DOMAIN-011` and `052`. |
| `MNIR-FUNC-028` | Retained and extended: ValueType changes preserve Function/Parameter identity. |
| `MNIR-FUNC-042`, `MNIR-FUNC-043` | Their type inputs are generalized to `ValueType`; allocation behavior remains governed by PSI. |
| `MNIR-FUNC-052`, `MNIR-FUNC-053`, `MNIR-FUNC-054` | Their intrinsic-type clauses are superseded by resolvable `ValueType` structural validity in `MNIR-DOMAIN-047`, `048`, `066`, and `069`. |
| `MNIR-FUNC-056`, `MNIR-FUNC-057` | Retained; presentation and entity identity remain distinct from the generalized signature shape. |
| `MNIR-FUNC-066`, `MNIR-FUNC-067` | Superseded only for Domain Type signature behavior explicitly defined here. |

All ownership, order, removal, presentation, transaction, and PSI-revised
identity clauses remain unchanged.

`AR-FUNC-005`, `006`, `007`, `012`, `020`, and `034` are generalized from
intrinsic types to `ValueType` and are covered for Domain signatures by
`AR-DOMAIN-015` through `018`, `036`, and `037`. Other `AR-FUNC-*`
requirements remain applicable under the specification 10 identity audit.

## 31.4 Expressions and Basic Function Bodies 0.1

| Earlier rule | Disposition under Domain Type Foundations 0.1 |
| --- | --- |
| `MNIR-EXPR-032` | Generalized by `MNIR-DOMAIN-050` from intrinsic type to current Parameter `ValueType`. |
| `MNIR-EXPR-033` through `MNIR-EXPR-035` | Superseded for the complete Expression universe by `MNIR-DOMAIN-062` through `064`; every successfully typed Expression has one `ValueType`. Existing literal derivations remain as intrinsic alternatives. |
| `MNIR-EXPR-040`, `MNIR-EXPR-041` | Generalized to exact `ValueType` equality by `MNIR-DOMAIN-053`. |
| `MNIR-EXPR-047` | Extended with Text/Bytes literal operations by `MNIR-DOMAIN-029` through `032`. |
| `MNIR-EXPR-068`, `MNIR-EXPR-100` | Extended to expose the new Expression kinds and all semantic data in `MNIR-DOMAIN-029`, `030`, `033`, and `037`. |
| `MNIR-EXPR-069`, `MNIR-EXPR-090`, `MNIR-EXPR-091` | Generalized by `MNIR-DOMAIN-062` through `064`; Domain resolution failures remain typed and non-poisoning. |
| `MNIR-EXPR-070`, `MNIR-EXPR-072`, `MNIR-EXPR-073` | Extended by Domain Expression dependencies and Domain structural validation. |
| `MNIR-EXPR-074` | Retained and extended: new pure Expressions gain no order from collection position. |
| `MNIR-EXPR-079` | Extended by the semantic contents listed in `MNIR-DOMAIN-078`; allocation state remains revised by PSI. |
| `MNIR-EXPR-082` | Superseded only for Text/Bytes literals and Domain construction/projection. |

Existing ownership, Return representation, cascade, inspection, and
PSI-revised identity rules remain unchanged. `MNIR-EXPR-088` remains valid;
this specification introduces no generic Node, Statement, or Terminator ID.

`AR-EXPR-004` through `008`, `014`, `015`, `026`, `028`, `041`, and `043`
retain their scenarios with `ValueType` wrapping/generalization. New Expression
and inspection evidence is provided by `AR-DOMAIN-003`, `004`, `019` through
`029`, `038`, and `039`. All other `AR-EXPR-*` requirements remain applicable
under later supersessions.

## 31.5 Arithmetic Expressions 0.1

| Earlier rule | Disposition under Domain Type Foundations 0.1 |
| --- | --- |
| `MNIR-ARITH-019` through `MNIR-ARITH-027` | Generalized by `MNIR-DOMAIN-055` and `056`. Support remains exactly intrinsic Int32/Int64; Text, Bytes, and Domain types are unsupported. |
| `MNIR-ARITH-028` through `MNIR-ARITH-030`, `MNIR-ARITH-090` through `MNIR-ARITH-095` | Existing recursive and precedence behavior is retained with `ValueType` inspection outcomes and Domain inspection failures. Diagnostic payload selection is separately versioned by `MNIR-DOMAIN-092` through `110`. |
| `MNIR-ARITH-031`, `MNIR-ARITH-032` | Retained: Domain-related type invalidity remains semantically representable. |
| `MNIR-ARITH-062`, `MNIR-ARITH-063` | Extended to mixed dependency graphs containing Domain Expressions. |
| `MNIR-ARITH-069`, `MNIR-ARITH-070` | Extended by shared Domain dependency integrity. |
| `MNIR-ARITH-072`, `MNIR-ARITH-074` | Generalized to `ValueType` inspection and Return comparison. |
| `MNIR-ARITH-082` | Superseded only for the type interactions explicitly defined here. |

Checked arithmetic and fault semantics remain unchanged and apply only after
the static type is valid intrinsic Int32 or Int64.

`AR-ARITH-001` through `009`, `013` through `016`, `021`, `022`, and `031`
retain their scenarios using intrinsic `ValueType` alternatives. Domain
non-unwrapping and explicit projection are covered by `AR-DOMAIN-030`, `033`,
and `038`. Remaining `AR-ARITH-*` requirements remain unchanged.

## 31.6 Semantic Verification and Diagnostics 0.1

| Earlier rule | Disposition under Domain Type Foundations 0.1 |
| --- | --- |
| `MNIR-VERIFY-004` through `MNIR-VERIFY-019` | Retained for V0_5, with rule-set applicability and binding extended by `MNIR-DOMAIN-084` through `091`. |
| `MNIR-VERIFY-022`, `MNIR-VERIFY-023`, `MNIR-VERIFY-050` through `MNIR-VERIFY-066`, `MNIR-VERIFY-086` | Historical V0_1 contracts remain unchanged. V0_5 adds subjects, structured payloads, duplicate prevention, and unordered collection semantics only as explicitly defined by `MNIR-DOMAIN-092` through `110`. |
| `MNIR-VERIFY-027` through `MNIR-VERIFY-036` | Historical V0_1 arithmetic triggers and `IntrinsicType` payloads remain unchanged. For V0_5 only, intrinsic cases reuse the historical codes and exact schemas, while Domain-valued cases use `MNIR-DIAG-018` or `019` under `MNIR-DOMAIN-105`. |
| `MNIR-VERIFY-038`, `MNIR-VERIFY-039`, `MNIR-VERIFY-041` | Historical V0_1 through V0_4 intrinsic-only Return meaning remains unchanged. Their intrinsic-only trigger clauses are superseded for V0_5 only by the `ValueType`-based rules in `MNIR-DOMAIN-107`. |
| `MNIR-VERIFY-040`, `MNIR-VERIFY-042` | The historical `MNIR-DIAG-004` `IntrinsicType` payload and duplicate rule remain unchanged. V0_5 Domain-valued single-Block mismatch uses new `MNIR-DIAG-022`; no historical payload field is widened. |
| `MNIR-VERIFY-043` | Extended by `MNIR-DOMAIN-104` to Text/Bytes literals. Domain construction/projection are excluded from this no-diagnostic rule because they have explicit semantic checks. |
| `MNIR-VERIFY-044` | Retained for structurally valid ParameterReference; Domain resolution is a structural prerequisite. |
| `MNIR-VERIFY-048`, `MNIR-VERIFY-049`, `MNIR-VERIFY-071` | Retained; Domain presentation names do not affect verification. |
| `MNIR-VERIFY-067`, `MNIR-VERIFY-068` | V0_1 completeness remains immutable; V0_5 completeness is defined by `MNIR-DOMAIN-088` through `091`. |
| `MNIR-VERIFY-080`, `MNIR-VERIFY-084` | Superseded only for V0_5 and Domain diagnostics explicitly defined here; speculative diagnostic systems remain excluded. |

`AR-VERIFY-001` through `029` and `031` through `036` remain valid when their
selected rule set is applicable. Payload assertions in `AR-VERIFY-005` through
`010` and `021` remain exact `IntrinsicType` contracts under V0_1 through V0_4
and whenever V0_5 reuses their historical codes. Domain-valued V0_5 evidence
uses the new version-specific codes and is provided by `AR-DOMAIN-048` through
`055` and `071` through `077`.

## 31.7 Comparison Expressions 0.1

| Earlier rule | Disposition under Domain Type Foundations 0.1 |
| --- | --- |
| `MNIR-CMP-001` through `MNIR-CMP-004`, `MNIR-CMP-101` through `MNIR-CMP-106` | V0_1/V0_2 identity remains immutable and applicability is further restricted by `MNIR-DOMAIN-084` through `086`. |
| `MNIR-CMP-024` through `MNIR-CMP-040` | Generalized by `MNIR-DOMAIN-058` through `060`: Text/Bytes gain exact equality in V0_5, Domain types gain no equality or ordering, and type-inspection outcome values use `ValueType`. Diagnostic payloads remain versioned under `MNIR-DOMAIN-092` through `110`. |
| `MNIR-CMP-044` through `MNIR-CMP-048` | Retained; Text/Bytes equality is added only under V0_5 and existing intrinsic meanings do not change. |
| `MNIR-CMP-057` through `MNIR-CMP-062` | Extended to shared graphs containing Domain Expressions; structural/semantic separation remains. |
| `MNIR-CMP-064` through `MNIR-CMP-066` | Result is `ValueType::Intrinsic(Bool)` and Return uses exact `ValueType` equality. |
| `MNIR-CMP-070`, `MNIR-CMP-071`, `MNIR-CMP-076` through `MNIR-CMP-079` | Historical V0_2 meaning remains unchanged. V0_5 reuses the unavailable code and independently applies required unavailable propagation under `MNIR-DOMAIN-102` and `106`. |
| `MNIR-CMP-072` through `MNIR-CMP-075` | Historical V0_2 triggers and `IntrinsicType` payloads remain unchanged. For V0_5 only, intrinsic mismatch/unsupported cases reuse those exact schemas, while Domain-valued cases use `MNIR-DIAG-020` or `021` under `MNIR-DOMAIN-106`. |
| `MNIR-CMP-092`, `MNIR-CMP-094` | No implicit conversion remains; unresolved status is superseded only for interactions defined here. |

`AR-CMP-001` through `010`, `016`, `017`, and `021` through `027` retain
their intrinsic scenarios. Text/Bytes equality and Domain non-inheritance are
covered by `AR-DOMAIN-031`, `032`, `034`, and `038`. V0_1/V0_2 evidence is
supplemented by `AR-DOMAIN-048`. Other `AR-CMP-*` requirements remain
unchanged.

## 31.8 Conditional Control Flow 0.1

| Earlier rule | Disposition under Domain Type Foundations 0.1 |
| --- | --- |
| `MNIR-CFG-039` through `MNIR-CFG-042`, `MNIR-CFG-056` | Generalized by `MNIR-DOMAIN-054`: valid Branch type is exactly `ValueType::Intrinsic(Bool)`. |
| `MNIR-CFG-069` through `MNIR-CFG-072` | Extended to preserve Domain Expressions and TypeId references without changing CFG identity or order semantics. |
| `MNIR-CFG-075` through `MNIR-CFG-077`, `MNIR-CFG-129`, `MNIR-CFG-130` | Older applicability behavior is retained and extended through V0_4 by `MNIR-DOMAIN-084` through `086`. |
| `MNIR-CFG-078` through `MNIR-CFG-082`, `MNIR-CFG-126` through `MNIR-CFG-128` | V0_3 identity and binding remain unchanged; V0_5 is a distinct later rule set. |
| `MNIR-CFG-085`, `MNIR-CFG-087` | Historical V0_3/V0_4 intrinsic-only Branch triggers remain unchanged. For V0_5 only, they are superseded by the complete `ValueType` selection in `MNIR-DOMAIN-108`. |
| `MNIR-CFG-086`, `MNIR-CFG-088`, `MNIR-CFG-089` | Historical Branch payload schemas and exclusivity remain unchanged. V0_5 Domain Branch conditions use new `MNIR-DIAG-023`; `MNIR-DIAG-009.actual_type` remains `IntrinsicType`. |
| `MNIR-CFG-090`, `MNIR-CFG-091`, `MNIR-CFG-094` | Historical V0_3/V0_4 single- and multi-Block Return behavior remains unchanged. Intrinsic-only trigger clauses are superseded for V0_5 only by `MNIR-DOMAIN-107`, including unavailable mismatch suppression. |
| `MNIR-CFG-092`, `MNIR-CFG-093` | The historical `MNIR-DIAG-010` subject and `IntrinsicType` payload remain unchanged. V0_5 Domain-valued multi-Block mismatch uses new `MNIR-DIAG-024`; no historical field is widened. |
| `MNIR-CFG-095` through `MNIR-CFG-100` | Retained and included in complete V0_5 traversal and duplicate prevention. |

`AR-CFG-027` through `033`, `036`, `044`, and `045` retain their scenarios;
Branch and Return types are represented as `ValueType` under V0_5. A Bool-backed
Domain rejection is added by `AR-DOMAIN-035`. All other `AR-CFG-*`
requirements remain unchanged.

## 31.9 Function Calls and Sequencing Foundations 0.1

| Earlier rule | Disposition under Domain Type Foundations 0.1 |
| --- | --- |
| `MNIR-CALL-029` through `MNIR-CALL-032` | Generalized by `MNIR-DOMAIN-051` and `064`: Call result type is target return `ValueType`; unresolved Domain return references are typed inspection failures in repairable working state. |
| `MNIR-CALL-056`, `MNIR-CALL-057` | Extended to Domain Expression dependencies without making pure Domain Expressions sequence entries. |
| `MNIR-CALL-071` through `MNIR-CALL-076` | Generalized by `MNIR-DOMAIN-052`: argument compatibility is exact `ValueType` equality; result type remains derivable despite argument semantic invalidity. |
| `MNIR-CALL-077` through `MNIR-CALL-079` | Extended to preserve TypeId references exactly; PSI still prohibits normal-fork remapping. |
| `MNIR-CALL-083`, `MNIR-CALL-084` | Extended by new Expression and type state in `MNIR-DOMAIN-078`; EffectSequence semantics are unchanged. |
| `MNIR-CALL-085` through `MNIR-CALL-087` | V0_1 through V0_3 Call applicability remains and V0_4 gains the new inapplicability conditions in `MNIR-DOMAIN-085`. |
| `MNIR-CALL-088` through `MNIR-CALL-095` | V0_4 meaning and binding remain immutable; V0_5 is separately selected and bound. |
| `MNIR-CALL-100`, `MNIR-CALL-101` | Historical V0_4 intrinsic-unavailable meaning and payload remain unchanged. For V0_5 only, the trigger is superseded by absence of a successfully derived argument `ValueType` under `MNIR-DOMAIN-109`; `MNIR-DIAG-012` remains index-based. |
| `MNIR-CALL-102`, `MNIR-CALL-103`, `MNIR-CALL-107` | Historical V0_4 mismatch triggers, `IntrinsicType` payload, ordering, and coexistence remain unchanged. For V0_5 only, `MNIR-DOMAIN-109` selects historical `MNIR-DIAG-013` for all-intrinsic entries or new `MNIR-DIAG-025` when any entry involves Domain. |
| `MNIR-CALL-104` through `MNIR-CALL-106`, `MNIR-CALL-135` | Retained. Counts, unavailable payloads, target IDs, corresponding-position limits, and zero-based indices remain unchanged under V0_5. |
| `MNIR-CALL-120`, `MNIR-CALL-121` | Superseded only for Domain-aware Call typing explicitly defined here. |

Effectfulness, exact-once sequencing, dependency/effect-order consistency, and
Call execution exclusions remain unchanged.

`AR-CALL-011` through `013`, `023` through `025`, `035` through `043`, `048`,
`053`, `054`, and `058` retain their scenarios with `ValueType` where V0_5 is
selected. Domain Call evidence is added by `AR-DOMAIN-017`, `018`, `036`, and
`037`. Other `AR-CALL-*` requirements remain unchanged.

## 31.10 Persistent Semantic Identity 0.1

| Earlier rule | Disposition under Domain Type Foundations 0.1 |
| --- | --- |
| `MNIR-PSI-003` | Extended by `MNIR-DOMAIN-111` to include Domain Type/`TypeId`. |
| `MNIR-PSI-004`, `MNIR-PSI-006` through `MNIR-PSI-011` | Applied to the new typed `TypeId` category by `MNIR-DOMAIN-013` and `014`. |
| `MNIR-PSI-022` through `MNIR-PSI-045` | Applied to TypeId through the same allocation authority and issuance lifecycle by `MNIR-DOMAIN-015` through `018`. |
| `MNIR-PSI-046` through `MNIR-PSI-054` | Retained: TypeId allocation metadata is outside semantic revision state, creates no revision by itself, and guarantees non-reuse without full historical sets. |
| `MNIR-PSI-055` through `MNIR-PSI-060` | Extended to TypeId and Domain state by `MNIR-DOMAIN-079` and `112`. |
| `MNIR-PSI-061` through `MNIR-PSI-073` | Extended to preserve TypeIds and all TypeId references exactly by `MNIR-DOMAIN-080`, `081`, and `112`. |
| `MNIR-PSI-074` through `MNIR-PSI-077` | Extended so active TypeIds satisfy the same identity and authority invariants and dangling TypeId references are structurally rejected. |
| `MNIR-PSI-078` through `MNIR-PSI-081` | Earlier verifier compatibility remains true for Programs in those rule sets' applicability domains; V0_5 is introduced by this document. |
| `MNIR-PSI-082` through `MNIR-PSI-104` | Extended to TypeId where applicable; ordinary Domain Type creation uses no fresh entropy and introduces no persistence/serialization implementation. |

No rule in specification 10 is contradicted and specification 10 is not
modified. `AR-PSI-001` through `044` remain applicable to their original five
categories. TypeId-specific extension evidence is defined by
`AR-DOMAIN-005` through `014`, `042` through `045`, and `056` through `059`.

---

# 32. Implementation boundaries

## MNIR-DOMAIN-114 — Core ownership

`IntrinsicType`, `ValueType`, `TypeId`, Domain Type representation, the new
Expression kinds, structural validation, and controlled mutation MUST be owned
by `mnir-core`.

V0_5, `VerifiedProgram` rule-set binding, and semantic diagnostics MUST be
owned by `mnir-verify`. `mnir-core` MUST NOT depend on `mnir-verify`.

---

## MNIR-DOMAIN-115 — Read-only traversal

`mnir-core` MUST expose the minimum read-only traversal necessary for V0_5 to
enumerate Domain Types, inspect representations and ValueTypes, and inspect
new Expression data without exposing unrestricted mutation.

---

## MNIR-DOMAIN-116 — Controlled mutation and encapsulation

Production fields whose direct mutation could violate identity, ownership,
reference, or transaction invariants MUST remain encapsulated. Domain Type and
Domain Expression mutation MUST occur only through controlled operations.

---

## MNIR-DOMAIN-117 — Safe Rust

Implementing this specification MUST NOT require `unsafe` Rust.

Expected API failures MUST use typed errors rather than panics.

---

## MNIR-DOMAIN-118 — No speculative hierarchy

Implementation MUST extend the existing Program and Expression models. It
MUST NOT introduce a parallel Domain AST, generic Node hierarchy, generic type
registry, capability/trait hierarchy, or speculative type graph solely for
future features.

---

# 33. Explicitly deferred topics

## MNIR-DOMAIN-119 — Deferred topics are not 0.1 semantics

Domain Type Foundations 0.1 MUST NOT define or require:

- composite or record types;
- tuples;
- enums, unions, or variants;
- generics;
- aliases;
- recursive types or representation graphs;
- representation by another Domain Type;
- traits or capabilities;
- Domain equality or ordering capabilities;
- automatic numeric Domain semantics;
- validation or refinement types;
- security qualifiers;
- constrained strings or regular-expression validation;
- units of measure;
- methods or associated Functions;
- implicit conversions or coercions;
- Text encoding or decoding;
- Unicode normalization;
- collections;
- `Option` or `Result` types;
- package export/import semantics;
- canonical serialization;
- runtime layout, allocation, or memory management;
- execution or evaluation; or
- EasyH syntax.

---

## MNIR-DOMAIN-120 — Undefined behavior is a specification gap

If implementation requires externally observable Domain Type behavior not
defined by this document or inherited specifications, the implementation MUST
NOT invent it as MNIR semantics. The missing behavior MUST be reported as a
`SPECIFICATION GAP` under repository policy.

---

# 34. Acceptance requirements

Domain Type Foundations 0.1 conformance MUST demonstrate
`AR-DOMAIN-001` through `AR-DOMAIN-077`.

## AR-DOMAIN-001 — Complete intrinsic set

Demonstrate that the closed safe `IntrinsicType` representation has exactly
`Int32`, `Int64`, `Bool`, `Unit`, `Text`, and `Bytes`.

## AR-DOMAIN-002 — Text and Bytes distinction

Demonstrate that `Text` and `Bytes` are globally identified, distinct from
each other and all earlier intrinsic types, and receive no `TypeId`.

## AR-DOMAIN-003 — TextLiteral

Create Text literals containing empty text, non-ASCII scalar values, and two
canonically equivalent but scalar-distinct sequences. Verify exact scalar
preservation, `Intrinsic(Text)` typing, and no normalization.

## AR-DOMAIN-004 — BytesLiteral

Create Bytes literals including empty bytes, `0`, and `255`. Verify exact
length/order preservation and `Intrinsic(Bytes)` typing. Verify equal literal
contents do not imply equal `ExpressionId`.

## AR-DOMAIN-005 — TypeId persistent shape

Create a Domain Type and verify its `TypeId` exposes the lineage active
`AllocationNamespaceId` and issued monotonic counter while remaining opaque in
all other respects.

## AR-DOMAIN-006 — Typed TypeId category

Demonstrate that `TypeId` remains unequal to each existing persistent entity ID
category even under controlled evidence with equal namespace/counter
components.

## AR-DOMAIN-007 — Shared allocation ordering

Interleave Domain Type creation with Module, Function, Parameter, Block, and
Expression creation. Verify all six categories consume one namespace-wide
monotonic counter without category-specific counters. Using controlled
instrumentation, verify Domain Type creation keeps the active namespace and
does not invoke fresh Program/namespace identity generation.

## AR-DOMAIN-008 — Duplicate preferred names

Create two Domain Types in one Module with equal preferred names and equal
representations. Verify both are valid, have distinct TypeIds, and remain
nominally distinct. Update presentation metadata and verify TypeId,
representation, ownership, and type equality are unchanged while normal
revision behavior still applies.

## AR-DOMAIN-009 — Nominal equality

Verify two Domain Types represented by `Int64` have unequal
`ValueType::Domain` values, while two references containing the same `TypeId`
are equal.

## AR-DOMAIN-010 — Representation mutation

Change one Domain Type representation from `Int64` to `Text`. Verify `TypeId`,
owner, and presentation metadata are preserved and a semantic revision is
created.

## AR-DOMAIN-011 — Validate before TypeId reserve

Attempt Domain Type creation in an unknown Module. Verify failure occurs before
reservation, consumes no counter value, and follows existing poisoning rules.

## AR-DOMAIN-012 — TypeId discard non-reuse

Successfully reserve a TypeId and discard the transaction. In a later
transaction verify that counter is skipped and cannot be reused.

## AR-DOMAIN-013 — TypeId failed-commit non-reuse

Reserve a TypeId, create a different structural failure, and fail commit.
Verify semantic rollback and permanent TypeId non-reuse. In a separate
scenario, reserve a TypeId, trigger an operation failure that poisons the
transaction, discard it, and verify the TypeId is likewise never reused.

## AR-DOMAIN-014 — TypeId create-then-remove

Create and remove a Domain Type in one transaction and commit. Verify the
semantic contents are a no-op, the explicit commit creates a new RevisionId,
and the issued TypeId remains consumed.

## AR-DOMAIN-015 — Domain Parameter

Create and mutate a Parameter to a Domain `ValueType`. Verify exact TypeId
inspection and preservation of `ParameterId` and position.

## AR-DOMAIN-016 — Domain return

Create and mutate a Function return to a Domain `ValueType`. Verify exact
TypeId inspection and preservation of `FunctionId`.

## AR-DOMAIN-017 — Matching Domain Call

Construct a Call whose argument and target Parameter use the same Domain
`ValueType`. Verify the Call argument is valid and the result is the target's
current Domain return `ValueType`. Verify V0_5 emits neither `MNIR-DIAG-012`
nor either Call mismatch diagnostic.

## AR-DOMAIN-018 — Intrinsic does not satisfy Domain Call

Pass `Intrinsic(Int64)` to a Parameter of a Domain Type represented by
`Int64`. Verify structural representability and V0_5
`MNIR-DIAG-025` with exact `ValueType` expected/actual payload and no
`MNIR-DIAG-013`.

## AR-DOMAIN-019 — Valid DomainConstruct

Construct an `Int64`-represented Domain value from an `Int64` Expression.
Verify the construct derives `Domain(type_id)` and emits no Domain diagnostic.

## AR-DOMAIN-020 — DomainConstruct mismatch

Construct the same Domain Type from `Bool`. Verify the construct remains
structurally representable and typed as the Domain, and V0_5 emits
`MNIR-DIAG-015` with expected `Int64` and actual `Intrinsic(Bool)`.

## AR-DOMAIN-021 — DomainConstruct unavailable

Use a semantically type-invalid source Expression. Verify the construct result
remains Domain-typed and V0_5 emits `MNIR-DIAG-014`, not `MNIR-DIAG-015`.

## AR-DOMAIN-022 — Valid DomainProject

Project a valid Domain value. Verify the project derives the Domain Type's
current intrinsic representation and emits no Domain diagnostic.

## AR-DOMAIN-023 — DomainProject source not Domain

Project an intrinsic literal. Verify structural representability, a typed
`DomainProjectSourceNotDomain` inspection failure, and V0_5
`MNIR-DIAG-017` with the actual intrinsic `ValueType`.

## AR-DOMAIN-024 — DomainProject unavailable source

Project an Expression whose type is unavailable. Verify the project produces
the unavailable inspection outcome and V0_5 emits `MNIR-DIAG-016`, not
`MNIR-DIAG-017`.

## AR-DOMAIN-025 — Live representation affects construction

Create a valid construct for an `Int64` representation, then change the
representation to `Text`. Verify the construct keeps its Domain result type
but V0_5 now reports representation mismatch.

## AR-DOMAIN-026 — Live representation affects projection

For the same representation change, verify an existing valid projection's
derived type changes from `Intrinsic(Int64)` to `Intrinsic(Text)` without
changing any ExpressionId or TypeId.

## AR-DOMAIN-027 — Same-Block construction dependency

Verify unknown and cross-Block source Expressions are rejected before
ExpressionId reservation for both DomainConstruct and DomainProject.

## AR-DOMAIN-028 — Shared DAG acyclicity

Using internal corruption evidence or another safe conformance mechanism,
demonstrate that cycles mixing DomainConstruct/DomainProject with arithmetic,
comparison, or Call dependencies are structurally rejected.

## AR-DOMAIN-029 — Pure Expressions and EffectSequence

Verify TextLiteral, BytesLiteral, DomainConstruct, and DomainProject cannot
appear in EffectSequence, create no implicit sequence entries, and receive no
order from Expression enumeration. Separately construct the transitive path
required by `AR-DOMAIN-076`.

## AR-DOMAIN-030 — Arithmetic does not unwrap Domain

Use equal operands of the same `Int64`-represented Domain Type in arithmetic.
Verify `UnsupportedOperandType(Domain(type_id))`, not valid Int64 arithmetic.

## AR-DOMAIN-031 — Comparison does not unwrap Domain

Compare equal operands of the same Domain Type using equality and ordering.
Verify both are unsupported rather than inherited from the representation.

## AR-DOMAIN-032 — Text and Bytes comparison

Under V0_5, verify exact-sequence equality and inequality for Text and Bytes,
and verify ordering for either type is unsupported. Compare `Text == Bytes`
and verify both operand types are available, the comparison is invalid, and
historical `MNIR-DIAG-006` carries `IntrinsicType::Text` as `left_type` and
`IntrinsicType::Bytes` as `right_type`.

## AR-DOMAIN-033 — Projection enables intrinsic arithmetic

Project two values of an `Int64`-represented Domain Type and use the projections
in arithmetic. Verify intrinsic Int64 arithmetic typing succeeds.

## AR-DOMAIN-034 — Projection enables intrinsic comparison

Project Domain values and compare the projections. Verify the operation uses
only the representation's explicitly supported intrinsic comparison semantics.

## AR-DOMAIN-035 — Bool-backed Domain Branch

Use a Bool-represented Domain value directly as a Branch condition. Verify
`MNIR-DIAG-023` reports `actual_type = Domain(type_id)`, with no
`MNIR-DIAG-008` or `MNIR-DIAG-009`. Project it explicitly and verify the
projected intrinsic Bool is a valid condition.

## AR-DOMAIN-036 — Domain Return match

Return an Expression having exactly the Function's declared Domain
`ValueType`. Verify V0_5 reports no Return mismatch.

## AR-DOMAIN-037 — Domain Return mismatch

Return the underlying intrinsic representation or a different Domain Type.
Verify `MNIR-DIAG-022` for a single-Block body or `MNIR-DIAG-024` for a
multi-Block body contains exact expected and actual `ValueType` values and the
applicable normative primary subject. Verify nominally distinct Domain Types
mismatch even when their representations are equal.

## AR-DOMAIN-038 — Generalized type inspection

Exercise every Expression family in `MNIR-DOMAIN-063` and every failure family
in `MNIR-DOMAIN-064`. Verify successful inspection returns `ValueType`, error
precedence is deterministic, and read-only failures do not poison a transaction.

## AR-DOMAIN-039 — New Expression inspection data

Verify read-only inspection distinguishes TextLiteral, BytesLiteral,
DomainConstruct, and DomainProject and exposes all normative semantic data.

## AR-DOMAIN-040 — Unreferenced Domain Type removal

Remove an unreferenced Domain Type and commit. Verify removal succeeds,
unrelated state is unchanged, and the TypeId remains unavailable for reuse.

## AR-DOMAIN-041 — Referenced Domain Type removal rejects commit

Remove a Domain Type referenced by a surviving signature and DomainConstruct.
Verify removal itself succeeds without poisoning and commit fails structurally
with no Domain semantic diagnostic.

## AR-DOMAIN-042 — Signature removal repair

Repair dangling signature references using type mutation or entity removal and
verify commit succeeds without recreating or reusing the removed TypeId.

## AR-DOMAIN-043 — DomainConstruct removal repair

Repair a dangling DomainConstruct only through an available enclosing
Block/body/Function/Module cascade, including required CFG repair. Verify no
individual Expression removal or retargeting API is required.

## AR-DOMAIN-044 — Module removal with cross-Module references

Remove a Module owning Domain Types referenced from another Module. Verify the
surviving references temporarily dangle, unrelated Modules are not silently
removed, and commit requires explicit repair.

## AR-DOMAIN-045 — Snapshot preservation

Verify a snapshot preserves Domain Type identity, ownership, representation,
presentation, all Domain references, literal contents, Expression data, and
its immutable allocation-authority observation.

## AR-DOMAIN-046 — Fork preservation

Fork a Program containing signatures, constructs, projections, Calls, CFG,
and EffectSequence. Verify all inherited TypeIds and references are exact,
semantic contents are preserved, and no remapping occurs.

## AR-DOMAIN-047 — No name-derived validation or security

Create identically represented Domain Types named `EmailAddress`,
`PasswordHash`, `Secret`, and `Validated`. Verify no validation, security,
conversion, arithmetic, or comparison semantics arise from the names.

## AR-DOMAIN-048 — Older rule-set applicability

Independently construct each unsupported-form scenario in `AR-DOMAIN-071`.
For each applicable historical rule set V0_1 through V0_4, also place the
unsupported form only in a late Module and independent earlier semantic errors
in another Module. Verify complete applicability scanning returns only
rule-set inapplicability, with no partial diagnostics or `VerifiedProgram`.

## AR-DOMAIN-049 — Older rule sets remain usable

Verify V0_1 through V0_4 retain their existing results and rule-set bindings
for Programs wholly within each rule set's prior applicability domain. Include
the focused historical diagnostic payload evidence in `AR-DOMAIN-072`.

## AR-DOMAIN-050 — V0_5 success and binding

Verify a valid Program using Domain signatures, Text/Bytes literals,
construction, projection, Calls, arithmetic/comparison, Branch, and Return
produces a `VerifiedProgram` bound explicitly to V0_5, ProgramId, RevisionId,
and the immutable snapshot.

## AR-DOMAIN-051 — Domain diagnostics

Independently trigger `MNIR-DIAG-014` through `MNIR-DIAG-025`. Verify exact
codes, Error severity, normative primary subjects, payload field names, field
types, and payload ordering where specified.

## AR-DOMAIN-052 — V0_5 historical-diagnostic reuse

Under V0_5, independently trigger historical codes `MNIR-DIAG-002`,
`MNIR-DIAG-003`, `MNIR-DIAG-004`, `MNIR-DIAG-006`, `MNIR-DIAG-007`,
`MNIR-DIAG-009`, `MNIR-DIAG-010`, and `MNIR-DIAG-013` using only intrinsic
types. Verify every type-valued field retains its exact historical
`IntrinsicType` schema rather than `ValueType::Intrinsic`.

## AR-DOMAIN-053 — Unchanged diagnostic payloads

Under V0_5, trigger codes `MNIR-DIAG-001`, `MNIR-DIAG-005`,
`MNIR-DIAG-008`, `MNIR-DIAG-011`, and `MNIR-DIAG-012`. Verify their
existing non-type payload meanings, Function/Expression identities, counts,
and zero-based argument indices remain unchanged.

## AR-DOMAIN-054 — Domain diagnostic duplicate prevention

Verify at most one V0_5 diagnostic of a code is emitted for each normative
primary subject—ExpressionId, FunctionId, or BlockId as specified—and that
global diagnostic order is non-semantic.

## AR-DOMAIN-055 — Complete V0_5 traversal

Place independent Domain construction/projection errors, inherited arithmetic,
comparison, Branch, Return, and Call errors across multiple Modules and
unreturned Expressions. Verify every required diagnostic is discovered without
execution or repair.

## AR-DOMAIN-056 — Post-fork TypeId allocation isolation

Create an inherited Domain Type, then record the source allocation authority
immediately before and after fork creation and verify fork creation leaves it
unchanged. Record the fork authority. Allocate a Domain Type in the source and
verify only source authority advances; allocate one in the fork and verify
only fork authority advances. Verify new TypeIds use their own lineage
namespace while inherited TypeIds remain equal.

## AR-DOMAIN-057 — TypeId exhaustion last issuance

With controlled allocator state `Available(u64::MAX)`, create a Domain Type and
verify `namespace:MAX` is issued exactly once and authority becomes
`Exhausted`. Discard directly and verify the state remains exhausted and the ID
cannot be reused.

## AR-DOMAIN-058 — Allocation from Exhausted

In a separate scenario, attempt Domain Type creation from `Exhausted`. Verify
failure before issuance, transaction poisoning according to existing rules,
and no namespace rotation.

## AR-DOMAIN-059 — Allocator-only TypeId observation

Snapshot one Program revision, reserve a TypeId, and discard. Snapshot again.
Verify equal ProgramId/RevisionId and semantic contents, the same allocation
namespace, an advanced second counter observation, and immutability of the
first observation.

## AR-DOMAIN-060 — Existing conformance preservation

Run all acceptance evidence for specifications `01` through `10`. Verify it
remains valid wherever not explicitly superseded by section 31, with older
verifier rule sets evaluated only inside their unchanged applicability domains.

## AR-DOMAIN-061 — Domain Type ownership and unordered collection

Create Domain Types in multiple Modules. Verify each has exactly one owner,
lookup is by TypeId, duplicate names do not affect lookup, enumeration order is
non-semantic, and no Domain Type move operation is exposed as 0.1 behavior.

## AR-DOMAIN-062 — Representation restriction

Verify every Domain Type has exactly one of the six intrinsic representations
and that safe public APIs cannot create a Domain-represented, recursive,
missing, or unknown representation.

## AR-DOMAIN-063 — Cross-Module Domain references

Use a Domain Type owned by one Module in signatures and DomainConstruct
Expressions owned through another Module. Verify the references are valid
within one Program and remain exact identity references.

## AR-DOMAIN-064 — Unknown Domain mutation targets

Attempt representation, presentation, and removal operations against an
unknown or already removed TypeId. Verify typed operation failure, operation
atomicity, and existing transaction poisoning behavior without identity
allocation.

## AR-DOMAIN-065 — Domain Expression identity and cascades

Verify all four new Expression kinds use ordinary ExpressionId allocation,
uniqueness, retirement, and non-reuse. Verify Block, body, Function, and Module
removal cascades remove their new Expressions without affecting unrelated
entities.

## AR-DOMAIN-066 — Presentation-independent V0_5 result

Change only Domain Type preferred name or documentation and verify Domain
identity, `ValueType` equality, type inspection, and V0_5 semantic result are
unchanged, while the explicit successful commit still creates a new revision.

## AR-DOMAIN-067 — V0_5 read-only determinism

Run V0_5 repeatedly against the same immutable revision. Verify equivalent
codes, primary subjects, and normative payloads; no Program mutation,
allocation, repair, execution, or new RevisionId; and no reliance on collection
iteration order.

## AR-DOMAIN-068 — Semantic invalidity remains representable

Commit structurally valid examples of every semantic invalidity listed in
`MNIR-DOMAIN-068` and verify they are rejected only by the applicable V0_5
diagnostics, not structural commit validation.

## AR-DOMAIN-069 — Structural corruption rejection

Using narrowly scoped internal corruption evidence, verify duplicate TypeId,
invalid Domain ownership, unknown representation, dangling signature TypeId,
and dangling DomainConstruct TypeId are rejected structurally and do not
produce semantic diagnostics.

## AR-DOMAIN-070 — Scope and architecture boundaries

Verify `mnir-core` owns the type/model/mutation behavior, `mnir-verify` owns
V0_5, dependency direction remains one-way, no `unsafe` is used, and no
deferred construct, evaluator, serializer, EasyH syntax, generic Node/type
registry, or security inference is introduced.

## AR-DOMAIN-071 — Independent historical applicability matrix

Independently create each of these six Program scenarios:

1. `Intrinsic(Text)` appears in a Function signature;
2. `Intrinsic(Bytes)` appears in a Function signature;
3. an otherwise unused Domain Type declaration exists in a Module;
4. `ValueType::Domain(type_id)` appears in a Function signature;
5. a `DomainConstruct` exists; and
6. a `DomainProject` exists.

For each scenario, independently request V0_1, V0_2, V0_3, and V0_4 and verify
every request is rule-set inapplicable, emits no semantic Diagnostic, produces
no partial result, and produces no `VerifiedProgram`. The unused declaration
scenario MUST NOT rely on any Domain use to establish inapplicability.

## AR-DOMAIN-072 — Historical diagnostic payload compatibility

After `ValueType` is introduced, use Programs within the historical rule-set
applicability domains to trigger at least:

- a single-Block Return with expected `Int64` and actual `Int32` under each of
  V0_1, V0_2, V0_3, and V0_4, producing historical `MNIR-DIAG-004` with
  `expected_type: IntrinsicType::Int64` and
  `actual_type: IntrinsicType::Int32`;
- intrinsic non-Bool Branch condition under V0_3 and V0_4;
- intrinsic Call argument mismatch under V0_4; and
- intrinsic arithmetic or comparison operand mismatch under its applicable
  historical rule set.

Verify the historical codes, primary subjects, and payload field types remain
exactly as previously specified. In particular, every expected, actual,
left, right, or operand type field MUST be externally observable as
`IntrinsicType`, not `ValueType::Intrinsic`.

## AR-DOMAIN-073 — Complete V0_5 diagnostic selection

For arithmetic, comparison, single-Block Return, multi-Block Return, Branch,
and Call mismatches, exercise both the all-intrinsic path and every applicable
Domain-valued path. Verify the deterministic selection in
`MNIR-DOMAIN-105` through `109`, including that a construct receives exactly
one applicable type diagnostic per semantic category and no historical code's
payload schema is widened.

## AR-DOMAIN-074 — Nominal Domain Call mismatch

Create `CustomerId` and `OrderId` with equal `Int64` representations. Call a
Function whose Parameter is `Domain(CustomerId)` using an argument of
`Domain(OrderId)`. Verify both `ValueType` values are available, their
representations are equal, their nominal TypeIds differ, the Call is invalid,
and V0_5 emits `MNIR-DIAG-025` whose payload preserves both exact TypeIds.
Verify no `MNIR-DIAG-013` is emitted. In a separate Call, pass the underlying
`Intrinsic(Int64)` value and verify it remains invalid under the same nominal
rule.

## AR-DOMAIN-075 — Representation mutation preserves nominal Call matching

Create Domain Type `T` represented by `Int64`, a Function Parameter of
`Domain(T)`, and a Call argument produced by `DomainConstruct(T, int64_source)`.
Change `T`'s representation to `Text` while preserving `TypeId`. Verify the
Parameter and construct result remain exactly `Domain(T)` and therefore still
match nominally for Call argument checking. Separately verify that the now
invalid construct from the old Int64 source emits `MNIR-DIAG-015`. The Call
MUST NOT receive a mismatch or unavailable diagnostic merely because the
construction producing its nominally matching argument is semantically
invalid.

## AR-DOMAIN-076 — Transitive EffectSequence dependency

Construct a valid same-Block dependency path:

```text
E1 = Call(F)
E2 = DomainConstruct(T, E1)
E3 = DomainProject(E2)
E4 = Call(G, [E3])
```

Arrange the signatures and representation so construction and projection are
otherwise valid. Verify `E2` and `E3` are pure and absent from EffectSequence,
the shared Expression DAG preserves the transitive `E1 -> E2 -> E3 -> E4`
dependency, and existing Call sequencing semantics require EffectSequence
`[E1, E4]`. Verify reversing the two Call entries is structurally invalid.

## AR-DOMAIN-077 — Required unavailable propagation

Independently feed an unavailable inner Expression type into arithmetic,
comparison, a corresponding Call argument, `DomainConstruct`, and
`DomainProject`. Verify each dependent construct emits the unavailable
diagnostic required by its own normative rule, without discretionary
suppression. Separately return an Expression without a successfully derived
`ValueType` and verify Return mismatch remains suppressed because no Return
unavailable diagnostic exists.
