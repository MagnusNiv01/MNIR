# MNIR Specification — Expressions and Basic Function Bodies

**Document:** `04-expressions-and-basic-function-bodies.md`
**Specification status:** Draft
**Specification version:** 0.1
**Normative:** Yes

---

## 1. Purpose

This document introduces the first executable-body structure in MNIR.

Specification version 0.1 defines:

* optional Function bodies;
* one basic Block per Function body;
* `BlockId`;
* Expressions;
* `ExpressionId`;
* intrinsic literal Expressions;
* Parameter reference Expressions;
* a Return terminator;
* expression ownership and reference integrity;
* controlled construction and removal of Function bodies.

The target capability is to represent a Function conceptually equivalent to:

```text
identity(value: Int32) -> Int32 {
    return value
}
```

This document intentionally does not define:

* arithmetic;
* `Add`;
* comparisons;
* local variables;
* assignments;
* multiple Blocks;
* branches;
* loops;
* Function calls;
* side effects;
* expression evaluation order;
* overflow behavior;
* semantic type verification;
* diagnostics;
* EasyH syntax;
* execution backends.

Those concepts are introduced by later specification increments.

---

# 2. Normative language

Every declarative statement inside a numbered `MNIR-EXPR-*` rule is normative unless explicitly stated otherwise.

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** retain their normative meanings when used inside numbered rules.

Text outside numbered `MNIR-EXPR-*` rules is informative unless explicitly stated otherwise.

Acceptance requirements use identifiers of the form:

```text
AR-EXPR-NNN
```

All applicable `AR-EXPR-*` requirements are mandatory project-level acceptance requirements for the initial implementation increment defined by this document.

Acceptance requirements demonstrate normative behavior but do not independently introduce MNIR semantics.

---

# 3. Dependencies

This specification builds on:

* Program Model 0.1;
* Type System Foundations 0.1;
* Functions and Parameters 0.1.

Existing semantics remain unchanged.

In particular:

* `Program` remains the aggregate root;
* Functions remain owned by Modules;
* Parameters remain owned by Functions;
* mutation remains Program-transaction based;
* intrinsic types remain Program-independent;
* committed semantic identifiers are never reused within a Program lineage.

---

# 4. Terminology

## 4.1 Function body

A **Function body** is the executable-body structure optionally owned by a Function.

In version 0.1, a Function body contains exactly one Block.

A Function may exist without a body.

The absence of a body does not, by itself, define external linkage, abstract Functions, imports, native Functions, or any other execution semantics.

Those concepts remain unresolved.

---

## 4.2 Block

A **Block** is an independently addressable semantic entity owned by exactly one Function body.

Version 0.1 supports exactly one Block per Function body.

The Block contains:

* zero or more Expressions;
* exactly one Return terminator before the body can be committed as structurally valid.

---

## 4.3 Expression

An **Expression** is an independently addressable semantic entity that represents a value-producing program construct.

Version 0.1 defines only:

```text
Int32Literal
Int64Literal
BoolLiteral
UnitLiteral
ParameterReference
```

Arithmetic and other composed Expressions are not part of this specification.

---

## 4.4 Terminator

A **terminator** defines how execution leaves a Block.

Version 0.1 defines exactly one terminator kind:

```text
Return
```

A Return terminator refers to one Expression in the same Block.

A terminator is not an Expression.

---

# 5. Function body presence

## MNIR-EXPR-001 — Optional Function body

A Function MAY own zero or one Function body.

A Function MUST NOT own more than one Function body in version 0.1.

---

## MNIR-EXPR-002 — Existing bodyless Functions remain valid

A Function without a body remains structurally valid under this specification if it satisfies all applicable Functions and Parameters 0.1 rules.

This specification does not assign execution semantics to a bodyless Function.

---

## MNIR-EXPR-003 — Function body ownership

A Function body MUST belong to exactly one Function.

A Function body MUST NOT be shared between Functions.

---

# 6. Block identity

## MNIR-EXPR-004 — Block identity

Every Block MUST have exactly one `BlockId`.

`BlockId` is a distinct semantic identifier category.

---

## MNIR-EXPR-005 — BlockId uniqueness domain

A committed `BlockId` MUST be unique among all Block identities allocated within one Program lineage.

The uniqueness domain is the Program lineage rather than the Function.

---

## MNIR-EXPR-006 — BlockId opacity

The raw representation of a `BlockId` MUST NOT carry normative meaning beyond identity.

Consumers MUST NOT infer:

* Function ownership;
* creation order;
* execution order;
* revision;
* or control-flow position

from the raw identifier representation.

---

## MNIR-EXPR-007 — BlockId retirement

Once a `BlockId` has become part of a committed Program revision, that identifier MUST never be allocated to another Block identity in the same Program lineage.

Removing a Function body MUST NOT release its committed `BlockId`.

---

# 7. Single-Block body model

## MNIR-EXPR-008 — Exactly one Block when body exists

A Function body MUST contain exactly one Block in version 0.1.

---

## MNIR-EXPR-009 — No Block ordering semantics

