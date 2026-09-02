# MNIR Specification — Conditional Control Flow

**Document:** `08-conditional-control-flow.md`
**Specification status:** Draft
**Specification version:** 0.1
**Normative:** Yes

---

## 1. Purpose

This document introduces the first explicit control-flow graph in MNIR.

Conditional Control Flow 0.1 extends Function bodies from exactly one Block to a finite acyclic graph of Blocks.

This specification introduces:

* multiple Blocks per Function body;
* one distinguished entry Block;
* identity-based Block collections;
* `Branch` terminators;
* true and false successor Blocks;
* Branch condition Expressions;
* CFG structural validity;
* CFG reachability;
* acyclic control flow;
* Block creation and removal;
* terminator replacement;
* semantic verification of Branch conditions;
* semantic verification of multiple Return terminators;
* Semantic Verification rule set `V0_3`.

The target capability is to represent a Function conceptually equivalent to:

```text
max(a: Int32, b: Int32) -> Int32 {
    if a > b {
        return a
    } else {
        return b
    }
}
```

This specification intentionally does not define:

* loops;
* cyclic control flow;
* `while`;
* `for`;
* `break`;
* `continue`;
* switch/match;
* fallthrough;
* local values;
* local variables;
* block parameters;
* phi nodes;
* Function calls;
* exceptions;
* logical AND/OR/NOT;
* short-circuit evaluation;
* path-sensitive analysis;
* constant branch folding;
* unreachable-code diagnostics;
* control-flow execution;
* EasyH syntax.

---

# 2. Normative language

Every declarative statement inside a numbered `MNIR-CFG-*` rule is normative unless explicitly stated otherwise.

Acceptance requirements use identifiers of the form:

```text
AR-CFG-NNN
```

All applicable acceptance requirements are mandatory for completion of this implementation increment.

Text outside numbered rules is informative unless explicitly stated otherwise.

---

# 3. Dependencies

This specification builds on specifications `01` through `07`.

It extends:

```text
FunctionBody
Block
BlockId
Return
Expression
ExpressionId
Program mutation transactions
Structural validation
Semantic verification
VerificationRuleSet
VerifiedProgram
Diagnostic
```

Existing semantics remain unchanged except where this specification explicitly extends Function-body and terminator semantics.

---

## MNIR-CFG-115 — Explicit supersession of earlier Function-body rules

When Conditional Control Flow 0.1 is applicable, the rules in this specification supersede earlier normative rules only where explicitly stated below.

Earlier rules that do not conflict with this specification remain applicable.

This specification MUST NOT be interpreted as implicitly discarding unrelated semantics from Expressions and Basic Function Bodies 0.1.

---

## MNIR-CFG-116 — Multiple-Block supersession

For implementations supporting Conditional Control Flow 0.1:

`MNIR-CFG-001` through `MNIR-CFG-008` supersede the single-Block restrictions established by:

```text
MNIR-EXPR-008
MNIR-EXPR-009
MNIR-EXPR-010
```

The earlier requirement that a Function body contain exactly one Block is therefore no longer applicable under Conditional Control Flow 0.1.

A single-Block Function body remains valid as the one-Block case of the new model.

---

## MNIR-CFG-117 — Terminator supersession

For implementations supporting Conditional Control Flow 0.1:

`MNIR-CFG-016` through `MNIR-CFG-033` supersede the single-Return-terminator restrictions established by:

```text
MNIR-EXPR-036
MNIR-EXPR-037
MNIR-EXPR-038
```

to the extent that those earlier rules require Return to be the only possible committed Block terminator.

Existing Return ownership and Expression-reference semantics remain applicable where they do not conflict with the Branch model.

---

## MNIR-CFG-118 — Function-body structural-validity supersession

`MNIR-CFG-073` and `MNIR-CFG-074` supersede the Function-body structural-validity definition established by:

```text
MNIR-EXPR-072
MNIR-EXPR-073
```

for Program revisions using Conditional Control Flow 0.1.

Expression-level structural rules from earlier specifications remain applicable.

---

## MNIR-CFG-119 — Control-flow exclusion supersession

`MNIR-CFG-001` through `MNIR-CFG-108` supersede `MNIR-EXPR-078` only for:

```text
multiple Blocks
Branch terminators
acyclic conditional control flow
```

The exclusions in `MNIR-EXPR-078` concerning constructs not introduced by this specification remain in force.

In particular, Conditional Control Flow 0.1 still does not introduce loops, exception edges, or other unresolved control-flow constructs.

---

# 4. Function body Block model

## MNIR-CFG-001 — Function body contains one or more Blocks

A Function body MUST contain one or more Blocks.

A Function body MUST NOT exist with zero Blocks.

---

## MNIR-CFG-002 — Entry Block

Every Function body MUST identify exactly one Block as its:

```text
entry Block
```

---

## MNIR-CFG-003 — Entry Block identity

The entry Block MUST be identified by its existing `BlockId`.

This specification MUST NOT introduce a separate entry-point identity.

---

## MNIR-CFG-004 — Entry Block belongs to body

The entry `BlockId` MUST identify a Block owned by that Function body.

---

## MNIR-CFG-005 — Body creation preserves existing behavior

Creating a Function body MUST initially create exactly one Block.

That Block becomes the Function body's entry Block.

The `BlockId` exposed by body creation is therefore the entry `BlockId`.

---

# 5. Block collection semantics

## MNIR-CFG-006 — Identity-based Block collection

