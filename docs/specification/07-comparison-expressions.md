# MNIR Specification — Comparison Expressions

**Document:** `07-comparison-expressions.md`
**Specification status:** Draft
**Specification version:** 0.1
**Normative:** Yes

---

## 1. Purpose

This document introduces comparison Expressions in MNIR.

Specification version 0.1 defines:

* equality comparison;
* inequality comparison;
* signed integer ordering comparisons;
* comparison operand references;
* comparison type derivation;
* comparison semantic validity;
* comparison inspection;
* comparison verification diagnostics;
* Semantic Verification rule set version 0.2.

The defined comparison Expression kinds are:

```text
Equal
NotEqual
LessThan
LessThanOrEqual
GreaterThan
GreaterThanOrEqual
```

The target capability is to represent Expressions conceptually equivalent to:

```text
a == b
a != b
a < b
a <= b
a > b
a >= b
```

and to derive:

```text
Bool
```

for valid comparisons.

This specification intentionally does not define:

* logical AND or OR;
* logical NOT;
* branching;
* `if`;
* multiple Blocks;
* control-flow graphs;
* pattern matching;
* numeric conversions;
* comparison operator overloading;
* user-defined comparison semantics;
* string comparison;
* domain-type comparison;
* runtime evaluation;
* constant folding;
* EasyH comparison syntax.

---

# 2. Normative language

Every declarative statement inside a numbered `MNIR-CMP-*` rule is normative unless explicitly stated otherwise.

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** retain their normative meanings when used inside numbered rules.

Text outside numbered `MNIR-CMP-*` rules is informative unless explicitly stated otherwise.

Acceptance requirements use identifiers of the form:

```text
AR-CMP-NNN
```

All applicable `AR-CMP-*` requirements are mandatory for completion of this implementation increment.

---

# 3. Dependencies

This specification builds on specifications `01` through `06`.

In particular it extends:

```text
Expression
ExpressionKind
ExpressionId
Expression type inspection
mnir-verify
Diagnostic
VerifiedProgram
VerificationRuleSet
```

Existing semantics remain unchanged unless explicitly extended by this specification.

---

# 4. Verification compatibility principle

Semantic Verification and Diagnostics 0.1 established:

```text
SemanticVerificationAndDiagnosticsV0_1
```

as a fixed verification rule set.

Comparison Expressions MUST NOT retroactively change the semantics of that rule set.

---

## MNIR-CMP-001 — Verification V0_1 remains stable

`SemanticVerificationAndDiagnosticsV0_1` MUST retain the semantic meaning defined by Semantic Verification and Diagnostics 0.1.

Comparison-specific semantic diagnostics MUST NOT silently become part of `V0_1`.

---

## MNIR-CMP-002 — Verification V0_2

This specification introduces:

```text
SemanticVerificationAndDiagnosticsV0_2
```

`V0_2` consists of:

```text
all V0_1 verification rules
+
all comparison verification rules defined by this specification
```

---

## MNIR-CMP-003 — Rule-set identity is semantic

A `VerifiedProgram` produced under `V0_1` and a `VerifiedProgram` produced under `V0_2` represent different verification claims even when they refer to the same:

```text
ProgramId
RevisionId
```

---

## MNIR-CMP-004 — No implicit verification upgrade

A Program successfully verified under `V0_1` MUST NOT be treated as verified under `V0_2` without a successful `V0_2` verification run.

---

## MNIR-CMP-101 — Verification rule-set applicability

A verification rule set is applicable only to Program revisions whose semantic constructs are understood by that rule set.

A rule set MUST NOT silently assign semantics to a construct introduced after that rule set was defined.

---

## MNIR-CMP-102 — V0_1 does not support Comparison Expressions

`SemanticVerificationAndDiagnosticsV0_1` does not understand Comparison Expressions.

If a Program revision contains one or more Comparison Expressions, `V0_1` MUST be considered not applicable to that Program revision.

`V0_1` MUST NOT attempt to interpret a Comparison Expression as:

* a valid `Bool` Expression;
* an unavailable Expression type;
* an opaque Expression;
* an arithmetic type failure;
* or another pre-existing semantic construct.

---

## MNIR-CMP-103 — V0_1 applicability failure is not a semantic Diagnostic

Failure because a Program revision contains semantic constructs unsupported by the requested verification rule set MUST NOT be represented as:

```text
MNIR-DIAG-001
MNIR-DIAG-002
MNIR-DIAG-003
MNIR-DIAG-004
MNIR-DIAG-005
MNIR-DIAG-006
MNIR-DIAG-007
```

The condition is a verification rule-set applicability failure, not a semantic error in the Program.

The exact Rust error representation is implementation-defined.

---

## MNIR-CMP-104 — No partial V0_1 verification

When `V0_1` is not applicable because the Program contains a Comparison Expression, the verifier MUST NOT produce partial semantic verification results for that Program revision.

It MUST NOT:

* emit V0_1 semantic diagnostics for only the constructs it recognizes;
* create a `VerifiedProgram`;
* or claim partial verification success.

---

## MNIR-CMP-105 — Applicability checks cover the complete Program

