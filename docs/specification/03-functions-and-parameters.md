# MNIR Specification — Functions and Parameters

**Document:** `03-functions-and-parameters.md`
**Specification status:** Draft
**Specification version:** 0.1
**Normative:** Yes

---

## 1. Purpose

This document defines the first MNIR representation of function signatures.

Specification version 0.1 introduces:

* `Function`
* `FunctionId`
* `Parameter`
* `ParameterId`
* Module ownership of Functions
* Function ownership of Parameters
* semantic Parameter ordering
* Function return types
* Parameter types
* controlled mutation of Function signatures
* persistent Function and Parameter identity

The purpose of this increment is to make MNIR capable of representing a function signature conceptually equivalent to:

```text
add(a: Int32, b: Int32) -> Int32
```

This specification does not define:

* Function bodies
* Statements
* Expressions
* Return statements
* Function invocation
* Local variables
* Closures
* Lambdas
* Methods
* Constructors
* Function overloading
* Name-based Function lookup
* Default Parameters
* Optional Parameters
* Variadic Parameters
* Generic Functions
* Generic Parameters
* Type Parameters
* Parameter passing conventions
* References or borrowing
* Async Functions
* Effects
* Contracts
* Visibility
* Imports or exports
* EasyH Function syntax
* ABI or calling conventions

Those concepts are introduced by later specification increments.

---

# 2. Normative language

Every declarative statement inside a numbered `MNIR-FUNC-*` rule is normative unless the rule explicitly states otherwise.

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** retain their normative meanings when used inside numbered rules.

Text outside numbered `MNIR-FUNC-*` rules is informative unless explicitly stated otherwise.

Acceptance requirements use identifiers of the form:

```text
AR-FUNC-NNN
```

All applicable `AR-FUNC-*` requirements are mandatory project-level acceptance requirements for the initial implementation increment defined by this document.

Acceptance requirements demonstrate normative behavior but do not independently introduce MNIR semantics.

---

# 3. Dependencies

This specification builds on:

* MNIR Program Model 0.1
* MNIR Type System Foundations 0.1

The concepts defined by those specifications retain their existing semantics.

In particular:

* `Program` remains the aggregate root.
* mutations remain Program-level mutation transactions.
* committed identifiers are never reused within a Program lineage.
* intrinsic types remain Program-independent.
* textual source code is not required.

---

# 4. Terminology

## 4.1 Function

A **Function** is an independently addressable semantic entity owned by exactly one Module.

In version 0.1, a Function contains only information required to describe its signature and presentation metadata.

A Function has:

```text
FunctionId
ordered Parameters
return type
optional preferred name
optional documentation
```

A Function does not yet contain executable behavior.

---

## 4.2 Parameter

A **Parameter** is an independently addressable semantic entity owned by exactly one Function.

A Parameter has:

```text
ParameterId
IntrinsicType
optional preferred name
optional documentation
```

Its position in the Function's ordered Parameter sequence is semantically significant.

---

## 4.3 Function signature

For this specification increment, the **Function signature shape** consists of:

```text
ordered Parameter types
return type
```

Presentation names are not part of the semantic signature shape.

This specification does not define overload resolution, name-based Function identity, or signature-based Function lookup.

---

# 5. Function identity

## MNIR-FUNC-001 — Function identity

Every Function MUST have exactly one `FunctionId`.

`FunctionId` is a distinct semantic identifier category.

---

## MNIR-FUNC-002 — FunctionId uniqueness domain

A committed `FunctionId` MUST be unique among all Function identities allocated within one Program lineage.

The uniqueness domain of `FunctionId` is the Program lineage, not the Module.

Therefore, two Functions in different Modules of the same Program lineage MUST NOT share the same `FunctionId`.

---

## MNIR-FUNC-003 — FunctionId opacity

The raw representation of a `FunctionId` MUST NOT carry normative semantic meaning beyond identity.

Consumers MUST NOT infer:

* Module ownership,
* creation order,
* revision,
* name,
* position,
* or Function semantics

from the raw identifier representation.

---

## MNIR-FUNC-004 — Function identity preservation

Updating an existing Function through a Function update operation MUST preserve its `FunctionId`.

Changes that preserve Function identity include:

* preferred name changes,
* documentation changes,
* return type changes,
* Parameter additions,
* Parameter removals,
* Parameter metadata changes,
* and Parameter type changes.

A change in Function signature does not by itself create a new Function identity.

---

## MNIR-FUNC-005 — Function identity retirement

Once a `FunctionId` has become part of a committed Program revision, that identifier MUST never be allocated to another Function identity in the same Program lineage.

Removing a Function MUST NOT release its committed `FunctionId` for reuse.

---

# 6. Function ownership

## MNIR-FUNC-006 — Module ownership

Every committed Function MUST be owned by exactly one Module in the committed Program revision.

