# MNIR Specification — Arithmetic Expressions

**Document:** `05-arithmetic-expressions.md`
**Specification status:** Draft
**Specification version:** 0.1
**Normative:** Yes

---

## 1. Purpose

This document defines the first arithmetic Expressions in MNIR.

Specification version 0.1 introduces:

* `Add`
* `Subtract`
* `Multiply`
* `Divide`
* `Remainder`
* arithmetic operand references
* arithmetic type derivation
* checked signed-integer arithmetic semantics
* abstract arithmetic fault conditions
* arithmetic Expression construction and inspection

Arithmetic Expressions operate only on:

```text
Int32
Int64
```

The target capability is to represent a Function conceptually equivalent to:

```text
add(a: Int32, b: Int32) -> Int32 {
    return a + b
}
```

This specification intentionally does not define:

* implicit numeric conversions
* explicit numeric conversions
* unsigned integers
* floating-point arithmetic
* decimal arithmetic
* unary negation
* increment or decrement
* bitwise operations
* wrapping arithmetic
* saturating arithmetic
* arithmetic fault handling
* exceptions
* `Result`
* fault catching
* semantic verification
* constant folding
* optimization
* runtime execution
* backend lowering

Those concepts require later specification increments.

---

# 2. Normative language

Every declarative statement inside a numbered `MNIR-ARITH-*` rule is normative unless explicitly stated otherwise.

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** retain their normative meanings when used inside numbered rules.

Text outside numbered `MNIR-ARITH-*` rules is informative unless explicitly stated otherwise.

Acceptance requirements use identifiers of the form:

```text
AR-ARITH-NNN
```

All applicable `AR-ARITH-*` requirements are mandatory project-level acceptance requirements for the implementation increment defined by this document.

Acceptance requirements demonstrate normative behavior but do not independently introduce MNIR semantics.

---

# 3. Dependencies

This specification builds on:

* Program Model 0.1
* Type System Foundations 0.1
* Functions and Parameters 0.1
* Expressions and Basic Function Bodies 0.1

Existing semantics remain unchanged.

Arithmetic operators introduced by this document are additional Expression kinds.

They use the existing:

```text
ExpressionId
Block
Function body
Return
IntrinsicType
```

models.

---

# 4. Terminology

## 4.1 Arithmetic Expression

An **Arithmetic Expression** is an Expression representing one binary signed-integer arithmetic operation.

Every Arithmetic Expression has:

```text
ExpressionId
operator kind
left ExpressionId
right ExpressionId
```

The Arithmetic Expression itself is owned by the same Block as its operands.

---

## 4.2 Operand

The **left operand** and **right operand** are Expressions referenced by an Arithmetic Expression.

Operand position is semantic.

For example:

```text
a - b
```

and:

```text
b - a
```

are different Arithmetic Expressions.

---

## 4.3 Arithmetic fault

An **arithmetic fault** is an abstract execution condition defined by arithmetic semantics when an operation cannot produce a value in its required intrinsic value domain.

Version 0.1 defines two arithmetic fault conditions:

```text
Overflow
DivisionByZero
```

An arithmetic fault is not:

* an MNIR value
* an Expression
* a type
* an exception object
* a `Result`
* a control-flow node

This specification defines when arithmetic faults occur but does not define how they are represented, propagated, caught, reported, or executed by a future runtime.

---

# 5. Arithmetic Expression kinds

## MNIR-ARITH-001 — Defined arithmetic operators

Arithmetic Expressions 0.1 MUST define exactly these arithmetic Expression kinds:

```text
Add
Subtract
Multiply
Divide
Remainder
```

---

## MNIR-ARITH-002 — Binary operators

Every arithmetic Expression introduced by this specification MUST contain exactly:

```text
one left ExpressionId
one right ExpressionId
```

---

## MNIR-ARITH-003 — No separate arithmetic identity

An Arithmetic Expression MUST use the existing `ExpressionId` identity model.

This specification MUST NOT introduce:

```text
ArithmeticExpressionId
OperatorId
ArithmeticNodeId
```

or another arithmetic-specific semantic identity.

---

# 6. Operand order

## MNIR-ARITH-004 — Operand position is semantic

The distinction between left and right operand MUST be preserved as part of the Arithmetic Expression's semantic contents.

---

## MNIR-ARITH-005 — No commutative canonicalization

Version 0.1 MUST NOT automatically exchange, sort, canonicalize, or deduplicate operands merely because an operator is mathematically commutative.

For example, the representation of:

```text
a + b
```

MUST NOT be silently rewritten to:

```text
b + a
```

as part of Arithmetic Expressions 0.1 semantics.

The same applies to `Multiply`.

---

# 7. Operand ownership

## MNIR-ARITH-006 — Existing operands

Both operand `ExpressionId` values MUST resolve to existing Expressions when an Arithmetic Expression is structurally valid.

---

## MNIR-ARITH-007 — Same-Block operand ownership

Both operand Expressions MUST belong to the same Block that owns the Arithmetic Expression.

Cross-Block arithmetic operand references are structurally invalid.