Rule-set applicability MUST be determined across the complete Program revision.

A Comparison Expression makes `V0_1` inapplicable even when that Expression:

* is not returned;
* is not referenced by another Expression;
* is in another Module;
* or would otherwise be considered dead or unreachable by a future analysis.

Version 0.1 defines no reachability exemption.

---

## MNIR-CMP-106 — V0_2 supports Comparison Expressions

`SemanticVerificationAndDiagnosticsV0_2` understands all semantic constructs supported by `V0_1` plus Comparison Expressions defined by this specification.

A structurally valid Program containing Comparison Expressions MAY therefore be semantically verified under `V0_2`.

---

# 5. Comparison Expression kinds

## MNIR-CMP-005 — Defined comparison kinds

Comparison Expressions 0.1 MUST define exactly:

```text
Equal
NotEqual
LessThan
LessThanOrEqual
GreaterThan
GreaterThanOrEqual
```

as comparison Expression kinds.

---

## MNIR-CMP-006 — Existing Expression identity

Comparison Expressions MUST use the existing `ExpressionId` identity model.

This specification MUST NOT introduce:

```text
ComparisonExpressionId
ComparisonId
OperatorId
```

or another comparison-specific semantic identity.

---

## MNIR-CMP-007 — Binary comparison

Every comparison Expression MUST contain exactly:

```text
left: ExpressionId
right: ExpressionId
```

---

# 6. Operand identity and order

## MNIR-CMP-008 — Operand position is semantic

Left and right operand position MUST be preserved in MNIR.

---

## MNIR-CMP-009 — No comparison canonicalization

Comparison Expressions 0.1 MUST NOT automatically:

* swap operands;
* sort operands;
* invert operators;
* canonicalize mathematically equivalent comparisons.

For example:

```text
a > b
```

MUST NOT automatically become:

```text
b < a
```

as part of MNIR semantic construction.

Likewise:

```text
a == b
```

MUST NOT be reordered merely because equality is symmetric.

---

## MNIR-CMP-010 — Same Expression may occupy both positions

The same `ExpressionId` MAY be used as both operands.

Examples:

```text
Equal(E1, E1)
GreaterThan(E2, E2)
```

are structurally valid provided all other structural requirements hold.

---

# 7. Operand ownership

## MNIR-CMP-011 — Existing operands

Both comparison operand identifiers MUST resolve to existing Expressions for a structurally valid comparison Expression.

---

## MNIR-CMP-012 — Same-Block operands

Both operands MUST belong to the same Block that owns the comparison Expression.

Cross-Block comparison references are structurally invalid.

---

## MNIR-CMP-013 — Semantic references

Operands MUST be referenced through `ExpressionId`.

Comparison semantics MUST NOT depend on:

* presentation names;
* textual source;
* Expression collection position;
* iteration order.

---

# 8. Dependency graph

Comparison operands extend the existing Expression dependency graph.

---

## MNIR-CMP-014 — Comparison dependencies are acyclic

A comparison Expression MUST NOT directly or indirectly depend on itself.

---

## MNIR-CMP-015 — Existing dependency semantics apply

Comparison operand references participate in the same acyclic Expression dependency graph as arithmetic references.

Arithmetic and comparison Expressions MAY depend on each other when all other semantic and structural rules permit it.

---

## MNIR-CMP-016 — Safe construction prevents cycles

The normal safe mutation API MUST require both comparison operands to exist before allocating the new comparison Expression.

Because version 0.1 provides no comparison operand-retargeting operation, normal safe construction MUST NOT permit dependency-cycle creation.

Structural validation MUST still reject internally corrupted cyclic candidates.

---

# 9. Comparison construction

## MNIR-CMP-017 — Required mutation operations

The controlled mutation API MUST provide operations equivalent to:

```text
add_equal_expression
add_not_equal_expression
add_less_than_expression
add_less_than_or_equal_expression
add_greater_than_expression
add_greater_than_or_equal_expression
```

Exact Rust names are implementation-defined.

---

## MNIR-CMP-018 — Construction behavior

Every comparison construction operation MUST:

* require an existing Block;
* require an existing left Expression;
* require an existing right Expression;
* require both operands to belong to that Block;
* allocate one provisional `ExpressionId`;
* create the requested comparison Expression;
* preserve operand position;
* expose the provisional `ExpressionId`.

---

## MNIR-CMP-019 — Semantic type validity does not gate construction

Comparison construction MUST NOT require operands to satisfy comparison type rules.

For example, structurally valid MNIR MAY represent:

```text
Int32 == Bool
Bool < Bool
Int32 < Int64
```

provided structural references are valid.

Such Expressions may later be semantically invalid.

---

## MNIR-CMP-020 — Structural construction failure

Comparison construction MUST fail and poison the transaction when:

* the Block is unknown;
* the left Expression is unknown;
* the right Expression is unknown;
* or an operand belongs to another Block.

Semantic operand-type invalidity MUST NOT poison construction.

---

# 10. Comparison inspection

## MNIR-CMP-021 — Comparison kind inspection

The public Expression inspection API MUST distinguish all six comparison Expression kinds.