Because a version 0.1 Function body contains exactly one Block, no sibling Block ordering semantics exist.

---

## MNIR-EXPR-010 — No control-flow graph

Version 0.1 MUST NOT establish multiple Blocks, branches, jumps, predecessors, successors, or control-flow graph semantics.

---

# 8. Expression identity

## MNIR-EXPR-011 — Expression identity

Every Expression MUST have exactly one `ExpressionId`.

`ExpressionId` is a distinct semantic identifier category.

---

## MNIR-EXPR-012 — ExpressionId uniqueness domain

A committed `ExpressionId` MUST be unique among all Expression identities allocated within one Program lineage.

The uniqueness domain is the Program lineage.

---

## MNIR-EXPR-013 — ExpressionId opacity

The raw representation of an `ExpressionId` MUST NOT carry normative semantic meaning beyond identity.

Consumers MUST NOT infer:

* Expression kind;
* owning Block;
* intrinsic type;
* creation order;
* evaluation order;
* or revision

from its raw representation.

---

## MNIR-EXPR-014 — Expression identity retirement

Once an `ExpressionId` has become part of a committed Program revision, it MUST never be allocated to another Expression identity in the same Program lineage.

Removing a Function body MUST NOT release its committed Expression identifiers.

---

# 9. Expression ownership

## MNIR-EXPR-015 — Block ownership of Expressions

Every committed Expression MUST be owned by exactly one Block.

---

## MNIR-EXPR-016 — No shared Expressions

An Expression MUST NOT simultaneously belong to multiple Blocks.

---

## MNIR-EXPR-017 — Expression collection semantics

A Block's Expression collection MUST be identity-based.

Collection position MUST NOT determine Expression identity.

---

## MNIR-EXPR-018 — Expression collection order is non-semantic

Version 0.1 defines no semantic ordering between sibling Expressions stored in a Block.

An implementation MAY use any collection representation consistent with this specification.

Iteration order MUST NOT be interpreted as execution order.

---

# 10. Int32 literal Expression

## MNIR-EXPR-019 — Int32 literal

An `Int32Literal` Expression MUST contain one mathematical integer value belonging to the abstract `Int32` value domain defined by Type System Foundations 0.1.

---

## MNIR-EXPR-020 — Int32 literal type

The semantic type of an `Int32Literal` Expression MUST be `Int32`.

---

## MNIR-EXPR-021 — Int32 literal range

The public mutation API MUST NOT permit construction of an `Int32Literal` whose value lies outside the `Int32` abstract value domain.

The concrete Rust input type used to represent the literal value is implementation-defined.

---

# 11. Int64 literal Expression

## MNIR-EXPR-022 — Int64 literal

An `Int64Literal` Expression MUST contain one mathematical integer value belonging to the abstract `Int64` value domain.

---

## MNIR-EXPR-023 — Int64 literal type

The semantic type of an `Int64Literal` Expression MUST be `Int64`.

---

## MNIR-EXPR-024 — Int64 literal range

The public mutation API MUST NOT permit construction of an `Int64Literal` whose value lies outside the `Int64` abstract value domain.

The concrete Rust input type is implementation-defined.

---

# 12. Bool literal Expression

## MNIR-EXPR-025 — Bool literal

A `BoolLiteral` Expression MUST contain exactly one logical `Bool` value:

```text
true
false
```

---

## MNIR-EXPR-026 — Bool literal type

The semantic type of a `BoolLiteral` Expression MUST be `Bool`.

---

# 13. Unit literal Expression

## MNIR-EXPR-027 — Unit literal

A `UnitLiteral` Expression represents the single semantic value belonging to the `Unit` intrinsic type.

---

## MNIR-EXPR-028 — Unit literal type

The semantic type of a `UnitLiteral` Expression MUST be `Unit`.

---

# 14. Parameter reference Expression

## MNIR-EXPR-029 — Parameter reference

A `ParameterReference` Expression MUST refer to exactly one `ParameterId`.

---

## MNIR-EXPR-030 — Parameter ownership constraint

The referenced Parameter MUST belong to the same Function that owns the Function body containing the Parameter reference.

A Parameter reference to a Parameter owned by another Function is structurally invalid.

---

## MNIR-EXPR-031 — Existing Parameter requirement

The referenced Parameter MUST exist in the Function's current body-containing Program state.

A dangling Parameter reference is structurally invalid.

---

## MNIR-EXPR-032 — Parameter reference type

The semantic type of a `ParameterReference` Expression MUST be the intrinsic type of the referenced Parameter.

Changing the Parameter's type therefore changes the semantic type of the Parameter reference without changing the `ExpressionId`.

---

# 15. Expression semantic type

## MNIR-EXPR-033 — Defined Expression type

Every Expression kind introduced by version 0.1 MUST have exactly one semantic intrinsic type.

The type is derived from Expression semantics rather than assigned an independent Program-local `TypeId`.

---

## MNIR-EXPR-034 — Type derivation

The semantic Expression type MUST be derived as follows:

```text
Int32Literal        -> Int32
Int64Literal        -> Int64
BoolLiteral         -> Bool
UnitLiteral         -> Unit
ParameterReference  -> referenced Parameter type
```