---

## MNIR-FUNC-007 — Function containment

A Module MAY own zero or more Functions.

A Function MUST NOT simultaneously belong to multiple Modules in the same committed Program revision.

---

## MNIR-FUNC-008 — Function ownership is semantic

The owning Module of a Function is part of the semantic Program structure.

Module ownership MUST NOT be inferred from presentation names or textual layout.

---

## MNIR-FUNC-075 — Function collection is identity-based

A Module's Function collection MUST be treated as an identity-based collection.

Function collection position MUST NOT contribute to Function identity or Function semantics.

Functions and Parameters 0.1 defines no semantic ordering between sibling Functions owned by the same Module.

---

## MNIR-FUNC-076 — Function iteration order is non-semantic

If an implementation exposes iteration over Functions in a Module, the observed iteration order MUST NOT be interpreted as MNIR semantic ordering.

An implementation MAY use any internal collection representation consistent with the other requirements of this specification.

Future serialization or presentation specifications MAY define deterministic output ordering without making that ordering part of Function semantics.

---

## MNIR-FUNC-009 — No Function move operation

Functions and Parameters 0.1 does not define an operation that moves an existing Function from one Module to another while preserving its `FunctionId`.

An implementation MUST NOT expose Function movement as normative 0.1 behavior.

A future specification MAY define Function movement semantics.

---

# 7. Parameter identity

## MNIR-FUNC-010 — Parameter identity

Every Parameter MUST have exactly one `ParameterId`.

`ParameterId` is a distinct semantic identifier category.

---

## MNIR-FUNC-011 — ParameterId uniqueness domain

A committed `ParameterId` MUST be unique among all Parameter identities allocated within one Program lineage.

The uniqueness domain of `ParameterId` is the Program lineage, not the owning Function.

Two Parameters belonging to different Functions in the same Program lineage MUST NOT share the same `ParameterId`.

---

## MNIR-FUNC-012 — ParameterId opacity

The raw representation of a `ParameterId` MUST NOT carry normative semantic meaning beyond identity.

Consumers MUST NOT infer:

* owning Function,
* Parameter position,
* Parameter type,
* preferred name,
* creation order,
* or revision

from its raw representation.

---

## MNIR-FUNC-013 — Parameter identity preservation

Updating an existing Parameter MUST preserve its `ParameterId`.

This includes:

* preferred name changes,
* documentation changes,
* and type changes.

Reordering Parameters, if introduced by a future specification, does not necessarily imply new Parameter identity.

Version 0.1 does not define a general reorder operation.

---

## MNIR-FUNC-014 — Parameter identity retirement

Once a `ParameterId` has become part of a committed Program revision, that identifier MUST never be allocated to another Parameter identity in the same Program lineage.

Removing a Parameter MUST NOT release its committed `ParameterId` for reuse.

---

## MNIR-FUNC-015 — Function removal retires Parameter identities

When a committed Function is removed, every committed Parameter belonging to that Function ceases to exist in the new Program revision.

Their committed `ParameterId` values MUST remain permanently retired within the Program lineage.

---

# 8. Parameter ownership

## MNIR-FUNC-016 — Function ownership of Parameters

Every committed Parameter MUST be owned by exactly one Function.

---

## MNIR-FUNC-017 — No shared Parameters

A Parameter MUST NOT simultaneously belong to multiple Functions.

---

## MNIR-FUNC-018 — Parameter ownership is semantic

The owning Function of a Parameter is part of the semantic Program structure.

Parameter ownership MUST NOT be inferred from presentation names or Parameter position alone.

---

## MNIR-FUNC-019 — No Parameter move operation

Version 0.1 does not define moving an existing Parameter between Functions while preserving its `ParameterId`.

An implementation MUST NOT expose such movement as normative 0.1 behavior.

---

# 9. Parameter ordering

## MNIR-FUNC-020 — Ordered Parameters

Each Function MUST own an ordered sequence of Parameters.

The order of that sequence is semantically significant.

---

## MNIR-FUNC-021 — Position affects signature shape

Two Function signature shapes with identical Parameter types in different positions are distinct whenever the ordered type sequences differ.

For example:

```text
(Int32, Bool) -> Unit
```

and:

```text
(Bool, Int32) -> Unit
```

are different signature shapes.

---

## MNIR-FUNC-022 — Parameter identity is independent of position

Parameter position MUST NOT define Parameter identity.

A `ParameterId` identifies the Parameter itself.

Position identifies the Parameter's current place in the ordered sequence.

---

## MNIR-FUNC-023 — Parameter append semantics

The version 0.1 `add_parameter` operation MUST append the new Parameter to the end of the Function's current Parameter sequence.

Version 0.1 does not define insertion at an arbitrary position.

---

## MNIR-FUNC-024 — Parameter removal ordering