A Function body's Blocks MUST form an identity-based collection.

Collection position MUST NOT determine Block identity.

---

## MNIR-CFG-007 — Block collection order is non-semantic

No semantic ordering exists between sibling Blocks.

Any exposed iteration order MUST NOT be interpreted as:

* control-flow order;
* execution order;
* source order;
* branch priority.

---

## MNIR-CFG-008 — BlockId identity rules remain unchanged

`BlockId` remains Program-lineage-scoped according to Expressions and Basic Function Bodies 0.1.

Committed Block identifiers MUST remain permanently retired after removal.

---

# 6. Additional Block construction

## MNIR-CFG-009 — Add Block operation

The controlled mutation API MUST provide an operation equivalent to:

```text
add_block(function_id)
```

for adding a Block to the existing Function body owned by the identified Function.

Because Function bodies have no separate semantic identifier in version 0.1, `FunctionId` is the semantic target used to identify the body.

The exact Rust API signature is implementation-defined.

---

## MNIR-CFG-010 — Add Block behavior

`add_block` MUST:

* require an existing `FunctionId`;
* require that the Function owns a Function body;
* allocate exactly one provisional `BlockId`;
* create one initially unterminated Block;
* add it to that Function body's Block collection;
* expose the provisional `BlockId`.

The new Block MUST NOT replace or alter the entry Block.

---

## MNIR-CFG-121 — Unknown Function add target

`add_block` MUST fail and poison the transaction when its `FunctionId` does not identify an existing Function in the transaction working state.

---

## MNIR-CFG-122 — Bodyless Function add target

`add_block` MUST fail and poison the transaction when the target Function exists but does not own a Function body.

The operation MUST NOT implicitly create a Function body.

Function-body creation remains the responsibility of the existing `create_function_body` operation.

---

## MNIR-CFG-011 — Additional Blocks are initially unreachable

A newly added non-entry Block MAY temporarily be unreachable while an Active mutation transaction is being constructed.

Such a working state does not poison the transaction.

The Block MUST become reachable or be removed before successful commit.

---

# 7. Block removal

## MNIR-CFG-012 — Remove Block operation

The controlled mutation API MUST provide an operation equivalent to:

```text
remove_block
```

for removing a non-entry Block.

---

## MNIR-CFG-013 — Entry Block cannot be individually removed

`remove_block` MUST fail and poison the transaction when applied to the Function body's entry Block.

The entry Block can only disappear through removal of the entire Function body, Function, or owning Module.

---

## MNIR-CFG-014 — Block removal cascade

Removing a Block MUST remove from the transaction working state:

* the Block;
* all Expressions owned by that Block;
* its terminator.

Committed Block and Expression identifiers remain retired.

---

## MNIR-CFG-015 — Incoming Branch references may temporarily dangle

Removing a non-entry Block that is still targeted by a Branch MAY temporarily create a structurally invalid working state.

The removal operation itself MUST NOT poison the transaction solely because incoming Branch references exist.

Structural validity MUST be restored before commit.

---

## MNIR-CFG-123 — Unknown Block removal target

`remove_block` MUST fail and poison the transaction when the supplied `BlockId` does not identify an existing Block in the transaction working state.

---

## MNIR-CFG-124 — Block removal target is identity-based

`remove_block` MUST identify the target through `BlockId`.

The operation MUST NOT depend on Block collection position, iteration order, or presentation metadata.

---

# 8. Terminator model

Conditional Control Flow 0.1 defines exactly two terminator kinds:

```text
Return
Branch
```

---

## MNIR-CFG-016 — Every committed Block has one terminator

Every Block in a committed Function body MUST contain exactly one terminator.

---

## MNIR-CFG-017 — Unterminated working Blocks

An Active transaction MAY temporarily contain an unterminated Block.

A Program containing such a Block MUST NOT commit successfully.

---

## MNIR-CFG-018 — Terminator is not an Expression

Neither `Return` nor `Branch` is an Expression.

Terminators MUST NOT receive `ExpressionId`.

---

## MNIR-CFG-019 — No TerminatorId

Conditional Control Flow 0.1 MUST NOT introduce:

```text
TerminatorId
StatementId
BranchId
ReturnId
```

---

# 9. Terminator representation

## MNIR-CFG-020 — Terminator kinds are inspectable

The public read-only API MUST permit consumers to distinguish:

```text
Return
Branch
```

terminators.

---

## MNIR-CFG-021 — Return data

A Return terminator contains exactly:

```text
expression: ExpressionId
```

---

## MNIR-CFG-022 — Branch data

A Branch terminator contains exactly:

```text
condition: ExpressionId
true_block: BlockId
false_block: BlockId
```

---

## MNIR-CFG-023 — Sufficient terminator inspection

The public API MUST expose sufficient semantic data to reconstruct the version 0.1 meaning of either terminator kind.

---

# 10. Return extension

## MNIR-CFG-024 — Return belongs to its Block

The Return Expression MUST belong to the same Block as the Return terminator.

Existing Return ownership rules remain applicable.

---

## MNIR-CFG-025 — Multiple Returns across body

Different Blocks in one Function body MAY each have a Return terminator.

---

## MNIR-CFG-026 — Return terminator replacement

The existing `set_return` operation MUST replace any existing terminator on the target Block.

Therefore `set_return` MAY replace:

* an existing Return;
* or an existing Branch.

The resulting Block contains exactly one Return terminator.

---

# 11. Branch terminator

## MNIR-CFG-027 — Set Branch operation

