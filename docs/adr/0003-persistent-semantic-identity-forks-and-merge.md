# ADR 0003: Persistent Semantic Identity, Forks, and Merge

## Status

Accepted

## Context

MNIR currently scopes persistent semantic entity identities to a Program
lineage. Several normative specifications therefore permit a fork to preserve
or remap raw identifiers while treating an inherited entity as a different
semantic identity because the fork has a different `ProgramId`. The current
model also retains full committed identifier-allocation histories so that
deleted identifiers are never reused within a lineage.

That model was sufficient while Programs evolved only as isolated in-memory
lineages. It is not a suitable foundation for persistent serialization,
independent offline forks, packages, semantic diff, or merge. If an entity's
identity changes merely because a Program is forked, two branches cannot
recognize inherited entities as the same semantic entities without remapping
tables or additional ancestry-dependent interpretation.

MNIR now needs an identity model in which an entity can survive process and
storage boundaries and retain its identity while different lineages evolve it
independently. The model must also let independent writers allocate new IDs
without online coordination and without realistic collision risk.

This ADR records the architectural direction. It is informative and does not
itself change normative MNIR semantics. A follow-up normative specification
will explicitly supersede the affected rules identified below before the
implementation changes.

## Decision

### Persistent entity identity

Persistent semantic entity IDs remain stable across:

- process restarts;
- serialization and deserialization;
- snapshots;
- forks;
- future semantic diff; and
- future semantic merge.

Existing semantic entities inherited through a fork preserve their IDs. A
normal fork does not remap inherited semantic entity IDs.

This decision applies to all currently defined persistent semantic entity ID
categories:

```text
ModuleId
FunctionId
ParameterId
BlockId
ExpressionId
```

It is also the default architectural rule for future persistent semantic
entity ID categories unless a later ADR and specification establish a concrete
reason for different behavior. Typed ID categories remain distinct. For
example, equal namespace and counter components do not make a `FunctionId` and
an `ExpressionId` interchangeable.

`ProgramId` and `RevisionId` retain their separate roles described below; they
are not entity ID categories governed by this construction.

### Identity construction

The selected architectural model is equivalent to:

```text
EntityId = AllocationNamespaceId + MonotonicCounter
```

The exact binary and wire encoding is deferred. The model does not require
UUID textual syntax.

An `AllocationNamespaceId` must provide sufficient collision resistance for
independent offline allocation across processes and machines without
coordination. The architectural target is approximately 128 bits of
collision-resistant namespace identity, or equivalent collision strength.
Within a namespace, a monotonically advancing counter provides compact and
deterministic local allocation.

The namespace and counter are identity components, not business data.
Consumers must not infer creation time, ownership hierarchy, trust,
presentation order, or business meaning from them. Counter order is an
allocator property and does not establish semantic entity order.

### Allocation namespace is not Program identity

An allocation namespace records where an identity was minted. It does not
identify the Program that currently contains the entity.

A Program may contain semantic entities originating from multiple allocation
namespaces as a result of fork ancestry and, in the future, merge, import, or
packages. Therefore these concepts must remain distinct:

```text
ProgramId
    = identity of a Program lineage

AllocationNamespaceId
    = minting domain for persistent semantic entity IDs
```

The presence of an entity from namespace `A` in a Program does not give that
Program authority to allocate another identity in namespace `A`.

### Namespace ownership invariant

The foundational allocation invariant is:

> A Program lineage MUST NOT allocate new semantic entity identities in an
> allocation namespace that it does not own for allocation.

Inherited or imported namespaces may be represented in Program state, but
they are not allocation authorities for that Program lineage.

A newly created Program lineage owns an allocation namespace. A fork:

1. receives a new `ProgramId`;
2. preserves all inherited persistent entity IDs unchanged; and
3. obtains its own allocation namespace for identities created after the
   fork.

Conceptually:

```text
original lineage:
    existing Function A:42
    new Function      A:57

fork lineage:
    inherited Function A:42
    new Function       B:1
```

`A:42` denotes the same persistent semantic Function identity in both
lineages. The two immutable or evolving Program states may contain different
versions of that entity without changing its identity. `B:1` is a distinct
entity minted by the fork.

