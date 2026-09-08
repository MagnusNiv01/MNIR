# MNIR Specification — Function Calls and Sequencing Foundations

**Document:** `09-function-calls-and-sequencing-foundations.md`
**Specification status:** Draft
**Specification version:** 0.1
**Normative:** Yes

---

## 1. Purpose

This document introduces Function Calls and the first explicit intra-Block effect sequencing model in MNIR.

Specification version 0.1 defines:

* `Call` Expressions;
* direct `FunctionId` call targets;
* ordered Call arguments;
* Call result type derivation;
* recursive and cross-Module calls within one Program;
* one ordered `EffectSequence` per Block;
* explicit sequencing of effectful Calls;
* consistency between value dependencies and effect order;
* structural validation of effect sequencing;
* semantic verification of Call arguments;
* Semantic Verification rule set `V0_4`.

The target capability is to represent conceptually:

```text
caller(a: Int32) -> Int32 {
    x = call foo(a)
    y = call bar(x)
    return y
}
```

as semantic MNIR equivalent to:

```text
E1 = ParameterReference(a)
E2 = Call(foo, [E1])
E3 = Call(bar, [E2])

EffectSequence:
    E2
    E3

Return(E3)
```

The textual examples in this document are informative. They are neither EasyH nor serialized MNIR syntax.

This specification intentionally does not define:

* pure Function Calls;
* effect declarations;
* effect inference;
* Function purity;
* effect polymorphism;
* asynchronous Calls;
* concurrency;
* partial-order effects;
* effect tokens;
* exceptions;
* Call fault propagation;
* external/package Calls;
* dynamic dispatch;
* methods;
* closures;
* Function values;
* indirect Calls;
* variadic arguments;
* named arguments;
* default arguments;
* argument evaluation order beyond explicit effect sequencing;
* runtime execution;
* ABI or calling convention;
* EasyH Call syntax.

---

# 2. Normative language

Every declarative statement inside a numbered `MNIR-CALL-*` rule is normative unless explicitly stated otherwise.

Acceptance requirements use identifiers of the form:

```text
AR-CALL-NNN
```

All applicable acceptance requirements are mandatory for completion of this implementation increment.

Text outside numbered rules is informative unless explicitly stated otherwise.

---

# 3. Dependencies

This specification builds on specifications `01` through `08`.

It also follows the architectural direction documented by:

```text
ADR 0001 — Value Identity and Sequencing Direction
ADR 0002 — Explicit Effect Sequencing
```

The ADRs are informative architectural rationale. This specification is authoritative for the normative semantics introduced here.

---

# 4. Core sequencing model

## MNIR-CALL-001 — Pure Expression model remains unchanged

Existing pure Expressions MUST continue to use the semantic dependency-DAG model established by earlier specifications.

Their collection order MUST remain non-semantic.

---

## MNIR-CALL-002 — EffectSequence

Every Block MUST contain one ordered:

```text
EffectSequence
```

The EffectSequence contains zero or more `ExpressionId` references.

---

## MNIR-CALL-003 — EffectSequence order is semantic

The position of an `ExpressionId` inside a Block's EffectSequence MUST represent semantic effect order within that Block.

Unlike ordinary Expression collection order, EffectSequence order is intentionally semantic.

---

## MNIR-CALL-004 — No EffectSequence identity

This specification MUST NOT introduce:

```text
EffectSequenceId
EffectId
OperationId
```

solely to represent sequencing.

The EffectSequence is semantic state owned by the Block.

---

## MNIR-CALL-005 — Existing Blocks start with empty effect sequence

A Block containing no effectful Expressions has an empty EffectSequence.

Programs representable before this specification remain structurally compatible with an empty EffectSequence.

---

# 5. Function Call Expression

## MNIR-CALL-006 — Call is an Expression

A Function Call MUST be represented as an ordinary Expression using the existing:

```text
ExpressionId
```

identity model.

---

## MNIR-CALL-007 — No Call-specific identity

This specification MUST NOT introduce:

```text
CallId
InvocationId
OperationId
```

for direct Function Calls.

---

## MNIR-CALL-008 — Call semantic contents

A Call Expression MUST contain exactly:

```text
target: FunctionId
arguments: ordered sequence of ExpressionId
```

---

## MNIR-CALL-009 — Call target uses semantic identity

The target Function MUST be referenced by `FunctionId`.

Call target resolution MUST NOT depend on:

* preferred Function name;
* textual source name;
* Module name;
* collection position.

---

# 6. Call target domain

## MNIR-CALL-010 — Existing target Function

A structurally valid committed Call MUST target an existing Function in the same Program revision.

---

## MNIR-CALL-011 — Cross-Module Calls

A Call MAY target a Function owned by another Module in the same Program.

Module boundaries do not prevent direct Function calls within one Program.

---

## MNIR-CALL-012 — Cross-Program Calls excluded

Function Calls and Sequencing Foundations 0.1 MUST NOT define Calls to Functions owned by another Program.

Package and dependency semantics are deferred.

---

## MNIR-CALL-013 — Bodyless target Function

A Call MAY target a bodyless Function.

This specification does not assign runtime linkage or external-function meaning to a bodyless target.

---

## MNIR-CALL-014 — Recursive Call allowed

A Function MAY directly or indirectly call itself.

This specification defines no acyclicity requirement for the Function call graph.

Function-call recursion is distinct from the intra-Function CFG acyclicity defined by Conditional Control Flow 0.1.

---

# 7. Call argument ordering

## MNIR-CALL-015 — Arguments are ordered

Call arguments MUST form an ordered sequence.

Argument position is semantic.

---

## MNIR-CALL-016 — Parameter-position correspondence

Argument position `N` corresponds to Parameter position `N` of the target Function.

---

## MNIR-CALL-017 — Duplicate argument Expressions

The same `ExpressionId` MAY appear in multiple argument positions.

For example:

```text
Call(F, [E1, E1])
```

is structurally valid.

---

## MNIR-CALL-018 — No argument canonicalization

The implementation MUST NOT reorder, deduplicate, or otherwise canonicalize Call arguments.

---

# 8. Argument ownership