Removing a Parameter MUST preserve the relative ordering of all remaining Parameters.

For example, removing `P2` from:

```text
P1, P2, P3
```

produces:

```text
P1, P3
```

with `P1` remaining before `P3`.

---

# 10. Function and Parameter types

## MNIR-FUNC-025 — Parameter type

Every Parameter MUST have exactly one type.

In Functions and Parameters 0.1, the Parameter type MUST be one of the intrinsic types defined by Type System Foundations 0.1:

```text
Int32
Int64
Bool
Unit
```

---

## MNIR-FUNC-026 — Function return type

Every Function MUST have exactly one return type.

In Functions and Parameters 0.1, the return type MUST be one of:

```text
Int32
Int64
Bool
Unit
```

---

## MNIR-FUNC-027 — No implicit type relation

This specification introduces no conversion, subtype, coercion, compatibility, or assignment relationship between Function or Parameter types.

A Parameter of type `Int32` has type `Int32`.

A return type of `Int64` has type `Int64`.

No further relation is defined by this increment.

---

## MNIR-FUNC-028 — Type changes preserve entity identity

Changing the type of an existing Parameter MUST preserve its `ParameterId`.

Changing the return type of an existing Function MUST preserve its `FunctionId`.

Both operations produce Program mutations and therefore participate in the revision semantics defined by Program Model 0.1.

---

# 11. Function presentation metadata

A Function may carry:

```text
preferred_name
documentation
```

using the same semantic separation established for Module presentation metadata.

---

## MNIR-FUNC-029 — Function preferred name

A Function MAY have a preferred human-readable name.

The preferred name MUST NOT determine `FunctionId`.

---

## MNIR-FUNC-030 — Duplicate Function preferred names

Multiple Functions in the same Module MAY have the same preferred name in version 0.1.

This specification defines no Function name resolution or overloading semantics.

---

## MNIR-FUNC-031 — Function documentation

A Function MAY contain documentation text.

Documentation MUST NOT affect Function identity or signature shape.

---

# 12. Parameter presentation metadata

A Parameter may carry:

```text
preferred_name
documentation
```

---

## MNIR-FUNC-032 — Parameter preferred name

A Parameter MAY have a preferred human-readable name.

The preferred name MUST NOT determine `ParameterId`.

---

## MNIR-FUNC-033 — Duplicate Parameter preferred names

Multiple Parameters of the same Function MAY have identical preferred names in version 0.1.

Name uniqueness and name-based Parameter lookup are not defined.

Parameter identity and Parameter position remain independent of names.

---

## MNIR-FUNC-034 — Parameter documentation

A Parameter MAY contain documentation text.

Documentation MUST NOT affect Parameter identity, position, or type.

---

# 13. Committed identifier history

## MNIR-FUNC-035 — Function allocation history

A mutable Program lineage MUST maintain sufficient committed allocation history to ensure that committed `FunctionId` values are never reused.

---

## MNIR-FUNC-036 — Parameter allocation history

A mutable Program lineage MUST maintain sufficient committed allocation history to ensure that committed `ParameterId` values are never reused.

---

## MNIR-FUNC-037 — Persistence requirement

Any future persisted representation intended to restore a Program lineage for continued mutation MUST preserve enough Function and Parameter identifier allocation history to maintain the non-reuse guarantees defined by this specification.

The serialization representation remains outside this specification's scope.

---

# 14. Provisional identities

The provisional identity semantics established by Program Model 0.1 are extended to Functions and Parameters.

---

## MNIR-FUNC-038 — Provisional FunctionId

A `FunctionId` allocated inside an Active mutation transaction is provisional until successful commit.

A provisional Function may be addressed by its provisional `FunctionId` within the same transaction.

---

## MNIR-FUNC-039 — Provisional ParameterId

A `ParameterId` allocated inside an Active mutation transaction is provisional until successful commit.

A provisional Parameter may be addressed by its provisional `ParameterId` within the same transaction.

---

## MNIR-FUNC-040 — Successful provisional identity commit

When a transaction commits successfully, every Function and Parameter identifier allocated during that transaction MUST become part of the Program lineage's committed allocation history.

This applies even when an entity is created and removed again inside the same successfully committed transaction.

---

## MNIR-FUNC-041 — Failed provisional identity

If a transaction fails or is discarded without commit, provisional Function and Parameter identifiers do not become committed semantic identities.

Their raw values MAY be reused later according to the implementation-defined allocator strategy.

Consumers MUST NOT rely on provisional identity durability after failure or discard.

---

# 15. Controlled mutation

Functions and Parameters use the Program-level transaction model defined by Program Model 0.1.

No independently committed Function or Module transaction exists in this specification.

---

## MNIR-FUNC-042 — Initial Function operations

The version 0.1 controlled mutation API MUST provide operations equivalent to:

```text
add_function
remove_function
set_function_preferred_name
set_function_documentation
set_function_return_type
```

The exact Rust API names are implementation-defined.

`add_function` MUST allocate the `FunctionId` through the Program lineage.

The normal public mutation API MUST NOT allow callers to select the raw `FunctionId`.

A new Function MUST be created with an explicit intrinsic return type.

---

## MNIR-FUNC-043 — Initial Parameter operations

The version 0.1 controlled mutation API MUST provide operations equivalent to:

```text
add_parameter
remove_parameter
set_parameter_preferred_name
set_parameter_documentation
set_parameter_type
```

The exact Rust API names are implementation-defined.

`add_parameter` MUST allocate the `ParameterId` through the Program lineage.

The normal public mutation API MUST NOT allow callers to select the raw `ParameterId`.

A new Parameter MUST be created with an explicit intrinsic type.

---

# 16. Mutation target validity

## MNIR-FUNC-044 — Unknown Module for Function creation

`add_function` MUST fail if the target `ModuleId` does not identify a Module in the transaction's current working state.

The failure MUST poison the mutation transaction according to Program Model 0.1.

---

## MNIR-FUNC-045 — Unknown Function mutation

An operation requiring an existing Function MUST fail if the `FunctionId` does not identify a Function in the transaction's current working state.

This applies at minimum to:

* removing a Function,
* adding a Parameter,
* setting Function preferred name,
* setting Function documentation,
* and changing Function return type.

The failure MUST poison the transaction.

---

## MNIR-FUNC-046 — Unknown Parameter mutation

An operation requiring an existing Parameter MUST fail if the `ParameterId` does not identify a Parameter in the transaction's current working state.

This applies at minimum to:

* removing a Parameter,
* setting Parameter preferred name,
* setting Parameter documentation,
* and changing Parameter type.

The failure MUST poison the transaction.

---

# 17. Sequential transaction behavior

## MNIR-FUNC-047 — Sequential visibility

Function and Parameter mutation operations MUST observe all earlier successful operations in the same Active mutation transaction.

For example:

```text
add function F
add parameter P to F
```

is valid.

Likewise:

```text
add function F
set preferred name of F
```

is valid.

---

## MNIR-FUNC-048 — Removed Function visibility

After a Function has been removed from a transaction's working state:

* the Function no longer exists in that working state;
* its Parameters no longer exist in that working state;
* subsequent operations requiring that Function MUST fail;
* subsequent operations requiring one of its Parameters MUST fail.

---

## MNIR-FUNC-049 — Removed Parameter visibility

After a Parameter has been removed from the transaction's working state, subsequent operations requiring that Parameter MUST fail.

---

# 18. Function removal semantics

## MNIR-FUNC-050 — Function removal

Removing a Function MUST remove that Function from its owning Module in the transaction working state.

All Parameters owned by that Function MUST also be removed from the working state.

---

## MNIR-FUNC-051 — Removal does not affect unrelated Functions

Removing one Function MUST NOT alter the identity, ownership, Parameter ordering, Parameter types, return type, or presentation metadata of unrelated Functions.

---

# Module removal semantics

The Program Model `remove_module` operation is extended by this specification because Modules may now own Functions and Parameters.

## MNIR-FUNC-077 — Module removal cascades

Removing a Module from a mutation transaction working state MUST also remove from that working state:

* every Function owned by that Module;
* and every Parameter owned by those Functions.

No Function or Parameter formerly owned by the removed Module may remain structurally present in the resulting working state.

---

## MNIR-FUNC-078 — Removed Module descendants become unavailable

After a Module has been removed from a transaction working state:

* operations requiring that Module MUST fail;
* operations requiring any Function formerly owned by that Module MUST fail;
* operations requiring any Parameter formerly owned by those Functions MUST fail.

Such failures MUST follow the transaction poisoning semantics defined by Program Model 0.1.

---

## MNIR-FUNC-079 — Module removal preserves committed identity retirement

Removing a Module MUST NOT release any committed `FunctionId` or `ParameterId` that belonged to descendants of that Module.

Those committed identifiers remain permanently retired within the Program lineage.

---

## MNIR-FUNC-080 — Provisional descendants of removed Module

If a Function or Parameter was allocated provisionally during the current transaction and is subsequently removed because its owning Module is removed, its identifier MUST follow `MNIR-FUNC-040` and `MNIR-FUNC-041`.

Therefore:

* if the transaction commits successfully, the provisionally allocated identifier becomes part of committed allocation history even though the entity is absent from the resulting revision;
* if the transaction fails or is discarded, the identifier does not become committed.

---

## MNIR-FUNC-081 — Module removal isolation

Removing one Module MUST NOT alter the identity, contents, Function membership, Parameter membership, signatures, or presentation metadata of another Module or of Functions and Parameters owned by another Module.

---

# 19. Structural validity