The follow-up specification will decide the exact representation of allocation
authority and whether a lineage may ever rotate or own more than one active
allocation namespace. Such a mechanism must preserve the invariant above.

### Coordination-free independent forks

Independent offline forks must be able to allocate new semantic IDs without
coordination and without realistic collision risk:

```text
common ancestor:
    Function A:42

fork 1:
    inherited Function A:42
    new Function       B:1

fork 2:
    inherited Function A:42
    new Function       C:1
```

The collision-resistant namespace construction makes `B != C` overwhelmingly
likely even when both forks are created on different machines without a shared
allocator.

### ProgramId, snapshots, revisions, and VerifiedProgram

`ProgramId` remains the identity of a Program lineage. It does not form part
of the persistent identity of a contained semantic entity.

Identity reasoning therefore moves away from:

```text
(ProgramId, FunctionId)
```

and toward:

```text
FunctionId(AllocationNamespaceId, MonotonicCounter)
```

The same applies to the other persistent entity ID categories. An inherited
entity remains the same entity even when the containing fork has a different
`ProgramId`.

A snapshot remains an immutable observation of a revision in the same Program
lineage. It preserves `ProgramId`, `RevisionId`, entity IDs, semantic
references, and allocation state needed for its defined use.

A fork is a new Program lineage and receives a new `ProgramId`. Its initial
state preserves inherited entity IDs and references exactly, while its new
allocation namespace prevents independently added entities from colliding
with additions in the source lineage or sibling forks.

`RevisionId` continues to identify a committed state within a Program lineage.
Revision ancestry and revision identity remain separate from contained entity
identity. Editing an entity creates a new Program revision but normally
preserves that entity's ID.

`VerifiedProgram` remains evidence bound to the exact `ProgramId`,
`RevisionId`, immutable snapshot, and verification rule set that were
verified. Fork-preserved entity IDs do not let verification evidence for one
lineage or revision imply verification of another lineage or revision.

### Allocation state and non-reuse

The current architecture retains full committed-ID sets to prevent reuse.
Under the selected model, non-reuse for identities minted by an owned
namespace can instead be guaranteed by durable monotonic allocator state,
conceptually:

```text
AllocationNamespaceState {
    namespace_id
    next_counter
}
```

The exact in-memory and persisted representation is deferred. Once the model
is normatively specified and implemented, explicit full historical-ID sets may
be unnecessary solely for non-reuse.

Deletion does not roll back the counter and does not make an ID reusable.
Issuing an identity provisionally also must not cause the namespace counter to
move backward when a transaction fails or is discarded. The normative
specification must define how allocator advancement is persisted and how it
interacts with transaction atomicity and crashes, but it must not reintroduce
reuse of an issued namespace/counter pair.

Allocation state is not entity membership. A counter gap is valid and conveys
no semantic meaning. No-op and revision rules must distinguish semantic
Program contents from allocator-state advancement explicitly rather than
depending on historical-ID set equality.

### Merge motivation

Persistent, fork-preserved identities are foundational for future semantic
diff and merge. Given:

```text
ancestor:
    Function A:42

branch 1:
    modifies Function A:42

branch 2:
    modifies Function A:42
```

a semantic merge system can recognize that both branches modified the same
Function. Independent additions remain distinct without remapping:

```text
branch 1:
    Function B:1

branch 2:
    Function C:1
```

This ADR does not define diff algorithms, merge algorithms, conflicts,
ancestry representation, or merge resolution.

### Canonical serialization consequence

Future canonical serialization must preserve persistent semantic identity and
the allocation state required for continued safe mutation.

Canonical determinism means:

> The same MNIR identity state must serialize canonically and deterministically
> independent of process, machine, `HashMap` iteration order, or load history.

It does not mean that independently constructed, behaviorally equivalent
Programs with different entity identities must produce identical serialized
bytes.

The architecture therefore distinguishes conceptually between:

```text
identity equivalence
semantic or behavioral equivalence
```

This ADR does not normatively define either equivalence relation. In
particular, identity-sensitive canonicalization and a future behavior-oriented
comparison serve different purposes.

Canonical MNIR storage does not need to be human-readable text merely to work
well with Git. The preferred future integration sequence is:

```text
Canonical MNIR serialization
    ↓
Git storage
    ↓
Git textconv projection
    ↓
human-readable diffs
    ↓
MNIR semantic diff
    ↓
semantic merge driver later
```

Git `textconv` is an intended presentation technique for readable diffs. This
ADR does not select the canonical serialization format or normatively specify
Git configuration.

### Import, packages, and the trust boundary

Collision-resistant allocation namespace identity provides uniqueness
properties. It does not provide trust, provenance, or authority.

An imported package or external MNIR artifact may claim foreign entity IDs or
allocation namespaces. Random-looking or collision-resistant namespace data
does not prove that the claimant minted those identities legitimately.

Malicious identity claims, namespace provenance, signatures, trusted
publishers, package integrity, and namespace authorization are outside this
ADR. Future package, import, and security semantics must define that trust
boundary explicitly.

## Rationale

Stable identity across forks gives semantic tools a direct way to recognize
common inherited entities. A collision-resistant namespace permits offline
lineages to create independent allocation domains, while a monotonic counter
keeps allocation within each domain compact and straightforward. Separating
namespace origin from Program ownership also permits a future merged or
importing Program to contain identities from several origins without claiming
allocation authority over them.

The design preserves typed IDs, normal update identity, Program lineage
identity, revision-bound verification, and immutable snapshots. It changes the
composition and allocation domain of persistent entity identity rather than
collapsing those distinct concepts.

## Consequences

- Normal forks preserve inherited `ModuleId`, `FunctionId`, `ParameterId`,
  `BlockId`, and `ExpressionId` identities without remapping.
- A fork still has a new `ProgramId` and independent revision history.
- Each lineage allocates new entity IDs only through a namespace for which it
  has allocation authority.
- Programs may contain IDs minted by multiple namespaces.
- Independent offline forks can allocate without a shared counter service.
- Entity ID representations become larger than the current compact
  implementation-defined raw values.
- Full historical-ID sets may be removed when durable monotonic namespace
  state provides the required non-reuse guarantee.
- Allocator advancement may survive failed or discarded transactions and can
  produce harmless counter gaps.
- Snapshot, fork, serialization, diff, and merge implementations must preserve
  entity IDs and all references to them.
- Semantic merge gains stable correspondence for inherited entities, but merge
  policy and conflict handling remain future work.
- Canonical serialization is identity-sensitive; behavioral equivalence is a
  separate future concern.
- Namespace collision resistance does not establish artifact trust.

## Alternatives considered

### Program-lineage-scoped entity IDs

This is the current normative model. It is not selected for persistent future
identity because an inherited entity becomes a different semantic identity in
each fork. Semantic diff and merge then require remapping or ancestry side
information merely to recover entity correspondence.

### Fork may preserve or remap IDs

Several current specifications permit this. It is rejected for normal forks.
Normal fork semantics preserve inherited entity IDs. Explicit remapping may
remain available to a future, separately specified import, copy, or
transformation operation, but it is not normal fork behavior.

### Globally random ID for every entity

This is a valid architecture and is not inherently incorrect. It naturally
supports coordination-free allocation, but requires random material in every
entity ID and loses the compact monotonic allocation structure available
inside a namespace. The namespace-plus-counter design provides one
collision-resistant origin identifier followed by deterministic local
allocation.

### Allocation namespace plus monotonic counter

Selected. It combines coordination-free namespace creation with compact,
deterministic allocation within a namespace and supports non-reuse without a
full set of every previously committed ID.

### Content-addressed semantic IDs

Not selected. Persistent entity identity must survive ordinary semantic edits.
An ID derived from entity contents would change whenever those contents change
and would turn mutation into identity replacement.

## Normative impact audit

The audit covered every normative document from `01` through `09`, including
every rule and acceptance-requirement section, and searched specifically for
Program-lineage scope, raw-value equality, allocation and provisional
allocation, retirement and reuse, snapshots, forks and remapping, reference
preservation, no-op state, `ProgramId`, and verification evidence.

The labels below mean:

- **Supersede**: the present rule conflicts with this decision and must be
  replaced explicitly.
- **Revise**: the requirement remains materially valid but its identity domain,
  allocator, fork wording, or conformance evidence must change.
- **Review/retain**: the rule is identity-adjacent and must be checked and
  carried forward without accidentally changing its existing semantic role.