---

## MNIR-CMP-022 — Operand inspection

The public API MUST expose:

```text
left ExpressionId
right ExpressionId
```

for each comparison Expression.

---

## MNIR-CMP-023 — Sufficient semantic data

Comparison inspection MUST expose enough information to reconstruct the version 0.1 semantic meaning of the Expression:

```text
comparison kind
left ExpressionId
right ExpressionId
```

---

# 11. Equality comparison types

The equality family consists of:

```text
Equal
NotEqual
```

---

## MNIR-CMP-024 — Equality requires equal operand types

A semantically valid equality comparison MUST have two operands whose intrinsic types are both successfully derivable and equal.

---

## MNIR-CMP-025 — Equality-supported intrinsic types

Equality comparison is supported for same-typed operands of:

```text
Int32
Int64
Bool
Unit
```

---

## MNIR-CMP-026 — Equality result type

A semantically valid:

```text
Equal
NotEqual
```

Expression MUST derive:

```text
Bool
```

regardless of the supported operand type.

---

## MNIR-CMP-027 — Mixed-type equality is invalid

Examples such as:

```text
Int32 == Int64
Int32 == Bool
Bool != Unit
```

are semantically type-invalid.

No implicit conversion is applied.

---

# 12. Ordering comparison types

The ordering family consists of:

```text
LessThan
LessThanOrEqual
GreaterThan
GreaterThanOrEqual
```

---

## MNIR-CMP-028 — Ordering requires equal operand types

A semantically valid ordering comparison MUST have two operands whose intrinsic types are successfully derivable and equal.

---

## MNIR-CMP-029 — Ordering-supported types

Ordering comparison is supported only for:

```text
Int32
Int64
```

in version 0.1.

---

## MNIR-CMP-030 — Ordering result type

A semantically valid ordering comparison MUST derive:

```text
Bool
```

---

## MNIR-CMP-031 — Bool ordering is unsupported

Expressions conceptually equivalent to:

```text
Bool < Bool
Bool >= Bool
```

are semantically invalid.

---

## MNIR-CMP-032 — Unit ordering is unsupported

Expressions conceptually equivalent to:

```text
Unit < Unit
Unit > Unit
```

are semantically invalid.

---

## MNIR-CMP-033 — Mixed-width ordering is invalid

Expressions such as:

```text
Int32 < Int64
Int64 >= Int32
```

are semantically invalid.

No implicit widening or narrowing occurs.

---

# 13. Comparison type-inspection outcomes

Comparison type inspection uses the semantic outcome categories already established for arithmetic type inspection:

```text
ValidType(IntrinsicType)
OperandTypeUnavailable
OperandTypeMismatch
UnsupportedOperandType
```

---

## MNIR-CMP-034 — Valid comparison type

A semantically valid comparison MUST produce:

```text
ValidType(Bool)
```

---

## MNIR-CMP-035 — Operand type unavailable

If one or both operand types cannot be derived, comparison type inspection MUST produce:

```text
OperandTypeUnavailable
```

---

## MNIR-CMP-036 — Operand type mismatch

If both operand types are derivable but differ, comparison type inspection MUST produce:

```text
OperandTypeMismatch
```

---

## MNIR-CMP-037 — Unsupported equality operand type

For the intrinsic types defined in MNIR 0.1, equal same-typed operands are all supported by:

```text
Equal
NotEqual
```

Therefore an equality comparison between equal current intrinsic types does not produce `UnsupportedOperandType`.

Future types may extend this rule through later specifications.

---

## MNIR-CMP-038 — Unsupported ordering operand type

An ordering comparison whose operands both derive:

```text
Bool
```

or both derive:

```text
Unit
```

MUST produce:

```text
UnsupportedOperandType
```

---

## MNIR-CMP-039 — Deterministic outcome precedence

Comparison type inspection MUST use:

```text
1. if either operand type cannot be derived:
       OperandTypeUnavailable

2. otherwise, if operand types differ:
       OperandTypeMismatch

3. otherwise, if the comparison family does not support that type:
       UnsupportedOperandType

4. otherwise:
       ValidType(Bool)
```

The result MUST NOT depend on operand-inspection order or collection iteration order.

---

## MNIR-CMP-040 — Nested type failure

If an operand Expression itself lacks a valid derived type because of an arithmetic or comparison semantic error, the containing comparison MUST treat that operand as unavailable.

---

## MNIR-CMP-041 — Inspection failure is read-only

Comparison type-inspection failure MUST NOT poison an Active transaction.

---

# 14. Structural versus semantic validity

## MNIR-CMP-042 — Type-invalid comparisons are representable

A structurally valid comparison Expression MAY be committed even when its operand types make it semantically invalid.

---

## MNIR-CMP-043 — Comparison type invalidity is not structural corruption

The following MUST NOT by themselves make Program structure invalid:

```text
operand type mismatch
unsupported ordering operand type
unavailable semantic operand type
```

provided structural references remain valid.

---

# 15. Equality abstract semantics

No evaluator is required by this specification, but the abstract comparison meanings are normative.

---