---

## MNIR-ARITH-008 — No textual operand references

Arithmetic operands MUST be referenced by `ExpressionId`.

Arithmetic operands MUST NOT be identified through:

* preferred names
* source text
* textual variable names
* collection position

---

## MNIR-ARITH-096 — Same Expression may occupy both operand positions

The same `ExpressionId` MAY be used as both the left and right operand of one Arithmetic Expression.

For example:

```text
Add(E1, E1)
Multiply(E2, E2)
```

are structurally valid provided all other structural requirements are satisfied.

Using the same Expression in both operand positions does not create a dependency cycle by itself.

---

# 8. Expression dependency graph

Arithmetic operand references introduce dependencies between Expressions.

Conceptually:

```text
E1 = ParameterReference(P1)
E2 = ParameterReference(P2)
E3 = Add(E1, E2)
```

means that `E3` depends on `E1` and `E2`.

---

## MNIR-ARITH-009 — Acyclic Expression dependencies

The Expression dependency graph inside a Block MUST be acyclic.

An Arithmetic Expression MUST NOT directly or indirectly depend on itself.

---

## MNIR-ARITH-010 — No semantic collection order

Expression dependency MUST be represented by semantic references.

Dependency MUST NOT be inferred from `HashMap`, array, serialization, or iteration order.

---

## MNIR-ARITH-011 — Creation dependency requirement

The standard safe mutation API MUST require arithmetic operands to exist in the transaction working state before the new Arithmetic Expression is created.

Because version 0.1 defines no operand-retargeting operation, this requirement prevents dependency cycles through normal safe construction.

Structural validation MUST nevertheless reject a cyclic internally constructed candidate.

---

# 9. Arithmetic mutation operations

## MNIR-ARITH-012 — Arithmetic construction operations

The version 0.1 mutation API MUST provide operations equivalent to:

```text
add_add_expression
add_subtract_expression
add_multiply_expression
add_divide_expression
add_remainder_expression
```

The exact Rust method names are implementation-defined.

---

## MNIR-ARITH-013 — Arithmetic construction behavior

Each arithmetic construction operation MUST:

* require an existing Block;
* require an existing left operand Expression;
* require an existing right operand Expression;
* require both operands to belong to that Block;
* allocate exactly one provisional `ExpressionId`;
* create the requested arithmetic Expression;
* preserve left and right operand position;
* and return or otherwise directly expose the provisional `ExpressionId`.

---

## MNIR-ARITH-014 — Type validity does not gate structural construction

Arithmetic construction MUST NOT require the operand types to be semantically valid for the requested arithmetic operation.

For example, structurally valid MNIR MAY represent:

```text
Int32 + Bool
```

or:

```text
Int32 + Int64
```

provided all Expression references and ownership relationships are structurally valid.

Such Expressions are semantically type-invalid according to this specification.

---

## MNIR-ARITH-015 — Invalid structural target poisons transaction

Arithmetic construction MUST fail and poison the transaction if:

* the Block does not exist;
* either operand Expression does not exist;
* or either operand belongs to another Block.

Semantic operand-type mismatch MUST NOT poison the transaction during construction.

---

# 10. Arithmetic inspection

## MNIR-ARITH-016 — Operator inspection

The public read-only API MUST permit a consumer to determine which arithmetic operator kind an Arithmetic Expression represents.

---

## MNIR-ARITH-017 — Operand inspection

The public read-only API MUST expose:

```text
left ExpressionId
right ExpressionId
```

for every Arithmetic Expression.

---

## MNIR-ARITH-018 — Expression data sufficiency

The semantic data exposed for an Arithmetic Expression MUST be sufficient to reconstruct its version 0.1 meaning:

```text
operator
left ExpressionId
right ExpressionId
```

No textual arithmetic syntax is required.

---

# 11. Supported arithmetic operand types

## MNIR-ARITH-019 — Supported arithmetic types

Arithmetic Expressions 0.1 supports arithmetic only when both operands have exactly the same intrinsic type and that type is:

```text
Int32
```

or:

```text
Int64
```

---

## MNIR-ARITH-020 — Int32 arithmetic

When both operands derive type `Int32`, the arithmetic Expression derives type `Int32`.

---

## MNIR-ARITH-021 — Int64 arithmetic

When both operands derive type `Int64`, the arithmetic Expression derives type `Int64`.

---

## MNIR-ARITH-022 — Mixed integer types are invalid

An Arithmetic Expression with operand types:

```text
Int32
Int64
```

in either order is semantically type-invalid.

Version 0.1 defines no implicit widening from `Int32` to `Int64`.

---

## MNIR-ARITH-023 — Bool arithmetic is invalid

`Bool` MUST NOT be a valid operand type for arithmetic operators introduced by this specification.

---

## MNIR-ARITH-024 — Unit arithmetic is invalid

`Unit` MUST NOT be a valid operand type for arithmetic operators introduced by this specification.

---

# 12. Arithmetic type derivation

## MNIR-ARITH-025 — Arithmetic Expression type derivation

An Arithmetic Expression derives its intrinsic type only when:

1. both operand types can be derived;
2. both operand types are equal;
3. and that type is `Int32` or `Int64`.

The resulting Arithmetic Expression type is the common operand type.

---

## MNIR-ARITH-026 — Operand type mismatch

If the operand types are both derivable but differ, arithmetic Expression type inspection MUST report a typed semantic type error.

It MUST NOT arbitrarily select either operand type.

---

## MNIR-ARITH-027 — Unsupported operand type

If both operands have the same type but that type is not supported for arithmetic, arithmetic Expression type inspection MUST report a typed semantic type error.

This includes:

```text
Bool + Bool
Unit + Unit
```

and equivalent use with the other arithmetic operators.

---

## MNIR-ARITH-028 — Nested type derivation

Arithmetic type derivation MUST recursively use the derived types of its operand Expressions.

For example:

```text
E1 : Int32
E2 : Int32
E3 = Add(E1, E2)        -> Int32
E4 = Multiply(E3, E1)   -> Int32
```

---

## MNIR-ARITH-029 — Operand type inspection failure

If the type of an operand cannot currently be derived, the containing Arithmetic Expression's type also cannot be derived.

The inspection API MUST report a typed error or unavailable-type result.

It MUST NOT invent an operand or result type.

---

## MNIR-ARITH-030 — Inspection errors do not poison transactions

Arithmetic Expression type-inspection failure is read-only behavior.

Such inspection MUST NOT poison an Active mutation transaction.

---

## MNIR-ARITH-090 — Arithmetic type-inspection outcomes

Arithmetic Expression type inspection MUST distinguish at least the following semantic outcomes:

```text
ValidType(IntrinsicType)

OperandTypeUnavailable

OperandTypeMismatch

UnsupportedOperandType
```

The exact Rust representation, type names, enum layout, payloads, source-cause representation, and public API shape are implementation-defined.

This rule defines semantic distinguishability only.

It does not introduce the future MNIR diagnostics system.

---

## MNIR-ARITH-091 — Operand type unavailable

Arithmetic type inspection MUST produce the semantic outcome:

```text
OperandTypeUnavailable
```

when the intrinsic type of one or both operands cannot currently be derived.

This includes an operand whose type inspection fails because of a temporarily dangling `ParameterReference`.

The implementation MAY preserve the underlying operand error as diagnostic or error-source information.

Such cause representation is implementation-defined.

---

## MNIR-ARITH-092 — Operand type mismatch

If both operand types are successfully derived but are different, arithmetic type inspection MUST produce the semantic outcome:

```text
OperandTypeMismatch
```

For example:

```text
Int32 + Int64
```

produces `OperandTypeMismatch`.

The implementation MAY expose the left and right intrinsic types as error payload.

The exact payload representation is implementation-defined.

---

## MNIR-ARITH-093 — Unsupported operand type

If both operand types are successfully derived, are equal, but are not supported arithmetic types under `MNIR-ARITH-019`, arithmetic type inspection MUST produce:

```text
UnsupportedOperandType
```

Examples include:

```text
Bool + Bool
Unit * Unit
```

The implementation MAY expose the unsupported intrinsic type as error payload.

---

## MNIR-ARITH-094 — Deterministic type-outcome precedence

Arithmetic type inspection MUST determine its semantic outcome using the following precedence:

```text
1. If either operand type cannot be derived:
       OperandTypeUnavailable

2. Otherwise, if the derived operand types differ:
       OperandTypeMismatch

3. Otherwise, if the common operand type is not Int32 or Int64:
       UnsupportedOperandType

4. Otherwise:
       ValidType(common operand type)
```

This precedence is semantic.

It MUST NOT depend on:

* `HashMap` iteration order;
* expression creation order;
* memory layout;
* or whichever operand an implementation happens to inspect first.

If both operands independently have unavailable types, the implementation is not required to choose one underlying cause as normatively primary.

---

## MNIR-ARITH-095 — Recursive arithmetic type failure

When an operand is itself an Arithmetic Expression whose type inspection does not produce a valid intrinsic type, the containing Arithmetic Expression MUST treat that operand type as unavailable for purposes of `MNIR-ARITH-094`.

The outer Expression MUST NOT reinterpret an inner arithmetic type error as a valid operand type.

The implementation MAY preserve nested error-source information.

Nested error representation is implementation-defined.

---

# 13. Structurally valid but semantically invalid arithmetic

## MNIR-ARITH-031 — Type-invalid arithmetic may be committed

An Arithmetic Expression that violates `MNIR-ARITH-019` MAY exist in a committed Program revision if all structural requirements are satisfied.

---

## MNIR-ARITH-032 — Type invalidity is not structural invalidity

Arithmetic operand type mismatch or unsupported arithmetic operand type MUST NOT be treated as structural corruption.

A future semantic verifier will detect and report such conditions.

---

## MNIR-ARITH-033 — Semantic verifier remains deferred

Arithmetic Expressions 0.1 MUST NOT introduce a semantic verifier or `VerifiedProgram` merely to reject type-invalid arithmetic.

---