Structural validity in this specification extends Program Model structural validity.

---

## MNIR-FUNC-052 — Structurally valid Function

A Function is structurally valid under Functions and Parameters 0.1 when:

1. it contains exactly one `FunctionId`;
2. it is owned by exactly one existing Module;
3. it has exactly one intrinsic return type;
4. it contains an ordered sequence of zero or more Parameters;
5. no two Functions in the Program share its `FunctionId`;
6. its optional presentation metadata is representable by the implementation.

---

## MNIR-FUNC-053 — Structurally valid Parameter

A Parameter is structurally valid when:

1. it contains exactly one `ParameterId`;
2. it is owned by exactly one existing Function;
3. it has exactly one intrinsic type;
4. its position occurs exactly once in the owning Function's ordered Parameter sequence;
5. no two Parameters in the Program share its `ParameterId`;
6. its optional presentation metadata is representable by the implementation.

---

## MNIR-FUNC-054 — Structural rejection

A transaction MUST NOT commit a Program revision that violates any applicable Function or Parameter structural rule.

A structural commit failure MUST preserve the atomicity guarantees of Program Model 0.1.

---

# 20. Function signature semantics

## MNIR-FUNC-055 — Signature shape

The Function signature shape in version 0.1 consists of:

```text
ordered sequence of Parameter intrinsic types
Function intrinsic return type
```

---

## MNIR-FUNC-056 — Presentation excluded from signature shape

The following MUST NOT affect Function signature shape:

* Function preferred name,
* Function documentation,
* Parameter preferred names,
* Parameter documentation,
* FunctionId raw representation,
* ParameterId raw representations.

---

## MNIR-FUNC-057 — Identity distinct from signature shape

Function identity and Function signature shape are distinct concepts.

Two different Functions MAY have identical signature shapes.

Changing an existing Function's signature shape MUST NOT by itself change its `FunctionId`.

---

# 21. No executable behavior

## MNIR-FUNC-058 — No Function body

Functions and Parameters 0.1 MUST NOT introduce a Function body representation.

A Function defined by this specification represents only a signature and associated presentation metadata.

---

## MNIR-FUNC-059 — No Function invocation

Functions and Parameters 0.1 MUST NOT define Function calls, invocation expressions, dispatch, argument binding, or execution behavior.

---

## MNIR-FUNC-060 — No return statement

The Function return type defined by this specification MUST NOT be interpreted as defining a return statement or return expression.

Executable return behavior belongs to a later specification.

---

# 22. No name-based semantics

## MNIR-FUNC-061 — Names are not lookup semantics

Functions and Parameters 0.1 MUST NOT establish semantic Function or Parameter lookup based on preferred names.

---

## MNIR-FUNC-062 — No overloading semantics

The presence of multiple Functions with equal preferred names MUST NOT be interpreted as Function overloading under this specification.

Overloading, if introduced, requires a later normative specification.

---

# 23. Fork and snapshot behavior

## MNIR-FUNC-063 — Snapshot identity preservation

A Program snapshot MUST preserve all committed `FunctionId` and `ParameterId` values belonging to that revision.

---

## MNIR-FUNC-064 — Fork raw identifier preservation

A Program fork MAY preserve the raw Function and Parameter identifier values from its source snapshot.

Because the fork receives a new `ProgramId`, those Function and Parameter identities belong to a different Program lineage.

---

## MNIR-FUNC-065 — Fork allocation history

A fork that preserves existing Function or Parameter raw identifier values MUST initialize its own allocation history so those preserved identifiers cannot later be reused for different entities inside the forked lineage.

---

# 24. No-op behavior

## MNIR-FUNC-082 — Extended no-op comparison

For Programs containing Functions and Parameters, the Program Model 0.1 no-op comparison MUST additionally include:

* Function membership;
* Function identities;
* Function ownership;
* Function return types;
* Function presentation metadata;
* Parameter membership;
* Parameter identities;
* Parameter ownership;
* Parameter order;
* Parameter types;
* Parameter presentation metadata;
* committed `FunctionId` allocation history;
* committed `ParameterId` allocation history.

The comparison excludes the new `RevisionId` allocated by commit, consistent with Program Model 0.1.

---

## MNIR-FUNC-083 — Allocate-then-remove is not a no-op after successful commit

A transaction that allocates a provisional Function or Parameter and removes that entity before successful commit MUST NOT be classified as a no-op when successful commit advances Function or Parameter committed allocation history.

This rule applies whether removal occurs directly or transitively through Function or Module removal.

---

# 25. Specification gaps

## MNIR-FUNC-066 — Undefined Function semantics

If implementation requires externally observable Function or Parameter behavior that is not defined by this specification, the implementation MUST NOT establish that behavior as normative MNIR semantics.

The missing behavior MUST be reported as a specification gap according to the repository's established process.

---