## MNIR-CMP-044 — Int32 equality

For `Int32` values:

```text
Equal(a, b)
```

denotes whether mathematical integer values `a` and `b` are equal.

`NotEqual(a, b)` denotes the logical negation of that equality.

---

## MNIR-CMP-045 — Int64 equality

`Int64` equality and inequality use mathematical signed-integer equality.

---

## MNIR-CMP-046 — Bool equality

For `Bool`:

```text
Equal(true, true)   = true
Equal(false, false) = true
Equal(true, false)  = false
Equal(false, true)  = false
```

`NotEqual` is the logical inverse.

---

## MNIR-CMP-047 — Unit equality

Because `Unit` has exactly one semantic value:

```text
Unit == Unit
```

is semantically true.

```text
Unit != Unit
```

is semantically false.

---

# 16. Ordering abstract semantics

## MNIR-CMP-048 — Signed integer ordering

Ordering operators compare `Int32` and `Int64` according to their mathematical signed-integer values.

---

## MNIR-CMP-049 — LessThan

```text
LessThan(a, b)
```

is true exactly when:

```text
a < b
```

in mathematical signed-integer ordering.

---

## MNIR-CMP-050 — LessThanOrEqual

```text
LessThanOrEqual(a, b)
```

is true exactly when:

```text
a <= b
```

---

## MNIR-CMP-051 — GreaterThan

```text
GreaterThan(a, b)
```

is true exactly when:

```text
a > b
```

---

## MNIR-CMP-052 — GreaterThanOrEqual

```text
GreaterThanOrEqual(a, b)
```

is true exactly when:

```text
a >= b
```

---

# 17. No comparison faults

## MNIR-CMP-053 — Comparison defines no arithmetic fault

The comparison operators introduced by this specification do not produce:

```text
Overflow
DivisionByZero
```

or another comparison-specific runtime fault for valid supported operand values.

---

# 18. Expression purity

## MNIR-CMP-054 — No comparison side effects

This specification defines no side effects for comparison Expressions.

---

## MNIR-CMP-055 — No operand evaluation order

Comparison Expressions 0.1 defines no left-first or right-first execution order.

Future effectful Expression specifications must define observable ordering if required.

---

# 19. Comparison immutability

## MNIR-CMP-056 — No operand retargeting

Version 0.1 defines no mutation operation that changes:

* comparison kind;
* left operand;
* right operand

after creation.

---

# 20. Snapshots and forks

## MNIR-CMP-057 — Snapshot preservation

Snapshots MUST preserve:

* comparison Expression identity;
* comparison kind;
* left operand;
* right operand.

---

## MNIR-CMP-058 — Fork preservation

Forks MUST preserve comparison semantic contents.

If Expression IDs are remapped, all comparison operand references MUST be remapped consistently and atomically.

---

# 21. Removal cascade

## MNIR-CMP-059 — Body removal

Removing a Function body MUST remove descendant comparison Expressions through the existing Expression cascade semantics.

---

## MNIR-CMP-060 — Function and Module removal

Function and Module cascade removal MUST transitively remove comparison Expressions.

Committed Expression identifiers remain retired according to existing rules.

---

# 22. Structural validity

## MNIR-CMP-061 — Structurally valid comparison

A comparison Expression is structurally valid when:

1. it has exactly one `ExpressionId`;
2. it belongs to exactly one existing Block;
3. it has exactly one defined comparison kind;
4. its left operand exists in the same Block;
5. its right operand exists in the same Block;
6. its dependency relationships are acyclic;
7. all existing Expression identity rules are satisfied.

Operand semantic type validity is not structural validity.

---

## MNIR-CMP-062 — Structural commit rejection

A Program revision violating `MNIR-CMP-061` MUST NOT commit successfully.

---

# 23. Extended Expression type inspection

## MNIR-CMP-063 — Core Expression inspection

The existing `mnir-core` Expression type-inspection API MUST support comparison Expressions according to this specification.

---

## MNIR-CMP-064 — Comparison result type

When comparison type inspection succeeds, it MUST return:

```text
Bool
```

---

# 24. Return integration

## MNIR-CMP-065 — Comparisons may be returned

A Return terminator MAY reference a comparison Expression in the same Block.

---

## MNIR-CMP-066 — Existing Return compatibility semantics

The existing Function Return semantic rule applies to comparison Expressions.

For example:

```text
Function return type: Bool
Return: Int32 comparison
```

is semantically valid when the comparison itself is valid.

A Function declared:

```text
-> Int32
```

returning a valid comparison has Return type mismatch because the comparison derives `Bool`.

---

# 25. Existing no-op semantics

## MNIR-CMP-067 — Comparison data participates in Program state

For no-op comparison purposes, comparison Expression state includes:

```text
comparison kind
left ExpressionId
right ExpressionId
```

---

# 26. Comparison verification diagnostics

Semantic Verification rule set `V0_2` introduces exactly three new diagnostic codes:

```text
MNIR-DIAG-005  ComparisonOperandTypeUnavailable
MNIR-DIAG-006  ComparisonOperandTypeMismatch
MNIR-DIAG-007  ComparisonUnsupportedOperandType
```