# 14. Checked arithmetic principle

## MNIR-ARITH-034 — Arithmetic is checked

`Add`, `Subtract`, `Multiply`, `Divide`, and `Remainder` MUST have checked signed-integer semantics.

Arithmetic operations MUST NOT silently wrap, saturate, truncate to another integer width, or otherwise manufacture a value different from the mathematical result defined by this specification.

---

## MNIR-ARITH-035 — Arithmetic result domain

When an arithmetic operation mathematically produces a result inside the operand type's abstract value domain, the arithmetic value is that mathematical result.

When the operation cannot produce a valid result according to the rules below, an arithmetic fault condition occurs.

---

## MNIR-ARITH-036 — Fault does not change static type

An arithmetic fault condition does not create a separate static Expression type.

A type-correct Arithmetic Expression retains the derived intrinsic type defined by `MNIR-ARITH-025`.

For example:

```text
Int32 + Int32 -> Int32
```

remains the static type relation even when particular runtime operand values would overflow.

---

# 15. Abstract fault semantics

## MNIR-ARITH-037 — Defined arithmetic fault conditions

Arithmetic Expressions 0.1 defines exactly these abstract arithmetic fault conditions:

```text
Overflow
DivisionByZero
```

---

## MNIR-ARITH-038 — Arithmetic faults are not values

An arithmetic fault MUST NOT be interpreted as an `Int32`, `Int64`, `Bool`, `Unit`, or other normal MNIR value.

---

## MNIR-ARITH-039 — Fault handling deferred

This specification does not define:

* catching arithmetic faults;
* converting faults into values;
* `Result`;
* exceptions;
* panic semantics;
* process termination;
* stack unwinding;
* error effects;
* fault propagation APIs.

No implementation of those concepts is required by this increment.

---

## MNIR-ARITH-040 — Future execution must preserve fault semantics

A future execution backend implementing these arithmetic operators MUST preserve the arithmetic fault conditions defined by this specification.

It MUST NOT silently replace an arithmetic fault with wrapping or saturating arithmetic behavior.

---

# 16. Addition semantics

## MNIR-ARITH-041 — Addition mathematical result

For same-typed valid signed-integer operands:

```text
a + b
```

denotes mathematical integer addition.

---

## MNIR-ARITH-042 — Addition overflow

If the mathematical result of `a + b` lies outside the abstract value domain of the operand type, the operation has the arithmetic fault:

```text
Overflow
```

Otherwise the result is the mathematical sum.

---

# 17. Subtraction semantics

## MNIR-ARITH-043 — Subtraction mathematical result

For same-typed valid signed-integer operands:

```text
a - b
```

denotes mathematical integer subtraction with preserved left/right operand roles.

---

## MNIR-ARITH-044 — Subtraction overflow

If the mathematical result of `a - b` lies outside the abstract value domain of the operand type, the operation has the arithmetic fault:

```text
Overflow
```

Otherwise the result is the mathematical difference.

---

# 18. Multiplication semantics

## MNIR-ARITH-045 — Multiplication mathematical result

For same-typed valid signed-integer operands:

```text
a * b
```

denotes mathematical integer multiplication.

---

## MNIR-ARITH-046 — Multiplication overflow

If the mathematical result of `a * b` lies outside the abstract value domain of the operand type, the operation has the arithmetic fault:

```text
Overflow
```

Otherwise the result is the mathematical product.

---

# 19. Division semantics

## MNIR-ARITH-047 — Division by zero

For:

```text
a / 0
```

the operation has the arithmetic fault:

```text
DivisionByZero
```

---

## MNIR-ARITH-048 — Signed division rounding

For a nonzero divisor, signed integer division MUST produce the mathematical quotient truncated toward zero.

Examples:

```text
 7 /  3 =  2
-7 /  3 = -2
 7 / -3 = -2
-7 / -3 =  2
```

---

## MNIR-ARITH-049 — Division overflow

If the truncated mathematical quotient lies outside the abstract value domain of the operand type, the operation has:

```text
Overflow
```

For the intrinsic signed integer types currently defined, this includes:

```text
Int32::MIN / -1
Int64::MIN / -1
```

---

## MNIR-ARITH-050 — Valid division result

If the divisor is nonzero and the truncated quotient lies inside the operand type's abstract value domain, the operation produces that quotient.

---

# 20. Remainder semantics

## MNIR-ARITH-051 — Remainder by zero

For:

```text
a % 0
```

the operation has the arithmetic fault:

```text
DivisionByZero
```

---

## MNIR-ARITH-052 — Remainder definition

For nonzero divisor `b`, Remainder semantics are defined over mathematical integers.

Conceptually:

```text
q = truncate_toward_zero(a / b)
r = a - (q * b)
```

where the conceptual intermediate calculations used to define `q` and `r` are mathematical integer operations and are not themselves MNIR `Int32` or `Int64` runtime operations.

The Remainder result is `r`.

This definition ensures that the conceptual quotient used to define Remainder does not independently introduce an `Overflow` fault.

The special case defined by `MNIR-ARITH-054` therefore remains:

```text
Int32::MIN % -1 = 0
Int64::MIN % -1 = 0
```

---

## MNIR-ARITH-053 — Remainder sign

A nonzero remainder MUST have the same sign as the left operand.

The absolute value of the remainder MUST be less than the absolute value of the right operand.

---

## MNIR-ARITH-054 — Minimum divided by negative one remainder

For:

```text
Int32::MIN % -1
Int64::MIN % -1
```

the Remainder result is:

```text
0
```

This operation does not have `Overflow`, because the Remainder result itself is representable.

---

# 21. No implicit conversion

## MNIR-ARITH-055 — No arithmetic widening

Arithmetic Expressions 0.1 MUST NOT implicitly convert:

```text
Int32 -> Int64
```

or any other numeric type.

---

## MNIR-ARITH-056 — No arithmetic narrowing

Arithmetic Expressions 0.1 MUST NOT implicitly convert:

```text
Int64 -> Int32
```

---

## MNIR-ARITH-057 — No operand normalization through conversion

A mixed arithmetic Expression such as:

```text
Int32 + Int64
```

MUST remain semantically type-invalid rather than being silently normalized through conversion.

---

# 22. Literal arithmetic and static faults

## MNIR-ARITH-058 — Guaranteed arithmetic faults are representable

A structurally valid and type-correct Arithmetic Expression MAY represent operand values that necessarily produce an arithmetic fault.

For example, an Expression conceptually equivalent to:

```text
2147483647 + 1
```

using `Int32` operands is representable.

---

## MNIR-ARITH-059 — Constant fault detection deferred

Arithmetic Expressions 0.1 does not require constant folding or compile-time detection of guaranteed arithmetic faults.

A future verifier MAY introduce diagnostics or rejection rules for statically provable arithmetic faults.

---

# 23. Expression purity and ordering

## MNIR-ARITH-060 — Arithmetic operators define no side effects

This specification defines no side effects for:

```text
Add
Subtract
Multiply
Divide
Remainder
```

---

## MNIR-ARITH-061 — No operand evaluation order

Arithmetic Expressions 0.1 does not define left-first or right-first runtime operand evaluation order.

Because this increment defines no side effects for arithmetic operands introduced here, collection or storage order MUST NOT be interpreted as execution order.

Future execution/effect specifications must define any ordering that becomes observably relevant.

---

# 24. Nested arithmetic

## MNIR-ARITH-062 — Arithmetic operands may be arithmetic Expressions

An Arithmetic Expression MAY reference another Arithmetic Expression in the same Block as an operand.

---

## MNIR-ARITH-063 — Nested reference integrity

All nested arithmetic operand references MUST satisfy the same ownership, existence, and acyclicity rules as other arithmetic references.

---

# 25. Snapshot behavior

## MNIR-ARITH-064 — Snapshot arithmetic preservation

A Program snapshot MUST preserve:

* arithmetic Expression identity;
* arithmetic operator kind;
* left operand reference;
* right operand reference.

---

# 26. Fork behavior

## MNIR-ARITH-065 — Fork arithmetic preservation

A Program fork MUST preserve arithmetic Expression semantics.

If Expression identifiers are remapped during fork construction, all arithmetic operand references MUST be updated consistently and atomically.

---

## MNIR-ARITH-066 — Fork reference integrity

The resulting fork MUST preserve valid arithmetic Expression ownership and dependency relationships.

---

# 27. Removal cascade

Arithmetic Expressions use the existing Expression ownership model.

There is no individual Expression removal operation in version 0.1.

---

## MNIR-ARITH-067 — Body removal removes arithmetic Expressions

Removing a Function body MUST remove all Arithmetic Expressions owned by its Block.

Committed `ExpressionId` values remain retired according to the existing Expression identity rules.

---

## MNIR-ARITH-068 — Function and Module cascades

Function and Module removal MUST transitively remove descendant Arithmetic Expressions through the existing body-removal semantics.

---

# 28. Structural validity

## MNIR-ARITH-069 — Structurally valid Arithmetic Expression

An Arithmetic Expression is structurally valid when:

1. it has exactly one valid `ExpressionId`;
2. it belongs to exactly one existing Block;
3. it has exactly one arithmetic operator kind defined by this specification;
4. it references exactly one existing left Expression in the same Block;
5. it references exactly one existing right Expression in the same Block;
6. its operand dependency graph is acyclic;
7. all applicable Expression identity rules are satisfied.

Operand type compatibility is not a structural validity requirement.

---

## MNIR-ARITH-070 — Structural commit rejection

A Program revision containing an Arithmetic Expression that violates `MNIR-ARITH-069` MUST NOT commit successfully.

---

# 29. Extended Expression inspection

## MNIR-ARITH-071 — Arithmetic kind participates in Expression inspection

The public `Expression` inspection model MUST distinguish the five arithmetic Expression kinds from all previously defined Expression kinds.

---

## MNIR-ARITH-072 — Arithmetic Expression type inspection