The controlled mutation API MUST provide an operation equivalent to:

```text
set_branch
```

---

## MNIR-CFG-028 — Set Branch inputs

`set_branch` MUST require:

```text
source BlockId
condition ExpressionId
true target BlockId
false target BlockId
```

---

## MNIR-CFG-029 — Branch condition ownership

The Branch condition Expression MUST exist and belong to the source Block containing the Branch terminator.

---

## MNIR-CFG-030 — Branch target ownership

Both Branch target Blocks MUST exist and belong to the same Function body as the source Block.

Cross-Function or cross-body Branch targets are structurally invalid.

---

## MNIR-CFG-031 — Branch target roles are semantic

The distinction between:

```text
true_block
false_block
```

MUST be preserved.

The targets MUST NOT be automatically reordered.

---

## MNIR-CFG-032 — Identical Branch targets

The same `BlockId` MAY be used as both:

```text
true_block
false_block
```

provided all other structural requirements are satisfied.

---

## MNIR-CFG-033 — Branch replacement

Calling `set_branch` on a Block with an existing terminator MUST replace the previous terminator.

The previous terminator has no independent identity to retire.

---

# 12. Branch construction failures

## MNIR-CFG-034 — Unknown source Block

`set_branch` MUST fail and poison the transaction when the source Block does not exist.

---

## MNIR-CFG-125 — Unknown Branch source is an operation failure

Failure under `MNIR-CFG-034` occurs before any terminator replacement or other Branch mutation becomes visible in the transaction working state.

The transaction MUST enter the Failed state and no partial Branch mutation may remain.

---

## MNIR-CFG-035 — Unknown condition Expression

`set_branch` MUST fail and poison the transaction when the condition Expression does not exist.

---

## MNIR-CFG-036 — Foreign condition Expression

`set_branch` MUST fail and poison the transaction when the condition Expression belongs to another Block.

---

## MNIR-CFG-037 — Unknown target Block

`set_branch` MUST fail and poison the transaction when either target Block does not exist.

---

## MNIR-CFG-038 — Foreign target Block

`set_branch` MUST fail and poison the transaction when either target belongs to another Function body.

---

# 13. Branch type semantics

## MNIR-CFG-039 — Valid Branch condition type

A semantically valid Branch condition MUST derive intrinsic type:

```text
Bool
```

---

## MNIR-CFG-040 — Non-Bool Branch condition

A structurally valid Branch MAY reference a condition Expression whose valid derived intrinsic type is not `Bool`.

Such a Program is semantically invalid, not structurally invalid.

---

## MNIR-CFG-041 — Unavailable Branch condition type

A structurally valid Branch MAY reference a condition Expression whose valid intrinsic type cannot be derived because of another semantic Expression error.

The Branch is semantically invalid under control-flow verification.

---

## MNIR-CFG-042 — Branch condition failure does not affect structure

Branch-condition semantic type invalidity MUST NOT by itself make a committed Program structurally invalid.

---

# 14. Control-flow graph edges

## MNIR-CFG-043 — Return has no successors

A Return terminator contributes zero outgoing CFG edges.

---

## MNIR-CFG-044 — Branch contributes two semantic edges

A Branch contributes:

```text
source → true_block
source → false_block
```

Both edges are semantically meaningful even when the two targets are identical.

---

## MNIR-CFG-045 — CFG edges derive only from terminators

Control-flow edges MUST be determined exclusively by Block terminators.

Block collection order MUST NOT create implicit control-flow edges.

---

## MNIR-CFG-046 — No fallthrough

Conditional Control Flow 0.1 defines no implicit fallthrough from one Block to another.

---

# 15. Entry reachability

## MNIR-CFG-047 — Entry is reachable

The entry Block is reachable by definition.

---

## MNIR-CFG-048 — Successor reachability

A Block is reachable when it is:

* the entry Block;
* or the target of a Branch from another reachable Block.

---

## MNIR-CFG-049 — All committed Blocks must be reachable

Every Block in a committed Function body MUST be reachable from the entry Block.

---

## MNIR-CFG-050 — Unreachable working state allowed temporarily

An Active mutation transaction MAY contain unreachable Blocks.

Commit MUST fail unless every remaining Block is reachable.

---

# 16. Acyclic control flow

## MNIR-CFG-051 — CFG is acyclic

The committed control-flow graph MUST be acyclic.

---

## MNIR-CFG-052 — Self Branch is cyclic

A Branch whose source Block is also one of its targets creates a control-flow cycle and MUST NOT commit.

---

## MNIR-CFG-053 — Indirect cycles rejected

A cycle involving multiple Blocks MUST NOT commit.

Example:

```text
B1 → B2
B2 → B3
B3 → B1
```

is structurally invalid.

---

## MNIR-CFG-054 — Loops are deferred

The acyclic requirement is intentional.

Loops require a future specification that explicitly defines cyclic CFG semantics.

---

# 17. Termination property

## MNIR-CFG-055 — Every committed path terminates in Return

Because:

* the CFG is finite;
* every Block has one terminator;
* Branch targets remain inside the body;
* and the CFG is acyclic;

every path beginning at the entry Block MUST eventually terminate in a Return Block.

---

# 18. Structural versus semantic control flow

## MNIR-CFG-056 — Branch condition type is semantic

The intrinsic type of a Branch condition is not part of CFG structural validity.

---

## MNIR-CFG-057 — CFG shape is structural

The following are structural properties:

* entry Block exists;
* Branch targets exist;
* Branch targets belong to the body;
* Branch condition belongs to source Block;
* every Block has one terminator;
* all Blocks are reachable;
* CFG is acyclic.

---

# 19. Function-body inspection

## MNIR-CFG-058 — Entry Block inspection

The public read-only API MUST expose the Function body's entry `BlockId`.

---

## MNIR-CFG-059 — Block lookup

The public read-only API MUST permit Block lookup by `BlockId` within a Function body.

---

## MNIR-CFG-060 — Block enumeration

The public read-only API MUST permit enumeration of all Blocks in a Function body.

Enumeration order is non-semantic.

---

## MNIR-CFG-120 — Previous body Block inspection maps to entry Block

Where an existing read-only API or semantic concept from Expressions and Basic Function Bodies 0.1 exposes the singular Block of a Function body, that concept refers to the Function body's entry Block under Conditional Control Flow 0.1.

New control-flow-aware APIs MUST additionally permit inspection of all Blocks according to `MNIR-CFG-058` through `MNIR-CFG-060`.

The existence of a singular entry-Block view MUST NOT imply that the body contains only one Block.

---

# 20. Existing single-Block bodies

## MNIR-CFG-061 — Existing single-Block representation remains valid

A Function body containing exactly its entry Block and one Return terminator remains structurally valid.

---

## MNIR-CFG-062 — Single-Block semantic meaning preserved

Conditional Control Flow 0.1 MUST NOT alter the semantic meaning of valid Function bodies previously representable under Expressions and Basic Function Bodies 0.1.

---

# 21. Removal cascades

## MNIR-CFG-063 — Body removal

Removing a Function body MUST remove all Blocks, Expressions, and terminators owned by that body.

---

## MNIR-CFG-064 — Function removal

Function removal MUST transitively remove all body Blocks and their contents.

---

## MNIR-CFG-065 — Module removal

Module removal MUST transitively remove all descendant Function bodies and Blocks.

---

## MNIR-CFG-066 — Identifier retirement

Committed removed `BlockId` and `ExpressionId` values remain retired according to existing identity rules.

---

# 22. Provisional Blocks

## MNIR-CFG-067 — Provisional Block uniqueness

A provisional Block allocated in an Active transaction MUST NOT collide with:

* committed Block history;
* another provisional Block in that transaction.

---

## MNIR-CFG-068 — Successful provisional history

Every `BlockId` allocated during a transaction that commits successfully becomes part of committed Block allocation history even when that Block was removed before commit.

---

# 23. Snapshots

## MNIR-CFG-069 — Snapshot CFG preservation

A Program snapshot MUST preserve:

* Function-body Block membership;
* entry Block identity;
* terminator kind;
* Return Expression references;
* Branch condition references;
* Branch target references.

---

# 24. Forks

## MNIR-CFG-070 — Fork CFG preservation

A fork MUST preserve the semantic control-flow graph of the source snapshot.

---

## MNIR-CFG-071 — Reference-consistent remapping

If a fork remaps:

```text
BlockId
ExpressionId
```

then all affected:

```text
entry references
Return references
Branch condition references
Branch target references
Expression dependencies
```

MUST be updated consistently and atomically.

---

# 25. No-op comparison

## MNIR-CFG-072 — CFG participates in Program state

The Program no-op comparison MUST include:

* Block membership;
* entry Block identity;
* terminator kind;
* Return Expression reference;
* Branch condition;
* true target;
* false target;
* committed Block allocation history.

---

# 26. Structural validation

## MNIR-CFG-073 — Structurally valid Function body

A Function body is structurally valid under Conditional Control Flow 0.1 when:

1. it contains at least one Block;
2. exactly one owned Block is designated entry;
3. every Block has exactly one terminator;
4. every Return references an Expression in its Block;
5. every Branch condition references an Expression in its source Block;
6. every Branch target exists in the same Function body;
7. every Block is reachable from entry;
8. the CFG is acyclic;
9. all applicable Expression structural rules remain satisfied.

---

## MNIR-CFG-074 — Structural commit rejection

A candidate Program violating `MNIR-CFG-073` MUST NOT commit.

Commit failure preserves existing Program transaction atomicity.

---

## MNIR-CFG-131 — Invalid entry reference is structural corruption

A Function body whose entry `BlockId`:

* does not resolve to an existing owned Block;
* resolves to a Block owned by another Function body;
* or otherwise fails `MNIR-CFG-004`;

is structurally invalid.

Such a candidate MUST NOT commit.

---

## MNIR-CFG-132 — Invalid committed Branch references are structural corruption

A candidate Program containing a Branch whose:

* condition Expression is absent;
* condition Expression belongs to another Block;
* true target is absent;
* false target is absent;
* or either target belongs to another Function body;

is structurally invalid and MUST NOT commit.

This rule applies independently of whether the invalid state was created through internal corruption or another non-public mechanism.

The safe public mutation API MUST remain stricter and reject invalid Branch construction before commit.

---

# 27. Verification rule-set compatibility

Conditional Control Flow introduces semantic constructs not understood by V0_1 or V0_2 when a Function body contains multiple Blocks or a Branch terminator.

---

## MNIR-CFG-075 — V0_1 control-flow applicability

`SemanticVerificationAndDiagnosticsV0_1` is not applicable to a Program revision containing:

* a Branch terminator;
* or a Function body containing more than one Block.

---

## MNIR-CFG-076 — V0_2 control-flow applicability

