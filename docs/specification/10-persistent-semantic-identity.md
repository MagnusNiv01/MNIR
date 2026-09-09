# MNIR Specification — Persistent Semantic Identity

**Document:** `10-persistent-semantic-identity.md`

**Specification status:** Draft

**Specification version:** 0.1

**Normative:** Yes

---

## 1. Purpose

This document defines persistent semantic identity for the entity categories
introduced by specifications `01` through `09`.

It defines:

- `AllocationNamespaceId`;
- typed entity IDs composed from an allocation namespace and counter;
- coordination-free namespace and Program identity creation;
- namespace allocation authority;
- persistent non-reuse;
- identity issuance during mutation transactions;
- separation between semantic revision state and allocator state;
- snapshot identity and allocator-state preservation;
- normal fork identity and allocation behavior;
- compatibility with existing semantic references and verifier rule sets; and
- explicit supersession of earlier lineage-scoped identity, allocation-history,
  and fork-remapping rules.

This document does not define:

- canonical serialization encoding;
- a textual identifier syntax;
- import or package semantics;
- transfer of allocation authority;
- namespace provenance or authorization;
- signatures or trusted publishers;
- semantic diff algorithms;
- merge algorithms or conflict resolution;
- behavioral or whole-Program semantic equivalence;
- new semantic entity categories;
- a new verification rule set; or
- runtime execution behavior.

---

# 2. Normative language

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are
normative only when they occur inside a numbered MNIR rule.

Normative requirements in this document use identifiers of the form:

```text
MNIR-PSI-NNN
```

Acceptance requirements use identifiers of the form:

```text
AR-PSI-NNN
```

Acceptance requirements demonstrate normative behavior but do not independently
introduce MNIR semantics.

---

# 3. Authority and supersession

## MNIR-PSI-001 — Supersession authority

Persistent Semantic Identity 0.1 supersedes the identity, allocation-history,
provisional-allocation, no-op-comparison, and normal-fork clauses explicitly
listed in section 24.

When a listed earlier rule conflicts with this document, this document MUST
take precedence for a Program claiming conformance with Persistent Semantic
Identity 0.1.

Unlisted portions of earlier rules remain normative.

---

## MNIR-PSI-002 — Existing semantic behavior remains applicable

Except where section 24 explicitly states otherwise, the semantic and
structural behavior defined by specifications `01` through `09` MUST remain
unchanged.

Changing an identifier representation MUST NOT silently change ownership,
reference integrity, collection ordering, transaction atomicity, CFG,
EffectSequence, type inspection, diagnostic, or verification semantics.

---

# 4. Terminology

## 4.1 Persistent semantic entity

A **persistent semantic entity** is an independently identified MNIR entity
whose typed identity survives snapshots, process and persistence boundaries,
and normal forks.

Version 0.1 applies this term to:

```text
Module
Function
Parameter
Block
Expression
```

---

## 4.2 Allocation namespace

An **allocation namespace** is a persistent identity domain in which entity
counter values are issued.

Its identity is represented by `AllocationNamespaceId`.

---

## 4.3 Namespace presence

An allocation namespace is **present** in Program contents when at least one
contained persistent entity ID uses that namespace.

Namespace presence does not imply allocation authority.

---

## 4.4 Allocation authority

**Allocation authority** is the authority held by one mutable Program lineage
to issue new entity IDs in one allocation namespace.

Allocation authority is lineage metadata. It is not ownership of every entity
whose ID uses that namespace.

---

## 4.5 Issued identity

An entity identity is **issued** when its namespace/counter pair has been
successfully reserved by an allocation authority according to
`MNIR-PSI-031` through `MNIR-PSI-034`.

An issued identity may still be provisional with respect to committed Program
contents.

---

## 4.6 Semantic revision state

**Semantic revision state** is the committed Program content identified by a
`ProgramId` and `RevisionId`, including the semantic structures defined by the
applicable specifications.

Allocation-authority state is not semantic revision state.

---

# 5. Persistent typed entity IDs

## MNIR-PSI-003 — Covered entity ID categories

Persistent Semantic Identity 0.1 MUST apply to exactly the following currently
defined persistent entity ID categories:

```text
ModuleId
FunctionId
ParameterId
BlockId
ExpressionId
```

This document MUST NOT introduce another persistent semantic entity category.

---

## MNIR-PSI-004 — Typed categories remain distinct

`ModuleId`, `FunctionId`, `ParameterId`, `BlockId`, and `ExpressionId` MUST
remain distinct semantic identifier categories.

An ID from one category MUST NOT be accepted where another category is
required merely because their namespace and counter components are equal.

---

## MNIR-PSI-005 — AllocationNamespaceId is distinct

`AllocationNamespaceId` MUST be a distinct persistent identifier category.

It MUST NOT be interchangeable with:

```text
ProgramId
RevisionId
ModuleId
FunctionId
ParameterId
BlockId
ExpressionId
```

---

## MNIR-PSI-006 — Entity ID components

Every covered entity ID MUST consist semantically of exactly:

```text
namespace: AllocationNamespaceId
counter: MonotonicCounter
```

The exact Rust, binary, and wire representations are implementation-defined by
this specification.

Once issued, both components MUST remain stable across every supported
persistence, process-restart, snapshot, and normal-fork boundary.

---

## MNIR-PSI-007 — Same typed ID means same identity

Two entity IDs of the same typed category with equal namespace and counter
components MUST denote the same persistent semantic entity identity.

This remains true when the IDs appear in different snapshots or forked Program
lineages.

---

## MNIR-PSI-008 — Different typed ID means different identity

Two IDs MUST denote different persistent semantic entity identities when:

1. their typed categories differ; or
2. their typed categories are equal but either namespace or counter differs.

---

## MNIR-PSI-009 — Entity identity is not lineage-scoped

The persistent identity of a covered entity MUST NOT be scoped by Program
lineage.

`ProgramId` MUST NOT form part of `ModuleId`, `FunctionId`, `ParameterId`,
`BlockId`, or `ExpressionId` identity.

---

## MNIR-PSI-010 — Identity-preserving updates

An update operation defined by an earlier specification as preserving an
entity ID MUST continue to preserve the complete typed namespace/counter ID.

Presentation changes, type changes, and other previously identity-preserving
updates MUST NOT mint a replacement entity ID.

---

## MNIR-PSI-011 — Identifier opacity

Consumers MUST NOT infer semantic order, creation time, ownership, containment,
revision, execution order, trust, provenance, or business meaning from an
entity ID's namespace or counter.

Counter comparison is allocator state and MUST NOT establish semantic entity
order.

---

# 6. AllocationNamespaceId

## MNIR-PSI-012 — Coordination-free namespace creation

Creation of a new `AllocationNamespaceId` MUST be possible independently
across processes and machines without a central allocator, network service,
registry, shared counter, or prior coordination.

---

## MNIR-PSI-013 — Namespace collision resistance

The `AllocationNamespaceId` creation mechanism MUST provide collision
resistance appropriate to approximately 128 bits of random namespace entropy
or equivalent strength.

An implementation MUST NOT use a process-local sequential counter alone as an
`AllocationNamespaceId` generation mechanism.

If an implementation detects that a newly generated namespace collides with a
distinct known allocation namespace, it MUST reject that candidate and either
generate another candidate or fail namespace creation. It MUST NOT combine the
two allocation authorities.

---

## MNIR-PSI-014 — Namespace persistence

Once created, an `AllocationNamespaceId` MUST remain stable across every
supported persistence and process-restart boundary.

Reloading a lineage MUST NOT replace its allocation namespace merely because a
new process or machine performs the load.

---

## MNIR-PSI-015 — Namespace equality is Program-independent

`AllocationNamespaceId` equality MUST be independent of `ProgramId` and
`RevisionId`.

The same namespace may be present in multiple Program lineages through
inherited entity IDs without becoming a different namespace.

---

## MNIR-PSI-016 — No UUID text requirement

Persistent Semantic Identity 0.1 MUST NOT require UUID textual syntax or any
other textual namespace syntax.

An implementation MAY internally use a representation with the required
collision properties, but that choice does not define MNIR text or canonical
serialization.

---

## MNIR-PSI-017 — Namespace identity does not establish trust

An `AllocationNamespaceId` MUST NOT be treated as proof of namespace
provenance, allocation authorization, artifact integrity, package publisher
identity, or trust.

---

# 7. ProgramId role and creation

## MNIR-PSI-018 — ProgramId remains lineage identity

`ProgramId` MUST continue to identify one Program lineage independently of its
current revision.

It MUST remain distinct from `AllocationNamespaceId` and from all covered
entity ID categories.

---

## MNIR-PSI-019 — Persistent ProgramId generation

Creation of a new Program lineage, including a normal fork, MUST create a
`ProgramId` with collision resistance suitable for independent creation across
processes and machines without coordination.

The collision resistance MUST be appropriate to approximately 128 bits of
random identity entropy or equivalent strength.

A process-local sequential counter alone MUST NOT satisfy this requirement.

---

## MNIR-PSI-020 — ProgramId persistence

Once created, a `ProgramId` MUST remain stable across supported persistence,
deserialization, process-restart, and machine-transfer boundaries for that
lineage.

---

## MNIR-PSI-021 — Program identity collision handling remains required