The existing Expression type-inspection API MUST support arithmetic Expressions according to `MNIR-ARITH-025` through `MNIR-ARITH-030` and `MNIR-ARITH-090` through `MNIR-ARITH-095`.

---

# 30. Existing Return semantics

## MNIR-ARITH-073 — Arithmetic Expression may be returned

A Return terminator MAY reference a structurally valid Arithmetic Expression in the same Block.

---

## MNIR-ARITH-074 — Return type compatibility remains semantic

The declared Function return type and derived Arithmetic Expression type remain subject to the semantic Return type rule established by Expressions and Basic Function Bodies 0.1.

This specification does not introduce semantic verification of that relationship.

---

# 31. Extended no-op state

## MNIR-ARITH-075 — Arithmetic data participates in no-op comparison

The Program transaction no-op comparison MUST include, for every Arithmetic Expression:

* operator kind;
* left operand `ExpressionId`;
* right operand `ExpressionId`.

Changing any of those semantic contents would change Program state.

Version 0.1 defines no public arithmetic operand-retargeting operation.

---

# 32. No arithmetic mutation after creation

## MNIR-ARITH-076 — Arithmetic Expression immutability

After an Arithmetic Expression has been created, Arithmetic Expressions 0.1 defines no mutation operation that changes:

* operator kind;
* left operand;
* right operand.

Changing such contents requires a future specification.

---

# 33. No wrapping arithmetic

## MNIR-ARITH-077 — Wrapping operators excluded

Version 0.1 MUST NOT introduce wrapping variants of:

```text
Add
Subtract
Multiply
Divide
Remainder
```

---

# 34. No saturating arithmetic

## MNIR-ARITH-078 — Saturating operators excluded

Version 0.1 MUST NOT introduce saturating arithmetic semantics.

---

# 35. No arithmetic fault API

## MNIR-ARITH-079 — No required public ArithmeticFault representation

Although fault conditions are normative arithmetic semantics, the `mnir-core` implementation of this increment is not required to expose:

```text
ArithmeticFault
ArithmeticResult
ArithmeticError
```

as public runtime-evaluation APIs.

This increment represents arithmetic program semantics; it does not execute arithmetic Expressions.

---

## MNIR-ARITH-080 — No evaluator required

Implementation of Arithmetic Expressions 0.1 MUST NOT require a Function-body evaluator, interpreter, constant evaluator, or execution backend solely to demonstrate arithmetic semantics.

---

# 36. Specification gaps

## MNIR-ARITH-081 — Undefined arithmetic semantics

If implementation requires externally observable arithmetic behavior that is not defined by this specification, the implementation MUST NOT establish that behavior as normative MNIR semantics.

The missing behavior MUST be reported as a specification gap.

---

# 37. Explicitly unresolved topics

The following topics are intentionally unresolved:

* unary negation
* absolute value
* exponentiation
* bitwise operations
* shifts
* unsigned arithmetic
* floating-point arithmetic
* decimal arithmetic
* explicit numeric conversion
* implicit numeric conversion
* wrapping arithmetic
* saturating arithmetic
* arithmetic fault representation
* arithmetic fault propagation
* arithmetic fault handling
* exceptions
* `Result`
* semantic verification
* arithmetic diagnostics
* constant folding
* compile-time arithmetic evaluation
* optimization
* algebraic simplification
* arithmetic canonicalization
* common-subexpression elimination
* runtime evaluation
* operand execution order
* backend lowering
* EasyH arithmetic syntax
* serialization syntax

---

## MNIR-ARITH-082 — Unresolved topics are not 0.1 semantics

An implementation MUST NOT establish unresolved topics listed by this document as normative Arithmetic Expressions 0.1 semantics.

---

# 38. Acceptance requirements

The acceptance range for this increment is `AR-ARITH-001` through `AR-ARITH-032`.

The arithmetic fault semantics defined by `MNIR-ARITH-034` through `MNIR-ARITH-059` are normative even though Arithmetic Expressions 0.1 does not provide an evaluator.

Conformance for rules whose only observable behavior would require execution may therefore be demonstrated by specification-to-implementation conformance inspection in this increment.

A future evaluator or execution-backend specification will require executable tests for these semantics.

## AR-ARITH-001 — Int32 addition

Create two `Int32` Expressions and an `Add` Expression referencing them.

Verify:

* arithmetic Expression identity;
* operator kind;
* left/right references;
* derived type `Int32`.

---

## AR-ARITH-002 — Int64 addition

Demonstrate equivalent behavior for `Int64`.

---

## AR-ARITH-003 — Subtraction

Create and inspect a `Subtract` Expression.

Verify that left and right operand roles are preserved.

---

## AR-ARITH-004 — Multiplication

Create and inspect a `Multiply` Expression.

---

## AR-ARITH-005 — Division

Create and inspect a `Divide` Expression.

---

## AR-ARITH-006 — Remainder

Create and inspect a `Remainder` Expression.

---

## AR-ARITH-007 — Mixed integer arithmetic is representable but type-invalid

Create Arithmetic Expressions using both:

```text
left:  Int32
right: Int64
```

and:

```text
left:  Int64
right: Int32
```