---

## MNIR-EXPR-035 — No stored type identity requirement

An implementation is not required to redundantly store the derived intrinsic type inside every Expression.

The public behavior MUST remain consistent with the type derivation rules.

---

# 16. Return terminator

## MNIR-EXPR-036 — Exactly one Return terminator

A structurally valid version 0.1 Function body MUST contain exactly one Return terminator in its Block.

---

## MNIR-EXPR-037 — Return Expression reference

The Return terminator MUST refer to exactly one `ExpressionId`.

---

## MNIR-EXPR-038 — Return ownership constraint

The Expression referenced by Return MUST belong to the same Block as the Return terminator.

A dangling or cross-Block Return Expression reference is structurally invalid.

---

## MNIR-EXPR-039 — Return is not an Expression

The Return terminator MUST NOT have `ExpressionId` identity solely by virtue of being a Return terminator.

Version 0.1 does not introduce a general StatementId or TerminatorId.

---

## MNIR-EXPR-101 — Return representation

The Return terminator MAY be represented directly as an optional `ExpressionId` associated with the Block or by another equivalent internal representation.

The implementation MUST NOT introduce a general Statement or Terminator identity abstraction solely to represent Return in version 0.1.

---

# 17. Return type semantics

## MNIR-EXPR-040 — Return type requirement

A semantically valid Function body requires the intrinsic type of the Return Expression to equal the Function's declared return type.

For example:

```text
Function return type: Int32
Return Expression: ParameterReference to Int32 Parameter
```

is semantically valid with respect to this rule.

---

## MNIR-EXPR-041 — Return type mismatch is semantic invalidity

A structurally well-formed body MAY contain a Return Expression whose intrinsic type differs from the Function return type.

Such a Program is semantically invalid with respect to `MNIR-EXPR-040`.

Structural construction and storage of that mismatch MUST NOT, by itself, require rejection under this specification.

A future semantic verification specification will define detection and diagnostics for this condition.

---

# 18. Structural versus semantic validity

## MNIR-EXPR-042 — Structural validity does not imply Return type validity

Structural validation MUST verify reference integrity and ownership constraints defined by this specification.

Structural validation MUST NOT be treated as proof that `MNIR-EXPR-040` is satisfied.

---

## MNIR-EXPR-043 — Semantic verifier deferred

Expressions and Basic Function Bodies 0.1 MUST NOT introduce `VerifiedProgram` or a semantic verifier solely to enforce Return type compatibility.

Semantic verification is reserved for a later specification.

---

# 19. Function body controlled mutation

All body and Expression mutation uses the existing Program-level mutation transaction.

---

## MNIR-EXPR-044 — Create Function body

The version 0.1 mutation API MUST provide an operation equivalent to:

```text
create_function_body
```

The operation MUST:

* require an existing Function;
* fail if that Function already owns a body;
* allocate exactly one provisional `BlockId`;
* create an initially unterminated Block;
* associate that Block with the Function body;
* and return or otherwise directly expose the provisional `BlockId` to the caller.

The exact Rust API name and return type are implementation-defined.

---

## MNIR-EXPR-045 — Remove Function body

The version 0.1 mutation API MUST provide an operation equivalent to:

```text
remove_function_body
```

Removing a Function body MUST remove from the working state:

* its Block;
* all Expressions owned by that Block;
* and its Return terminator.

---

## MNIR-EXPR-046 — Unknown Function body target

Creating or removing a Function body against an unknown `FunctionId` MUST fail and poison the transaction.

Removing a Function body from a Function that has no body MUST fail and poison the transaction.

---

# 20. Literal creation operations

## MNIR-EXPR-047 — Literal mutation operations

The version 0.1 mutation API MUST provide operations equivalent to:

```text
add_int32_literal
add_int64_literal
add_bool_literal
add_unit_literal
```

Each operation MUST:

* require an existing Block in the transaction working state;
* allocate a provisional `ExpressionId`;
* create the corresponding Expression kind;
* return or otherwise expose the provisional `ExpressionId` for use within that transaction.

The exact Rust API names are implementation-defined.

---

# 21. Parameter reference creation

## MNIR-EXPR-048 — Parameter reference mutation operation

The version 0.1 mutation API MUST provide an operation equivalent to:

```text
add_parameter_reference
```

The operation MUST:

* require an existing Block;
* require an existing Parameter;
* require that the Parameter belongs to the Function owning the Block;
* allocate exactly one provisional `ExpressionId`;
* create one `ParameterReference` Expression owned by that Block;
* store the referenced `ParameterId`;
* and return or otherwise directly expose the provisional `ExpressionId`.

Violation of the Block, Parameter, or ownership requirements MUST fail and poison the transaction.

The exact Rust API name and return type are implementation-defined.

---

# 22. Return mutation

## MNIR-EXPR-049 — Set Return terminator

The version 0.1 mutation API MUST provide an operation equivalent to:

```text
set_return
```

It MUST require an existing Block and an existing Expression owned by that Block.