The collision detection and rejection behavior of `MNIR-CORE-008` and
`MNIR-CORE-053` MUST continue to apply.

Collision-resistant generation reduces collision probability; it MUST NOT
turn a detected collision into evidence of lineage identity.

---

# 8. Namespace allocation authority

## MNIR-PSI-022 — One active allocation authority

Every mutable Program lineage conforming to Persistent Semantic Identity 0.1
MUST hold exactly one active entity-allocation authority.

Namespace rotation and multiple simultaneously active allocation namespaces
for one lineage are outside version 0.1.

---

## MNIR-PSI-023 — Minimum allocation-authority state

The minimum persistent allocation-authority state MUST be semantically
equivalent to:

```text
AllocationAuthorityState {
    namespace_id: AllocationNamespaceId
    counter_state: CounterState
}

CounterState =
    Available(next_counter: MonotonicCounter)
    | Exhausted
```

`Available(next_counter)` means that `next_counter` is the current counter
value that the authority will reserve for its next successful issuance. It
MUST be greater than every counter already issued by that authority.

`Exhausted` means that the authority has issued the last representable counter
and no counter remains available for issuance.

The exact in-memory representation is implementation-defined.

---

## MNIR-PSI-024 — Allocation-authority persistence

Allocation-authority state MUST be preserved across every supported
persistence or process-restart boundary that permits continued mutation of the
same Program lineage.

Both `Available(next_counter)` and `Exhausted` are persistent counter states
and MUST be preserved exactly.

Process-local allocator state alone is insufficient.

---

## MNIR-PSI-025 — Presence and authority are distinct

A Program MAY contain entities whose IDs originate from multiple allocation
namespaces.

Presence of an entity from a namespace MUST NOT grant the containing lineage
allocation authority for that namespace.

---

## MNIR-PSI-026 — Allocation only through owned authority

A Program lineage MUST NOT issue a new entity ID in an allocation namespace
other than the namespace of its active allocation authority.

Inherited or future imported namespace presence MUST NOT be used as allocation
authority.

---

## MNIR-PSI-027 — No authority transfer semantics

Persistent Semantic Identity 0.1 MUST NOT define transfer of allocation
authority between lineages, delegation to another lineage, sharing with a
different lineage, import, or package distribution of allocation authority.

An implementation MUST NOT infer authority transfer from copying Program
contents or entity IDs.

Restoring the same mutable lineage according to `MNIR-PSI-024` and
`MNIR-PSI-058` through `MNIR-PSI-060` is continuation of its existing
authority, not transfer to another lineage.

---

# 9. Program creation

## MNIR-PSI-028 — New Program lineage state

A newly created Program lineage MUST receive:

1. a new `ProgramId` satisfying `MNIR-PSI-019`;
2. a newly created `AllocationNamespaceId` satisfying
   `MNIR-PSI-012` through `MNIR-PSI-014`; and
3. active allocation-authority state for that namespace.

---

## MNIR-PSI-029 — Initial counter state

The initial counter state MUST be `Available(first_counter)`, where
`first_counter` permits issuance of at least one valid entity ID and represents
that no counter has yet been issued by the new authority.

The concrete first valid counter value is implementation-defined.

---

## MNIR-PSI-030 — Independent Program creation

Two Programs created independently without coordination MUST receive distinct
`ProgramId` values from one another and distinct active
`AllocationNamespaceId` values from one another with overwhelming probability
under the collision requirements of this specification.

---

# 10. Counter allocation and issuance

## MNIR-PSI-031 — One shared namespace counter

One allocation authority MUST use one shared monotonically increasing counter
across every covered entity ID category.

The authority MUST NOT maintain independent counter sequences whose equal
namespace/counter pairs could be issued once per typed category.

---

## MNIR-PSI-032 — Counter selection

When the authority is in `Available(next_counter)`, a successful reservation
MUST select that current `next_counter` value for issuance.

When the authority is `Exhausted`, no counter may be selected. Allocation in
that state is governed by `MNIR-PSI-037`.

---

## MNIR-PSI-033 — Issuance point

Before an entity-creating mutation operation begins ID reservation, it MUST
validate every specification-defined operation precondition that may produce
an ordinary public mutation error. This includes, where applicable, unknown
parent entities, Blocks, Functions, or argument Expressions; foreign-Block
references; invalid public mutation input; and other preconditions defined by
specifications `01` through `09`.

If that validation fails, the operation MUST issue no entity ID, MUST consume
no counter, and MUST leave allocation-authority state unchanged by that
attempted allocation. Existing transaction failure and poisoning semantics
remain applicable.

After every required precondition has succeeded, the operation may begin ID
reservation. Successful reservation of the selected namespace/counter pair is
the normative issuance point of an entity ID.

A successful reservation MUST:

1. select the current allocatable counter according to `MNIR-PSI-032`;
2. permanently issue the typed namespace/counter identity;
3. atomically advance authoritative counter state according to
   `MNIR-PSI-034`; and
4. make the issued pair permanently unavailable for reissuance.

Issuance MUST occur before that ID is returned to a caller or inserted into a
transaction working state.

The entity MUST be created or inserted into transaction working state only
after successful reservation.

---

## MNIR-PSI-034 — Counter advancement

Successful reservation from `Available(counter)` MUST transition authoritative
counter state as follows:

1. when a greater representable counter remains available, transition to
   `Available(next_counter)`, where `next_counter` is greater than the issued
   `counter`; or
2. when `counter` is the last representable counter, issue that counter and
   transition to `Exhausted`.

The last representable counter MUST be valid for issuance and MUST NOT be
silently skipped solely to avoid representing exhaustion.

This advancement MUST NOT be rolled back in a way that permits the issued pair
to be issued again.

Successful reservation and authoritative counter-state advancement MUST
behave as one atomic allocation-authority transition with respect to identity
reuse. If that transition cannot complete, reservation MUST fail before
issuance and counter state MUST remain unchanged by the attempted reservation.

In a persistence-capable implementation, reservation MUST NOT be considered
successful unless the counter-state transition preventing reuse is durable
according to that implementation's persistence model. A process interruption
MUST NOT permit a successfully reserved identity to be issued again after
restart.

For a non-persistent implementation, the authoritative in-memory counter-state
transition MAY satisfy this requirement until persistence is introduced.

---

## MNIR-PSI-035 — Issued pair non-reuse

An allocation authority MUST NOT issue a namespace/counter pair more than
once.

This guarantee applies across entity categories, transactions, revisions,
deletion, persistence, process restart, and continued mutation.

---

## MNIR-PSI-036 — Allocation gaps are valid

Counter values need not be dense or contiguous.

Unused values below the counter in an `Available` state and values consumed by
failed, discarded, or removed provisional work MAY remain permanent gaps and
MUST NOT carry semantic meaning.

---

## MNIR-PSI-037 — Counter exhaustion

When allocation-authority counter state is `Exhausted`, an entity-creating
mutation operation MUST fail before issuance. It MUST issue no identity, leave
counter state `Exhausted`, and create no entity.

The failure MUST follow the existing transaction-poisoning behavior for
mutation-operation failure, including `MNIR-CORE-061`.

Allocation MUST NOT wrap or reuse a previously issued counter. Persistent
Semantic Identity 0.1 MUST NOT automatically rotate or replace the active
allocation namespace after exhaustion.

Such expected allocation failure MUST be represented by an error rather than a
panic.

The concrete counter width and error type are implementation-defined.

---

# 11. Transaction behavior

## MNIR-PSI-038 — Issued and committed are distinct states

An issued entity ID in an Active transaction remains provisional and MUST NOT
be treated as a committed semantic entity identity until the transaction
commits successfully.

Issuance nevertheless permanently consumes the namespace/counter pair.

---

## MNIR-PSI-039 — Successful semantic commit

When a transaction creating an entity commits successfully, that entity's
issued typed ID becomes part of committed semantic Program contents unless the
entity was removed from the working state before commit.

Every issued pair remains unavailable for future issuance in either case.

---

## MNIR-PSI-040 — Operation failure after issuance

If a mutation operation fails after issuing one or more entity IDs, every such
namespace/counter pair MUST remain permanently unavailable for reissuance.

This remains true when the entity was never inserted into transaction working
state or the issued ID was never returned to the caller.

The transaction poisoning rules of the applicable earlier specification remain
unchanged.

---

## MNIR-PSI-041 — Poisoned transaction

Poisoning a transaction MUST roll back or isolate its uncommitted semantic
Program changes according to existing transaction rules.

Poisoning MUST NOT roll allocation-authority state backward in a way that
permits reuse of an issued identity.

---

## MNIR-PSI-042 — Discarded transaction

Discarding an Active or Failed transaction MUST NOT commit its semantic working
state.

Every identity issued during that transaction MUST remain permanently
unavailable for reissuance.

---

## MNIR-PSI-043 — Failed commit

A structurally failed commit MUST leave committed semantic Program contents and
the current `RevisionId` unchanged according to existing atomicity rules.

Every identity issued by the transaction before commit failure MUST remain
permanently unavailable for reissuance.

---

## MNIR-PSI-044 — Successful create-then-remove transaction

When an entity is created and removed in the same transaction, its ID remains
issued regardless of whether the transaction later commits, fails, or is
discarded.