## MNIR-CALL-019 — Existing argument Expressions

Every argument `ExpressionId` MUST resolve to an existing Expression for a structurally valid Call.

---

## MNIR-CALL-020 — Argument Block ownership

Every Call argument Expression MUST belong to the same Block that owns the Call Expression.

Cross-Block Call arguments are structurally invalid in version 0.1.

---

## MNIR-CALL-021 — Argument references are semantic

Call argument references MUST use `ExpressionId`.

They MUST NOT be inferred from Expression collection order or presentation names.

---

## MNIR-CALL-131 — Call arguments are value dependencies

Every Call argument reference creates an Expression value-dependency edge from the Call Expression to the referenced argument Expression.

Conceptually:

```text
E3 = Call(F, [E1, E2])
```

means:

```text
E3 depends on E1
E3 depends on E2
```

These dependencies participate in the same Expression dependency graph used by arithmetic and comparison Expressions.

---

## MNIR-CALL-132 — Call dependencies participate in acyclicity

Call argument dependencies MUST satisfy all existing Expression dependency acyclicity requirements.

A Call Expression MUST NOT directly or indirectly depend on itself through:

* Call arguments;
* arithmetic operands;
* comparison operands;
* or a combination of those Expression families.

Using the same Expression in multiple argument positions does not by itself create a dependency cycle.

The target `FunctionId` is not an Expression dependency edge.

---

# 9. Call construction

## MNIR-CALL-022 — Call mutation operation

The controlled mutation API MUST provide an operation equivalent to:

```text
add_call_expression
```

---

## MNIR-CALL-023 — Call construction inputs

The operation MUST accept semantic inputs equivalent to:

```text
block_id: BlockId
target_function_id: FunctionId
arguments: ordered sequence of ExpressionId
```

The exact Rust API shape is implementation-defined.

---

## MNIR-CALL-024 — Call construction behavior

Call construction MUST:

* require an existing Block;
* require an existing target Function;
* require every argument Expression to exist;
* require every argument Expression to belong to the Block identified by `block_id`;
* allocate exactly one provisional `ExpressionId`;
* create one Call Expression;
* preserve argument order;
* expose the provisional `ExpressionId`.

---

## MNIR-CALL-133 — Call creation does not implicitly sequence the Call

Successful `add_call_expression` MUST NOT automatically modify the owning Block's EffectSequence.

The new Call MUST therefore be provisionally unsequenced immediately after successful Call construction.

Sequencing the Call requires an explicit EffectSequence mutation.

This separation is intentional: Call construction creates the semantic operation, while EffectSequence explicitly defines observable effect order.

---

## MNIR-CALL-025 — Arity and argument types do not gate structural construction

Call construction MUST NOT require:

* argument count to equal Parameter count;
* argument types to equal Parameter types.

Such conditions are semantic verification concerns.

---

## MNIR-CALL-026 — Invalid structural Call target poisons transaction

Call construction MUST fail and poison the transaction when:

* the Block is unknown;
* the target Function is unknown;
* an argument Expression is unknown;
* an argument Expression belongs to another Block.

---

# 10. Call Expression immutability

## MNIR-CALL-027 — Call target is immutable after creation

Version 0.1 defines no operation that retargets an existing Call Expression to another Function.

---

## MNIR-CALL-028 — Call arguments are immutable after creation

Version 0.1 defines no operation that changes, inserts, removes, or reorders arguments of an existing Call Expression.

---

# 11. Call result type

## MNIR-CALL-029 — Call result type

The derived intrinsic type of a structurally valid Call Expression MUST equal the declared return type of the target Function.

---

## MNIR-CALL-030 — Call type independent of argument validity

Call result type derivation MUST NOT depend on:

* argument count validity;
* argument semantic type validity.

A Call with semantically invalid arguments still derives the target Function's declared return type.

---

## MNIR-CALL-031 — Live target return-type derivation

If the target Function's return type changes while the Call retains the same `ExpressionId`, the derived Call Expression type MUST reflect the target Function's current return type in that Program revision or transaction working state.

---

## MNIR-CALL-032 — Unresolved Call target type inspection

If an Active transaction temporarily contains a Call whose target Function no longer exists, Call type inspection MUST report a typed/unavailable result rather than returning the former target return type as valid.

The exact Rust error representation is implementation-defined.

Read-only type inspection failure MUST NOT poison the transaction.

---

# 12. Target Function removal

## MNIR-CALL-033 — Removing a called Function may create temporary dangling Calls

Removing a Function targeted by existing Calls MAY temporarily leave dangling Call targets in an Active transaction.

The Function-removal operation itself MUST NOT poison the transaction solely because callers still exist.

---

## MNIR-CALL-034 — Dangling Call target cannot commit

A Program revision containing a Call whose target Function does not exist MUST NOT commit successfully.

---

## MNIR-CALL-035 — Repair paths for dangling target

Version 0.1 defines only the following repair paths after target Function removal:

* remove the Function body containing the Call;
* remove the caller Function;
* remove the caller's Module.

Version 0.1 defines no Call retargeting or individual Expression removal.

---

## MNIR-CALL-136 — Removing a target Module may create dangling Calls

Removing a Module that owns one or more Functions targeted by Calls elsewhere in the Program MAY temporarily leave those Calls with unresolved target `FunctionId` references in the Active transaction working state.

The Module-removal operation MUST NOT poison the transaction solely because such callers exist.

This behavior is equivalent to the direct target-Function removal behavior defined by `MNIR-CALL-033`.

---

## MNIR-CALL-137 — Dangling Calls after target Module removal cannot commit

A Program revision containing a Call whose target Function disappeared because its owning Module was removed MUST NOT commit successfully.

Structural validity may be restored using the existing repair paths:

* remove the Function body containing the dangling Call;
* remove the caller Function;
* remove the caller Module.

Version 0.1 defines no Call retargeting or individual Call removal.

If removal of the target Module also removes the caller through the same cascade, no dangling Call remains.

---

# 13. Call effectfulness

## MNIR-CALL-036 — All Calls are effectful in 0.1