---

## MNIR-CMP-068 — Comparison diagnostic severity

All comparison diagnostics introduced by this specification have severity:

```text
Error
```

---

## MNIR-CMP-069 — Diagnostic primary subject

The primary semantic subject of:

```text
MNIR-DIAG-005
MNIR-DIAG-006
MNIR-DIAG-007
```

MUST be the affected comparison `ExpressionId`.

---

# 27. Comparison unavailable diagnostic

## MNIR-CMP-070 — ComparisonOperandTypeUnavailable

Under verification rule set `V0_2`, when comparison type inspection produces:

```text
OperandTypeUnavailable
```

the verifier MUST emit exactly one:

```text
MNIR-DIAG-005
```

for that comparison Expression.

---

## MNIR-CMP-071 — Unavailable payload

`MNIR-DIAG-005` MUST expose at minimum:

```text
expression_id: ExpressionId
```

---

# 28. Comparison mismatch diagnostic

## MNIR-CMP-072 — ComparisonOperandTypeMismatch

When comparison type inspection produces:

```text
OperandTypeMismatch
```

`V0_2` MUST emit exactly one:

```text
MNIR-DIAG-006
```

---

## MNIR-CMP-073 — Mismatch payload

`MNIR-DIAG-006` MUST expose at minimum:

```text
expression_id: ExpressionId
left_type: IntrinsicType
right_type: IntrinsicType
```

The values MUST correspond to the successfully derived left and right operand types.

---

# 29. Comparison unsupported diagnostic

## MNIR-CMP-074 — ComparisonUnsupportedOperandType

When comparison type inspection produces:

```text
UnsupportedOperandType
```

`V0_2` MUST emit exactly one:

```text
MNIR-DIAG-007
```

---

## MNIR-CMP-075 — Unsupported payload

`MNIR-DIAG-007` MUST expose at minimum:

```text
expression_id: ExpressionId
operand_type: IntrinsicType
```

The comparison kind MUST also be programmatically inspectable either directly from the Diagnostic or through lookup of the identified Expression.

It does not need to be duplicated in the normative diagnostic payload.

---

# 30. Comparison diagnostic exclusivity

## MNIR-CMP-076 — One comparison diagnostic per Expression

A comparison Expression MUST NOT receive more than one of:

```text
MNIR-DIAG-005
MNIR-DIAG-006
MNIR-DIAG-007
```

during one `V0_2` verification run.

---

## MNIR-CMP-077 — Cascading comparison diagnostics

If an inner invalid Expression causes an outer comparison Expression to have unavailable operand type, both Expressions independently receive diagnostics required by their own semantic outcomes.

No cascading-diagnostic suppression is defined.

---

# 31. Return verification under V0_2

## MNIR-CMP-078 — V0_2 Return verification understands comparisons

`V0_2` Return verification MUST derive comparison Expression types according to this specification.

---

## MNIR-CMP-079 — Invalid comparison Return suppresses Return mismatch

If a returned comparison Expression has no valid derived type because of a comparison semantic error, `V0_2` MUST NOT additionally emit:

```text
MNIR-DIAG-004 ReturnTypeMismatch
```

for that Function.

The comparison diagnostic represents the semantic failure.

---

# 32. V0_1 behavior

## MNIR-CMP-080 — V0_1 comparison behavior

`SemanticVerificationAndDiagnosticsV0_1` MUST NOT semantically verify a Program revision containing Comparison Expressions.

Such a verification request MUST fail because the requested rule set is not applicable.

No comparison-specific or pre-existing semantic diagnostic may be used as a substitute for this applicability failure.

---

## MNIR-CMP-081 — V0_1 remains available

The public verification API MUST preserve the ability to request `SemanticVerificationAndDiagnosticsV0_1`.

For Program revisions containing only constructs understood by V0_1, its existing semantics MUST remain unchanged.

---

## MNIR-CMP-082 — V0_2 is explicitly selectable

The public verification API MUST permit callers to explicitly request:

```text
SemanticVerificationAndDiagnosticsV0_2
```

The exact Rust API shape is implementation-defined.

---

## MNIR-CMP-083 — Existing verify convenience behavior

If the existing public API:

```text
verify(snapshot)
```

is retained, its existing `V0_1` meaning MUST NOT silently change.

When such an API is used with a Program containing Comparison Expressions, it MUST produce the V0_1 rule-set applicability failure defined by this specification.

A new API equivalent to:

```text
verify_with_rule_set(snapshot, rule_set)
```

MAY be introduced.

---

## MNIR-CMP-107 — Comparison as Return under V0_1

A Program containing a Comparison Expression referenced by Return is not applicable to V0_1 verification.

V0_1 MUST NOT emit `MNIR-DIAG-004` based on the Comparison Expression's type because V0_1 does not define that type.

---

## MNIR-CMP-108 — Comparison nested with Arithmetic under V0_1

A Program containing an Arithmetic Expression that directly or indirectly depends on a Comparison Expression is not applicable to V0_1 verification.

V0_1 MUST NOT reinterpret the Comparison operand as `OperandTypeUnavailable`.