The operation MUST establish that Expression as the Block's Return Expression.

---

## MNIR-EXPR-050 — Return replacement

Calling `set_return` on a Block that already has a Return terminator MUST replace the referenced Return Expression while retaining a single Return terminator.

Because Return has no independent identity in version 0.1, replacement does not retire or allocate a terminator identifier.

---

# 23. Unterminated working state

## MNIR-EXPR-051 — Unterminated body during transaction

An Active mutation transaction MAY temporarily contain a Function body whose Block has no Return terminator.

This permits construction sequences such as:

```text
create body
add ParameterReference
set Return
commit
```

---

## MNIR-EXPR-052 — Unterminated body cannot commit

A Program revision containing a Function body without a Return terminator MUST NOT commit successfully.

Such a candidate is structurally invalid.

---

# 24. Function removal cascade

## MNIR-EXPR-053 — Function removal removes body

Removing a Function that owns a Function body MUST transitively remove:

* the Function body;
* its Block;
* all Expressions;
* and its Return terminator

from the transaction working state.

---

## MNIR-EXPR-054 — Function removal retires body identities

Committed `BlockId` and `ExpressionId` values belonging to a removed Function MUST remain permanently retired in the Program lineage.

---

# 25. Module removal cascade

## MNIR-EXPR-055 — Module removal removes descendant bodies

Removing a Module MUST transitively remove all Function bodies, Blocks, Expressions, and Return terminators owned through Functions in that Module.

---

## MNIR-EXPR-056 — Module removal preserves identity retirement

Committed `BlockId` and `ExpressionId` values removed through Module cascade MUST remain permanently retired.

---

# 26. Parameter removal and references

## MNIR-EXPR-057 — Referenced Parameter removal

A transaction MUST NOT commit a Function body containing a `ParameterReference` to a Parameter that no longer exists in that Function.

---

## MNIR-EXPR-058 — Temporary dangling Parameter reference

An Active transaction MAY temporarily remove a Parameter that is still referenced by a `ParameterReference` Expression.

Such a working state is structurally invalid but does not by itself poison the transaction.

Version 0.1 defines only the following ways to restore structural validity after this condition:

* remove the entire affected Function body;
* remove the owning Function;
* or remove the owning Module.

Version 0.1 does not define:

* removing an individual Expression;
* changing the referenced `ParameterId`;
* changing the Expression kind;
* or replacing one Expression with another.

Commit MUST fail unless structural validity has been restored before commit.

---

# 27. Provisional identity semantics

## MNIR-EXPR-059 — Provisional BlockId

A `BlockId` allocated in an Active transaction is provisional until successful commit.

It MAY be addressed within that transaction.

---

## MNIR-EXPR-060 — Provisional ExpressionId

An `ExpressionId` allocated in an Active transaction is provisional until successful commit.

It MAY be addressed within that transaction.

---

## MNIR-EXPR-097 — Provisional BlockId uniqueness

A provisional `BlockId` allocated in an Active transaction MUST NOT collide with:

* any committed `BlockId` in the Program lineage;
* or another provisional `BlockId` allocated in the same transaction.

---

## MNIR-EXPR-098 — Provisional ExpressionId uniqueness

A provisional `ExpressionId` allocated in an Active transaction MUST NOT collide with:

* any committed `ExpressionId` in the Program lineage;
* or another provisional `ExpressionId` allocated in the same transaction.

---

## MNIR-EXPR-061 — Successful identity history commit

On successful transaction commit, every Block and Expression identifier allocated during that transaction MUST become part of committed allocation history.

This applies even when the corresponding entity has been removed before commit.

---

## MNIR-EXPR-062 — Failed provisional identity

If a transaction fails or is discarded without commit, provisional Block and Expression identifiers do not become committed semantic identities.

Their raw values MAY later be reused according to the implementation-defined allocator strategy.

---

# 28. Allocation history

## MNIR-EXPR-063 — Block allocation history

A mutable Program lineage MUST maintain sufficient committed history to prevent reuse of committed `BlockId` values.

---

## MNIR-EXPR-064 — Expression allocation history

A mutable Program lineage MUST maintain sufficient committed history to prevent reuse of committed `ExpressionId` values.

---

## MNIR-EXPR-065 — Future persistence requirement

A future persisted representation intended to restore a Program lineage for continued mutation MUST preserve sufficient Block and Expression allocation history to maintain these non-reuse guarantees.

---

# 29. Snapshot and fork behavior

## MNIR-EXPR-066 — Snapshot identity preservation

A Program snapshot MUST preserve all committed `BlockId` and `ExpressionId` values present in that revision.

---

## MNIR-EXPR-067 — Fork identity behavior

A fork MUST preserve the body and Expression semantic contents of its source snapshot.

A fork MAY preserve or remap raw Block and Expression identifier values.

The fork MUST maintain fork-local allocation history sufficient to prevent reuse of preserved or remapped committed identities.

---

## MNIR-EXPR-099 — Fork reference consistency

If a fork implementation remaps any:

```text
BlockId
ExpressionId
ParameterId
```