Every Call Expression introduced by this specification MUST be classified as effectful for sequencing purposes.

No Function purity analysis or declaration is defined by this specification.

---

## MNIR-CALL-037 — Conservative effect assumption

A Call MUST NOT be treated as pure merely because:

* its result is unused;
* its target Function body appears to contain only pure Expressions;
* its target is bodyless;
* its preferred name suggests purity.

Purity requires a future Effects specification.

---

## MNIR-CALL-038 — Unit-returning Calls remain Expressions

A Call whose target Function returns `Unit` remains an ordinary value-producing Call Expression of type `Unit`.

It is still effectful and must be sequenced.

---

# 14. EffectSequence membership

## MNIR-CALL-039 — Every Call appears exactly once

Every Call Expression owned by a committed Block MUST appear exactly once in that Block's EffectSequence.

---

## MNIR-CALL-040 — Only Calls may appear in EffectSequence

Under Function Calls and Sequencing Foundations 0.1, every `ExpressionId` in EffectSequence MUST identify a Call Expression owned by that Block.

Pure Expressions MUST NOT appear in EffectSequence.

---

## MNIR-CALL-041 — Block-local sequence references

Every EffectSequence entry MUST reference an Expression owned by the same Block.

Cross-Block sequence references are structurally invalid.

---

## MNIR-CALL-042 — Duplicate sequence entry invalid

One Call Expression MUST NOT appear more than once in the same EffectSequence.

---

# 15. EffectSequence inspection

## MNIR-CALL-043 — Sequence inspection

The public read-only Block API MUST expose the Block's EffectSequence.

---

## MNIR-CALL-044 — Sequence order inspection

Consumers MUST be able to inspect EffectSequence entries in semantic order.

---

## MNIR-CALL-045 — Expression inspection distinguishes Calls

The public Expression inspection API MUST permit consumers to identify a Call Expression and inspect:

```text
target FunctionId
ordered argument ExpressionIds
```

---

# 16. EffectSequence mutation

## MNIR-CALL-046 — Set EffectSequence operation

The controlled mutation API MUST provide an operation equivalent to:

```text
set_effect_sequence
```

for replacing one Block's complete EffectSequence.

---

## MNIR-CALL-047 — Set EffectSequence inputs

The operation MUST accept:

```text
block_id: BlockId
ordered sequence of ExpressionId
```

The sequence supplied represents the complete intended effect order for that Block at the time of the operation.

---

## MNIR-CALL-048 — Unknown Block sequence target

`set_effect_sequence` MUST fail and poison the transaction when the target Block does not exist.

---

## MNIR-CALL-049 — Unknown sequence Expression

`set_effect_sequence` MUST fail and poison the transaction when a supplied `ExpressionId` does not exist.

---

## MNIR-CALL-050 — Foreign sequence Expression

`set_effect_sequence` MUST fail and poison the transaction when a supplied Expression belongs to another Block.

---

## MNIR-CALL-051 — Non-Call sequence entry

`set_effect_sequence` MUST fail and poison the transaction when a supplied Expression is not a Call Expression.

---

## MNIR-CALL-052 — Duplicate sequence entries

`set_effect_sequence` MUST fail and poison the transaction when the supplied sequence contains the same `ExpressionId` more than once.

---

## MNIR-CALL-053 — Complete sequence replacement

A successful `set_effect_sequence` operation MUST replace the previous sequence atomically.

No partially updated EffectSequence may remain.

---

# 17. Temporary sequence incompleteness

## MNIR-CALL-054 — Newly created Call may be temporarily unsequenced

An Active transaction MAY contain a newly created Call Expression that has not yet been inserted into EffectSequence.

This temporary working state does not poison the transaction.

---

## MNIR-CALL-055 — Incomplete sequence cannot commit

A candidate Program in which one or more Calls do not appear exactly once in their Block's EffectSequence MUST NOT commit successfully.

---

# 18. Value dependencies and effect order

## MNIR-CALL-056 — Value dependency precedes dependent Call

If Call Expression `A` transitively depends on Call Expression `B` through the Expression dependency graph, `B` MUST appear before `A` in the same Block's EffectSequence.

---

## MNIR-CALL-057 — Pure intermediate dependencies participate

The dependency relation in `MNIR-CALL-056` includes paths through pure Expressions.

For example:

```text
E1 = Call(F)
E2 = Add(E1, Int32Literal(1))
E3 = Call(G, [E2])
```

requires:

```text
EffectSequence:
    E1
    E3
```

---

## MNIR-CALL-058 — Effect order may constrain otherwise independent Calls

Two Calls with no value dependency MAY still be ordered by EffectSequence.

That order is semantic.

---

## MNIR-CALL-059 — Value/effect order conflict is invalid

An EffectSequence that orders a dependent Call before one of its transitive Call dependencies is structurally invalid.

---

## MNIR-CALL-060 — Sequence consistency is deterministic

EffectSequence consistency MUST be derived from:

* explicit Expression dependencies;
* explicit EffectSequence position.

It MUST NOT depend on Expression collection iteration order.

---

# 19. Terminator relationship

## MNIR-CALL-061 — EffectSequence precedes terminator

Within one Block, the semantic effect order is:

```text
EffectSequence
    ↓
Block Terminator
```

All sequenced Calls in the Block occur conceptually before the Block's Return or Branch terminator.

---

## MNIR-CALL-062 — Terminator may depend on Call result

A Return Expression or Branch condition MAY directly or indirectly depend on a Call Expression in the same Block.

The Call remains required to appear in EffectSequence.

---

## MNIR-CALL-063 — Unused Call remains semantically present

A Call Expression that is not referenced by another Expression or terminator still represents an effectful operation when it appears in EffectSequence.

Its lack of value consumers MUST NOT cause it to be ignored.

---

# 20. Calls across Blocks

## MNIR-CALL-064 — Call arguments remain Block-local

A Call in one Block MUST NOT use an Expression defined in another Block as an argument under version 0.1.

---

## MNIR-CALL-065 — CFG orders effects across Blocks

Observable effect order between Calls in different Blocks is determined by:

* the control-flow graph between Blocks;
* the EffectSequence inside each Block.