`SemanticVerificationAndDiagnosticsV0_2` is not applicable to a Program revision containing:

* a Branch terminator;
* or a Function body containing more than one Block.

---

## MNIR-CFG-077 — Older rule-set applicability failure

When V0_1 or V0_2 encounters unsupported conditional-control-flow constructs, verification MUST fail with the existing rule-set applicability mechanism.

No semantic Diagnostic or partial `VerifiedProgram` may be produced.

---

## MNIR-CFG-129 — V0_1 and V0_2 applicability scan is Program-wide

Before V0_1 or V0_2 performs semantic verification, rule-set applicability MUST be determined across the complete Program revision.

The applicability scan MUST discover unsupported Conditional Control Flow constructs in every Module and Function.

If any Function body contains:

* more than one Block;
* or a Branch terminator;

the requested V0_1 or V0_2 verification run MUST fail as rule-set not applicable before semantic verification begins.

---

## MNIR-CFG-130 — No partial old-rule-set diagnostics after applicability failure

When the Program-wide applicability scan defined by `MNIR-CFG-129` fails:

* no semantic Diagnostic from that verification run may be produced;
* no partial semantic verification result may be exposed;
* no `VerifiedProgram` may be produced.

---

# 28. Verification V0_3

## MNIR-CFG-078 — Verification V0_3

This specification introduces:

```text
SemanticVerificationAndDiagnosticsV0_3
```

---

## MNIR-CFG-079 — V0_3 support

V0_3 understands:

* all constructs supported by V0_2;
* multiple Blocks;
* Branch terminators;
* Branch condition type verification;
* multiple Return terminators.

---

## MNIR-CFG-080 — V0_3 retains existing Expression verification

V0_3 MUST apply all applicable arithmetic and comparison Expression semantic verification rules.

---

## MNIR-CFG-081 — Explicit rule-set selection

The public verification API MUST permit explicit selection of V0_3.

---

## MNIR-CFG-082 — Existing convenience verify remains stable

Existing APIs with fixed V0_1 meaning MUST NOT silently change to V0_3.

---

## MNIR-CFG-126 — V0_3 VerifiedProgram rule-set binding

A `VerifiedProgram` produced by successful V0_3 verification MUST report:

```text
SemanticVerificationAndDiagnosticsV0_3
```

as its authoritative `VerificationRuleSet`.

---

## MNIR-CFG-127 — V0_3 success requirements

A Program revision MAY produce a V0_3 `VerifiedProgram` only when:

* the requested rule set is applicable to the complete Program revision;
* every semantic rule inherited from V0_2 succeeds;
* every Branch-condition verification rule defined by this specification succeeds;
* every applicable Return verification rule defined by this specification succeeds.

Any applicable Error diagnostic prevents V0_3 `VerifiedProgram` creation.

---

## MNIR-CFG-128 — Older verification evidence does not satisfy V0_3

A `VerifiedProgram` produced under:

```text
SemanticVerificationAndDiagnosticsV0_1
```

or:

```text
SemanticVerificationAndDiagnosticsV0_2
```

MUST NOT satisfy an API or semantic requirement for V0_3 verification evidence.

The `VerificationRuleSet` exposed by `VerifiedProgram` is authoritative.

---

# 29. Branch diagnostics

V0_3 introduces:

```text
MNIR-DIAG-008 BranchConditionTypeUnavailable
MNIR-DIAG-009 BranchConditionNotBool
```

---

## MNIR-CFG-083 — Branch diagnostics have Error severity

Both Branch diagnostics have severity:

```text
Error
```

---

## MNIR-CFG-084 — Branch diagnostic primary subject

The primary semantic subject for both Branch diagnostics MUST be:

```text
BlockId
```

of the Block containing the Branch.

---

## MNIR-CFG-085 — BranchConditionTypeUnavailable

When the Branch condition Expression does not have a valid derivable intrinsic type, V0_3 MUST emit exactly one:

```text
MNIR-DIAG-008
```

for that Branch Block.

---

## MNIR-CFG-086 — Unavailable Branch payload

`MNIR-DIAG-008` MUST expose at minimum:

```text
block_id: BlockId
condition_expression_id: ExpressionId
```

---

## MNIR-CFG-087 — BranchConditionNotBool

When the Branch condition derives a valid intrinsic type other than:

```text
Bool
```

V0_3 MUST emit exactly one:

```text
MNIR-DIAG-009
```

---

## MNIR-CFG-088 — Non-Bool Branch payload

`MNIR-DIAG-009` MUST expose:

```text
block_id: BlockId
condition_expression_id: ExpressionId
actual_type: IntrinsicType
```

---

## MNIR-CFG-089 — Branch diagnostic exclusivity

One Branch MUST NOT receive both:

```text
MNIR-DIAG-008
MNIR-DIAG-009
```

during the same verification run.

---

# 30. Return verification in V0_3

Existing `MNIR-DIAG-004` was defined for the single-Block Function-body model and uses `FunctionId` as its primary subject.

Multiple Return terminators require Block-scoped diagnostics.

V0_3 therefore introduces:

```text
MNIR-DIAG-010 ControlFlowReturnTypeMismatch
```

for Return mismatches in multi-Block bodies.

---

## MNIR-CFG-090 — Single-Block Return diagnostic compatibility

For a Function body containing exactly one Block, V0_3 MUST preserve the existing:

```text
MNIR-DIAG-004 ReturnTypeMismatch
```

semantics.

---