values, every semantic reference affected by that remapping MUST be updated consistently within the fork.

This includes at minimum:

* Return → Expression references;
* ParameterReference → Parameter references.

The resulting fork MUST preserve the semantic contents and structural reference integrity of the source snapshot.

Identifier remapping MUST occur atomically as part of fork construction.

---

# 30. Expression inspection

## MNIR-EXPR-068 — Expression kind inspection

The public read-only API MUST permit consumers to determine which version 0.1 Expression kind an Expression represents.

---

## MNIR-EXPR-069 — Expression semantic type inspection

The public read-only API MUST permit consumers to determine the derived intrinsic type of a version 0.1 Expression.

For `ParameterReference`, this type is derived from the referenced Parameter in the same Program revision or transaction working state.

---

## MNIR-EXPR-090 — Expression type inspection requires resolvable references

The derived intrinsic type of a `ParameterReference` Expression is available only when the referenced `ParameterId` resolves to an existing Parameter owned by the Function containing that Expression.

If a transaction working state temporarily contains a dangling `ParameterReference` permitted by `MNIR-EXPR-058`, type inspection of that Expression MUST report that the type cannot currently be derived.

The public API MUST NOT invent, cache as authoritative, or return the former Parameter type as though the reference were structurally valid.

The exact Rust error type is implementation-defined.

---

## MNIR-EXPR-091 — Type inspection failure is not transaction poisoning

Failure to inspect the derived type of a temporarily dangling `ParameterReference` is a read-only inspection failure.

Such inspection MUST NOT by itself poison an otherwise Active mutation transaction.

---

## MNIR-EXPR-100 — Expression data inspection

For each version 0.1 Expression kind, the public read-only API MUST expose sufficient semantic data to reconstruct that Expression's version 0.1 meaning.

At minimum:

```text
Int32Literal
    literal value

Int64Literal
    literal value

BoolLiteral
    literal value

UnitLiteral
    Expression kind is sufficient

ParameterReference
    referenced ParameterId
```

This requirement does not define serialization or textual formatting.

---

# 31. Function body and Block inspection

## MNIR-EXPR-092 — Function body presence inspection

The public read-only API MUST permit a consumer to determine whether a Function owns a Function body.

---

## MNIR-EXPR-093 — Function body Block inspection

When a Function owns a body, the public read-only API MUST permit a consumer to determine the body's `BlockId`.

---

## MNIR-EXPR-094 — Block Expression lookup

The public read-only API MUST permit lookup of an Expression owned by a Block using `ExpressionId`.

Lookup MUST NOT depend on Expression collection iteration order.

---

## MNIR-EXPR-095 — Block Expression enumeration

The public read-only API MUST permit inspection of the Expressions currently owned by a Block.

Any exposed iteration order is non-semantic according to `MNIR-EXPR-018`.

---

## MNIR-EXPR-096 — Return inspection

The public read-only API MUST permit a consumer to determine:

* whether a working-state Block currently has a Return terminator;
* and, when present, the `ExpressionId` referenced by Return.

A committed structurally valid Function body always has a Return terminator.

---

# 32. Expression reference integrity

## MNIR-EXPR-070 — Existing Expression references

Every committed Expression reference introduced by this specification MUST resolve to an existing Expression of the required ownership domain.

---

## MNIR-EXPR-071 — No textual references

Expression and Return references MUST use semantic identifiers rather than presentation names or textual source fragments.

---

# 33. Structural validity

## MNIR-EXPR-072 — Structurally valid Function body

A Function body is structurally valid when:

1. it is owned by exactly one existing Function;
2. it contains exactly one Block;
3. the Block has exactly one valid `BlockId`;
4. every Expression has exactly one valid `ExpressionId`;
5. every Expression belongs to that Block;
6. all Parameter references resolve to Parameters owned by the Function;
7. the Block has exactly one Return terminator;
8. the Return Expression exists in the same Block;
9. all applicable committed identity non-reuse rules are satisfied.

---

## MNIR-EXPR-073 — Structural commit rejection

A transaction MUST NOT commit a Program revision violating `MNIR-EXPR-072` or another applicable structural rule from this specification.

---

# 34. No expression evaluation order

## MNIR-EXPR-074 — No sibling Expression execution order

Version 0.1 MUST NOT infer execution or evaluation order from Expression collection iteration order.

This specification defines no side effects for the Expression kinds introduced by version 0.1.

Future specifications introducing effectful Expressions or sequencing MUST define execution ordering explicitly.

---

# 35. No arithmetic

## MNIR-EXPR-075 — Arithmetic excluded

Expressions and Basic Function Bodies 0.1 MUST NOT define:

* addition;
* subtraction;
* multiplication;
* division;
* remainder;
* numeric conversion;
* overflow behavior.

Those semantics require a later specification.

---

# 36. No local variables

## MNIR-EXPR-076 — Local variables excluded

Version 0.1 MUST NOT introduce local variables, assignment, mutable locals, immutable bindings, or local variable references.

---

# 37. No Function invocation

## MNIR-EXPR-077 — Invocation excluded