Successful commit MUST NOT place the removed entity in committed semantic
Program contents and MUST NOT make its ID reusable.

---

## MNIR-PSI-045 — Semantic rollback does not imply allocator rollback

Transaction rollback, isolation, or atomic rejection of semantic changes MUST
NOT be interpreted as rollback of issued namespace/counter pairs.

---

# 12. Semantic state, allocator state, and revisions

## MNIR-PSI-046 — State separation

A mutable Program lineage MUST distinguish:

1. committed semantic revision state; and
2. persistent allocation-authority state.

Allocation-authority state is required for safe future mutation but MUST NOT be
treated as an MNIR semantic entity or semantic Program content.

---

## MNIR-PSI-047 — Allocation state excluded from no-op comparison

Allocation-authority state, including the complete counter state, MUST be
excluded when determining whether transaction working contents are
semantically equal to their source Program revision for no-op classification.

Existing no-op comparison clauses concerning committed full identifier-history
sets are superseded.

---

## MNIR-PSI-048 — Issuance alone creates no revision

Issuing entity IDs or advancing allocation-authority state MUST NOT by itself
create a new semantic Program revision or allocate a new `RevisionId`.

This applies when a transaction operation later fails, a transaction is
discarded, or no semantic commit occurs.

---

## MNIR-PSI-049 — Create-then-remove no-op classification

A transaction whose only net semantic activity is creation and removal of
provisional entities MUST be classified as a semantic no-op when its working
semantic contents otherwise equal its source revision.

Issued IDs and allocation-authority advancement MUST NOT make it semantically
non-no-op.

---

## MNIR-PSI-050 — Explicit no-op commit retains existing revision behavior

If an Active semantic no-op transaction is explicitly committed, the commit
MUST continue to produce a new `RevisionId` according to `MNIR-CORE-030` and
`MNIR-CORE-039`.

The new revision results from the explicit successful commit rule, not from
allocation-authority advancement.

---

## MNIR-PSI-051 — Revision contents exclude allocator metadata

Two observations MAY represent the same `ProgramId`, `RevisionId`, and semantic
Program contents while carrying allocation-authority observations taken at
different times.

Such allocator-state differences MUST NOT retroactively change the immutable
semantic revision represented by either observation.

---

# 13. Removal and historical-ID sets

## MNIR-PSI-052 — Deleted identity non-reuse

Removing a committed Module, Function, Parameter, Block, or Expression MUST NOT
permit its typed entity ID or namespace/counter pair to be issued to another
entity.

---

## MNIR-PSI-053 — Monotonic state is sufficient for owned namespace non-reuse

For identities issued by the lineage's active allocation authority, correct
persistent monotonic counter state, including `Exhausted`, MUST be sufficient
to demonstrate non-reuse.

An implementation MUST NOT be required to retain a complete set of deleted or
previously committed entity IDs solely to prevent reuse when monotonic
allocation state provides that guarantee.

---

## MNIR-PSI-054 — Historical sets remain an implementation option

An implementation MAY retain historical-ID sets for diagnostics, migration,
validation, or another independently justified purpose.

Such sets MUST NOT replace correct persistent allocation-authority state and
MUST NOT make allocator history semantic Program content.

---

# 14. Snapshots and persistence observations

## MNIR-PSI-055 — Snapshot identity preservation

A Program snapshot MUST preserve the following identity-related state exactly:

- `ProgramId`;
- the applicable `RevisionId` semantics;
- every contained typed entity ID; and
- every semantic reference between those IDs.

Snapshot creation MUST NOT remap or mint an entity identity merely because a
snapshot is created.

---

## MNIR-PSI-056 — Snapshot allocation-state observation

Every Program snapshot MUST retain or be paired with an immutable observation
of the lineage's allocation-authority state at the time the snapshot is
created.

That observation MUST preserve `namespace_id` and the complete counter state
exactly, including whether it is `Available(next_counter)` or `Exhausted`.

---

## MNIR-PSI-057 — Snapshot does not grant authority

Possession or copying of a snapshot's allocation-state observation MUST NOT by
itself grant authority to allocate in that namespace.

Allocation authority belongs to the mutable lineage, not to arbitrary snapshot
copies.

The only same-lineage restoration behavior defined by this specification is
the safe continuation governed by `MNIR-PSI-058` through `MNIR-PSI-060`.

---

## MNIR-PSI-058 — Same-lineage restoration requires current safe state

A persisted representation used to restore the mutable continuation of the
same Program lineage MUST restore allocation-authority state that is at least
as advanced as every successful reservation previously performed by that
authority.

An `Available(next_counter)` restoration MUST use a counter greater than every
counter previously issued by that authority. An `Exhausted` restoration MUST
remain `Exhausted`.

Restoration MUST NOT use a stale allocator observation when doing so could
permit reuse.

---

## MNIR-PSI-059 — Historical snapshots remain non-mutation roots

The historical snapshot restrictions of `MNIR-CORE-058` and
`MNIR-CORE-059` remain unchanged.

Continuing from a historical snapshot requires a normal fork and therefore a
fresh allocation namespace.

---

## MNIR-PSI-060 — Safe restoration failure

If an implementation cannot establish that restored allocation-authority state
satisfies `MNIR-PSI-058`, it MUST reject continued mutation of that lineage
rather than risk identity reuse.

The error representation is implementation-defined.

---

# 15. Normal forks

## MNIR-PSI-061 — Normal fork definition

A **normal fork** is the operation defined by Program Model for creating a new
independently evolving Program lineage from one immutable Program revision.

Import, export, copy-as-new-identity, transformation, and merge operations are
not normal forks and remain undefined.

---

## MNIR-PSI-062 — Fork ProgramId

A normal fork MUST receive a new `ProgramId` satisfying `MNIR-PSI-019`.

The fork MUST NOT reuse the source lineage's `ProgramId`.

---

## MNIR-PSI-063 — Exact inherited entity identity

A normal fork MUST preserve every inherited `ModuleId`, `FunctionId`,
`ParameterId`, `BlockId`, and `ExpressionId` exactly.

The same inherited typed ID in the source and fork denotes the same persistent
semantic entity identity.

---

## MNIR-PSI-064 — Exact inherited references

A normal fork MUST preserve every inherited semantic reference exactly,
including:

- Module ownership of Functions;
- Function ownership and Parameter order;
- Parameter references;
- arithmetic and comparison operand references;
- Function-body Block membership and entry Block references;
- Return Expression references;
- Branch condition and target references;
- Call target and argument references; and
- EffectSequence entries and order.

---

## MNIR-PSI-065 — No normal-fork remapping

A normal fork MUST NOT remap an inherited covered entity ID.

No reference rewriting is required or permitted merely to construct a normal
fork.

---

## MNIR-PSI-066 — Fresh fork allocation authority

A normal fork MUST receive a newly created `AllocationNamespaceId` and active
allocation-authority state satisfying `MNIR-PSI-012` through
`MNIR-PSI-014` and `MNIR-PSI-022` through `MNIR-PSI-024`.

New entity IDs issued by the fork after divergence MUST use that fresh
namespace.

---

## MNIR-PSI-067 — Ancestor authority is not inherited

The fork MUST NOT receive allocation authority for the source lineage's active
namespace.

Inherited IDs from that namespace remain present and usable as semantic
references, but the fork MUST NOT issue another ID in it.

---

## MNIR-PSI-068 — Original-lineage authority remains valid

Forking MUST NOT transfer, revoke, advance, or otherwise modify the source
lineage's allocation authority.

After the fork, the source lineage MUST remain able to issue new IDs in its
existing active namespace.

---

## MNIR-PSI-069 — Independently created forks

Two forks created independently from the same source revision MUST be able to
issue new entity IDs without coordination and with negligible collision
probability.

Their inherited IDs MUST remain equal, while their newly issued IDs MUST be
distinct with the collision guarantees of `MNIR-PSI-013`.

---

## MNIR-PSI-070 — Fork semantic preservation

Except for `ProgramId`, fork revision identity as governed by existing rules,
and the fork's fresh allocation-authority state, a newly created normal fork
MUST preserve the source snapshot's semantic Program contents.

---

# 16. Semantic references

## MNIR-PSI-071 — Exact typed references

All semantic references defined by specifications `01` through `09` MUST
continue to use complete typed entity IDs.

A reference MUST compare namespace and counter as part of the referenced typed
ID.

---

## MNIR-PSI-072 — Reference ownership remains structural

Persistent identity across lineages MUST NOT weaken same-Program,
same-Function, same-body, or same-Block reference constraints defined by
earlier specifications.

An equal persistent ID does not authorize a reference to an entity absent from
the current Program revision.

---

## MNIR-PSI-073 — Fork reference rewriting prohibited

Because normal forks preserve inherited IDs, normal fork construction MUST NOT
rewrite inherited Function, Parameter, Block, Expression, Branch, Return,
Call, arithmetic, comparison, or EffectSequence references.

---

# 17. Structural validity

## MNIR-PSI-074 — Valid covered entity identity

Every contained covered entity MUST have exactly one valid ID of its required
typed category, and that ID MUST contain exactly one valid namespace and
counter.

---

## MNIR-PSI-075 — Program-state identity uniqueness