---

## MNIR-CMP-109 — Arithmetic nested inside Comparison under V0_1

A Program containing a Comparison Expression whose operand is an Arithmetic Expression is still not applicable to V0_1 verification because the Comparison Expression itself is unsupported by V0_1.

The fact that the Arithmetic operand is understood by V0_1 does not make the complete Program applicable.

---

## MNIR-CMP-110 — V0_2 mixed Expression graph

V0_2 type inspection MUST support Expression dependency graphs containing both Arithmetic and Comparison Expressions.

For example:

```text
E1 : Int32
E2 : Int32
E3 = Add(E1, E2)          -> Int32
E4 = GreaterThan(E3, E1)  -> Bool
```

is semantically type-valid.

Likewise:

```text
E1 : Int32
E2 : Int32
E3 = GreaterThan(E1, E2)  -> Bool
E4 = Equal(E3, BoolExpr)   -> Bool
```

is semantically type-valid when `BoolExpr` derives `Bool`.

---

# 33. VerifiedProgram V0_2

## MNIR-CMP-084 — V0_2 rule-set binding

A `VerifiedProgram` produced by successful `V0_2` verification MUST report:

```text
SemanticVerificationAndDiagnosticsV0_2
```

as its verification rule set.

---

## MNIR-CMP-085 — V0_2 success

A Program revision produces a `V0_2` `VerifiedProgram` only when:

* every V0_1 verification rule succeeds;
* every comparison verification rule defined here succeeds.

---

## MNIR-CMP-086 — V0_1 does not imply V0_2

Possession of a `V0_1` `VerifiedProgram` MUST NOT satisfy an API requirement for a `V0_2` verification artifact.

---

## MNIR-CMP-111 — VerifiedProgram rule-set inspection is authoritative

The `VerificationRuleSet` exposed by `VerifiedProgram` is the authoritative public identity of the verification claim.

A consumer requiring V0_2 verification MUST explicitly determine that:

```text
verified_program.rule_set()
```

represents:

```text
SemanticVerificationAndDiagnosticsV0_2
```

A V0_1 artifact does not satisfy that requirement even when the underlying Program revision would also verify successfully under V0_2.

---

# 34. Verification traversal

## MNIR-CMP-087 — All comparison Expressions are verified

`V0_2` MUST verify every committed comparison Expression in every Function body.

This applies even when the comparison Expression is not returned.

No reachability exemption exists.

---

## MNIR-CMP-088 — Verification order is non-semantic

Comparison verification results MUST NOT depend on Module, Function, or Expression collection iteration order.

---

# 35. No execution

## MNIR-CMP-089 — Verifier does not evaluate comparisons

Comparison verification MUST NOT execute comparisons to determine semantic type validity.

---

## MNIR-CMP-090 — No constant comparison folding

Version 0.1 MUST NOT require or establish constant-folding semantics such as simplifying:

```text
1 < 2
```

to:

```text
true
```

---

# 36. No future comparison abstractions

## MNIR-CMP-091 — No generic comparison trait system

Version 0.1 MUST NOT introduce:

```text
Comparable
Ordered
Equality
ComparisonTrait
OperatorOverload
```

or similar public semantic abstractions solely in anticipation of future type systems.

---

## MNIR-CMP-092 — No implicit conversion

Comparison Expressions 0.1 MUST NOT introduce implicit numeric conversion.

---

# 37. Specification gaps

## MNIR-CMP-093 — Undefined comparison semantics

If implementation requires externally observable comparison or comparison-verification behavior not defined by this specification, it MUST NOT establish that behavior as normative MNIR semantics.

The missing behavior MUST be reported as a specification gap.

---

# 38. Explicitly unresolved topics

The following remain unresolved:

* logical AND;
* logical OR;
* logical NOT;
* branches;
* conditional control flow;
* multiple Blocks;
* control-flow graph semantics;
* user-defined comparison;
* domain-type comparison;
* Text comparison;
* Bytes comparison;
* floating-point comparison;
* numeric conversion;
* comparison overload resolution;
* comparison evaluation;
* constant folding;
* comparison canonicalization;
* semantic simplification;
* reachability;
* diagnostic suppression;
* source locations;
* EasyH syntax;
* serialization syntax.

---

## MNIR-CMP-094 — Unresolved topics are not 0.1 semantics

An implementation MUST NOT establish unresolved topics as normative Comparison Expressions 0.1 semantics.

---

# 39. Acceptance requirements

This implementation increment covers:

```text
AR-CMP-001 .. AR-CMP-038
```

## AR-CMP-001 — Int32 equality

Create:

```text
Int32 == Int32
```

Verify construction succeeds and derived type is `Bool`.

---

## AR-CMP-002 — Int64 inequality

Create `NotEqual` using two `Int64` operands and verify derived `Bool`.

---

## AR-CMP-003 — Bool equality

Verify:

```text
Bool == Bool
```

is semantically type-valid and derives `Bool`.

---

## AR-CMP-004 — Unit equality

Verify:

```text
Unit == Unit
```

is semantically type-valid and derives `Bool`.

---