Verify for both:

* structural construction succeeds;
* structurally valid commit succeeds;
* type inspection produces `OperandTypeMismatch`.

No implicit conversion is introduced.

---

## AR-ARITH-008 — Bool arithmetic is representable but type-invalid

Create arithmetic using `Bool` operands and demonstrate the same structural-versus-semantic distinction.

---

## AR-ARITH-009 — Unit arithmetic is representable but type-invalid

Demonstrate equivalent behavior for `Unit`.

---

## AR-ARITH-010 — Cross-Block operands rejected

Demonstrate arithmetic construction failure when:

1. the left operand belongs to another Block;
2. the right operand belongs to another Block.

For each case verify:

* operation failure;
* transaction poisoning;
* and atomic non-commit of transaction-local changes.

The two cases MAY be covered by one parameterized test.

---

## AR-ARITH-011 — Unknown operands rejected

Demonstrate arithmetic construction failure when:

1. the left operand is an unknown `ExpressionId`;
2. the right operand is an unknown `ExpressionId`.

For each case verify:

* operation failure;
* transaction poisoning;
* and atomicity.

The two cases MAY be covered by one parameterized test.

---

## AR-ARITH-012 — Unknown Block rejected

Attempt arithmetic construction against an unknown `BlockId`.

Verify failure and transaction poisoning.

---

## AR-ARITH-013 — Nested arithmetic type derivation

Construct conceptually:

```text
E3 = Add(E1, E2)
E4 = Multiply(E3, E1)
```

using `Int32`.

Verify `E4` derives `Int32`.

---

## AR-ARITH-014 — Nested type failure propagation

Create an inner Arithmetic Expression whose type inspection produces either:

```text
OperandTypeMismatch
```

or:

```text
UnsupportedOperandType
```

Use it as an operand of another Arithmetic Expression.

Verify that outer type inspection produces:

```text
OperandTypeUnavailable
```

rather than inventing an intrinsic type.

Nested diagnostic cause preservation is implementation-defined.

---

## AR-ARITH-015 — Dangling ParameterReference type failure propagation

Create arithmetic depending on a `ParameterReference`.

Temporarily remove the referenced Parameter.

Verify:

* the `ParameterReference` type cannot be derived;
* the containing Arithmetic Expression reports `OperandTypeUnavailable`;
* inspection does not poison the transaction;
* structural validity may still be restored using the previously specified repair path.

---

## AR-ARITH-016 — Operand order preservation

Create two Expressions `E1` and `E2`.

Construct:

```text
Subtract(E1, E2)
```

and separately:

```text
Subtract(E2, E1)
```

Verify the stored semantic operand references remain distinct and ordered.

---

## AR-ARITH-017 — Arithmetic ExpressionId uniqueness

Create multiple arithmetic Expressions in one and multiple Function bodies.

Verify committed and provisional `ExpressionId` uniqueness continues to satisfy the existing Expression identity rules.

---

## AR-ARITH-018 — Arithmetic Expression identity retirement

Commit an Arithmetic Expression, remove its body, later create another Expression, and verify the committed arithmetic `ExpressionId` is not reused.

---

## AR-ARITH-019 — Snapshot preservation

Verify snapshot preservation of arithmetic operator kind and operand references.

---

## AR-ARITH-020 — Fork preservation

Verify fork preservation of arithmetic semantics and operand reference consistency.

---

## AR-ARITH-021 — Return arithmetic Expression

Construct a Function conceptually equivalent to:

```text
add(a: Int32, b: Int32) -> Int32 {
    return a + b
}
```

Verify structural commit succeeds and the Return terminator references the `Add` Expression.

---

## AR-ARITH-022 — Return mismatch remains semantic

Construct arithmetic deriving `Int64` in a Function declared to return `Int32`.

Verify structural commit succeeds.

Do not introduce semantic verification.

---

## AR-ARITH-023 — Guaranteed overflow Expression is representable

Using literals, construct conceptually:

```text
2147483647 + 1
```

as `Int32` arithmetic.

Verify the MNIR structure is representable and type-correct.

No evaluator or constant-folding behavior is required.

---

## AR-ARITH-024 — Checked arithmetic conformance

Demonstrate by conformance inspection that the implementation introduces only the checked arithmetic Expression kinds defined by this specification and no wrapping or saturating alternatives.

No runtime evaluator is required.

---

## AR-ARITH-025 — Division semantic definition conformance

Demonstrate by conformance review that the implementation does not introduce conflicting signed-division, divide-by-zero, or overflow semantics.

No executable division evaluator is required.

---

## AR-ARITH-026 — Remainder semantic definition conformance

Demonstrate equivalent conformance for Remainder semantics.

---

## AR-ARITH-027 — Expression dependency acyclicity

Demonstrate that the safe public mutation API cannot create an arithmetic dependency cycle.

Using an internal test-only corruption mechanism if necessary, verify that a cyclic candidate cannot commit.

The production API MUST NOT be weakened for this test.

---

## AR-ARITH-028 — No arithmetic operand retargeting