This specification introduces no cross-Block EffectSequence.

---

## MNIR-CALL-066 — Branch-dependent effects

Calls in different Branch successor Blocks are ordered only according to the CFG path that is taken.

This specification does not execute the Branch or determine which path is taken.

---

# 21. Structural validity extension

## MNIR-CALL-067 — Structurally valid Call

A Call Expression is structurally valid when:

1. it has exactly one valid `ExpressionId`;
2. it belongs to exactly one existing Block;
3. its target `FunctionId` resolves to an existing Function in the same Program;
4. every argument `ExpressionId` resolves to an Expression in the same Block;
5. argument order is preserved;
6. all existing Expression dependency rules remain satisfied.

Argument count and argument type compatibility are not structural validity.

---

## MNIR-CALL-068 — Structurally valid EffectSequence

A Block's EffectSequence is structurally valid when:

1. every entry resolves to a Call Expression in that Block;
2. every Call Expression in that Block appears exactly once;
3. no Call appears more than once;
4. sequence order satisfies all transitive Call value dependencies.

---

## MNIR-CALL-069 — Block structural validity extension

For Blocks containing Calls, existing Block structural validity MUST additionally require `MNIR-CALL-068`.

---

## MNIR-CALL-070 — Structural commit rejection

A candidate Program violating Call target, Call argument, EffectSequence membership, or sequence-order consistency MUST NOT commit.

---

# 22. Call argument semantic validity

## MNIR-CALL-071 — Valid Call arity

A semantically valid Call MUST have exactly the same number of arguments as the target Function has Parameters.

---

## MNIR-CALL-072 — Valid Call argument type

For every argument position that has a corresponding target Parameter, the argument Expression's successfully derived intrinsic type MUST equal the Parameter's intrinsic type.

---

## MNIR-CALL-073 — No implicit argument conversion

Call verification MUST NOT introduce implicit:

```text
Int32 → Int64
Int64 → Int32
```

or another type conversion.

---

## MNIR-CALL-074 — Semantically invalid Calls remain structurally representable

A Call with incorrect:

* argument count;
* argument types;

MAY exist in committed structurally valid MNIR.

---

# 23. Call argument type inspection

## MNIR-CALL-075 — Argument type unavailable

If an argument Expression does not have a successfully derivable intrinsic type, that argument is semantically unavailable for Call verification.

---

## MNIR-CALL-076 — Call result type remains derivable

Argument semantic failure MUST NOT make the Call Expression result type unavailable when the target Function itself is structurally resolvable.

The Call result type remains the target Function's declared return type.

---

# 24. Snapshots

## MNIR-CALL-077 — Snapshot Call preservation

A Program snapshot MUST preserve:

* Call `ExpressionId`;
* target `FunctionId`;
* ordered argument references;
* Block EffectSequence contents and order.

---

# 25. Forks

## MNIR-CALL-078 — Fork Call preservation

A fork MUST preserve Call and EffectSequence semantic contents.

---

## MNIR-CALL-079 — Fork reference-consistent remapping

If a fork remaps any:

```text
FunctionId
ExpressionId
BlockId
```

it MUST atomically update:

* Call target references;
* Call argument references;
* EffectSequence references;
* all previously defined Expression and CFG references.

---

# 26. Removal cascades

## MNIR-CALL-080 — Body removal removes Calls and sequence

Removing a Function body removes all descendant Call Expressions together with its Blocks and EffectSequences.

---

## MNIR-CALL-081 — Function and Module removal cascade

Existing Function and Module cascade semantics MUST transitively remove descendant Calls and EffectSequences.

---

## MNIR-CALL-138 — Block removal removes its EffectSequence

Removing a Block MUST remove:

* all Expressions owned by that Block;
* the Block's EffectSequence;
* and its terminator

as part of the same Block-removal cascade.

EffectSequence has no independent identity or allocation history.

Committed Call `ExpressionId` retirement continues to follow the existing Expression identity rules.

---

## MNIR-CALL-082 — Expression identity retirement

Committed Call `ExpressionId` values remain retired when Calls are removed through existing cascade semantics.

---

# 27. Program no-op semantics

## MNIR-CALL-083 — Call data participates in Program state

Program no-op comparison MUST include for every Call:

* target `FunctionId`;
* ordered argument `ExpressionId` sequence.

---

## MNIR-CALL-084 — EffectSequence participates in Program state

EffectSequence contents and order are semantic Program state.

Changing only EffectSequence order constitutes a Program state change.

---

# 28. Verification rule-set applicability

Function Calls introduce semantic constructs not understood by V0_1, V0_2, or V0_3.

---

## MNIR-CALL-085 — V0_1 through V0_3 applicability

`SemanticVerificationAndDiagnosticsV0_1`, `V0_2`, and `V0_3` are not applicable to a Program revision containing a Call Expression.

---

## MNIR-CALL-086 — Applicability scan is Program-wide

Older verification rule sets MUST detect Calls across the complete Program before beginning semantic verification.

---

## MNIR-CALL-087 — Applicability failure is not semantic diagnostic

Older rule-set applicability failure caused by Calls MUST:

* produce no semantic Diagnostic;
* produce no partial verification result;
* produce no `VerifiedProgram`.

---

# 29. Verification V0_4

## MNIR-CALL-088 — Verification V0_4

This specification introduces:

```text
SemanticVerificationAndDiagnosticsV0_4
```

---

## MNIR-CALL-089 — V0_4 coverage

V0_4 understands:

* all constructs understood by V0_3;
* Call Expressions;
* Call result type derivation;
* Call argument semantic verification;
* EffectSequence semantics defined by this specification.

---

## MNIR-CALL-090 — V0_4 inherits previous semantic verification

V0_4 MUST apply all applicable:

* arithmetic;
* comparison;
* Branch;
* Return;

semantic verification defined by earlier rule sets.

---

## MNIR-CALL-091 — Explicit V0_4 selection

The public verification API MUST permit explicit selection of V0_4.

---

## MNIR-CALL-092 — Older verification APIs remain stable

Existing fixed-rule-set verification API meanings MUST NOT silently change.