## AR-CMP-005 — Int32 ordering

Demonstrate all four ordering operators using `Int32`.

Verify each derives `Bool`.

---

## AR-CMP-006 — Int64 ordering

Demonstrate ordering with `Int64`.

---

## AR-CMP-007 — Mixed equality mismatch

Create both:

```text
Int32 == Int64
Int64 == Int32
```

Verify structural commit succeeds and type inspection produces `OperandTypeMismatch`.

---

## AR-CMP-008 — Mixed ordering mismatch

Demonstrate equivalent behavior for ordering comparisons.

---

## AR-CMP-009 — Bool ordering unsupported

Create:

```text
Bool < Bool
```

Verify structural commit succeeds and type inspection produces `UnsupportedOperandType`.

---

## AR-CMP-010 — Unit ordering unsupported

Demonstrate equivalent behavior for `Unit`.

---

## AR-CMP-011 — Cross-Block operand rejection

Demonstrate both left and right cross-Block operand failure.

Verify transaction poisoning and atomicity.

---

## AR-CMP-012 — Unknown operand rejection

Demonstrate both left and right unknown `ExpressionId` failures.

---

## AR-CMP-013 — Unknown Block rejection

Verify comparison construction against an unknown Block fails and poisons the transaction.

---

## AR-CMP-014 — Operand order preserved

Construct:

```text
GreaterThan(E1, E2)
GreaterThan(E2, E1)
```

and verify stored operands remain distinct and ordered.

---

## AR-CMP-015 — Identical operand allowed

Construct:

```text
Equal(E1, E1)
```

and verify it is structurally valid and does not create a cycle.

---

## AR-CMP-016 — Nested comparison

Create a valid comparison and use it as an equality operand with another Bool Expression.

Verify recursive type derivation.

---

## AR-CMP-017 — Nested unavailable propagation

Use a semantically invalid inner Expression as a comparison operand.

Verify the outer comparison produces `OperandTypeUnavailable`.

---

## AR-CMP-018 — Comparison Expression identity

Verify comparison Expressions use ordinary unique `ExpressionId` identities and committed identifiers remain retired after removal.

---

## AR-CMP-019 — Snapshot preservation

Verify snapshot preservation of comparison kind and operands.

---

## AR-CMP-020 — Fork preservation

Verify fork preservation and reference consistency.

---

## AR-CMP-021 — Comparison may be returned

Construct a Bool-returning Function whose Return Expression is a valid comparison.

Verify structural commit and type derivation.

---

## AR-CMP-022 — Comparison Return mismatch

Return a valid comparison from a Function declared `Int32`.

Under `V0_2`, verify `MNIR-DIAG-004`.

---

## AR-CMP-023 — V0_2 valid comparison verification

Verify a Program containing valid comparisons under `V0_2`.

Verify zero comparison diagnostics and successful `VerifiedProgram`.

---

## AR-CMP-024 — Comparison mismatch diagnostic

Under `V0_2`, verify a mixed-type comparison produces exactly one:

```text
MNIR-DIAG-006
```

with normative payload.

---

## AR-CMP-025 — Comparison unsupported diagnostic

Verify:

```text
Bool < Bool
```

produces exactly one:

```text
MNIR-DIAG-007
```

---

## AR-CMP-026 — Comparison unavailable diagnostic

Create nested semantic failure causing a comparison operand type to be unavailable.

Verify:

```text
MNIR-DIAG-005
```

---

## AR-CMP-027 — Invalid comparison Return suppression

Return an invalid comparison.

Verify its comparison diagnostic is emitted and `MNIR-DIAG-004` is not additionally emitted.

---

## AR-CMP-028 — Dead comparison is still verified

Create an invalid comparison not referenced by Return.

Verify `V0_2` still reports it.

---

## AR-CMP-029 — V0_1 remains unchanged and rejects unsupported comparison semantics

Verify that the existing V0_1 acceptance suite continues to pass unchanged for Program revisions containing only constructs understood by V0_1.

Additionally verify V0_1 rule-set applicability failure for each of the following Program shapes:

### Case A — Valid Comparison returned from Bool Function

```text
Function return type: Bool
Return: valid Comparison
```

V0_1 MUST fail as not applicable.

It MUST NOT produce `MNIR-DIAG-004`.

### Case B — Valid Comparison returned from Int32 Function

```text
Function return type: Int32
Return: valid Comparison
```

V0_1 MUST fail as not applicable.

It MUST NOT produce `MNIR-DIAG-004`.

### Case C — Invalid unreturned Comparison

A Program containing a semantically invalid Comparison that is not returned MUST still make V0_1 inapplicable.

V0_1 MUST NOT emit comparison diagnostics.

### Case D — Comparison used as Arithmetic operand

A Program containing Arithmetic that depends on a Comparison MUST make V0_1 inapplicable.

V0_1 MUST NOT reinterpret the Comparison as `OperandTypeUnavailable`.

### Case E — Arithmetic used as Comparison operand

A Program containing a Comparison that depends on Arithmetic MUST make V0_1 inapplicable because the Comparison construct itself is unsupported.