# 26. Explicitly unresolved topics

The following topics are intentionally unresolved:

* Function bodies
* statements
* expressions
* return statements
* Function calls
* argument expressions
* argument-to-Parameter binding
* named arguments
* local variables
* Function references as values
* Function types
* first-class Functions
* closures
* lambdas
* methods
* constructors
* destructors
* receiver Parameters
* Function visibility
* Module exports
* Function imports
* cross-Program Function references
* overloading
* name resolution
* unique Function names
* unique Parameter names
* arbitrary Parameter insertion
* Parameter reorder operations
* default Parameters
* optional Parameters
* variadic Parameters
* generics
* type Parameters
* constraints
* effects
* contracts
* async
* exceptions
* ABI
* calling conventions
* stack representation
* EasyH syntax
* serialization syntax
* semantic verification beyond structural validity

---

## MNIR-FUNC-067 — Unresolved topics are not 0.1 semantics

An implementation MUST NOT establish topics listed as unresolved in this document as normative Functions and Parameters 0.1 semantics.

Experimental internal details are permitted only when they do not create externally observable MNIR semantics or conflict with another normative rule.

---

# 27. Acceptance requirements

## AR-FUNC-001 — Function creation

Demonstrate creating a Function in an existing Module through controlled mutation.

Verify after commit that:

* the Function has a `FunctionId`;
* the Function belongs to the requested Module;
* the Function has the requested intrinsic return type;
* and a new Program revision was created.

---

## AR-FUNC-002 — FunctionId uniqueness

Demonstrate that committed Functions within one Program lineage have distinct `FunctionId` values, including Functions owned by different Modules.

---

## AR-FUNC-003 — FunctionId retirement

Demonstrate:

1. create and commit a Function;
2. remove and commit that Function;
3. create and commit another Function.

Verify that the new Function does not reuse the removed Function's identity.

---

## AR-FUNC-004 — Function presentation identity

Demonstrate changing Function preferred name and documentation without changing:

```text
ProgramId
ModuleId
FunctionId
```

A successful commit produces a new `RevisionId`.

---

## AR-FUNC-005 — Function return type mutation

Demonstrate changing an existing Function return type from one intrinsic type to another.

Verify that:

* `FunctionId` is preserved;
* the new return type is visible after commit;
* and a new Program revision is created.

---

## AR-FUNC-006 — Parameter creation and ordering

Create a Function and append two Parameters:

```text
a: Int32
b: Bool
```

Verify that:

* both receive distinct `ParameterId` values;
* both belong to the Function;
* their order is preserved as `a`, then `b`;
* and their intrinsic types are preserved.

---

## AR-FUNC-007 — Parameter order is semantic

Create two distinct Functions.

Give the first Function Parameters with the ordered type sequence:

```text
Int32
Bool
```

Give the second Function Parameters with the ordered type sequence:

```text
Bool
Int32
```

Verify that the two Functions expose different signature shapes because their ordered Parameter type sequences differ.

No Parameter reorder operation is required.

---

## AR-FUNC-008 — ParameterId Program-lineage uniqueness

Demonstrate that Parameters belonging to different Functions still receive distinct committed `ParameterId` values within the same Program lineage.

---

## AR-FUNC-009 — ParameterId retirement

Demonstrate that a committed Parameter ID is not reused after that Parameter is removed and another Parameter is later created.

---

## AR-FUNC-010 — Function removal retires Parameter identities

Create and commit a Function containing Parameters.

Remove and commit the Function.

Create a new Function and new Parameters.

Verify that none of the removed Function's committed Parameter identities are reused.

---

## AR-FUNC-011 — Parameter presentation identity

Demonstrate changing Parameter preferred name and documentation without changing its `ParameterId`.

---

## AR-FUNC-012 — Parameter type mutation

Demonstrate changing a Parameter intrinsic type while preserving its `ParameterId`.

---

## AR-FUNC-013 — Parameter removal preserves relative order

Given:

```text
P1
P2
P3
```

remove `P2` and verify that the remaining sequence is:

```text
P1
P3
```

with identities preserved.

---

## AR-FUNC-014 — Sequential provisional Function operations

Within one transaction:

1. add a Function;
2. use its provisional `FunctionId`;
3. set its preferred name;
4. add a Parameter;
5. commit.

Verify that all changes are present in the committed revision.

---

## AR-FUNC-015 — Sequential provisional Parameter operations

Within one transaction:

1. add a Parameter;
2. use its provisional `ParameterId`;
3. change its preferred name;
4. change its type;
5. commit.

Verify that the committed Parameter preserves that identity and contains the final transaction state.

---

## AR-FUNC-016 — Unknown Function poisons transaction

Perform at least one successful transaction-local mutation and then attempt a Function operation against an unknown `FunctionId`.

Verify that:

* the operation fails;
* the transaction becomes Failed;
* commit is impossible;
* and no transaction-local changes become committed.

---

## AR-FUNC-017 — Unknown Parameter poisons transaction

Perform at least one successful transaction-local mutation and then attempt a Parameter operation against an unknown `ParameterId`.

Verify the same atomic failure guarantees as `AR-FUNC-016`.

---

## AR-FUNC-018 — Removed Function invalidates transaction-local references

Within one transaction:

1. remove an existing Function;
2. attempt to modify that Function.

Verify that the second operation fails and poisons the transaction.

Also verify that an operation targeting a Parameter formerly owned by the removed Function fails.

---

## AR-FUNC-019 — Duplicate presentation names permitted

Demonstrate that:

* two Functions may share a preferred name;
* two Parameters of one Function may share a preferred name.

No name-resolution semantics should be introduced.

---

## AR-FUNC-020 — Zero-Parameter Function

Demonstrate a structurally valid Function containing zero Parameters and one intrinsic return type.

---

## AR-FUNC-021 — Snapshot identity preservation

Demonstrate that a Program snapshot preserves committed:

```text
FunctionId
ParameterId
```

values.

---

## AR-FUNC-022 — Fork identity domain

Demonstrate that a Program fork:

* receives a new `ProgramId`;
* preserves the Function and Parameter semantic contents of the source snapshot;
* and maintains fork-local allocation history sufficient to prevent identifier reuse inside the new lineage.

If the implementation preserves raw Function or Parameter identifier values across the fork, verify that those preserved identifiers remain retired from future allocation to different entities in the forked lineage.

If the implementation remaps raw identifier values instead, document that implementation-defined choice and verify equivalent identity and non-reuse guarantees.

---

## AR-FUNC-023 — No executable Function representation

Demonstrate by conformance inspection that this increment introduces no:

* Function body,
* expression,
* statement,
* invocation,
* return statement,
* or executable Function behavior.

---

## AR-FUNC-024 — No speculative Function semantics

Demonstrate by conformance inspection that this implementation introduces no:

* overloading,
* default Parameters,
* variadic Parameters,
* generics,
* methods,
* closures,
* Function values,
* or name-resolution semantics.

---

## AR-FUNC-025 — Existing conformance preserved

All Program Model 0.1 and Type System Foundations 0.1 acceptance tests MUST continue to pass without semantic regression.

---

## AR-FUNC-026 — Function collection has no semantic order

Create multiple Functions in one Module.

Demonstrate through API behavior and conformance inspection that:

* each Function is addressed by identity;
* no Function semantic identity depends on collection position;
* and the implementation does not establish sibling Function ordering as MNIR semantics.

The implementation is not required to randomize or vary iteration order for this test.

---

## AR-FUNC-027 — Module removal cascades

Create and commit a Module containing:

* at least two Functions;
* and at least one Parameter in each Function.

Remove and commit the Module.

Verify that the resulting Program revision contains:

* no removed Module;
* none of its Functions;
* and none of its Parameters.

---

## AR-FUNC-028 — Removed Module descendant references fail

Within one Active transaction:

1. remove a Module containing Functions and Parameters;
2. attempt an operation targeting one of its former Functions.

Verify that the operation fails and poisons the transaction.

Repeat or otherwise demonstrate the same behavior for one of the Module's former Parameters.

---

## AR-FUNC-029 — Module removal does not affect unrelated Modules

Create at least two Modules containing Functions.

Remove one Module and commit.

Verify that the other Module and its Functions and Parameters preserve:

* identities;
* ownership;
* Parameter ordering;
* types;
* return types;
* and presentation metadata.

---

## AR-FUNC-030 — Create-then-remove changes allocation history

Within one transaction:

1. create a Function;
2. create at least one Parameter in that Function;
3. remove the Function before commit;
4. commit successfully.

Verify that:

* the resulting Program contains neither the Function nor the Parameter;
* the transaction is not classified as a no-op;
* the provisional `FunctionId` becomes part of committed Function allocation history;
* the provisional `ParameterId` becomes part of committed Parameter allocation history;
* and later allocations do not reuse either committed identifier.

---

## AR-FUNC-031 — Unknown Module Function creation poisons transaction

Perform at least one successful transaction-local mutation.

Then attempt `add_function` using an unknown `ModuleId`.

Verify that:

* the operation fails;
* the transaction becomes Failed;
* commit is impossible;
* and earlier transaction-local changes do not become committed.

---

## AR-FUNC-032 — Removed Parameter becomes unavailable

Within one transaction:

1. remove an existing Parameter;
2. attempt another operation targeting that `ParameterId`.

Verify that the second operation fails and poisons the transaction.

---

## AR-FUNC-033 — Function and Parameter identifier categories are distinct

Demonstrate through Rust compile-time typing that:

```text
FunctionId
ParameterId
ProgramId
RevisionId
ModuleId
```