---

# 30. V0_4 VerifiedProgram

## MNIR-CALL-093 — V0_4 binding

A `VerifiedProgram` produced by successful V0_4 verification MUST report:

```text
SemanticVerificationAndDiagnosticsV0_4
```

as its authoritative rule set.

---

## MNIR-CALL-094 — V0_4 success requirements

A Program revision may produce a V0_4 `VerifiedProgram` only when:

* V0_4 is applicable;
* all inherited V0_3 semantic checks succeed;
* all Call argument verification rules succeed.

Structural EffectSequence validity is a prerequisite to verification input.

---

## MNIR-CALL-095 — Older verification evidence does not satisfy V0_4

V0_1, V0_2, or V0_3 verification evidence MUST NOT satisfy a requirement for V0_4 verification.

---

# 31. Call diagnostics

V0_4 introduces:

```text
MNIR-DIAG-011  CallArgumentCountMismatch
MNIR-DIAG-012  CallArgumentTypeUnavailable
MNIR-DIAG-013  CallArgumentTypeMismatch
```

---

## MNIR-CALL-096 — Call diagnostic severity

All Call diagnostics have severity:

```text
Error
```

---

## MNIR-CALL-097 — Call diagnostic primary subject

The primary semantic subject for:

```text
MNIR-DIAG-011
MNIR-DIAG-012
MNIR-DIAG-013
```

MUST be the affected Call `ExpressionId`.

---

## MNIR-CALL-134 — Diagnostic FunctionId identifies the Call target

For:

```text
MNIR-DIAG-011
MNIR-DIAG-012
MNIR-DIAG-013
```

the normative `function_id` payload field MUST identify the Function targeted by the affected Call Expression.

It does not identify the Function containing the Call.

The containing caller Function may be obtained separately through Program structure when required.

---

## MNIR-CALL-135 — Argument indices are zero-based

Every argument index exposed by Function Calls and Sequencing Foundations 0.1 is zero-based.

Therefore, for:

```text
Call(F, [E1, E2, E3])
```

the argument positions are:

```text
E1 → 0
E2 → 1
E3 → 2
```

This applies to:

```text
argument_index
argument_indices
```

and every mismatch or unavailable-type payload defined by this specification.

Indices refer to the semantic ordered argument sequence, not collection iteration order.

---

# 32. Argument count diagnostic

## MNIR-CALL-098 — CallArgumentCountMismatch

When argument count differs from target Parameter count, V0_4 MUST emit exactly one:

```text
MNIR-DIAG-011
```

for that Call.

---

## MNIR-CALL-099 — Argument count payload

`MNIR-DIAG-011` MUST expose:

```text
expression_id: ExpressionId
function_id: FunctionId
expected_count
actual_count
```

Counts MUST be machine-readable non-negative integers.

---

# 33. Argument unavailable diagnostic

## MNIR-CALL-100 — CallArgumentTypeUnavailable

For supplied argument positions that correspond to existing Parameters, V0_4 MUST identify every argument whose intrinsic type cannot be successfully derived.

If one or more such positions exist, V0_4 MUST emit exactly one:

```text
MNIR-DIAG-012
```

for the Call.

---

## MNIR-CALL-101 — Unavailable argument payload

`MNIR-DIAG-012` MUST expose:

```text
expression_id: ExpressionId
function_id: FunctionId
argument_indices: ordered sequence of non-negative indices
```

The indices MUST be strictly increasing and correspond to argument positions whose type is unavailable.

---

# 34. Argument mismatch diagnostic

## MNIR-CALL-102 — CallArgumentTypeMismatch

For supplied argument positions that correspond to existing Parameters, V0_4 MUST identify every position where:

* argument type is successfully derived;
* argument type differs from Parameter type.

If one or more mismatches exist, V0_4 MUST emit exactly one:

```text
MNIR-DIAG-013
```

for the Call.

---

## MNIR-CALL-103 — Mismatch payload

`MNIR-DIAG-013` MUST expose an ordered sequence of mismatch entries.

Each entry MUST contain:

```text
argument_index
expected_type: IntrinsicType
actual_type: IntrinsicType
```

Entries MUST appear in strictly increasing argument-index order.

The Diagnostic MUST also expose:

```text
expression_id: ExpressionId
function_id: FunctionId
```

---

# 35. Diagnostic coexistence

## MNIR-CALL-104 — Count and type diagnostics may coexist

A Call with incorrect argument count MAY also receive:

```text
MNIR-DIAG-012
MNIR-DIAG-013
```

for argument positions that have corresponding Parameters.

---

## MNIR-CALL-105 — Extra arguments have no expected type

Arguments beyond the target Function's Parameter count MUST NOT receive type-mismatch diagnostics because no corresponding Parameter type exists.

They remain represented by the count mismatch.

---

## MNIR-CALL-106 — Missing arguments have no argument-expression diagnostic

Missing argument positions MUST NOT produce:

```text
MNIR-DIAG-012
MNIR-DIAG-013
```

because no argument Expression exists.

They remain represented by the count mismatch.

---

## MNIR-CALL-107 — Unavailable and mismatch diagnostics may coexist

One Call MAY receive both:

```text
MNIR-DIAG-012
MNIR-DIAG-013
```

when different corresponding argument positions independently satisfy those categories.

---

## MNIR-CALL-139 — V0_4 diagnostic order is non-semantic

V0_4 diagnostic collection order remains non-semantic.

Consumers MUST NOT infer:

* execution order;
* argument order;
* Call order;
* EffectSequence order;
* severity priority;
* or causality

from diagnostic collection position.

Normative ordering inside diagnostic payloads, such as `argument_indices` and mismatch-entry sequences, remains semantic where explicitly specified.

---

# 36. Call verification scope

## MNIR-CALL-108 — Every Call is verified

V0_4 MUST semantically verify every committed Call Expression in every Block.

This applies whether or not its result contributes to the Block terminator.

---

## MNIR-CALL-109 — Unused Call still verified

A Call used only for its effect remains subject to all Call argument verification rules.

---

# 37. Interaction with Return and Branch verification