The future normative specification is explicitly authorized to supersede and
revise the listed earlier rules. Until that specification is accepted, all
current normative text remains authoritative.

### `01-program-model.md`

Rules:

- **Supersede:** `MNIR-CORE-004`, `MNIR-CORE-011`, `MNIR-CORE-013`,
  `MNIR-CORE-015`, `MNIR-CORE-054` through `MNIR-CORE-057`,
  `MNIR-CORE-017`, `MNIR-CORE-034`, `MNIR-CORE-064` through
  `MNIR-CORE-066`, and item 6 of `MNIR-CORE-040`.
- **Revise:** `MNIR-CORE-001`, `MNIR-CORE-002`, `MNIR-CORE-006` through
  `MNIR-CORE-010`, `MNIR-CORE-012`, `MNIR-CORE-016`, `MNIR-CORE-018`,
  `MNIR-CORE-019`, `MNIR-CORE-033`, `MNIR-CORE-038`, and
  `MNIR-CORE-039` together with the section 3.8 no-op definition.
- **Review/retain:** `MNIR-CORE-005`, `MNIR-CORE-014`,
  `MNIR-CORE-020` through `MNIR-CORE-024`, `MNIR-CORE-026` through
  `MNIR-CORE-031`, `MNIR-CORE-053`, `MNIR-CORE-058`, and
  `MNIR-CORE-059`, together with `MNIR-CORE-041` and
  `MNIR-CORE-042`.

Acceptance requirements:

- **Revise:** `AR-CORE-001`, `AR-CORE-002`, `AR-CORE-004`,
  `AR-CORE-005`, `AR-CORE-007`, `AR-CORE-009`, and `AR-CORE-013`
  through `AR-CORE-015`.
- **Review/retain:** `AR-CORE-003`, `AR-CORE-006`, and `AR-CORE-012`.

### `02-type-system-foundations.md`

Intrinsic types are global semantic values rather than persistent Program
entities, so this ADR does not add `AllocationNamespaceId` to them.

- **Review/retain rules:** `MNIR-TYPE-003` through `MNIR-TYPE-007`,
  `MNIR-TYPE-019`, `MNIR-TYPE-020`, and `MNIR-TYPE-022`.
- **Review/retain acceptance requirements:** `AR-TYPE-002` through
  `AR-TYPE-004` and `AR-TYPE-012`.

### `03-functions-and-parameters.md`

Rules:

- **Supersede:** `MNIR-FUNC-002`, `MNIR-FUNC-005`, `MNIR-FUNC-011`,
  `MNIR-FUNC-014`, `MNIR-FUNC-015`, `MNIR-FUNC-035` through
  `MNIR-FUNC-037`, `MNIR-FUNC-040`, `MNIR-FUNC-041`,
  `MNIR-FUNC-064`, `MNIR-FUNC-065`, `MNIR-FUNC-079`,
  `MNIR-FUNC-082`, and `MNIR-FUNC-083`.
- **Revise:** `MNIR-FUNC-001`, `MNIR-FUNC-003`, `MNIR-FUNC-004`,
  `MNIR-FUNC-010`, `MNIR-FUNC-012`, `MNIR-FUNC-013`,
  `MNIR-FUNC-038`, `MNIR-FUNC-039`, `MNIR-FUNC-042`,
  `MNIR-FUNC-043`, `MNIR-FUNC-063`, and `MNIR-FUNC-080`.
- **Review/retain:** `MNIR-FUNC-006` through `MNIR-FUNC-009`,
  `MNIR-FUNC-016` through `MNIR-FUNC-019`, `MNIR-FUNC-022`,
  `MNIR-FUNC-028`, `MNIR-FUNC-052` through `MNIR-FUNC-054`,
  `MNIR-FUNC-057`, `MNIR-FUNC-070`, and `MNIR-FUNC-075`.

Acceptance requirements:

- **Revise:** `AR-FUNC-001` through `AR-FUNC-006`, `AR-FUNC-008` through
  `AR-FUNC-015`, `AR-FUNC-021`, `AR-FUNC-022`, `AR-FUNC-027` through
  `AR-FUNC-030`, `AR-FUNC-033`, `AR-FUNC-034`, and `AR-FUNC-035`.