are distinct identifier categories where technically practical.

Accidental interchange between `FunctionId` and `ParameterId` SHOULD fail at compile time.

A compile-fail doctest or equivalent compile-time evidence MAY be used.

---

## AR-FUNC-034 — Structurally invalid Function state cannot commit

Demonstrate that a transaction candidate violating a Function or Parameter structural invariant cannot become committed Program state.

The test MAY use an internal test-only construction mechanism if the normal safe public API intentionally makes the invalid state impossible to construct.

Verify that:

* commit does not publish the invalid candidate;
* no new committed revision is produced;
* and the source committed revision remains unchanged.

This acceptance requirement MUST NOT require weakening the public mutation API merely to construct invalid state.

---

## AR-FUNC-035 — Function removal does not affect unrelated Functions

Create multiple Functions in the same Module.

Remove one Function and commit.

Verify that unrelated Functions preserve:

* `FunctionId`;
* Parameter membership and order;
* Parameter identities;
* types;
* return type;
* and presentation metadata.

---

# 28. Acceptance verification mechanisms

Acceptance requirements may be demonstrated through:

```text
automated runtime tests
compile-time/type-system evidence
documented conformance inspection
```

Runtime tests should be preferred for observable mutation and identity behavior.

Compile-time evidence should be preferred for typed identifier separation.

Conformance inspection is appropriate for negative architectural requirements where adding a public API purely for testing would increase implementation scope.

---

# 29. Implementation constraints

## MNIR-FUNC-068 — Core ownership

The Function and Parameter representation defined by this specification MUST belong to:

```text
mnir-core
```

---

## MNIR-FUNC-069 — Dependency direction

Implementing this specification MUST NOT cause `mnir-core` to depend on:

```text
mnir-verify
easyh-render
mnir-cli
```

---

## MNIR-FUNC-070 — Typed identifiers

The implementation SHOULD use distinct Rust types for:

```text
FunctionId
ParameterId
```

and SHOULD make accidental interchange with other identifier categories fail at compile time where practical.

---

## MNIR-FUNC-071 — Controlled mutation only

The public API MUST NOT expose unrestricted mutable access that allows callers to bypass Function or Parameter structural invariants.

---

## MNIR-FUNC-072 — Safe Rust

The implementation MUST NOT use or require `unsafe` Rust to satisfy this specification.

---

## MNIR-FUNC-073 — No speculative general node abstraction

This specification does not require a generic:

```text
Node
NodeId
EntityId
SemanticReference
```

abstraction.

The implementation MUST NOT introduce such an abstraction as normative MNIR semantics solely in anticipation of future specifications.

---

## MNIR-FUNC-074 — No semantic verifier

This specification extends structural validity only.

Implementation MUST NOT introduce `VerifiedProgram` or semantic Function verification merely to satisfy this increment.

---

# 30. Implementation scope summary

The expected conceptual scope is:

```text
Program
└── Module
    └── Function
        ├── FunctionId
        ├── preferred_name?
        ├── documentation?
        ├── return_type: IntrinsicType
        └── Parameters [ordered]
            └── Parameter
                ├── ParameterId
                ├── preferred_name?
                ├── documentation?
                └── type: IntrinsicType
```

```text
Module Function collection
    = identity-based, semantically unordered

Function Parameter collection
    = ordered, order is semantically significant
```

This distinction is intentional.

Together with Program-level controlled mutation for these entities.

No executable Function representation is part of this increment.

---

# 31. Target example

After implementation, MNIR must be able to represent the semantic Function signature conceptually equivalent to:

```text
add(a: Int32, b: Int32) -> Int32
```

with semantic identity equivalent to:

```text
ModuleId(M1)
└── FunctionId(F1)
    ├── preferred_name: "add"
    ├── return_type: Int32
    ├── ParameterId(P1)
    │   ├── preferred_name: "a"
    │   └── type: Int32
    └── ParameterId(P2)
        ├── preferred_name: "b"
        └── type: Int32
```

The textual representation above is informative.

It is not EasyH syntax and is not a serialization format.

---

# 32. Foundational invariants

The following is an informative summary:

```text
Functions are owned by Modules.

Parameters are owned by Functions.

FunctionId and ParameterId are Program-lineage-scoped identities.

Committed Function and Parameter IDs are never reused.

Parameter order is semantically significant.

Parameter identity is independent of Parameter position.

Function and Parameter names are presentation metadata.

Duplicate preferred names are allowed.

Parameters use current intrinsic types.

Functions have current intrinsic return types.

Changing a signature preserves Function identity.

Changing a Parameter type preserves Parameter identity.

Functions may have zero Parameters.

No Function bodies exist yet.

No invocation semantics exist yet.

No name-resolution or overloading semantics exist yet.

All changes use the existing Program mutation transaction model.
```