Within one Program revision, no two distinct entities of the same typed
category may carry the same complete typed entity ID.

This structural uniqueness rule does not make entity identity Program-scoped.

---

## MNIR-PSI-076 — Allocation-authority invariant

For a mutable lineage, an active `Available(next_counter)` state MUST use a
counter greater than every counter previously issued through that authority.
An active `Exhausted` state MUST issue no further counters.

Contained entities from other namespaces impose no allocation authority over
those namespaces.

---

## MNIR-PSI-077 — Structural rejection

A candidate Program state violating `MNIR-PSI-074` or `MNIR-PSI-075` MUST NOT
commit successfully.

Rejection MUST preserve existing transaction atomicity while retaining the
non-reuse of every ID already issued by the transaction.

---

# 18. Verification compatibility

## MNIR-PSI-078 — Existing verifier rule sets remain applicable

Semantic Verification rule sets V0_1 through V0_4 MUST retain their existing
applicability, diagnostics, traversal, success, and failure semantics.

Persistent entity ID representation alone MUST NOT create a new applicability
condition.

---

## MNIR-PSI-079 — VerifiedProgram binding

`VerifiedProgram` MUST continue to bind verification evidence to exactly the
verified:

```text
ProgramId
RevisionId
VerificationRuleSet
immutable Program snapshot
```

`ProgramId` identifies the verified lineage. Persistent entity IDs inside the
snapshot retain their Program-independent identity.

---

## MNIR-PSI-080 — No cross-lineage verification implication

Equal inherited entity IDs across two forked Program lineages MUST NOT make a
`VerifiedProgram` for one lineage evidence that the other lineage or revision
was verified.

---

## MNIR-PSI-081 — No new verification rule set

Implementation of Persistent Semantic Identity 0.1 MUST NOT introduce a new
semantic verification rule set solely because identifier representation
changed.

---

# 19. Persistence and serialization implications

## MNIR-PSI-082 — Minimum future persistent representation

Any future persistent representation capable of restoring a mutable lineage
MUST preserve:

- `ProgramId`;
- relevant `RevisionId` and revision contents;
- every typed entity ID;
- every semantic reference;
- active `AllocationNamespaceId`; and
- complete counter state sufficient to prevent reuse, including `Exhausted`
  when applicable.

---

## MNIR-PSI-083 — Process-local state is insufficient

An implementation claiming persistence across process restart MUST NOT rely
solely on process-local counters, memory addresses, runtime-global state, or
load order for Program, namespace, or entity identity safety.

---

## MNIR-PSI-084 — Canonical encoding deferred

Persistent Semantic Identity 0.1 MUST NOT define canonical byte encoding,
field ordering, textual identifier syntax, Git representation, or a
serialization file format.

---

# 20. Import, merge, and trust

## MNIR-PSI-085 — Import and merge algorithms excluded

Persistent Semantic Identity 0.1 MUST NOT define import, package resolution,
semantic diff, merge, merge-conflict, or merge-commit algorithms.

---

## MNIR-PSI-086 — Future ancestry matching

Equal inherited persistent IDs MAY be used by future specifications as the
identity foundation for recognizing a shared semantic entity across lineages.

This rule does not define a diff or merge result.

---

## MNIR-PSI-087 — Independent additions remain distinct

Entities independently added by lineages using distinct allocation namespaces
MUST have distinct persistent entity IDs even when their semantic contents are
otherwise equal.

---

## MNIR-PSI-088 — Trust remains unresolved

Collision-resistant namespace identity MUST NOT be interpreted as protection
against malicious namespace claims or as evidence of provenance,
authorization, signatures, trusted publishers, or package integrity.

Those concerns require future package, import, and security specifications.

---

# 21. Equality scope

## MNIR-PSI-089 — Identity equality only

The equality rules in this document define persistent entity identity only.

They MUST NOT be interpreted as defining whole-Program semantic equivalence,
behavioral equivalence, content equality, revision equivalence, or merge
equivalence.

---

## MNIR-PSI-090 — Equal identity may have revision-specific contents

The same persistent entity ID MAY identify revision-specific versions with
different semantic contents in different revisions or forked lineages.

Equal entity identity MUST NOT imply equal current entity contents.

---

# 22. Specification gaps and unresolved topics

## MNIR-PSI-091 — Undefined persistent identity behavior

If implementation requires externally observable persistent identity behavior
not defined by this specification, the implementation MUST NOT establish that
behavior as normative MNIR semantics.

The missing behavior MUST be reported as a specification gap.

---

## MNIR-PSI-092 — Unresolved topics are not 0.1 semantics

An implementation MUST NOT establish the following unresolved topics as
normative Persistent Semantic Identity 0.1 semantics:

- exact ID bit widths or binary layout;
- canonical serialization;
- UUID or other textual syntax;
- namespace rotation;
- multiple active authorities per lineage;
- transfer or delegation of allocation authority;
- counter compaction or renumbering;
- legacy-ID migration;
- explicit identity-remapping operations;
- import and package identity validation;
- namespace provenance and trust;
- cryptographic signatures;
- semantic diff and merge algorithms;
- behavioral equivalence; or
- future persistent entity categories.

---

# 23. Acceptance requirements

The acceptance range for this increment is `AR-PSI-001` through
`AR-PSI-044`.

## AR-PSI-001 — AllocationNamespaceId category

Demonstrate that `AllocationNamespaceId` is a typed category distinct from
`ProgramId`, `RevisionId`, and every covered entity ID category.

---

## AR-PSI-002 — Typed namespace/counter entity identity

For each of `ModuleId`, `FunctionId`, `ParameterId`, `BlockId`, and
`ExpressionId`, demonstrate inspection or equivalent compile-time evidence of
one namespace component and one counter component.

---

## AR-PSI-003 — Same typed ID equality

Demonstrate that equal namespace and counter components in the same ID category
compare as the same persistent identity independently of `ProgramId`.

---

## AR-PSI-004 — Typed categories remain distinct

Demonstrate that entity ID categories remain non-interchangeable even if their
namespace and counter components are equal.

Compile-time evidence SHOULD be used where practical.

---

## AR-PSI-005 — ProgramId is not entity identity

Demonstrate that the same inherited entity ID in Programs with different
`ProgramId` values denotes the same persistent entity identity.

---

## AR-PSI-006 — Coordination-free Program creation

Create multiple independent Program lineages and demonstrate distinct
`ProgramId` and active `AllocationNamespaceId` values.

Conformance inspection MUST also verify that neither creation mechanism relies
on a process-local sequential counter alone or requires online coordination.

Using deterministic test control or equivalent evidence, also demonstrate that
a detected candidate namespace or Program identity collision is rejected or
retried rather than treated as shared identity or authority.

---

## AR-PSI-007 — Shared monotonic allocation

Allocate multiple different entity categories in one lineage and verify that
all IDs use the active namespace and that no namespace/counter pair repeats.

---

## AR-PSI-008 — Snapshot identity preservation

Create a Program containing all covered entity categories and every reference
family defined through specification `09`.

Verify a snapshot preserves all entity IDs and references exactly and creates
no new entity ID.

---

## AR-PSI-009 — Snapshot allocation-state preservation

Begin with a snapshot `S1` whose allocator observation is equivalent to:

```text
ProgramId = P
RevisionId = R
namespace_id = A
counter_state = Available(100)
```

Successfully reserve counter `100` without semantic commit, for example by
subsequently discarding the transaction, and then create snapshot or
observation `S2`. Use a bounded test configuration in which `100` is not the
last representable counter.

Verify:

- `S1` remains immutable and continues to observe `Available(100)`;
- `S2` observes `Available(n)`, where `n > 100`;
- both observations preserve `ProgramId = P` and `RevisionId = R`;
- both represent the same semantic Program revision;
- allocator advancement alone created no semantic revision; and
- neither snapshot observation independently grants allocation authority.

Also verify a snapshot taken while counter state is `Exhausted` preserves that
state exactly.

---

## AR-PSI-010 — Fork receives new ProgramId

Create a normal fork and verify its `ProgramId` differs from the source
lineage's `ProgramId`.

---

## AR-PSI-011 — Fork preserves inherited IDs

Verify every inherited `ModuleId`, `FunctionId`, `ParameterId`, `BlockId`, and
`ExpressionId` is bit-for-bit or component-for-component equal in source and
normal fork.

---

## AR-PSI-012 — Same FunctionId across forks

Create two forks from one Function-containing ancestor and verify the inherited
`FunctionId` denotes the same persistent semantic entity identity in the
ancestor and both forks.

---

## AR-PSI-013 — Fork receives fresh namespace

Verify a normal fork's active allocation namespace differs from the source
lineage's active namespace.

---

## AR-PSI-014 — Original authority survives fork

Given a source lineage with `ProgramId = P1`, active namespace `A`, and counter
state `Available(n)`, create a normal fork and verify immediately afterward:

- the source still has `ProgramId = P1`;
- the source still has active namespace `A`;
- the source counter state remains exactly `Available(n)` solely because fork
  creation occurred;
- the source retains allocation authority for `A`;
- the fork has a new `ProgramId = P2`;
- the fork has a fresh active namespace `B`;
- every inherited semantic entity ID remains unchanged; and
- the fork has no allocation authority for `A`.

