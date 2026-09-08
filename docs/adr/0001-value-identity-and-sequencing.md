# ADR 0001: Value Identity and Sequencing Direction

## Status

Accepted

## Context

MNIR already gives each Expression an `ExpressionId`, represents pure
Expression dependencies through typed identity references, and assigns no
semantic meaning to Expression collection order. Future human-oriented
frontends nevertheless need to present familiar constructs such as:

```text
let sum = a + b
```

This creates a design question: whether MNIR should introduce a separate local
value or binding identity, or whether the Expression result itself should be
the locally computed value.

A related but distinct question concerns observable sequencing. Pure data-flow
dependencies do not by themselves define the order of future effectful
operations. Memory management and explicit resource lifetime introduce further
concerns that must not be conflated with source-language naming or scope.

This ADR records an architectural direction. It is informative and does not
define normative MNIR semantics. Normative behavior remains the responsibility
of specifications under `docs/specification/`.

## Decision

### Architectural principle

> MNIR should express identity, data flow, effects, sequencing, and resource
> lifetime explicitly where they have semantic meaning, while avoiding
> presentation or implementation details becoming semantics when they do not
> need to be.

### Value identity

`ExpressionId` is the semantic identity of a locally computed value.

MNIR will not introduce a separate `LocalValueId`, `Bind`, or
`LocalReference` solely to represent source-language constructs such as a
`let` binding. Conceptually:

```text
let sum = a + b
```

may correspond to:

```text
E3 = Add(E1, E2)
preferred_name(E3) = "sum"
```

The exact presentation-metadata mechanism for Expression names is not decided
by this ADR.

### Names

Human-oriented local names are presentation information rather than semantic
identity. Preferred names:

- may be absent;
- may be duplicated;
- are not used for semantic references;
- do not affect semantic equality or verification.

Semantic references continue to use typed MNIR identifiers. Renaming a
presentation name does not change the referenced semantic identity.

Future source-language frontends such as EasyH are responsible for scope, name
resolution, unique source-language identifiers where required, shadowing
rules, and deterministic collision resolution during rendering. These frontend
responsibilities do not turn names into canonical MNIR identities.

### Pure Expressions

Pure Expressions remain a dependency DAG. Expression collection position stays
non-semantic; data dependencies carry the required relationships between pure
computed values. MNIR will not add semantic execution order to pure Expressions
merely to reproduce human `let` syntax.

### Parameters

The existing `ParameterReference` model remains in place. This decision does
not replace Parameters with entry-Block value definitions and does not
introduce a general cross-Block `ValueId`.

A broader SSA or value-flow model may be reconsidered if future semantics for
dominance, merge points, Function calls, or related capabilities require it.

### Sequencing

Observable sequencing is separate from pure value dependency. When MNIR gains
effectful operations, required ordering must be represented explicitly. Effect
ordering must not be inferred from:

- `HashMap` iteration;
- Expression collection order;
- incidental serialization order;
- presentation or source order.

This ADR does not select the future sequencing representation. Candidate
models to evaluate are:

1. ordered effectful operations per Block;
2. explicit effect dependencies;
3. effect-state or effect-token threading.

### Memory and lifetime direction

The architecture distinguishes three concerns:

1. **Value liveness.** Liveness of semantic values such as Expression results
   should be analyzable from value and control flow and should not require
   explicit `free` operations in normal MNIR.
2. **Managed runtime memory.** Future values such as Text, collections, and
   managed objects may require heap storage. The memory-management strategy may
   be backend- or runtime-specific; this ADR does not choose between tracing
   garbage collection, reference counting, regions, or other strategies.
3. **Explicit resources.** Resources such as `File`, `Socket`, `Lock`,
   `DatabaseTransaction`, and `SecretBuffer` should eventually have explicit
   semantic ownership and lifetime rules. Correctness must not depend solely on
   garbage collection, and future verification should be able to identify
   invalid resource lifecycle behavior.

### Scope is not runtime lifetime

Source-language scope and runtime resource lifetime are separate. A name
leaving EasyH scope does not automatically define the fundamental MNIR lifetime
of its underlying semantic value or resource.

## Rationale

The current model already gives a computed Expression a stable typed identity.
Reusing it as local value identity avoids redundant semantic structure and
keeps human naming separate from canonical references. A dependency DAG is
sufficient for the relationships between current pure Expressions and matches
the existing non-semantic collection-order model.

At the same time, future observable effects cannot safely depend on incidental
container or presentation order. Treating sequencing as an explicit future
design problem preserves room for a model that can express and verify the
ordering that effects actually require.

Separating liveness, managed memory, and explicit resources likewise prevents
source scope or one runtime's allocation strategy from becoming accidental
MNIR semantics.

## Consequences

- Future local-value work starts from `ExpressionId` rather than adding a
  parallel identity hierarchy.
- Frontends may render and parse local names, but canonical references remain
  identity-based.
- Renames remain presentation changes rather than semantic rewrites.
- Pure Expression storage and serialization do not acquire execution-order
  meaning.
- Effectful Function calls cannot be specified safely until an explicit
  sequencing model is defined or selected.
- Future cross-Block value flow may require an additional architectural or
  normative decision; this ADR does not preclude one.
- Resource lifetime will require semantics distinct from both value liveness
  and managed-memory reclamation.

## Alternatives considered

### Separate `LocalValueId` / `Bind` / `LocalReference`

Rejected for the current direction. It would introduce a redundant identity
layer and extra indirection when `ExpressionId` already identifies the computed
value. It would also risk making source naming define canonical semantic
structure, contrary to the separation between presentation and identity.

### Ordered SSA-style instruction list for all Expressions

Not selected now. Ordered SSA instruction sequences are an established and
valid representation, but current MNIR Expressions are pure and their
collection order is explicitly non-semantic. Imposing order on all pure
Expressions would add semantics that are not presently required.

This alternative is not permanently rejected. Future value-flow or effect
requirements may justify reconsidering it.

### Pure Expression DAG plus explicit future effect sequencing

Selected as the current architectural direction. Pure computations retain the
existing dependency-DAG model, while any observable effect ordering will be
represented explicitly by a future sequencing model. The exact sequencing
representation remains deferred.

## Deferred questions

- What is the exact representation of effectful sequencing?
- Will MNIR introduce effect tokens or explicit effect dependencies?
- How will cross-Block value references be represented?
- What dominance rules will apply to cross-Block values?
- Will merge points use Block parameters, phi-like constructs, or another
  model?
- How will mutable state be represented?
- How will Function-call effects be declared and sequenced?
- What ownership and lifetime model will explicit resources use?
- Which managed-memory semantics, if any, belong in MNIR rather than a backend?
- What liveness analyses will be required or normative?
- What naming, scope, shadowing, and collision rules will EasyH use?
- How will Expression presentation metadata or a presentation side table be
  represented?