Demonstrate by API inspection that this increment exposes no mutation operation for changing an arithmetic operator or operand after creation.

---

## AR-ARITH-029 — No arithmetic evaluator

Demonstrate by conformance inspection that this increment introduces no Function-body interpreter, arithmetic evaluator, constant evaluator, or backend.

---

## AR-ARITH-030 — Existing conformance preserved

All previous Program Model, Type System, Functions/Parameters, and Expressions/Body acceptance suites MUST continue to pass without semantic regression.

---

## AR-ARITH-031 — Arithmetic type-inspection outcome categories

Demonstrate that the public arithmetic type-inspection behavior can distinguish:

```text
ValidType
OperandTypeUnavailable
OperandTypeMismatch
UnsupportedOperandType
```

Exact Rust type names are implementation-defined.

This requirement MUST NOT be satisfied by introducing the future MNIR diagnostics framework.

---

## AR-ARITH-032 — Same Expression as both operands

Create an Arithmetic Expression equivalent to:

```text
Add(E1, E1)
```

Verify that:

* construction succeeds;
* both stored operand references equal `E1`;
* structural validation succeeds;
* no dependency cycle is inferred solely from using the same operand twice.

---

# 39. Implementation constraints

## MNIR-ARITH-083 — Core ownership

Arithmetic Expression representation MUST belong to:

```text
mnir-core
```

---

## MNIR-ARITH-084 — Existing Expression model extension

Arithmetic operators MUST extend the existing Expression model rather than introducing a parallel arithmetic AST or independently owned expression hierarchy.

---

## MNIR-ARITH-085 — Dependency direction

Implementing Arithmetic Expressions 0.1 MUST NOT cause `mnir-core` to depend on:

```text
mnir-verify
easyh-render
mnir-cli
```

---

## MNIR-ARITH-086 — Controlled mutation only

The public API MUST NOT expose unrestricted mutable access that allows callers to bypass arithmetic reference or structural invariants.

---

## MNIR-ARITH-087 — Safe Rust

The implementation MUST NOT use or require `unsafe` Rust to satisfy this specification.

---

## MNIR-ARITH-088 — No speculative arithmetic hierarchy

This specification does not require:

```text
ArithmeticNode
ArithmeticId
BinaryOperationId
NumericType
ArithmeticResult
ArithmeticFault object
```

as general public abstractions.

The implementation MUST NOT introduce them as normative MNIR semantics solely in anticipation of future specifications.

---

## MNIR-ARITH-089 — No semantic verifier

Implementation MUST NOT introduce a semantic verifier merely to reject invalid arithmetic operand types.

---

# 40. Expected implementation shape

Conceptually:

```text
Expression
├── Int32Literal(i32)
├── Int64Literal(i64)
├── BoolLiteral(bool)
├── UnitLiteral
├── ParameterReference(ParameterId)
├── Add {
│   ├── left: ExpressionId
│   └── right: ExpressionId
│   }
├── Subtract {
│   ├── left: ExpressionId
│   └── right: ExpressionId
│   }
├── Multiply {
│   ├── left: ExpressionId
│   └── right: ExpressionId
│   }
├── Divide {
│   ├── left: ExpressionId
│   └── right: ExpressionId
│   }
└── Remainder {
    ├── left: ExpressionId
    └── right: ExpressionId
    }
```

The concrete Rust representation is implementation-defined.

---

# 41. Target example

After implementation, MNIR must be able to represent:

```text
add(a: Int32, b: Int32) -> Int32 {
    return a + b
}
```

conceptually as:

```text
FunctionId(F1)
├── return_type: Int32
├── ParameterId(P1)
│   └── type: Int32
├── ParameterId(P2)
│   └── type: Int32
└── Body
    └── BlockId(B1)
        ├── ExpressionId(E1)
        │   └── ParameterReference(P1)
        ├── ExpressionId(E2)
        │   └── ParameterReference(P2)
        ├── ExpressionId(E3)
        │   └── Add
        │       ├── left: E1
        │       └── right: E2
        └── Return(E3)
```

This representation is informative.

It is neither EasyH syntax nor MNIR serialization syntax.

---

# 42. Foundational invariants

The following is an informative summary:

```text
Arithmetic operators are Expressions.

Arithmetic Expressions reuse ExpressionId.

Arithmetic operands reference Expressions in the same Block.

Arithmetic dependencies are acyclic.

Left and right operand position is semantic.

Arithmetic construction does not reject type-invalid operands.

Arithmetic supports only same-typed Int32 and Int64 operands.

No implicit numeric conversions exist.

Arithmetic Expression types are derived recursively.

Type-inspection failures do not poison transactions.

Arithmetic is checked.

Overflow never silently wraps.

Division by zero is an arithmetic fault.

Signed division truncates toward zero.

Remainder follows truncation-toward-zero division semantics.

Int MIN % -1 produces zero.

Arithmetic faults are semantic conditions, not values.

No fault-handling mechanism exists yet.

No evaluator exists yet.

No semantic verifier exists yet.

No wrapping or saturating arithmetic exists yet.
```