Then allocate separately in source and fork and verify only those later
allocations advance their respective counter states and that the source
allocation uses namespace `A`.

---

## AR-PSI-015 — Fork cannot allocate in ancestor namespace

After forking, allocate a new entity in the fork and verify it uses the fork's
fresh namespace rather than the ancestor namespace.

Demonstrate that normal safe mutation APIs provide no way for the fork to
select the ancestor namespace.

---

## AR-PSI-016 — Independent offline forks

Construct two forks independently from the same snapshot without shared
counter state.

Verify inherited IDs remain equal and newly allocated IDs are distinct because
the forks use distinct namespaces.

---

## AR-PSI-017 — Deleted IDs are never reused

For every covered entity category, commit an entity, remove it, allocate later
entities, and verify the removed ID and namespace/counter pair are never
reused.

---

## AR-PSI-018 — Identity-preserving updates

Demonstrate existing presentation, type, and other identity-preserving updates
retain complete typed IDs after the representation change.

---

## AR-PSI-019 — Provisional identity use

Demonstrate that an issued provisional ID can address its entity within the
issuing Active transaction and becomes a committed entity identity only after
successful semantic commit.

---

## AR-PSI-020 — Issuance boundary

Demonstrate both sides of the normative issuance boundary.

For pre-reservation failure, begin with counter state `Available(100)` and
perform an entity-creation operation with invalid public input that is
normatively detectable during precondition validation. Verify:

- no entity ID is issued;
- counter state remains `Available(100)`;
- the failed mutation follows existing transaction failure and poisoning
  semantics; and
- in a fresh transaction, the next successful reservation is still permitted
  to issue counter `100` unless another legitimate allocation occurred in
  between.

Separately, successfully reserve an ID and then prevent semantic commit through
later failure, discard, or structural commit failure. Verify the reservation
advanced authoritative counter state atomically and durably as applicable,
and that the issued pair remains unavailable despite the later outcome.

The evidence MUST NOT depend on implementation-defined reservation timing or
reservation occurring before specification-defined public precondition
validation.

---

## AR-PSI-021 — Poisoned transaction non-reuse

Issue an ID, poison the transaction through a later failing operation, and
verify semantic changes do not commit while the issued pair remains
unavailable.

---

## AR-PSI-022 — Discarded transaction non-reuse

Issue one or more IDs, discard the transaction, and verify later allocations
do not reuse the issued pairs.

---

## AR-PSI-023 — Failed commit non-reuse

Issue one or more IDs, cause structural commit failure, and verify:

- committed semantic contents remain unchanged;
- `RevisionId` remains unchanged; and
- later allocations do not reuse the issued pairs.

---

## AR-PSI-024 — Create-then-remove allocation

Create and remove provisional entities before successful commit.

Verify they are absent from committed semantic contents, their issued IDs are
never reused, and complete historical-ID sets are not required as the sole
non-reuse mechanism.

---

## AR-PSI-025 — Allocation gaps

Produce at least one gap through failed, discarded, or removed provisional work
and verify later allocation succeeds using a greater counter without requiring
the gap to be filled.

---

## AR-PSI-026 — Allocator advancement creates no revision

Record `ProgramId`, `RevisionId`, and committed semantic contents.

Issue one or more IDs and then discard or fail the transaction without
semantic commit.

Verify allocation-authority state advanced while `ProgramId`, `RevisionId`, and
committed semantic contents remained unchanged.

---

## AR-PSI-027 — Explicit no-op commit

Create and remove provisional entities in one Active transaction, then commit.

Verify:

- the transaction is classified as a semantic no-op before commit;
- allocation state is excluded from that comparison;
- explicit commit succeeds;
- a new `RevisionId` is produced according to existing no-op commit rules;
- semantic contents remain equal; and
- issued IDs remain unavailable.

---

## AR-PSI-028 — Safe process restart

Demonstrate through persistence round-trip testing when persistence support
exists, or otherwise through documented conformance inspection, that restoring
the same mutable lineage preserves `ProgramId`, active namespace, and safe
complete counter state, including `Exhausted` when applicable.

Verify process-local allocator state alone cannot satisfy the design.

---

## AR-PSI-029 — Stale allocator state rejected

Demonstrate through automated test or documented conformance inspection that a
stale allocation-state observation cannot restore mutable authority when doing
so could reuse an issued counter.

---

## AR-PSI-030 — Fork references require no remapping

Construct a Program containing ParameterReference, arithmetic, comparison,
Branch, Return, Call, and EffectSequence references.

Verify a normal fork preserves every referenced ID exactly without a remapping
table or reference rewrite.

---

## AR-PSI-031 — Multiple namespace presence

After a normal fork allocates a new entity, verify the fork contains inherited
entities from the ancestor namespace and new entities from its own namespace,
while retaining authority only for its own namespace.

---

## AR-PSI-032 — Structural identity rejection

Demonstrate that a candidate Program containing two distinct same-category
entities with the same complete typed ID cannot commit.

Test-only structural corruption support MAY be used without weakening public
encapsulation.

---

## AR-PSI-033 — VerifiedProgram binding remains lineage-specific

Verify an inherited snapshot in two Program lineages independently.

Demonstrate that equal inherited entity IDs do not make `VerifiedProgram`
evidence interchangeable because `ProgramId`, `RevisionId`, and rule-set
binding remain authoritative.

---

## AR-PSI-034 — V0_1 through V0_4 compatibility

Run the existing V0_1 through V0_4 verification acceptance suites after the ID
representation change.

Verify applicability, diagnostics, payload IDs, and successful rule-set
bindings retain existing semantic behavior and no new verification rule set is
required.

---

## AR-PSI-035 — Existing conformance compatibility

Run all acceptance suites from specifications `01` through `09`.

Requirements explicitly superseded by section 24 MUST be replaced by their
`AR-PSI-*` evidence. All other semantic behavior MUST remain valid.

---

## AR-PSI-036 — Scope exclusions

Demonstrate by conformance inspection that implementation of this specification
introduces no canonical serialization, import, package, trust, signature,
semantic diff, merge, identity-remapping, or new verifier-rule-set semantics.

---

## AR-PSI-037 — Counter exhaustion is safe

Using a bounded test allocator or equivalent test-only support, reach an
explicit `Exhausted` counter state.

Verify allocation returns an expected error without panic, wraparound, or
reuse of an issued namespace/counter pair, and verify the failed mutation
follows existing transaction-poisoning behavior.

---

## AR-PSI-038 — Implementation boundaries

Demonstrate through dependency inspection, compile-time evidence, source
inspection, and public-API tests as applicable that:

- identity and allocation authority belong to `mnir-core`;
- typed ID categories remain distinct;
- normal callers cannot select or mutate namespaces, counters, or authority;
- no `unsafe` Rust is used;
- no unjustified external dependency is introduced; and
- no speculative generic identity, import, package, or merge abstraction is
  introduced.

---

## AR-PSI-039 — Last counter and exhausted-state behavior

Using a bounded allocator whose last representable counter is `MAX`, begin in
`Available(MAX)` and verify:

1. successful reservation issues the typed identity with counter `MAX`;
2. the atomic reservation transition changes counter state to `Exhausted`;
3. the issued `MAX` identity may enter transaction working state normally;
4. a later allocation attempt while `Exhausted` fails before issuance;
5. the failed attempt leaves counter state `Exhausted` and creates no entity;
6. no namespace rotation or authority replacement occurs; and
7. the failed mutation follows existing transaction-poisoning semantics.

In a separate transaction, successfully reserve `MAX`, transition to
`Exhausted`, and then discard the transaction. Verify counter state remains
`Exhausted` and the `MAX` namespace/counter pair never becomes reusable.

---

## AR-PSI-040 — Program identity-generation failure

Using controlled or injected test support, force identity generation to fail
during creation of a new Program lineage.

Verify:

- Program creation returns an implementation-neutral typed MNIR/API error;
- no Program lineage is created or becomes observable;
- no partial Program or allocation-namespace state is exposed; and
- no provider-specific error type crosses the public MNIR API.

---

## AR-PSI-041 — Partial Program identity generation

Force fresh `ProgramId` generation to succeed and fresh
`AllocationNamespaceId` generation to fail during creation of a new Program
lineage.

Verify no Program lineage or partial allocation authority is produced or made
observable.

---

## AR-PSI-042 — Fork identity-generation failure

Begin with a valid committed source Program and force identity generation to
fail during normal fork creation.

Verify:

- no fork lineage is produced or made observable;
- the source `ProgramId` and `RevisionId` remain unchanged;
- the source semantic contents remain unchanged; and
- the source allocation namespace, counter state, and allocation authority
  remain unchanged.

---

## AR-PSI-043 — Partial fork identity generation

During normal fork creation, force generation of the new fork `ProgramId` to
succeed and generation of the new fork `AllocationNamespaceId` to fail.

Verify no fork or partial fork allocation authority becomes observable and
the complete source lineage remains unchanged.

---

## AR-PSI-044 — Existing-lineage allocation requires no fresh entropy

Using controlled internal instrumentation or equivalent conformance evidence,
create a Module, Function, Parameter, Block, and Expression in an existing
lineage.