## MNIR-CFG-091 — Multi-Block Return mismatch

For a Function body containing more than one Block, each Return terminator whose Return Expression derives a valid intrinsic type different from the Function return type MUST produce exactly one:

```text
MNIR-DIAG-010
```

---

## MNIR-CFG-092 — Multi-Block Return diagnostic subject

The primary semantic subject of `MNIR-DIAG-010` MUST be the Return Block's:

```text
BlockId
```

---

## MNIR-CFG-093 — Multi-Block Return mismatch payload

`MNIR-DIAG-010` MUST expose:

```text
function_id: FunctionId
block_id: BlockId
return_expression_id: ExpressionId
expected_type: IntrinsicType
actual_type: IntrinsicType
```

---

## MNIR-CFG-094 — Unavailable Return type

If a Return Expression does not have a valid derived intrinsic type, V0_3 MUST NOT emit:

```text
MNIR-DIAG-004
MNIR-DIAG-010
```

for that Return.

Underlying Expression diagnostics remain responsible for the semantic failure.

---

# 31. Verification coverage

## MNIR-CFG-095 — Every reachable Block is verified

V0_3 MUST verify every Block in the committed Function body.

All committed Blocks are structurally reachable.

---

## MNIR-CFG-096 — Every Branch is verified

Every Branch terminator MUST receive Branch-condition semantic verification.

---

## MNIR-CFG-097 — Every Return is verified

Every Return terminator MUST receive Return-type semantic verification.

---

## MNIR-CFG-098 — All Expressions remain verified

All arithmetic and comparison Expressions in every Block remain subject to their existing semantic verification rules, regardless of whether they directly contribute to the terminator.

---

# 32. Diagnostic ordering

## MNIR-CFG-099 — Control-flow diagnostics are unordered

Diagnostic collection order remains non-semantic.

---

## MNIR-CFG-100 — Duplicate prevention

V0_3 MUST prevent duplicate diagnostics using existing:

```text
diagnostic code
primary semantic subject
```

semantics.

---

# 33. No execution

## MNIR-CFG-101 — Branches are not executed during verification

Semantic verification MUST NOT evaluate Branch conditions or choose a control-flow path.

---

## MNIR-CFG-102 — No reachability through condition values

Reachability is structural.

The verifier MUST NOT classify one Branch successor as unreachable by evaluating or constant-folding the condition.

---

# 34. No loops

## MNIR-CFG-103 — Loop semantics excluded

Conditional Control Flow 0.1 MUST NOT permit cyclic committed CFGs.

---

## MNIR-CFG-104 — No loop constructs

This specification MUST NOT introduce:

```text
Loop
While
For
Break
Continue
```

as MNIR semantics.

---

# 35. No locals or block parameters

## MNIR-CFG-105 — Local values excluded

Conditional Control Flow 0.1 MUST NOT introduce local bindings or local variables.

---

## MNIR-CFG-106 — Block parameters excluded

Conditional Control Flow 0.1 MUST NOT introduce:

```text
BlockParameter
Phi
MergeValue
```

or equivalent value-merging semantics.

---

# 36. Specification gaps

## MNIR-CFG-107 — Undefined control-flow semantics

If implementation requires externally observable control-flow behavior not defined by this specification, it MUST NOT establish that behavior as normative MNIR semantics.

The missing behavior MUST be reported as a specification gap.

---

# 37. Explicitly unresolved topics

The following topics are intentionally unresolved:

* loops;
* cyclic CFGs;
* logical AND/OR/NOT;
* short-circuit behavior;
* local bindings;
* local variables;
* mutable state;
* block parameters;
* phi nodes;
* value merging;
* Function calls;
* switch;
* match;
* fallthrough;
* break;
* continue;
* exception flow;
* unreachable-code diagnostics;
* path-sensitive semantic verification;
* branch probability;
* branch evaluation;
* constant-condition folding;
* CFG optimization;
* Block ordering;
* EasyH control-flow syntax;
* serialized CFG syntax.

---

## MNIR-CFG-108 — Unresolved topics are not 0.1 semantics

An implementation MUST NOT establish unresolved topics as normative Conditional Control Flow 0.1 semantics.

---

# 38. Acceptance requirements

This implementation increment covers:

```text
AR-CFG-001 .. AR-CFG-048
```

## AR-CFG-001 — Existing single-Block body remains valid

Verify an existing single-Block Return Function remains valid and preserves its semantics.

---

## AR-CFG-002 — Add additional Blocks

Create a Function body and add two additional Blocks.

Verify distinct provisional and committed `BlockId` identities.

---

## AR-CFG-003 — Entry Block

Verify body creation identifies exactly one entry Block and additional Blocks do not replace it.

---

## AR-CFG-004 — Block collection order is non-semantic

Demonstrate Blocks are addressed by identity and no collection order becomes CFG semantics.

---

## AR-CFG-005 — Return replacement of Branch

Create a Branch terminator, replace it with Return, and verify exactly one Return remains.

---

## AR-CFG-006 — Branch replacement of Return

Create Return, replace it with Branch, and verify exactly one Branch remains.

---

## AR-CFG-007 — Valid Branch

Create a Branch with:

```text
Bool condition
true target
false target
```

Verify stored semantic references.

---

## AR-CFG-008 — Same true and false target

Verify a Branch may use the same Block as both targets without that fact alone making the CFG cyclic.

---

## AR-CFG-009 — Unknown Branch condition rejected

Verify operation failure, poisoning, and atomicity.