## MNIR-CALL-110 — Call result may be returned

A Return terminator MAY directly or indirectly depend on a Call result.

Existing Return verification uses the Call's derived target return type.

---

## MNIR-CALL-111 — Call result may be Branch condition

A Branch condition MAY directly or indirectly depend on a Call result.

Existing Branch verification uses the Call's derived target return type.

---

## MNIR-CALL-112 — Call argument errors do not suppress result-type use

Call argument semantic errors do not make the Call result type unavailable.

Therefore inherited Return, Branch, arithmetic, or comparison verification MAY independently produce diagnostics based on the target Function's declared return type.

---

# 38. No Call execution

## MNIR-CALL-113 — Verifier does not execute Calls

Semantic verification MUST NOT execute target Functions.

---

## MNIR-CALL-114 — No interprocedural behavior analysis

V0_4 MUST NOT infer Call effects, purity, runtime values, or faults by analyzing the target Function body.

---

## MNIR-CALL-115 — No constant Call evaluation

V0_4 MUST NOT constant-fold or evaluate Calls.

---

# 39. EffectSequence and verification

## MNIR-CALL-116 — EffectSequence errors are structural

EffectSequence membership, locality, uniqueness, completeness, and dependency-order consistency are structural validity requirements.

V0_4 MUST NOT introduce semantic Diagnostics as substitutes for those structural errors.

---

## MNIR-CALL-117 — V0_4 assumes structurally valid sequencing

Semantic verification operates only after EffectSequence structural validity has been established.

---

# 40. Sequence and Call purity evolution

## MNIR-CALL-118 — All Calls remain effectful under 0.1

V0_4 MUST treat all Calls as requiring EffectSequence membership.

It MUST NOT infer purity.

---

## MNIR-CALL-119 — Future Effects specification may supersede effectfulness

A future Effects specification MAY explicitly supersede the rule that all Calls are effectful.

No such purity semantics are introduced here.

---

# 41. Explicitly unresolved topics

The following remain unresolved:

* pure Calls;
* Function effect declarations;
* effect inference;
* effect polymorphism;
* effect tokens;
* partial-order effect dependencies;
* parallel Calls;
* async Calls;
* cross-Block values;
* block parameters;
* phi nodes;
* Call retargeting;
* Call argument mutation;
* individual Call removal;
* indirect Calls;
* Function values;
* methods;
* closures;
* cross-Program Calls;
* package Calls;
* external linkage;
* ABI;
* calling convention;
* exceptions;
* Call fault propagation;
* resource ownership;
* runtime Call stack;
* tail Calls;
* inlining;
* EasyH Call syntax;
* canonical serialization syntax.

---

## MNIR-CALL-120 — Unresolved topics are not 0.1 semantics

An implementation MUST NOT establish unresolved topics as normative Function Calls and Sequencing Foundations 0.1 semantics.

---

# 42. Specification gap handling

## MNIR-CALL-121 — Undefined Call or sequencing semantics

If implementation requires externally observable Function Call or EffectSequence behavior not defined by this specification, it MUST NOT invent normative MNIR semantics.

The missing behavior MUST be reported as a specification gap.

---

# 43. Acceptance requirements

This increment requires `AR-CALL-001 .. AR-CALL-059`.

## AR-CALL-001 — Basic Call construction

Create a Call to an existing Function with valid arguments.

Verify:

* Call `ExpressionId`;
* target `FunctionId`;
* ordered arguments;
* derived target return type.

---

## AR-CALL-002 — Zero-argument Call

Create a Call to a zero-Parameter Function and verify an empty argument sequence.

---

## AR-CALL-003 — Cross-Module Call

Call a Function owned by another Module in the same Program.

Verify structural success.

---

## AR-CALL-004 — Recursive Call

Create a direct recursive Call and verify no Function-call-cycle restriction rejects it.

---

## AR-CALL-005 — Bodyless target

Call an existing bodyless Function and verify structural representation succeeds.

---

## AR-CALL-006 — Unknown target rejected

Attempt Call construction with an unknown `FunctionId`.

Verify operation failure, transaction poisoning, and atomicity.

---

## AR-CALL-007 — Unknown argument rejected

Demonstrate unknown argument `ExpressionId` rejection.

---

## AR-CALL-008 — Cross-Block argument rejected

Demonstrate argument Expression from another Block is rejected.

---

## AR-CALL-009 — Argument order preserved

Create a Call with multiple arguments and verify exact stored order.

---

## AR-CALL-010 — Duplicate argument allowed

Create:

```text
Call(F, [E1, E1])
```

and verify structural validity.

---

## AR-CALL-011 — Call return type derivation

Verify Call derived type equals target Function return type.

---

## AR-CALL-012 — Target return-type mutation updates Call type

Commit a Call, later change the target Function return type, and verify the Call retains its `ExpressionId` while its derived type changes accordingly.

---

## AR-CALL-013 — Dangling target type inspection

Remove a called target Function during an Active transaction.

Verify:

* Call type inspection reports unresolved/unavailable target;
* inspection does not poison the transaction;
* commit fails unless structural validity is repaired.

---

## AR-CALL-014 — Empty EffectSequence for pure Block

Verify a Block containing no Calls has an empty EffectSequence and remains valid.

---

## AR-CALL-015 — Call initially may be unsequenced

Create a Call and verify the transaction remains Active before EffectSequence is completed.

---

## AR-CALL-016 — Unsequenced Call cannot commit

Attempt to commit a Block containing a Call absent from EffectSequence.

Verify atomic structural rejection.

---

## AR-CALL-017 — Valid EffectSequence

Create multiple Calls and set a complete valid EffectSequence.

Verify order is inspectable and committed.

---

## AR-CALL-018 — Duplicate sequence entry rejected

Verify `set_effect_sequence` rejects duplicate Call IDs and poisons the transaction.

---

## AR-CALL-019 — Non-Call sequence entry rejected

Attempt to sequence a pure Expression.

Verify rejection and poisoning.

---

## AR-CALL-020 — Unknown sequence Expression rejected

Verify unknown Expression rejection.

---