Verify all five identities are allocated from the lineage's active
namespace/counter authority and that no fresh Program or allocation-namespace
identity generation is invoked by these ordinary `MutationTransaction`
operations.

---

# 24. Explicit normative supersession and revision

This section is normative. “Supersedes” applies only to the identified rule or
clause. Text explicitly described as retained remains authoritative.

## 24.1 Program Model 0.1

| Earlier rule | Disposition under Persistent Semantic Identity 0.1 |
| --- | --- |
| `MNIR-CORE-004` | `MNIR-PSI-025`, `MNIR-PSI-063`, and `MNIR-PSI-075` supersede the prohibition on one inherited Module identity occurring across forked lineages. Exactly-one ownership within each Program revision remains required. |
| `MNIR-CORE-011` | Superseded by `MNIR-PSI-006` through `MNIR-PSI-009` and `MNIR-PSI-075`; `ModuleId` is no longer Program-lineage-scoped. |
| `MNIR-CORE-013`, `MNIR-CORE-015` | Their lineage-scoped lifetime clauses are superseded by `MNIR-PSI-010`, `MNIR-PSI-035`, and `MNIR-PSI-052`. Update preservation and replacement-as-new-identity remain required. |
| `MNIR-CORE-054` through `MNIR-CORE-057` | Superseded by `MNIR-PSI-023`, `MNIR-PSI-024`, `MNIR-PSI-053` through `MNIR-PSI-060`, and `MNIR-PSI-082`; full Module-ID history is no longer required solely for non-reuse. |
| `MNIR-CORE-017` | Superseded by `MNIR-PSI-062` through `MNIR-PSI-070`; normal forks preserve inherited Module identity rather than creating lineage-scoped distinct identity. |
| `MNIR-CORE-034` | Its Program-lineage allocator clause is superseded by `MNIR-PSI-026`, `MNIR-PSI-031` through `MNIR-PSI-034`. Its prohibition on caller-selected raw IDs remains valid. |
| `MNIR-CORE-064` through `MNIR-CORE-066` | Superseded where they permit failed provisional raw-value reuse or defer issuance durability, by `MNIR-PSI-033` through `MNIR-PSI-045`. Provisional entities remain uncommitted until successful commit. |
| `MNIR-CORE-040`, item 6 | Superseded by `MNIR-PSI-035`, `MNIR-PSI-052`, and `MNIR-PSI-076`; non-reuse derives from persistent authority state rather than Program-lineage committed history. |

The identity-related revision audit has the following result:

| Earlier rule | Result |
| --- | --- |
| `MNIR-CORE-001`, `MNIR-CORE-002` | Remain fully valid. Allocation authority is mutable-lineage metadata, not another semantic child entity or semantic revision field. |
| `MNIR-CORE-006` | Remains valid and is supplemented by `MNIR-PSI-006` and `MNIR-PSI-011`. Namespace/counter composition is normative, but its representation carries no other semantic meaning. |
| `MNIR-CORE-007` through `MNIR-CORE-010` | Their lineage, snapshot, and new-fork-ProgramId behavior remains valid. The implementation-defined generation clause in `MNIR-CORE-008` is superseded by `MNIR-PSI-019`; fork behavior is supplemented by `MNIR-PSI-062`. |
| `MNIR-CORE-012` | Remains fully valid; typed categories remain distinct even if components or raw representations match. |
| `MNIR-CORE-016` | Remains valid and is extended to all covered IDs and references by `MNIR-PSI-055`. |
| `MNIR-CORE-018`, `MNIR-CORE-019` | Remain fully valid. Import and merge remain undefined. |
| `MNIR-CORE-033` | Remains fully valid; transactions still target one lineage and source revision. |
| `MNIR-CORE-038` | Remains valid with its identity-allocation clause governed by `MNIR-PSI-026` and `MNIR-PSI-031` through `MNIR-PSI-039`. |
| `MNIR-CORE-039` | Remains fully valid. Explicit no-op commit behavior is clarified by `MNIR-PSI-050`. |
| Section 3.8 no-op definition | Allocation-history participation is superseded by `MNIR-PSI-046` through `MNIR-PSI-050`; all semantic-content comparisons remain valid. |

The review/retain set `MNIR-CORE-005`, `MNIR-CORE-014`,
`MNIR-CORE-020` through `MNIR-CORE-024`, `MNIR-CORE-026` through
`MNIR-CORE-031`, `MNIR-CORE-041`, `MNIR-CORE-042`, `MNIR-CORE-053`,
`MNIR-CORE-058`, and `MNIR-CORE-059` was checked and remains unchanged.

Affected acceptance requirements are revised as follows:

| Earlier acceptance requirements | Replacement or retained evidence |
| --- | --- |
| `AR-CORE-001`, `AR-CORE-002`, `AR-CORE-004` | Supplemented or replaced for identity representation by `AR-PSI-001` through `AR-PSI-007` and `AR-PSI-032`; aggregate construction and ownership expectations remain. |
| `AR-CORE-005`, `AR-CORE-013` | Replaced for non-reuse evidence by `AR-PSI-017`. |
| `AR-CORE-007` | Semantic atomicity remains; allocator non-rollback is added by `AR-PSI-020`, `AR-PSI-021`, `AR-PSI-023`, and `AR-PSI-026`. |
| `AR-CORE-009` | Revised by `AR-PSI-026` and `AR-PSI-027` to exclude allocator metadata from semantic no-op comparison while retaining explicit no-op commit revision behavior. |
| `AR-CORE-014` | Revised by `AR-PSI-019` through `AR-PSI-024` to distinguish issued from committed identity. |
| `AR-CORE-015` | Retained and supplemented by `AR-PSI-010` through `AR-PSI-016`. |

`AR-CORE-003`, `AR-CORE-006`, and `AR-CORE-012` were reviewed and
remain valid, with typed-ID and update-preservation evidence additionally
covered by `AR-PSI-004` and `AR-PSI-018`.

## 24.2 Type System Foundations 0.1

`MNIR-TYPE-003` through `MNIR-TYPE-007`, `MNIR-TYPE-019`,
`MNIR-TYPE-020`, and `MNIR-TYPE-022`, together with `AR-TYPE-002`
through `AR-TYPE-004` and `AR-TYPE-012`, were reviewed and remain unchanged.

Intrinsic types are specification-defined global identities and are not
persistent Program entities. They do not receive namespace/counter IDs.

## 24.3 Functions and Parameters 0.1

| Earlier rule | Disposition under Persistent Semantic Identity 0.1 |
| --- | --- |
| `MNIR-FUNC-002`, `MNIR-FUNC-011` | Superseded by `MNIR-PSI-006` through `MNIR-PSI-009` and `MNIR-PSI-075`; Function and Parameter identity is no longer Program-lineage-scoped. |
| `MNIR-FUNC-005`, `MNIR-FUNC-014`, `MNIR-FUNC-015`, `MNIR-FUNC-079` | Their lineage-scoped retirement mechanism is superseded by `MNIR-PSI-035` and `MNIR-PSI-052`; removal still never permits identity reuse. |
| `MNIR-FUNC-035` through `MNIR-FUNC-037` | Superseded by `MNIR-PSI-023`, `MNIR-PSI-024`, `MNIR-PSI-053`, `MNIR-PSI-054`, and `MNIR-PSI-082`. |
| `MNIR-FUNC-040`, `MNIR-FUNC-041`, `MNIR-FUNC-080` | Superseded for issuance, failure, discard, and reuse by `MNIR-PSI-033` through `MNIR-PSI-045`. |
| `MNIR-FUNC-064`, `MNIR-FUNC-065` | Superseded by `MNIR-PSI-062` through `MNIR-PSI-070`; inherited Function and Parameter IDs are preserved exactly and the fork allocates only in its fresh namespace. |
| `MNIR-FUNC-082` | Its committed Function/Parameter allocation-history bullets are superseded by `MNIR-PSI-046` through `MNIR-PSI-051`; all listed semantic Function and Parameter contents remain part of no-op comparison. |
| `MNIR-FUNC-083` | Superseded by `MNIR-PSI-047`, `MNIR-PSI-049`, and `MNIR-PSI-050`; create-then-remove is a semantic no-op, while explicit commit still creates a revision. |

The revision audit determined:

- `MNIR-FUNC-001`, `MNIR-FUNC-003`, `MNIR-FUNC-010`, and
  `MNIR-FUNC-012` remain valid and are supplemented by
  `MNIR-PSI-004`, `MNIR-PSI-006`, and `MNIR-PSI-011`.
- `MNIR-FUNC-004` and `MNIR-FUNC-013` remain fully valid under
  `MNIR-PSI-010`.
- `MNIR-FUNC-038` and `MNIR-FUNC-039` retain provisional entity
  semantics but are supplemented by the issued/committed distinction in
  `MNIR-PSI-038`.
- The Program-lineage allocation clauses of `MNIR-FUNC-042` and
  `MNIR-FUNC-043` are superseded by `MNIR-PSI-026` and
  `MNIR-PSI-031` through `MNIR-PSI-034`; caller-selected IDs remain
  prohibited.
- `MNIR-FUNC-063` remains valid and is strengthened by `MNIR-PSI-055`.