---

## AR-CFG-010 — Foreign Branch condition rejected

Verify a condition Expression from another Block is rejected.

---

## AR-CFG-011 — Unknown Branch target rejected

Demonstrate both true and false unknown target failures.

---

## AR-CFG-012 — Foreign Branch target rejected

Demonstrate cross-Function body target rejection.

---

## AR-CFG-013 — Unterminated Block cannot commit

Create an additional unterminated Block and verify structural commit failure.

---

## AR-CFG-014 — Unreachable Block cannot commit

Create a terminated but unreachable Block and verify structural commit failure.

---

## AR-CFG-015 — Direct CFG cycle rejected

Attempt to commit a self-targeting Branch and verify structural rejection.

---

## AR-CFG-016 — Indirect CFG cycle rejected

Construct an internal multi-Block cycle and verify atomic commit rejection.

---

## AR-CFG-017 — Valid acyclic CFG

Construct:

```text
B1 Branch → B2 / B3
B2 Return
B3 Return
```

and verify structural commit succeeds.

---

## AR-CFG-018 — Block removal creates temporary dangling Branch

Remove a targeted non-entry Block.

Verify the transaction remains Active.

Commit fails unless Branch references are repaired or the affected structure is removed.

---

## AR-CFG-019 — Branch repair after Block removal

Remove a target Block and repair the source terminator using `set_branch` or `set_return`.

Verify successful commit.

---

## AR-CFG-020 — Entry Block removal rejected

Verify `remove_block(entry)` fails and poisons the transaction.

---

## AR-CFG-021 — Block identity retirement

Commit, remove, and later allocate Blocks.

Verify committed Block identities are not reused.

---

## AR-CFG-022 — Snapshot CFG preservation

Verify entry, Block membership, terminators, and Branch references survive snapshot creation.

---

## AR-CFG-023 — Fork CFG preservation

Verify fork semantic preservation and reference consistency.

---

## AR-CFG-024 — V0_1 rejects control-flow Program as not applicable

Request V0_1 verification of a Program containing Branch/multiple Blocks.

Verify applicability failure and zero semantic diagnostics.

---

## AR-CFG-025 — V0_2 rejects control-flow Program as not applicable

Demonstrate equivalent V0_2 behavior.

---

## AR-CFG-026 — Explicit V0_3 selection

Verify the public API can request:

```text
SemanticVerificationAndDiagnosticsV0_3
```

---

## AR-CFG-027 — Valid Bool Branch verifies

Under V0_3, verify a valid Bool condition produces no Branch diagnostic.

---

## AR-CFG-028 — Non-Bool Branch diagnostic

Use an `Int32` Branch condition.

Verify exactly one:

```text
MNIR-DIAG-009
```

with required payload.

---

## AR-CFG-029 — Unavailable Branch diagnostic

Use a Branch condition whose Expression type is unavailable because of an underlying arithmetic or comparison error.

Verify:

* underlying Expression diagnostic;
* exactly one `MNIR-DIAG-008`;
* no `MNIR-DIAG-009`.

---

## AR-CFG-030 — Multiple valid Returns

Create multiple Blocks returning the Function's declared type.

Verify V0_3 semantic success.

---

## AR-CFG-031 — Multi-Block Return mismatch

Create one mismatching Return Block.

Verify exactly one:

```text
MNIR-DIAG-010
```

with required payload.

---

## AR-CFG-032 — Multiple Return mismatches

Create multiple Return Blocks with mismatching derived types.

Verify one `MNIR-DIAG-010` per affected Block.

---

## AR-CFG-033 — Unavailable Return suppresses mismatch

Return an Expression with unavailable type.

Verify underlying Expression diagnostic and no Return mismatch diagnostic for that Block.

---

## AR-CFG-034 — Single-Block V0_3 compatibility

Verify a single-Block Return mismatch under V0_3 still produces:

```text
MNIR-DIAG-004
```

rather than `MNIR-DIAG-010`.

---

## AR-CFG-035 — Target max Function

Construct conceptually:

```text
max(a: Int32, b: Int32) -> Int32 {
    if a > b {
        return a
    } else {
        return b
    }
}
```

Verify structural commit and V0_3 semantic success.

---

## AR-CFG-036 — Complete traversal

Create multiple Functions containing multi-Block CFGs across multiple Modules.

Verify V0_3 discovers and checks every Branch, Return, arithmetic Expression, and comparison Expression.

---

## AR-CFG-037 — No branch evaluation

Use a literal `true` or `false` condition.

Verify structural/verifier behavior does not remove or ignore either successor based on constant value.

---

## AR-CFG-038 — Read-only CFG inspection

Verify public APIs expose:

* entry Block;
* all Blocks;
* terminator kind;
* Return Expression;
* Branch condition;
* true target;
* false target.

---

## AR-CFG-039 — Provisional removed Block history

Allocate and remove a provisional Block before successful commit.

Verify its identity enters committed allocation history and is not reused.

---

## AR-CFG-040 — Existing conformance preserved

All acceptance suites from specifications `01` through `07` MUST continue to pass without semantic regression.

---

## AR-CFG-041 — add_block target failures

Demonstrate both:

### Unknown Function

Call `add_block` using an unknown `FunctionId`.

Verify:

* operation failure;
* transaction poisoning;
* atomic non-commit.

### Bodyless Function

Call `add_block` using an existing Function without a body.

Verify:

* operation failure;
* transaction poisoning;
* no implicit Function-body creation.