Version 0.1 MUST NOT introduce Function invocation, argument binding, call Expressions, or dispatch.

---

# 38. No control flow beyond Return

## MNIR-EXPR-078 — Control flow excluded

Version 0.1 MUST NOT introduce:

* branches;
* conditional execution;
* loops;
* jumps;
* multiple Blocks;
* exception edges;
* early exits other than the single Return terminator.

---

# 39. Extended no-op comparison

## MNIR-EXPR-079 — Body and Expression no-op state

The Program Model no-op comparison MUST additionally include:

* Function body presence;
* Block identity;
* Expression membership;
* Expression identities;
* Expression kinds and contained literal/reference data;
* Return Expression reference;
* committed `BlockId` allocation history;
* committed `ExpressionId` allocation history.

---

## MNIR-EXPR-080 — Allocate-then-remove affects no-op state

A transaction that allocates a provisional Block or Expression and later removes it before successful commit MUST NOT be classified as a no-op if successful commit advances committed allocation history.

---

# 40. Specification gaps

## MNIR-EXPR-081 — Undefined Expression semantics

If implementation requires externally observable Expression or Function-body behavior that is not defined by this specification, the implementation MUST NOT establish that behavior as normative MNIR semantics.

The missing behavior MUST be reported as a specification gap.

---

# 41. Explicitly unresolved topics

The following topics are intentionally unresolved:

* arithmetic Expressions;
* checked arithmetic;
* wrapping arithmetic;
* saturating arithmetic;
* arithmetic overflow execution behavior;
* comparisons;
* logical operators;
* local variables;
* bindings;
* assignment;
* multiple Blocks;
* branches;
* conditional Expressions;
* loops;
* Function invocation;
* arguments;
* call ordering;
* side effects;
* expression evaluation strategy;
* common subexpression semantics;
* expression canonicalization;
* expression deduplication;
* expression equivalence;
* individual Expression removal;
* Expression replacement;
* ParameterReference retargeting;
* Expression kind mutation;
* semantic verifier;
* diagnostic codes;
* dead-expression diagnostics;
* unreachable code;
* Function-body execution;
* backend lowering;
* serialization format;
* EasyH syntax.

---

## MNIR-EXPR-082 — Unresolved topics are not 0.1 semantics

An implementation MUST NOT establish topics listed as unresolved as normative Expressions and Basic Function Bodies 0.1 semantics.

---

# 42. Acceptance requirements

## AR-EXPR-001 — Body creation

Create a Function body for an existing Function.

Verify that:

* exactly one provisional `BlockId` is created;
* the Block belongs to the Function body;
* and the Function still preserves its existing `FunctionId`.

---

## AR-EXPR-002 — Bodyless Function remains valid

Demonstrate that an existing Function without a body remains structurally valid.

---

## AR-EXPR-003 — Duplicate body creation fails

Attempt to create a second body for a Function that already owns one.

Verify operation failure and transaction poisoning.

---

## AR-EXPR-004 — Int32 literal

Create and commit an `Int32Literal`.

Verify its stored value, Expression kind, `ExpressionId`, and derived type `Int32`.

---

## AR-EXPR-005 — Int64 literal

Create and commit an `Int64Literal`.

Verify its stored value and derived type `Int64`.

---

## AR-EXPR-006 — Bool literal

Create and commit both logical Bool literal values and verify their derived type.

---

## AR-EXPR-007 — Unit literal

Create and commit a Unit literal and verify its derived type.

---

## AR-EXPR-008 — Parameter reference

Create a `ParameterReference` Expression to a Parameter owned by the same Function.

Verify:

* referenced `ParameterId`;
* `ExpressionId`;
* owning Block;
* and derived intrinsic type.

---

## AR-EXPR-009 — Cross-Function Parameter reference fails

Attempt to create a Parameter reference using a Parameter owned by another Function.

Verify operation failure and transaction poisoning.

---

## AR-EXPR-010 — Return construction

Create a Parameter reference and establish it as the Block's Return Expression.

Commit successfully.

---

## AR-EXPR-011 — Return replacement

Set Return to one Expression and then replace it with another Expression in the same transaction.

Verify that the final committed body has exactly one Return terminator referring to the final Expression.

---

## AR-EXPR-012 — Cross-Block or foreign Return reference rejection

Demonstrate that Return cannot reference an Expression outside the owning Block.

Because version 0.1 permits only one Block per body, this MAY be demonstrated using an Expression belonging to another Function body.

---

## AR-EXPR-013 — Unterminated body cannot commit

Create a Function body without setting Return.

Verify that commit fails atomically and the previous committed Program revision remains unchanged.

---

## AR-EXPR-014 — Return type mismatch is representable

Create a structurally valid Function body whose Return Expression intrinsic type differs from the declared Function return type.

Verify that structural commit succeeds.

Do not introduce semantic verification or diagnostics.

---

## AR-EXPR-015 — Parameter type mutation changes reference type

Create and commit a Parameter reference.

Later change the referenced Parameter intrinsic type.

Verify that:

* the `ExpressionId` is unchanged;
* and the derived Expression type reflects the Parameter's new type.