The review/retain set `MNIR-FUNC-006` through `MNIR-FUNC-009`,
`MNIR-FUNC-016` through `MNIR-FUNC-019`, `MNIR-FUNC-022`,
`MNIR-FUNC-028`, `MNIR-FUNC-052` through `MNIR-FUNC-054`,
`MNIR-FUNC-057`, `MNIR-FUNC-070`, and `MNIR-FUNC-075` remains
unchanged.

Acceptance mapping:

- `AR-FUNC-001` through `AR-FUNC-006`, `AR-FUNC-008` through
  `AR-FUNC-015`, `AR-FUNC-033`, `AR-FUNC-034`, and `AR-FUNC-035`
  retain their semantic scenarios with identity evidence replaced or supplemented by
  `AR-PSI-002` through `AR-PSI-007`, `AR-PSI-017` through
  `AR-PSI-023`, `AR-PSI-032`, and `AR-PSI-035`.
- `AR-FUNC-021` is supplemented by `AR-PSI-008` and `AR-PSI-009`.
- `AR-FUNC-022` is replaced for fork identity by `AR-PSI-010` through
  `AR-PSI-016` and `AR-PSI-030`.
- `AR-FUNC-027` through `AR-FUNC-029` retain cascade and isolation
  behavior; identity retirement/preservation evidence uses `AR-PSI-017`
  and `AR-PSI-018`.
- `AR-FUNC-030` is replaced by `AR-PSI-024`, `AR-PSI-025`, and
  `AR-PSI-027`.
- `AR-FUNC-026` was reviewed and remains unchanged.

## 24.4 Expressions and Basic Function Bodies 0.1

| Earlier rule | Disposition under Persistent Semantic Identity 0.1 |
| --- | --- |
| `MNIR-EXPR-005`, `MNIR-EXPR-012` | Superseded by `MNIR-PSI-006` through `MNIR-PSI-009` and `MNIR-PSI-075`; Block and Expression identities are no longer Program-lineage-scoped. |
| `MNIR-EXPR-007`, `MNIR-EXPR-014`, `MNIR-EXPR-054`, `MNIR-EXPR-056` | Their lineage-scoped retirement mechanism is superseded by `MNIR-PSI-035` and `MNIR-PSI-052`; removal still never permits reuse. |
| `MNIR-EXPR-061` through `MNIR-EXPR-065` | Superseded by `MNIR-PSI-023`, `MNIR-PSI-024`, `MNIR-PSI-033` through `MNIR-PSI-045`, `MNIR-PSI-053`, `MNIR-PSI-054`, and `MNIR-PSI-082`. |
| `MNIR-EXPR-067`, `MNIR-EXPR-099` | Superseded by `MNIR-PSI-062` through `MNIR-PSI-073`; normal forks preserve IDs and references exactly and do not remap. |
| `MNIR-EXPR-097`, `MNIR-EXPR-098` | Their committed-history collision clauses are superseded by `MNIR-PSI-023` and `MNIR-PSI-031` through `MNIR-PSI-035`. Distinct provisional IDs remain required. |
| `MNIR-EXPR-080` | Superseded by `MNIR-PSI-047`, `MNIR-PSI-049`, and `MNIR-PSI-050`. |

The revision audit determined:

- `MNIR-EXPR-004`, `MNIR-EXPR-006`, `MNIR-EXPR-011`, and
  `MNIR-EXPR-013` remain valid and are supplemented by
  `MNIR-PSI-004`, `MNIR-PSI-006`, and `MNIR-PSI-011`.
- Identity allocation performed by `MNIR-EXPR-044`, `MNIR-EXPR-047`,
  and `MNIR-EXPR-048` is governed by `MNIR-PSI-026` and
  `MNIR-PSI-031` through `MNIR-PSI-039`; all other operation behavior
  remains valid.
- `MNIR-EXPR-059` and `MNIR-EXPR-060` retain provisional entity
  semantics but are supplemented by `MNIR-PSI-038` through
  `MNIR-PSI-045`.
- `MNIR-EXPR-066` remains valid and is strengthened by `MNIR-PSI-055`.
- The identity non-reuse clause of `MNIR-EXPR-072` is governed by
  `MNIR-PSI-035`, `MNIR-PSI-052`, and `MNIR-PSI-076`; all other body
  validity clauses remain valid.
- The historical-allocation bullets of `MNIR-EXPR-079` are superseded
  by `MNIR-PSI-046` through `MNIR-PSI-051`; all semantic-content bullets
  remain valid.

The review/retain set `MNIR-EXPR-017`, `MNIR-EXPR-035`,
`MNIR-EXPR-039`, `MNIR-EXPR-050`, `MNIR-EXPR-071`,
`MNIR-EXPR-085`, and `MNIR-EXPR-101` remains unchanged.

Acceptance mapping:

- `AR-EXPR-001`, `AR-EXPR-004` through `AR-EXPR-008`,
  `AR-EXPR-018` through `AR-EXPR-023`, `AR-EXPR-027`,
  `AR-EXPR-034`, `AR-EXPR-035`, `AR-EXPR-039`, `AR-EXPR-040`, and
  `AR-EXPR-044` retain their semantic scenarios with identity evidence
  replaced or supplemented by `AR-PSI-002`, `AR-PSI-004`,
  `AR-PSI-007`, and `AR-PSI-017` through `AR-PSI-025`.
- `AR-EXPR-024` is supplemented by `AR-PSI-008` and `AR-PSI-009`.
- `AR-EXPR-025` is replaced for normal-fork identity by
  `AR-PSI-010` through `AR-PSI-016` and `AR-PSI-030`.
- `AR-EXPR-026`, `AR-EXPR-029`, and `AR-EXPR-041` through
  `AR-EXPR-043` were reviewed and remain unchanged.

## 24.5 Arithmetic Expressions 0.1

The remapping alternative in `MNIR-ARITH-065` is superseded by
`MNIR-PSI-063` through `MNIR-PSI-065` and `MNIR-PSI-073`.

`MNIR-ARITH-003`, `MNIR-ARITH-013`, `MNIR-ARITH-064` through
`MNIR-ARITH-067`, and the identity clause of `MNIR-ARITH-069` remain
valid with persistent `ExpressionId` semantics supplied by
`MNIR-PSI-003` through `MNIR-PSI-011`, `MNIR-PSI-031` through
`MNIR-PSI-039`, `MNIR-PSI-052`, and `MNIR-PSI-055` through
`MNIR-PSI-073`.

`MNIR-ARITH-062`, `MNIR-ARITH-063`, and `MNIR-ARITH-068` were reviewed
and remain unchanged.

`AR-ARITH-001` through `AR-ARITH-006` and `AR-ARITH-017` through
`AR-ARITH-019` retain their scenarios with identity evidence supplemented by
`AR-PSI-002`, `AR-PSI-007`, `AR-PSI-008`, `AR-PSI-017`, and
`AR-PSI-018`. `AR-ARITH-020` is replaced for normal-fork identity and
reference handling by `AR-PSI-010` through `AR-PSI-016` and
`AR-PSI-030`. `AR-ARITH-013`, `AR-ARITH-016`, and `AR-ARITH-021`
were reviewed and remain unchanged.

## 24.6 Semantic Verification and Diagnostics 0.1

The complete review set from ADR 0003 remains unchanged:

```text
MNIR-VERIFY-004
MNIR-VERIFY-005
MNIR-VERIFY-009 through MNIR-VERIFY-016
MNIR-VERIFY-023 through MNIR-VERIFY-026
MNIR-VERIFY-030
MNIR-VERIFY-032
MNIR-VERIFY-034
MNIR-VERIFY-056 through MNIR-VERIFY-059
MNIR-VERIFY-086 through MNIR-VERIFY-089
```

`MNIR-PSI-078` through `MNIR-PSI-081` clarify that Program lineage and
revision binding remain distinct from persistent entity identity.

The reviewed acceptance requirements `AR-VERIFY-003`, `AR-VERIFY-014`
through `AR-VERIFY-017`, `AR-VERIFY-021` through `AR-VERIFY-023`,
`AR-VERIFY-033`, `AR-VERIFY-035`, and `AR-VERIFY-036` remain unchanged
and are additionally covered by `AR-PSI-033` and `AR-PSI-034`.

## 24.7 Comparison Expressions 0.1

The remapping alternative in `MNIR-CMP-058` is superseded by
`MNIR-PSI-063` through `MNIR-PSI-065` and `MNIR-PSI-073`.

`MNIR-CMP-006`, `MNIR-CMP-018`, `MNIR-CMP-057`, `MNIR-CMP-059`,
`MNIR-CMP-060`, and the identity clause of `MNIR-CMP-061` remain valid
with persistent `ExpressionId` behavior supplied by `MNIR-PSI-003` through
`MNIR-PSI-011`, `MNIR-PSI-031` through `MNIR-PSI-039`,
`MNIR-PSI-052`, and `MNIR-PSI-055` through `MNIR-PSI-073`.

`MNIR-CMP-016` and `MNIR-CMP-017` were reviewed and remain unchanged.

`AR-CMP-018` and `AR-CMP-019` retain their scenarios with identity evidence
supplemented by `AR-PSI-002`, `AR-PSI-007`, `AR-PSI-008`, and
`AR-PSI-017`. `AR-CMP-020` is replaced for normal-fork behavior by
`AR-PSI-010` through `AR-PSI-016` and `AR-PSI-030`.
`AR-CMP-014` through `AR-CMP-017` were reviewed and remain unchanged.