---

## AR-CFG-042 — Unknown Block removal fails

Call `remove_block` with an unknown `BlockId`.

Verify:

* operation failure;
* transaction poisoning;
* no committed state change.

---

## AR-CFG-043 — Unknown Branch source fails atomically

Call `set_branch` using an unknown source `BlockId`.

Verify:

* operation failure;
* transaction poisoning;
* no partial Branch or terminator change;
* no committed state change.

---

## AR-CFG-044 — V0_3 VerifiedProgram binding

Successfully verify a valid Conditional Control Flow Program using V0_3.

Verify:

```text
verified_program.rule_set()
```

identifies:

```text
SemanticVerificationAndDiagnosticsV0_3
```

Also demonstrate that V0_1 or V0_2 verification evidence does not satisfy the V0_3 rule-set identity requirement.

---

## AR-CFG-045 — Older rule sets scan all Modules before verification

Create a Program containing at least two Modules.

Place otherwise valid V0_1/V0_2-compatible content in one Module.

Place Conditional Control Flow unsupported by V0_1/V0_2 in another Module.

Request V0_1 and V0_2 verification separately.

Verify for both:

* rule-set applicability failure;
* no semantic Diagnostics;
* no partial verification result;
* no `VerifiedProgram`.

The test MUST NOT depend on Module iteration order.

---

## AR-CFG-046 — Corrupt entry reference cannot commit

Using an internal test-only construction mechanism if necessary, create a Function body whose entry `BlockId` does not resolve to a valid owned Block.

Verify:

* structural commit rejection;
* atomic preservation of the previous committed revision;
* no weakening of the safe public API.

---

## AR-CFG-047 — Corrupt Branch references cannot commit

Using internal test-only construction, demonstrate structural rejection for invalid Branch references, including at least:

* foreign/missing condition Expression;
* missing target Block;
* foreign target Block.

The cases MAY share test infrastructure.

The public production API MUST NOT be weakened to construct these invalid states.

---

## AR-CFG-048 — Singular body Block view identifies entry

For a multi-Block Function body, verify that any retained singular body-Block inspection API refers to the entry Block.

Also verify the control-flow-aware read-only API exposes all Blocks separately.

The singular API MUST NOT imply single-Block body semantics.

---

# 39. Implementation constraints

## MNIR-CFG-109 — Core CFG ownership

Function-body CFG representation MUST belong to:

```text
mnir-core
```

---

## MNIR-CFG-110 — Verification ownership

Branch and multi-Block Return verification MUST belong to:

```text
mnir-verify
```

---

## MNIR-CFG-111 — Existing Block identity model

Conditional Control Flow MUST reuse existing `BlockId`.

No parallel CFG-specific Block identity may be introduced.

---

## MNIR-CFG-112 — No generic control-flow node identity

This specification does not require:

```text
ControlFlowNodeId
TerminatorId
EdgeId
```

and they MUST NOT be introduced as normative semantics solely for future use.

---

## MNIR-CFG-113 — Controlled mutation only

The public API MUST preserve existing encapsulation and structural guarantees.

---

## MNIR-CFG-114 — Safe Rust

Implementation MUST NOT use or require `unsafe`.

---

The current Rust implementation may already expose a singular
FunctionBody Block accessor originating from Expressions and Basic
Function Bodies 0.1.

Conditional Control Flow 0.1 should preserve source compatibility where
practical by treating that accessor as the entry-Block view while adding
separate all-Block inspection APIs.

This is implementation guidance rather than a normative Rust method-name
requirement.

---

# 40. Expected conceptual representation

```text
Function
└── Body
    ├── entry: B1
    └── Blocks
        ├── B1
        │   ├── E1 = ParameterReference(a)
        │   ├── E2 = ParameterReference(b)
        │   ├── E3 = GreaterThan(E1, E2)
        │   └── Branch {
        │       condition: E3
        │       true: B2
        │       false: B3
        │   }
        │
        ├── B2
        │   ├── E4 = ParameterReference(a)
        │   └── Return(E4)
        │
        └── B3
            ├── E5 = ParameterReference(b)
            └── Return(E5)
```

This representation is informative.

It is neither EasyH nor serialized MNIR syntax.

---

# 41. Verification evolution

Conceptually:

```text
V0_1
    arithmetic + single-Block Return verification

V0_2
    V0_1
    + comparison verification

V0_3
    V0_2
    + conditional control flow
    + Branch condition verification
    + multi-Block Return verification
```

V0_1 and V0_2 remain immutable verification claims.

---

# 42. Foundational invariants

```text
Function bodies may contain multiple Blocks.

Every body has exactly one entry Block.

Block collection order is non-semantic.

Every committed Block has exactly one terminator.

Return and Branch are terminators, not Expressions.

Branch condition belongs to the source Block.

Branch targets belong to the same Function body.

True and false target roles are semantic.

There is no fallthrough.

All committed Blocks are reachable from entry.

The committed CFG is acyclic.

Every control-flow path eventually reaches Return.

Loops are not part of Conditional Control Flow 0.1.

Branch conditions must semantically derive Bool.

Type-invalid Branch conditions remain structurally representable.

V0_1 and V0_2 reject unsupported CFG constructs as rule-set inapplicable.

V0_3 verifies conditional control flow.

Single-Block Return mismatches retain MNIR-DIAG-004.

Multi-Block Return mismatches use MNIR-DIAG-010.

No control-flow execution or constant branch folding exists yet.
```