- **Review/retain:** `AR-FUNC-026`.

### `04-expressions-and-basic-function-bodies.md`

Rules:

- **Supersede:** `MNIR-EXPR-005`, `MNIR-EXPR-007`, `MNIR-EXPR-012`,
  `MNIR-EXPR-014`, `MNIR-EXPR-054`, `MNIR-EXPR-056`,
  `MNIR-EXPR-061` through `MNIR-EXPR-065`, `MNIR-EXPR-067`,
  `MNIR-EXPR-097` through `MNIR-EXPR-099`, and `MNIR-EXPR-080`.
- **Revise:** `MNIR-EXPR-004`, `MNIR-EXPR-006`, `MNIR-EXPR-011`,
  `MNIR-EXPR-013`, `MNIR-EXPR-044`, `MNIR-EXPR-047`,
  `MNIR-EXPR-048`, `MNIR-EXPR-059`, `MNIR-EXPR-060`,
  `MNIR-EXPR-066`, `MNIR-EXPR-072`, and `MNIR-EXPR-079`.
- **Review/retain:** `MNIR-EXPR-017`, `MNIR-EXPR-035`,
  `MNIR-EXPR-039`, `MNIR-EXPR-050`, `MNIR-EXPR-071`,
  `MNIR-EXPR-085`, and `MNIR-EXPR-101`.

Acceptance requirements:

- **Revise:** `AR-EXPR-001`, `AR-EXPR-004` through `AR-EXPR-008`,
  `AR-EXPR-018` through `AR-EXPR-025`, `AR-EXPR-027`,
  `AR-EXPR-034`, `AR-EXPR-035`, `AR-EXPR-039`, `AR-EXPR-040`, and
  `AR-EXPR-044`.
- **Review/retain:** `AR-EXPR-026`, `AR-EXPR-029`, and
  `AR-EXPR-041` through `AR-EXPR-043`.

### `05-arithmetic-expressions.md`

Rules:

- **Supersede:** the remapping alternative in `MNIR-ARITH-065`.
- **Revise:** `MNIR-ARITH-003`, `MNIR-ARITH-013`,
  `MNIR-ARITH-064` through `MNIR-ARITH-067`, and the identity clause of
  `MNIR-ARITH-069`.
- **Review/retain:** `MNIR-ARITH-062`, `MNIR-ARITH-063`, and
  `MNIR-ARITH-068`.

Acceptance requirements:

- **Revise:** `AR-ARITH-001` through `AR-ARITH-006` and
  `AR-ARITH-017` through `AR-ARITH-020`.
- **Review/retain:** `AR-ARITH-013`, `AR-ARITH-016`, and
  `AR-ARITH-021`.

### `06-semantic-verification-and-diagnostics.md`

The verification rule set does not allocate entity IDs, but its snapshot,
subject, and evidence bindings must remain precise under the new identity
model.

Rules:

- **Review/retain:** `MNIR-VERIFY-004`, `MNIR-VERIFY-005`,
  `MNIR-VERIFY-009` through `MNIR-VERIFY-016`, `MNIR-VERIFY-023`
  through `MNIR-VERIFY-026`, `MNIR-VERIFY-030`, `MNIR-VERIFY-032`,
  `MNIR-VERIFY-034`, `MNIR-VERIFY-056` through `MNIR-VERIFY-059`, and
  `MNIR-VERIFY-086` through `MNIR-VERIFY-089`. These rules continue to
  bind evidence to a Program lineage and revision while diagnostic subjects
  use the revised persistent entity IDs.

Acceptance requirements:

- **Review/retain:** `AR-VERIFY-003`, `AR-VERIFY-014` through
  `AR-VERIFY-017`, `AR-VERIFY-021` through `AR-VERIFY-023`,
  `AR-VERIFY-033`, `AR-VERIFY-035`, and `AR-VERIFY-036`.

### `07-comparison-expressions.md`

Rules:

- **Supersede:** the remapping alternative in `MNIR-CMP-058`.
- **Revise:** `MNIR-CMP-006`, `MNIR-CMP-018`, `MNIR-CMP-057`,
  `MNIR-CMP-059`, `MNIR-CMP-060`, and the identity clause of
  `MNIR-CMP-061`.