No partial V0_1 verification result may be produced.

---

## AR-CMP-030 — Explicit V0_2 selection

Demonstrate the public API can explicitly request `V0_2`.

---

## AR-CMP-031 — Rule-set binding

Verify a `V0_2` `VerifiedProgram` reports the V0_2 rule-set identity.

Verify a `V0_1` artifact does not satisfy V0_2 verification identity.

---

## AR-CMP-032 — Comparison diagnostics machine-readable

Demonstrate machine-readable access to:

```text
MNIR-DIAG-005
MNIR-DIAG-006
MNIR-DIAG-007
```

including normative payloads.

---

## AR-CMP-033 — No comparison evaluator

Demonstrate by conformance inspection that no comparison evaluator or constant folder is introduced.

---

## AR-CMP-034 — No speculative comparison abstractions

Verify no generic comparable/ordered/operator-overload abstraction is introduced.

---

## AR-CMP-035 — Existing conformance preserved

All acceptance suites from specifications `01` through `06` MUST continue to pass without semantic regression.

---

## AR-CMP-036 — Rule-set applicability failure is distinct from semantic failure

Request V0_1 verification of a structurally valid Program containing a Comparison Expression.

Verify that:

* verification fails;
* no `VerifiedProgram` is produced;
* the failure is distinguishable from semantic verification diagnostics;
* no semantic Diagnostic is required to represent the unsupported-rule-set condition.

Exact Rust error naming is implementation-defined.

---

## AR-CMP-037 — V0_2 supports mixed Arithmetic and Comparison dependency graphs

Under V0_2, construct and verify at least:

```text
Arithmetic → Comparison
```

and:

```text
Comparison → Equality Comparison
```

dependency chains.

Verify recursive type derivation and semantic verification operate correctly across both Expression families.

---

## AR-CMP-038 — Cross-family Expression cycles are structurally rejected

Using an internal test-only corruption mechanism if necessary, construct a dependency cycle containing at least:

* one Arithmetic Expression;
* and one Comparison Expression.

Verify:

* structural commit fails;
* the invalid candidate does not become committed state;
* the source committed revision remains unchanged.

The public safe mutation API MUST NOT be weakened to permit cycle construction.

---

# 40. Implementation constraints

## MNIR-CMP-095 — Core representation ownership

Comparison Expression representation MUST belong to:

```text
mnir-core
```

---

## MNIR-CMP-096 — Verifier extension ownership

Comparison verification and diagnostics MUST belong to:

```text
mnir-verify
```

---

## MNIR-CMP-097 — Existing Expression model extension

Comparison Expressions MUST extend the existing Expression model rather than introduce a parallel comparison AST.

---

## MNIR-CMP-098 — Dependency direction

Implementation MUST NOT cause:

```text
mnir-core → mnir-verify
```

dependency.

---

## MNIR-CMP-099 — Safe Rust

Implementation MUST NOT use or require `unsafe` Rust.

---

## MNIR-CMP-100 — Controlled mutation

The public API MUST NOT expose unrestricted mutable access capable of bypassing comparison structural invariants.

---

# 41. Expected conceptual representation

```text
Expression
├── existing Expression kinds
├── Equal {
│   left: ExpressionId
│   right: ExpressionId
│ }
├── NotEqual {
│   left: ExpressionId
│   right: ExpressionId
│ }
├── LessThan {
│   left: ExpressionId
│   right: ExpressionId
│ }
├── LessThanOrEqual {
│   left: ExpressionId
│   right: ExpressionId
│ }
├── GreaterThan {
│   left: ExpressionId
│   right: ExpressionId
│ }
└── GreaterThanOrEqual {
    left: ExpressionId
    right: ExpressionId
  }
```

No concrete Rust representation is mandated.

---

# 42. Target example

After implementation MNIR must be able to represent conceptually:

```text
is_greater(a: Int32, b: Int32) -> Bool {
    return a > b
}
```

as:

```text
FunctionId(F1)
├── return_type: Bool
├── ParameterId(P1) : Int32
├── ParameterId(P2) : Int32
└── Body
    └── BlockId(B1)
        ├── E1 = ParameterReference(P1)
        ├── E2 = ParameterReference(P2)
        ├── E3 = GreaterThan(E1, E2)
        └── Return(E3)
```

The representation is informative.

It is neither EasyH nor serialized MNIR syntax.

---

# 43. Foundational invariants

```text
Comparisons are ordinary Expressions.

Comparison operands use ExpressionId.

Operand position is semantic.

Comparisons participate in the shared acyclic Expression dependency graph.

Equality supports Int32, Int64, Bool, and Unit.

Ordering supports Int32 and Int64.

Valid comparisons derive Bool.

No implicit conversion exists.

Type-invalid comparisons remain structurally representable.

Comparison type-inspection errors do not poison transactions.

No comparison evaluator exists.

No branching exists yet.

Verification V0_1 remains unchanged.

Verification V0_2 extends V0_1 with comparison verification.

Comparison diagnostics are MNIR-DIAG-005 through MNIR-DIAG-007.

VerifiedProgram remains explicitly bound to its verification rule set.
```