---

## AR-EXPR-016 — Parameter removal creates structural commit failure

Create a committed body referencing a Parameter.

In a later transaction remove the referenced Parameter without removing or repairing the body.

Verify that commit fails structurally and atomically.

---

## AR-EXPR-017 — Parameter removal can be repaired by body removal

Create a committed Function body containing a `ParameterReference`.

In a later transaction:

1. remove the referenced Parameter;
2. verify that the transaction remains Active;
3. remove the affected Function body;
4. commit successfully.

Verify that the resulting Program contains neither the removed Parameter nor the removed body and contains no dangling Parameter reference.

No individual Expression removal or reference-rewrite operation is required.

---

## AR-EXPR-018 — Function removal cascades to body

Remove a Function containing a body.

Verify removal of:

* body;
* Block;
* Expressions;
* Return terminator.

Committed Block and Expression identities remain retired.

---

## AR-EXPR-019 — Module removal cascades to bodies

Remove a Module containing Functions with bodies.

Verify all descendant body structures are removed and committed identities remain retired.

---

## AR-EXPR-020 — BlockId retirement

Create and commit a Function body, remove it, create another Function body, and verify that the committed `BlockId` is not reused.

---

## AR-EXPR-021 — ExpressionId retirement

Create and commit an Expression, remove its body, later create another Expression, and verify that the committed `ExpressionId` is not reused.

---

## AR-EXPR-022 — Provisional identity history after body removal

Within one transaction:

1. create a Function body;
2. allocate at least one Expression inside its Block;
3. remove the Function body before commit;
4. commit successfully.

Verify that:

* the resulting Function has no body;
* the allocated provisional `BlockId` enters committed Block allocation history;
* the allocated provisional `ExpressionId` enters committed Expression allocation history;
* later allocations do not reuse either committed identifier;
* and the successful transaction is not classified as a no-op.

---

## AR-EXPR-023 — Failed provisional identity

Demonstrate that failed or discarded transactions do not commit provisional Block or Expression identities.

---

## AR-EXPR-024 — Snapshot identity preservation

Verify snapshots preserve committed `BlockId` and `ExpressionId` values.

---

## AR-EXPR-025 — Fork identity behavior

Verify a fork preserves body and Expression semantics and maintains sufficient fork-local allocation history.

Raw ID preservation or remapping is implementation-defined.

---

## AR-EXPR-026 — Expression collection has no semantic order

Create multiple Expressions in one Block.

Verify that they are addressed by `ExpressionId` and that no semantic execution order is derived from collection position.

---

## AR-EXPR-027 — Typed identifiers

Demonstrate compile-time distinction between:

```text
BlockId
ExpressionId
FunctionId
ParameterId
ModuleId
ProgramId
RevisionId
```

where technically practical.

---

## AR-EXPR-028 — Structural rejection

Using an internal test-only construction mechanism if necessary, demonstrate that invalid body/reference structure cannot become committed Program state.

Do not weaken the safe public API to create invalid state.

---

## AR-EXPR-029 — Existing conformance preserved

All Program Model 0.1, Type System Foundations 0.1, and Functions and Parameters 0.1 tests MUST continue to pass without semantic regression.

---

## AR-EXPR-030 — No arithmetic implementation

Demonstrate through conformance inspection that this increment introduces no arithmetic, overflow, conversion, or comparison semantics.

---

## AR-EXPR-031 — No speculative control-flow implementation

Demonstrate through conformance inspection that this increment introduces no multiple Blocks, branches, loops, calls, locals, or generic Statement/Node abstraction.

---

## AR-EXPR-032 — Int32 literal range enforcement

Demonstrate that the public safe mutation API cannot create an `Int32Literal` outside the abstract `Int32` value domain.

This MAY be demonstrated by the Rust input type itself if the API accepts a representation whose entire value range is valid `Int32`.

The implementation SHOULD NOT introduce a wider public input type solely to manufacture an out-of-range runtime test.

---

## AR-EXPR-033 — Int64 literal range enforcement

Demonstrate equivalent range safety for `Int64Literal`.

---

## AR-EXPR-034 — BlockId Program-lineage uniqueness

Create committed Function bodies for multiple Functions.

Verify that their committed `BlockId` values are distinct within the Program lineage.

---

## AR-EXPR-035 — ExpressionId Program-lineage uniqueness

Create committed Expressions in multiple Function bodies.

Verify that their committed `ExpressionId` values are distinct within the Program lineage.

---

## AR-EXPR-036 — Removing absent Function body poisons transaction

Attempt `remove_function_body` on an existing Function without a body.

Verify that:

* the operation fails;
* the transaction becomes Failed;
* and commit is impossible.

Also demonstrate equivalent failure for an unknown `FunctionId`.

---

## AR-EXPR-037 — Literal creation against unknown Block fails

Perform at least one successful transaction-local operation.

Then attempt to create a literal using an unknown `BlockId`.

Verify that:

* the operation fails;
* the transaction becomes Failed;
* and no transaction-local changes are committed.

---

## AR-EXPR-038 — Return target validation

