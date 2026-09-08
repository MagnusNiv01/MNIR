# ADR 0002: Explicit Effect Sequencing

## Status

Accepted

## Context

[ADR 0001](0001-value-identity-and-sequencing.md) establishes `ExpressionId`
as the identity of a locally computed value, preserves the pure Expression
dependency DAG, and separates observable sequencing from pure value
dependencies. The next architectural question is how future effectful
operations can have explicit order without making every Expression collection
an instruction sequence.

Function calls are expected to become the first operations for which this
distinction matters. A Call may produce a value, perform observable effects, or
do both. Inferring effect order from `HashMap` iteration, Expression creation,
serialization, or source presentation would make incidental representation
details semantic and would conflict with the existing unordered Expression
model.

This ADR records an architectural direction. It is informative and does not
define normative MNIR semantics. Future specifications must define the
concrete representation, validity rules, and verification behavior before an
`EffectSequence` becomes part of MNIR.

## Decision

### Pure Expression model remains unchanged

Pure Expressions continue to form a semantic dependency DAG. Expression
collection order remains non-semantic, and `ExpressionId` remains the identity
of the value produced by an Expression. Pure Expressions do not acquire
semantic execution positions merely to support future sequencing.

### Separate effect sequencing

Observable side-effect ordering will be represented separately from pure value
dependencies. Conceptually, a future Block may contain:

```text
Block
├── Expressions
│   └── unordered dependency DAG
│
├── EffectSequence
│   └── ordered ExpressionId references
│
└── Terminator
    ├── Return
    └── Branch
```

`EffectSequence` is a distinct semantic construct whose ordering is
intentionally meaningful. This does not change the non-semantic ordering of
ordinary Expression, Function, Module, or Block collections.

### Expression identity remains unified

MNIR will not introduce a separate `OperationId`, `CallId`, `EffectId`, or
`LocalValueId` solely because an Expression may have observable effects. An
effectful value-producing operation still uses an ordinary `ExpressionId`.

A future effectful Expression may therefore participate in both:

1. the value-dependency graph; and
2. the containing Block's effect sequence.

Conceptually:

```text
ExpressionId E17
    ├── value dependencies
    └── effect-sequence position
```

### Effectful Expression requirement

The intended future direction is that an Expression with observable effects
must be explicitly sequenced. Pure Expressions do not need to appear in the
effect sequence. The normative rules that determine whether an Expression is
effectful are deferred to future specifications.

### Intra-Block and inter-Block sequencing

The architectural separation is:

```text
Within a Block:
    EffectSequence defines observable effect order.

Between Blocks:
    CFG edges define control-flow order.
```

This ADR does not introduce effect-token propagation across Blocks.

### Initial sequence model

The initial architectural direction is a simple total order of effectful
Expressions within each Block. The first implementation will not introduce a
partial-order effect DAG or explicit effect-state/effect-token threading.

Both remain possible future extensions if concurrency, optimization, or
stronger effect-state reasoning creates a concrete need.

### Function calls

Function calls are expected to be the first concrete consumer of this
sequencing model. A future Call Expression may produce a value, have observable
effects, and require placement in `EffectSequence`.

Pure calls may eventually be exempt from sequencing once Function effects are
known. This ADR does not define how call purity or effectfulness is determined.

### Resources

Effect sequencing and resource ownership are separate semantic dimensions. For
example, future semantics may place:

```text
OpenTransaction
DatabaseWrite
CommitTransaction
```

in one effect sequence while a separate resource model verifies ownership and
lifecycle correctness. This ADR does not define resource semantics.

### AI authoring

The model allows AI tooling to construct and sequence semantic operations
without manipulating textual source order. Conceptually:

```text
create Call E17
create Call E18

effect sequence:
    E17
    E18
```

The future semantic API should make sequencing requirements discoverable and
verifiable so that AI tooling can construct a valid sequence through typed
operations rather than infer it from presentation.

### Serialization

Effect-sequence position will be canonical semantic data and must eventually be
serialized deterministically. This ADR does not define serialization syntax or
encoding.

## Rationale

Keeping pure value dependencies separate from effects preserves the existing
Expression model and avoids imposing unnecessary execution semantics on pure
computation. A Block-local total order is the smallest sequencing model that
can state an unambiguous order for ordinary sequential effects.

Reusing `ExpressionId` lets an effectful value-producing operation participate
in data flow and effect sequencing without a second identity and lookup layer.
CFG edges already describe which Block follows which; adding cross-Block effect
tokens before merges, loops, and their value-flow rules are designed would
prematurely constrain those future decisions.

## Consequences

- Current pure Expression semantics remain stable.
- Effect order becomes explicit only where it has semantic significance.
- `ExpressionId` remains the unified identity for values produced by pure and
  effectful Expressions.
- Function Calls can be introduced without making all Expressions ordered.
- An Expression may participate in both data dependencies and effect-sequence
  position.
- EffectSequence position becomes canonical semantic data.
- Future Effects verification can require effectful Expressions to be
  correctly sequenced.
- Deterministic serialization and semantic diff must account for effect order.
- Concurrency and partial-order effects remain deferred.

## Alternatives considered

### Ordered SSA instruction list for all Expressions

Not selected. This is a valid established architecture, but it would make
execution order semantic for pure Expressions even when only their data
dependencies matter. It remains available for reconsideration if later
requirements justify a unified ordered representation.

### Explicit effect-dependency DAG

Not selected for the initial model. A partial-order graph would require cycle,
predecessor/successor, concurrency, and scheduling semantics before the current
roadmap needs them. It may be reconsidered if parallel effects or optimization
require partial-order information.

### Effect-state or effect-token threading

Not selected initially. Token threading would require substantially more
value-flow semantics around Branches, merge points, dominance, and future
loops. It remains a possible future representation if explicit effect-state
reasoning becomes valuable.

### Separate ordered `EffectSequence`

Selected. It preserves the unordered pure Expression DAG while adding one
explicit, Block-local total order for operations whose effects are observable.

## Deferred questions

- What is the exact representation and mutation API of `EffectSequence`?
- How is Expression effectfulness determined?
- How are pure and effectful Function calls distinguished?
- Is a Call returning `Unit` still represented as an Expression?
- Must every effectful Expression appear exactly once in its Block's sequence?
- Must every EffectSequence reference resolve to an Expression in the same
  Block?
- How are conflicts between value-dependency order and effect-sequence order
  detected and rejected?
- What sequencing relationship exists between the final effect and the Block's
  terminator?
- How does effect sequencing interact with future loops?
- How does it interact with Block merges?
- How are concurrency and parallel effects represented?
- What resource ownership model accompanies effect sequencing?
- How are asynchronous operations represented and ordered?
- How do exceptions and runtime faults affect sequencing?
- What cross-Block effect reasoning is required beyond CFG order?
- How is EffectSequence represented in canonical serialization?
- How does semantic diff represent reordered effects?