## AR-CALL-021 — Cross-Block sequence entry rejected

Verify a Call from another Block cannot appear in the sequence.

---

## AR-CALL-022 — Sequence replacement

Set one valid EffectSequence, then replace it with another valid ordering.

Verify replacement is atomic and final order is semantic state.

---

## AR-CALL-023 — Dependency-consistent sequencing

Construct:

```text
E1 = Call(F)
E2 = Call(G, [E1])
```

Verify:

```text
[E1, E2]
```

is valid.

---

## AR-CALL-024 — Dependency-conflicting sequencing rejected

Using the same dependencies, attempt:

```text
[E2, E1]
```

Verify rejection and transaction poisoning or structural commit rejection according to the public mutation path.

---

## AR-CALL-025 — Pure intermediate dependency sequencing

Construct:

```text
E1 = Call(F)
E2 = Add(E1, literal)
E3 = Call(G, [E2])
```

Verify E1 must precede E3 in EffectSequence.

---

## AR-CALL-026 — Independent Call order is semantic Program state

Create two Calls with no value dependency between them.

Commit one revision with:

```text
EffectSequence:
    E1
    E2
```

and another semantic state with:

```text
EffectSequence:
    E2
    E1
```

Verify that the two states differ in semantic Program contents because EffectSequence ordering differs.

The test MUST NOT establish the distinction merely by observing different `RevisionId` values.

---

## AR-CALL-027 — Unused Unit Call retained

Create a `Unit`-returning Call unused by any value consumer.

Sequence it and commit successfully.

Verify the Call remains present and semantically ordered.

---

## AR-CALL-028 — EffectSequence precedes Return

Return a value while another independent Unit Call exists in the same Block.

Verify the Call must remain sequenced and the terminator remains after EffectSequence conceptually.

No execution test is required.

---

## AR-CALL-029 — EffectSequence with Branch

Create Calls in a Branch source Block and successor Blocks.

Verify each Block has its own local EffectSequence and CFG provides inter-Block order.

---

## AR-CALL-030 — Snapshot preservation

Verify snapshot preservation of Calls, arguments, targets, and EffectSequence order.

---

## AR-CALL-031 — Fork preservation

Verify fork preservation and reference-consistent remapping/preservation.

---

## AR-CALL-032 — Removal cascade

Remove a Function body containing Calls and verify Calls and EffectSequence disappear while committed Expression IDs remain retired.

---

## AR-CALL-033 — Called Function removal produces dangling structural state

Remove a Function targeted by another Function's Call.

Verify the transaction remains Active but commit fails if the caller remains unchanged.

---

## AR-CALL-034 — Dangling target repair

Remove a called target Function and then remove the caller body or caller Function.

Verify successful commit.

---

## AR-CALL-035 — Valid arity verifies

Under V0_4, verify a Call with correct argument count and types produces no Call diagnostics.

---

## AR-CALL-036 — Count mismatch diagnostic

Verify `MNIR-DIAG-011` and its machine-readable payload.

---

## AR-CALL-037 — Argument unavailable diagnostic

Create a Call argument whose type is unavailable because of an underlying semantic Expression error.

Verify `MNIR-DIAG-012` with ordered argument indices.

---

## AR-CALL-038 — Argument mismatch diagnostic

Create multiple mismatching argument positions.

Verify one `MNIR-DIAG-013` containing all mismatches in increasing argument-index order.

---

## AR-CALL-039 — Count and type diagnostics coexist

Create a Call with incorrect count plus at least one corresponding argument type error.

Verify the applicable count and type diagnostics coexist.

---

## AR-CALL-040 — Unavailable and mismatch diagnostics coexist

Create different argument positions satisfying unavailable and mismatch categories.

Verify both diagnostics are emitted for the same Call.

---

## AR-CALL-041 — Extra arguments only count as count mismatch

Verify extra positions without corresponding Parameters do not create argument type mismatch diagnostics.

---

## AR-CALL-042 — Missing arguments only count as count mismatch

Verify absent argument positions do not create unavailable or mismatch argument diagnostics.

---

## AR-CALL-043 — Call result remains usable despite argument error

Create a Call with invalid arguments but a known `Int32` target return type.

Use the Call result in valid `Int32` arithmetic or Return typing.

Verify downstream type derivation uses `Int32` while Call argument diagnostics are still produced.

---

## AR-CALL-044 — V0_1 through V0_3 reject Call Programs as not applicable

For each older rule set, request verification of a Program containing a Call.

Verify:

* applicability failure;
* zero semantic diagnostics;
* no partial verification;
* no `VerifiedProgram`.

---

## AR-CALL-045 — Explicit V0_4 selection

Verify the public API can request V0_4.

---

## AR-CALL-046 — V0_4 binding

Verify a successful `VerifiedProgram` reports:

```text
SemanticVerificationAndDiagnosticsV0_4
```

---

## AR-CALL-047 — Older verification evidence does not satisfy V0_4

Verify V0_3 evidence is distinguishable from and does not satisfy V0_4 rule-set identity.

---

## AR-CALL-048 — Complete V0_4 traversal

Create Calls across multiple Functions, Blocks, and Modules.

Verify every Call is semantically checked.

---

## AR-CALL-049 — Calls are not executed

Demonstrate by conformance inspection that verifier and core introduce no Call evaluator or interpreter.

---

## AR-CALL-050 — No purity inference

Demonstrate by conformance inspection that all Calls are treated as effectful regardless of target implementation or preferred name.

---

## AR-CALL-051 — Structural corruption of EffectSequence rejected

Using internal test-only corruption, verify commit rejection for at least:

* missing Call entry;
* duplicate Call entry;
* non-Call entry;
* foreign Block entry;
* value/effect dependency-order conflict.

Do not weaken the safe public API.

---

## AR-CALL-052 — Existing conformance preserved

All acceptance suites from specifications `01` through `08` MUST continue to pass without semantic regression.

---

## AR-CALL-053 — Call arguments participate in the shared Expression dependency graph

Create:

```text
E1 = ParameterReference(P1)
E2 = Call(F, [E1])
E3 = Add(E2, E1)
```