- **Review/retain:** `MNIR-CMP-016` and `MNIR-CMP-017`.

Acceptance requirements:

- **Revise:** `AR-CMP-018` through `AR-CMP-020`.
- **Review/retain:** `AR-CMP-014` through `AR-CMP-017`.

### `08-conditional-control-flow.md`

Rules:

- **Supersede:** `MNIR-CFG-068`, the remapping alternative in
  `MNIR-CFG-071`, and the committed allocation-history clause of
  `MNIR-CFG-072`.
- **Revise:** `MNIR-CFG-003`, `MNIR-CFG-006`, `MNIR-CFG-008`,
  `MNIR-CFG-010`, `MNIR-CFG-014`, `MNIR-CFG-066`, `MNIR-CFG-067`,
  `MNIR-CFG-069`, `MNIR-CFG-070`, `MNIR-CFG-111`, and
  `MNIR-CFG-124`.
- **Review/retain:** `MNIR-CFG-063` through `MNIR-CFG-065` and
  `MNIR-CFG-112`.

Acceptance requirements:

- **Revise:** `AR-CFG-002`, `AR-CFG-021` through `AR-CFG-023`, and
  `AR-CFG-039`.
- **Review/retain:** `AR-CFG-003`, `AR-CFG-004`, `AR-CFG-019`, and
  `AR-CFG-040`.

### `09-function-calls-and-sequencing-foundations.md`

Rules:

- **Supersede:** the remapping alternative in `MNIR-CALL-079`.
- **Revise:** `MNIR-CALL-024`, `MNIR-CALL-077`, `MNIR-CALL-078`,
  `MNIR-CALL-080` through `MNIR-CALL-082`, and `MNIR-CALL-138`.
- **Review/retain:** `MNIR-CALL-004`, `MNIR-CALL-006` through
  `MNIR-CALL-012`, `MNIR-CALL-083`, `MNIR-CALL-084`, and
  `MNIR-CALL-127`.

Acceptance requirements:

- **Revise:** `AR-CALL-001`, `AR-CALL-030` through `AR-CALL-032`, and
  `AR-CALL-057`.
- **Review/retain:** `AR-CALL-003`, `AR-CALL-004`, `AR-CALL-009`,
  `AR-CALL-012`, `AR-CALL-013`, `AR-CALL-026`, `AR-CALL-033`,
  `AR-CALL-034`, `AR-CALL-047`, `AR-CALL-053`, `AR-CALL-054`,
  `AR-CALL-056`, `AR-CALL-058`, and `AR-CALL-059`.

No normative rule or acceptance requirement in `00-introduction.md` defines
entity identity, fork allocation, allocation history, or identifier reuse.
Rules and acceptance requirements in `01` through `09` not listed above were
reviewed and do not constrain the architecture changed by this ADR.

## Follow-up normative specification

ADR 0003 will be followed by a new normative specification, tentatively named:

```text
Persistent Semantic Identity 0.1
```

That specification will:

- explicitly supersede affected earlier identity and fork rules;
- define the namespace/counter identity model normatively;
- define namespace ownership and allocation authority;
- define fork allocation behavior;
- define persistent non-reuse behavior, including provisional allocation;
- define compatibility with snapshots and revisions;
- update affected acceptance requirements; and
- drive the subsequent implementation refactor.

Canonical serialization remains a later specification and implementation
increment.

## Deferred questions

- What exact binary and wire representation will IDs use?
- How is an `AllocationNamespaceId` generated, validated, and persisted?
- Does a lineage own exactly one allocation namespace for its lifetime, or may
  it rotate namespaces under explicitly defined conditions?
- How is namespace allocation authority represented without confusing it with
  namespace provenance or trust?
- When and how is counter advancement durably recorded relative to a mutation
  transaction and process failure?
- What counter width and exhaustion behavior are required?
- How are legacy lineage-scoped IDs migrated into persistent namespaced IDs?
- Which explicit import or transformation operations may remap identity?
- How are revision ancestry, semantic diff, merge conflicts, and merge commits
  represented?
- What are the normative definitions of identity equivalence and behavioral
  equivalence?
- What canonical serialization format is selected?
- What package security mechanism validates claimed foreign namespaces and
  entity identities?