## 24.8 Conditional Control Flow 0.1

| Earlier rule | Disposition under Persistent Semantic Identity 0.1 |
| --- | --- |
| `MNIR-CFG-068` | Superseded by `MNIR-PSI-033` through `MNIR-PSI-045`; successful commit is no longer what first prevents provisional Block ID reuse. |
| `MNIR-CFG-071` | Its remapping alternative is superseded by `MNIR-PSI-063` through `MNIR-PSI-065` and `MNIR-PSI-073`. |
| `MNIR-CFG-072` | Its committed Block allocation-history bullet is superseded by `MNIR-PSI-046` through `MNIR-PSI-051`; all semantic CFG-state bullets remain valid. |

The revision audit determined:

- `MNIR-CFG-003`, `MNIR-CFG-006`, `MNIR-CFG-008`,
  `MNIR-CFG-010`, `MNIR-CFG-014`, `MNIR-CFG-066`, and
  `MNIR-CFG-067` remain valid with persistent Block/Expression IDs and
  allocation supplied by `MNIR-PSI-003` through `MNIR-PSI-011` and
  `MNIR-PSI-031` through `MNIR-PSI-045`.
- `MNIR-CFG-069` and `MNIR-CFG-070` remain valid and are strengthened by
  `MNIR-PSI-055` and `MNIR-PSI-062` through `MNIR-PSI-070`.
- `MNIR-CFG-111` and `MNIR-CFG-124` remain fully valid; existing
  `BlockId` is now the persistent ID defined by this specification.
- `MNIR-CFG-063` through `MNIR-CFG-065` and `MNIR-CFG-112` were
  reviewed and remain unchanged.

`AR-CFG-002`, `AR-CFG-021`, `AR-CFG-022`, and `AR-CFG-039` retain
their scenarios with identity evidence replaced or supplemented by
`AR-PSI-002`, `AR-PSI-007` through `AR-PSI-009`, and
`AR-PSI-017` through `AR-PSI-027`. `AR-CFG-023` is replaced for
normal-fork behavior by `AR-PSI-010` through `AR-PSI-016` and
`AR-PSI-030`. `AR-CFG-003`, `AR-CFG-004`, `AR-CFG-019`, and
`AR-CFG-040` were reviewed and remain unchanged.

## 24.9 Function Calls and Sequencing Foundations 0.1

The remapping alternative in `MNIR-CALL-079` is superseded by
`MNIR-PSI-063` through `MNIR-PSI-065` and `MNIR-PSI-073`.

`MNIR-CALL-024`, `MNIR-CALL-077`, `MNIR-CALL-078`,
`MNIR-CALL-080` through `MNIR-CALL-082`, and `MNIR-CALL-138` remain
valid with persistent identity, allocation, snapshot, fork, and non-reuse
behavior supplied by `MNIR-PSI-003` through `MNIR-PSI-011`,
`MNIR-PSI-031` through `MNIR-PSI-045`, `MNIR-PSI-052`, and
`MNIR-PSI-055` through `MNIR-PSI-073`.

The reviewed rules `MNIR-CALL-004`, `MNIR-CALL-006` through
`MNIR-CALL-012`, `MNIR-CALL-083`, `MNIR-CALL-084`, and
`MNIR-CALL-127` remain unchanged.

`AR-CALL-001`, `AR-CALL-030`, `AR-CALL-032`, and `AR-CALL-057`
retain their scenarios with identity evidence supplemented by `AR-PSI-002`,
`AR-PSI-007`, `AR-PSI-008`, `AR-PSI-017`, and `AR-PSI-018`.
`AR-CALL-031` is replaced for normal-fork behavior by `AR-PSI-010`
through `AR-PSI-016` and `AR-PSI-030`.

The reviewed acceptance requirements `AR-CALL-003`, `AR-CALL-004`,
`AR-CALL-009`, `AR-CALL-012`, `AR-CALL-013`, `AR-CALL-026`,
`AR-CALL-033`, `AR-CALL-034`, `AR-CALL-047`, `AR-CALL-053`,
`AR-CALL-054`, `AR-CALL-056`, `AR-CALL-058`, and `AR-CALL-059`
remain unchanged.

---

# 25. Implementation constraints

## MNIR-PSI-093 — Core ownership

Persistent semantic entity ID and allocation-authority representation MUST
belong to `mnir-core`.

`mnir-core` MUST NOT depend on `mnir-verify`, EasyH, or an execution backend to
implement this specification.

---

## MNIR-PSI-094 — Controlled identity allocation

Normal safe public entity-creation APIs MUST allocate IDs through the mutable
lineage's active allocation authority.

They MUST NOT permit callers to select arbitrary namespace or counter
components.

---

## MNIR-PSI-095 — Encapsulation

The public API MUST NOT expose unrestricted mutation of counter state,
allocation authority, or contained entity IDs.

Read-only inspection needed to demonstrate this specification MAY be exposed.

---

## MNIR-PSI-096 — Safe Rust

Implementation of Persistent Semantic Identity 0.1 MUST NOT use or require
`unsafe` Rust.

---

## MNIR-PSI-097 — No external dependency requirement

This specification MUST NOT require a new external dependency when the
collision and persistence requirements can be satisfied correctly without one.

Any dependency introduced by an implementation requires explicit technical
justification.

---

## MNIR-PSI-098 — No speculative identity hierarchy

Implementation MUST NOT introduce a generic public `EntityId`, `NodeId`,
identity registry, import remapper, merge identity, or package identity
abstraction solely in anticipation of future specifications.

Typed covered ID categories MUST remain explicit.

---

## MNIR-PSI-099 — Identity-generation backend

The concrete mechanism used to generate fresh `ProgramId` and
`AllocationNamespaceId` values is implementation-defined. This includes the
OS randomness source, host-provided entropy, runtime or platform backend,
library or provider choice, retry strategy, and retry count.

The generation backend, its configuration, and unused internal candidates are
not MNIR semantic state. An implementation MUST nevertheless satisfy the
coordination-free collision-resistance requirements of this specification.

Provider-specific types or errors MUST NOT become part of MNIR semantic
identity semantics or cross the public MNIR API. Generation failure exposed by
that API MUST use an implementation-neutral typed MNIR error.

---

## MNIR-PSI-100 — New Program identity-generation failure

Creation of a new Program lineage MAY fail if the implementation cannot
generate every fresh persistent identity required by this specification.

Such failure MUST expose an implementation-neutral typed MNIR/API error, MUST
create no partially initialized Program lineage, MUST make no new Program
observable, and MUST NOT establish partial allocation authority.

---

## MNIR-PSI-101 — Normal-fork identity-generation failure

A normal fork MAY fail if generation of its required fresh `ProgramId` and
`AllocationNamespaceId` cannot complete successfully.

Failure MUST create no fork lineage and expose no partially created fork. It
MUST leave the source lineage unchanged, including its `ProgramId`,
`RevisionId`, semantic state, active allocation namespace, counter state, and
allocation authority.

Normal-fork creation is therefore atomic with respect to creation of the new
lineage.

---

## MNIR-PSI-102 — Incomplete internal generation

If generation of one internal candidate succeeds but another identity
required for the same new lineage fails before that lineage becomes
observable, the incomplete lineage MUST NOT become semantic Program state or
expose partial allocation authority.

An unused internal random candidate produced during such an attempt is not a
persistent semantic identity and need not be retained.

---

## MNIR-PSI-103 — Identity-generation retries

An implementation MAY retry after a locally detected candidate collision or
provider failure where appropriate. The retry strategy and retry limit are
implementation-defined.

An implementation MUST NOT claim that local collision detection establishes
the absence of collisions among independent processes. The coordination-free
collision-resistance requirements remain authoritative.

---

## MNIR-PSI-104 — Existing-lineage transaction allocation

Fresh `ProgramId` or `AllocationNamespaceId` generation is required only when
creating a new Program lineage or a normal fork in Persistent Semantic
Identity 0.1.

Ordinary Module, Function, Parameter, Block, and Expression creation within
an existing lineage MUST allocate through that lineage's active
namespace/counter authority and MUST NOT require fresh entropy. Identity-
generation failure therefore cannot arise from those ordinary
`MutationTransaction` allocation operations, and this specification defines
no transaction-poisoning behavior for such a failure.

---

# 26. Implementation scope summary

The required conceptual model is:

```text
Mutable Program lineage
├── ProgramId
├── current semantic revision
│   ├── RevisionId
│   └── persistent typed entity IDs and exact references
└── AllocationAuthorityState
    ├── AllocationNamespaceId
    └── CounterState
        ├── Available(next_counter)
        └── Exhausted
```

After a normal fork:

```text
source lineage
├── ProgramId P1
├── inherited FunctionId A:42
└── active authority A

fork lineage
├── ProgramId P2
├── inherited FunctionId A:42
└── active authority B
```

The source may next issue `A:57`; the fork may issue `B:1`. The inherited
`A:42` is the same persistent Function identity in both lineages.

Canonical serialization, import, package trust, semantic diff, and merge remain
future work.