Verify that the Expression dependency model recognizes:

```text
E2 depends on E1
E3 depends on E2 and E1
```

Using internal test-only corruption if necessary, construct a dependency cycle involving at least one Call argument edge and one arithmetic or comparison edge.

Verify the cyclic candidate cannot commit.

The safe public API MUST NOT be weakened for this test.

---

## AR-CALL-054 — Call diagnostic target identity and zero-based indices

Create Call verification failures that exercise:

```text
MNIR-DIAG-011
MNIR-DIAG-012
MNIR-DIAG-013
```

Verify:

* `function_id` identifies the target/callee Function;
* argument indices are zero-based;
* ordered index payloads use increasing zero-based positions;
* the caller Function is not substituted for the target Function in normative payload.

---

## AR-CALL-055 — Call construction leaves EffectSequence unchanged

Begin with a Block having a known EffectSequence.

Create a new Call Expression.

Verify immediately after creation that:

* the Call exists;
* the transaction remains Active;
* the existing EffectSequence is unchanged;
* the new Call has not been automatically appended.

Then explicitly update EffectSequence and verify normal commit behavior.

---

## AR-CALL-056 — Target Module removal creates repairable dangling Calls

Create:

* caller Function in Module A;
* target Function in Module B;
* committed Call from A to B.

In a later transaction remove Module B.

Verify:

* the transaction remains Active immediately after Module removal;
* the Call target is unresolved in the working state;
* Call type inspection reports target unavailability rather than stale return type;
* commit fails if caller state is left unchanged.

Then demonstrate successful repair by removing the caller body, caller Function, or caller Module before commit.

---

## AR-CALL-057 — Block removal removes EffectSequence

Create a non-entry Block containing sequenced Calls.

Remove that Block using an otherwise valid CFG repair sequence.

Verify that the removed Block's:

* Calls;
* EffectSequence;
* terminator

are absent from the resulting committed Program.

Verify committed Call `ExpressionId` values remain retired.

---

## AR-CALL-058 — V0_4 diagnostic collection order is non-semantic

Create multiple independent Call and inherited semantic errors.

Verify the required diagnostic set and normative payload contents without assuming collection iteration order.

Where a diagnostic contains an explicitly ordered payload such as argument indices, verify that payload ordering independently.

---

## AR-CALL-059 — EffectSequence ordering differs independently of revision identity

Construct or compare two Program semantic states whose relevant Block contents differ only by the ordering of independent Calls in EffectSequence.

Verify semantic state comparison recognizes the ordering difference based on EffectSequence contents.

Do not use `RevisionId` inequality as the evidence of semantic difference.

---

# 44. Implementation constraints

## MNIR-CALL-122 — Core ownership

Call Expression and EffectSequence representation MUST belong to:

```text
mnir-core
```

---

## MNIR-CALL-123 — Verification ownership

Call semantic verification and diagnostics MUST belong to:

```text
mnir-verify
```

---

## MNIR-CALL-124 — Existing Expression model extension

Call MUST extend the existing Expression model rather than create a parallel Call AST.

---

## MNIR-CALL-125 — Existing Block model extension

EffectSequence MUST extend the existing Block model.

---

## MNIR-CALL-126 — No speculative Effects framework

This specification MUST NOT introduce a general:

```text
Effect
EffectSet
EffectType
EffectToken
EffectCapability
```

framework solely to classify Calls.

All Calls are conservatively effectful in this increment.

---

## MNIR-CALL-127 — No generic operation identity

The implementation MUST NOT introduce a separate generic operation identity solely for effect sequencing.

---

## MNIR-CALL-128 — Dependency direction

Implementation MUST NOT cause:

```text
mnir-core → mnir-verify
```

dependency.

---

## MNIR-CALL-129 — Controlled mutation

The public API MUST preserve existing encapsulation and structural integrity.

---

## MNIR-CALL-130 — Safe Rust

Implementation MUST NOT use or require `unsafe`.

---

# 45. Expected conceptual representation

```text
Block B1
├── Expressions
│   ├── E1 = ParameterReference(P1)
│   ├── E2 = Call(Foo, [E1])
│   ├── E3 = Int32Literal(1)
│   ├── E4 = Add(E2, E3)
│   └── E5 = Call(Bar, [E4])
│
├── EffectSequence
│   ├── E2
│   └── E5
│
└── Return(E5)
```

The ordinary Expression collection remains semantically unordered.

The EffectSequence is semantically ordered.

Informative note:

```text
Call argument references participate in the same Expression dependency
graph as arithmetic and comparison operand references.

EffectSequence does not replace that dependency graph.

The two structures answer different semantic questions:

Expression dependency graph:
    which values are required by which computations?

EffectSequence:
    in what order must observable Calls occur within the Block?
```

---

# 46. Verification evolution

```text
V0_1
    arithmetic + single-Block Return

V0_2
    V0_1 + comparisons

V0_3
    V0_2 + conditional control flow

V0_4
    V0_3
    + Function Calls
    + Call argument verification
    + sequencing-aware Call support
```

V0_1 through V0_3 remain immutable verification claims.

---

# 47. Foundational invariants

```text
Call is an ordinary Expression.

Call uses ordinary ExpressionId.

Call target uses FunctionId.

Arguments are ordered ExpressionId references.

Call arguments are Block-local in 0.1.

Recursive Calls are allowed.

Cross-Module Calls inside one Program are allowed.

Cross-Program Calls are not defined yet.

Call result type is the target Function return type.

Argument errors do not change Call result type.

All Calls are conservatively effectful.

Every committed Call appears exactly once in its Block EffectSequence.

Only Calls appear in EffectSequence in 0.1.

EffectSequence order is semantic.

Expression collection order remains non-semantic.

Value dependencies must be consistent with effect order.

CFG orders effects between Blocks.

EffectSequence orders effects within a Block.

Calls may be unused as values and still execute semantically through EffectSequence.

EffectSequence validity is structural.

V0_4 verifies Call argument semantics.

Older rule sets reject Call Programs as not applicable.

No purity inference exists yet.

No general Effects framework exists yet.

No Call execution exists yet.
```