Demonstrate that `set_return` fails and poisons the transaction when:

* the target `BlockId` is unknown;
* the target `ExpressionId` is unknown;
* or the Expression exists but belongs to another Block.

---

## AR-EXPR-039 — Provisional Block identity uniqueness

Allocate multiple Function bodies in one Active transaction.

Verify that all provisional `BlockId` values are distinct and do not collide with committed Block history.

---

## AR-EXPR-040 — Provisional Expression identity uniqueness

Allocate multiple Expressions in one Active transaction and verify that provisional `ExpressionId` values are distinct and do not collide with committed Expression history.

---

## AR-EXPR-041 — Function body and Block inspection

Demonstrate through the public read-only API that a consumer can determine:

* whether a Function owns a body;
* the body's `BlockId`;
* the Expressions owned by that Block;
* and Expression lookup by `ExpressionId`.

---

## AR-EXPR-042 — Return inspection

Demonstrate through the public read-only API that:

* an unterminated working-state Block reports no Return;
* after `set_return`, the referenced `ExpressionId` can be inspected;
* and a committed structurally valid body exposes the committed Return reference.

---

## AR-EXPR-043 — Dangling ParameterReference type inspection

Within one Active transaction:

1. create or begin with a valid `ParameterReference`;
2. remove the referenced Parameter;
3. inspect the Expression's derived type.

Verify that:

* inspection reports a typed/unavailable-type result rather than returning the former type as valid;
* the read-only inspection does not poison the transaction;
* the transaction can subsequently restore structural validity by removing the body.

---

## AR-EXPR-044 — Function and Module cascade preserve body identity retirement

Demonstrate that committed Block and Expression identifiers remain retired when removed transitively through:

* Function removal;
* and Module removal.

This MAY extend the existing `AR-EXPR-018` and `AR-EXPR-019` tests rather than requiring separate duplicate test scenarios.

---

# 43. Implementation constraints

## MNIR-EXPR-083 — Core ownership

Function bodies, Blocks, Expressions, and Return representation MUST belong to `mnir-core`.

---

## MNIR-EXPR-084 — Dependency direction

Implementing this specification MUST NOT cause `mnir-core` to depend on:

```text
mnir-verify
easyh-render
mnir-cli
```

---

## MNIR-EXPR-085 — Typed identifier implementation

The implementation SHOULD use distinct Rust types for:

```text
BlockId
ExpressionId
```

and SHOULD make accidental interchange with other identifier categories fail at compile time where practical.

---

## MNIR-EXPR-086 — Controlled mutation only

The public API MUST NOT expose unrestricted mutable access that permits callers to bypass Function-body or Expression structural invariants.

---

## MNIR-EXPR-087 — Safe Rust

The implementation MUST NOT use or require `unsafe` Rust to satisfy this specification.

---

## MNIR-EXPR-088 — No generic Statement or Node abstraction

This specification does not require:

```text
Node
NodeId
Statement
StatementId
TerminatorId
SemanticReference
```

as general abstractions.

The implementation MUST NOT introduce them as normative MNIR semantics solely in anticipation of future specifications.

---

## MNIR-EXPR-089 — No semantic verifier

Implementation MUST NOT introduce a semantic verifier or `VerifiedProgram` merely to enforce Return type compatibility.

---

# 44. Expected implementation shape

The expected conceptual model is:

```text
Program
└── Module
    └── Function
        ├── Signature
        └── Body? 
            └── Block
                ├── BlockId
                ├── Expressions { identity-based, unordered }
                │   ├── Int32Literal
                │   ├── Int64Literal
                │   ├── BoolLiteral
                │   ├── UnitLiteral
                │   └── ParameterReference
                └── Return(ExpressionId)
```

The exact Rust module and type names are implementation-defined unless otherwise specified.

---

# 45. Target example

After implementation, MNIR must be able to represent:

```text
identity(value: Int32) -> Int32 {
    return value
}
```

conceptually as:

```text
FunctionId(F1)
├── return_type: Int32
├── ParameterId(P1)
│   ├── preferred_name: "value"
│   └── type: Int32
└── Body
    └── BlockId(B1)
        ├── ExpressionId(E1)
        │   └── ParameterReference(P1)
        └── Return(E1)
```

The representation above is informative.

It is neither EasyH syntax nor MNIR serialization syntax.

---

# 46. Foundational invariants

The following is an informative summary:

```text
Functions may have zero or one body.

A body contains exactly one Block in 0.1.

Blocks and Expressions have Program-lineage-scoped identities.

Committed Block and Expression IDs are never reused.

Expressions are value-producing semantic entities.

Return is a terminator, not an Expression.

Expression collection order is non-semantic.

Literal types are intrinsic and derived.

ParameterReference derives its type from the referenced Parameter.

ParameterReference must target the owning Function's Parameter.

Return must reference an Expression in the same Block.

Return type mismatch is semantic invalidity, not structural invalidity.

No semantic verifier exists yet.

No arithmetic exists yet.

No multiple Blocks or branching exist yet.

All mutations use the existing Program transaction model.
```
