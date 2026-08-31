# MNIR Specification — Program Model

**Document:** `01-program-model.md`  
**Specification status:** Draft  
**Specification version:** 0.1  
**Normative:** Yes

---

## 1. Purpose

This document defines the foundational program container, identity, revision, mutation, and presentation model of MNIR.

This specification increment intentionally defines only the concepts required to establish a stable foundation for later MNIR semantics.

It defines:

- `Program`
- `Module`
- `ProgramId`
- `ModuleId`
- identifier uniqueness domains
- identifier lifetime
- program revisions
- controlled mutation
- atomic mutation commit
- structural validity
- presentation metadata
- foundational terminology for future semantic verification
- foundational terminology for future human-readable projections

This document does not define:

- functions
- parameters
- variables
- expressions
- primitive types
- semantic type checking
- semantic verification rules
- effects
- contracts
- security rules
- policies
- patterns
- module imports
- cross-program references
- serialization encoding
- EasyH syntax
- EasyH rendering
- execution backends

Those concepts are introduced by later specification increments.

---

# 2. Normative language

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative only when they occur inside a numbered MNIR rule.

Normative requirements in this document use stable identifiers of the form:

```text
MNIR-CORE-NNN
```

Acceptance requirements use identifiers of the form:

```text
AR-CORE-NNN
```

`AR-CORE-*` requirements are mandatory project-level acceptance requirements for the initial implementation increment defined by this document.

The initial implementation increment is complete only when every applicable `AR-CORE-*` requirement has been demonstrated by automated tests or another explicitly stated verification mechanism.

Acceptance requirements do not independently introduce MNIR semantics. They demonstrate behavior required by the numbered `MNIR-CORE-*` rules.

Examples, rationale, notes, and explanatory text do not introduce additional normative requirements.

If explanatory text appears to conflict with a numbered normative rule, the numbered rule takes precedence.

---

# 3. Terminology

## 3.1 Program

A **Program** is the aggregate root of an MNIR program state.

A Program owns the modules that belong to that Program.

Program-level revision and mutation operate on the Program as a whole.

The Rust implementation is expected to expose a concept corresponding to:

```text
MnirProgram
```

The exact Rust type name is implementation-defined.

---

## 3.2 Module

A **Module** is a semantic container owned by exactly one Program.

In specification version 0.1, a Module does not yet contain functions, types, expressions, or other child semantic entities.

The Module concept is introduced in version 0.1 only to establish:

- containment by a Program,
- typed semantic identity,
- controlled mutation,
- revision behavior,
- and presentation metadata.

---

## 3.3 Model

The term **model** is not a distinct normative aggregate in MNIR.

Documentation may use phrases such as:

```text
MNIR data model
MNIR program model
```

in their ordinary descriptive sense.

Normative rules concerning ownership, revision, mutation, and identity refer specifically to a **Program** or another explicitly defined entity.

---

## 3.4 Program lineage

A **Program lineage** is the sequence of committed revisions that represent the continued evolution of one Program identity.

Conceptually:

```text
ProgramId(P1)

Revision R1
    ↓
Revision R2
    ↓
Revision R3
```

All revisions in this sequence belong to the same Program lineage.

---

## 3.5 Snapshot

A **snapshot** is an immutable observation or copy of one specific committed Program revision.

Two snapshots may represent the same Program identity and revision.

A snapshot is not, by itself, a new Program lineage.

---

## 3.6 Fork

A **fork** creates a new Program lineage based on the contents of an existing Program revision.

The fork has a new Program identity.

Detailed merge semantics between Program lineages are outside the scope of version 0.1.

---

## 3.7 Current revision

A mutable Program lineage has one **current revision**, also referred to as its **head revision**.

The head revision is the latest committed revision from which additional revisions may be created within that Program lineage.

Specification version 0.1 defines Program lineage evolution as linear.

Older snapshots remain valid immutable observations of previous revisions, but they are not mutation roots for the same Program lineage.

Continuing development from an older snapshot requires creation of a fork with a new `ProgramId`.

---

## 3.8 No-op transaction

A **no-op transaction** is an Active transaction whose working Program state is equal to its source Program state immediately before commit when comparing:

- Module membership,
- Module identities,
- Module presentation metadata,
- and committed Module identifier allocation history,

while explicitly excluding the new `RevisionId` that commit itself will allocate.

An empty transaction is therefore a no-op transaction.

Setting a presentation field to the value it already contains may also result in a no-op transaction.

A transaction that allocates a Module identifier and later removes that Module before commit is not a no-op, because successful commit changes the committed Module identifier allocation history.

---

# 4. Program

## MNIR-CORE-001 — Program as aggregate root

A Program MUST be the aggregate root for the MNIR state defined by this specification.

Modules belonging to the Program MUST be owned through the Program.

Mutation transactions defined by this specification MUST target a Program revision rather than independently committing module revisions.

---

## MNIR-CORE-002 — Program structure

A structurally valid Program MUST contain:

- one `ProgramId`,
- one `RevisionId`,
- and a collection containing zero or more Modules.

The Module collection MAY be empty.

---

## MNIR-CORE-003 — Canonical semantic container

The Program MUST be representable without requiring textual source code.

The Program MUST NOT require EasyH source, parser state, source formatting, or another frontend representation to exist.

---

## MNIR-CORE-004 — Module ownership

Every Module contained in a Program MUST belong to that Program.

A Module MUST NOT simultaneously belong to multiple Program lineages as the same semantic Module identity.

A snapshot of a Program MAY contain a snapshot of that Module without creating a second ownership relationship.

---

# 5. Identifier categories

The initial specification defines three typed identifier categories:

```text
ProgramId
RevisionId
ModuleId
```

Future specifications may introduce additional categories such as:

```text
FunctionId
ParameterId
ExpressionId
TypeId
```

Those identifiers do not exist normatively until introduced by their corresponding specification.

---

## MNIR-CORE-005 — Typed identifier categories

`ProgramId`, `RevisionId`, and `ModuleId` MUST be distinct semantic identifier categories.

The implementation SHOULD use distinct Rust types for these categories where practical.

An identifier from one category MUST NOT be accepted where another identifier category is required merely because the internal representations happen to be compatible.

---

## MNIR-CORE-006 — Opaque identifiers

The internal representation of an identifier MUST NOT carry normative semantic meaning beyond identity.

Consumers MUST NOT infer:

- creation order,
- hierarchy,
- ownership,
- timestamp,
- memory location,
- or business meaning

from the raw representation of an identifier unless a future specification explicitly defines such semantics.

---

# 6. Program identity

## MNIR-CORE-007 — Program identity

Every Program lineage MUST have one `ProgramId`.

The `ProgramId` identifies that Program lineage independently of its current revision.

---

## MNIR-CORE-008 — Program identity uniqueness

Distinct Program lineages MUST have distinct `ProgramId` values.

If an implementation detects two distinct Program lineages carrying the same `ProgramId`, it MUST treat the condition as an identity collision.

An identity collision MUST NOT be silently interpreted as evidence that the two lineages are the same Program.

The concrete generation mechanism used to minimize or prevent `ProgramId` collisions is implementation-defined in specification version 0.1.

---

## MNIR-CORE-053 — Program identity collision rejection

An operation that requires combining, comparing as identical, attaching, or otherwise associating two Program lineages MUST reject the operation if the implementation knows that the lineages are distinct but their `ProgramId` values collide.

The diagnostic representation for this condition is outside the scope of version 0.1.

---

## MNIR-CORE-009 — Program snapshots

A snapshot of an existing Program revision MUST preserve the `ProgramId` and `RevisionId` of the revision being represented.

Creating another in-memory representation of the same snapshot MUST NOT implicitly create a new Program identity.

---

## MNIR-CORE-010 — Program fork

A fork that is intended to evolve independently from the source Program lineage MUST receive a new `ProgramId`.

The fork MAY initially contain equivalent Program contents.

A fork MUST NOT be represented as a new independent lineage using the source Program's `ProgramId`.

---

# 7. Module identity

## MNIR-CORE-011 — Module identity domain

A `ModuleId` MUST be unique among all Module identities allocated within one Program lineage.

The uniqueness domain of `ModuleId` is therefore the Program lineage.

Conceptually:

```text
ProgramId(P1)
    ModuleId(M1)
    ModuleId(M2)
```

is valid.

The following is not valid within the same Program lineage:

```text
ProgramId(P1)
    ModuleId(M1)
    ModuleId(M1)
```

---

## MNIR-CORE-012 — Typed raw values

Different typed identifier categories MAY use equal raw internal values.

For example, an implementation MAY internally represent:

```text
ProgramId(17)
ModuleId(17)
RevisionId(17)
```

at the same time.

These identifiers are not equal because their semantic identifier categories differ.

---

## MNIR-CORE-013 — Module identity lifetime

Once a `ModuleId` has become part of a successfully committed Program revision, that `ModuleId` MUST NOT later be assigned to a different Module identity in the same Program lineage.

This rule continues to apply after the original Module has been removed.

Identifiers allocated provisionally by transactions that do not commit are governed by `MNIR-CORE-066`.

---

## MNIR-CORE-014 — Module identity preservation

Updating an existing Module through an operation defined as an update MUST preserve that Module's `ModuleId`.

This includes presentation metadata updates defined by this specification.

---

## MNIR-CORE-015 — Module replacement

Removing a Module and later creating another Module constitutes creation of a new Module identity.

The newly created Module MUST receive a `ModuleId` that has not previously become a committed Module identity within that Program lineage.

The implementation MUST NOT infer identity preservation merely because the new Module has presentation metadata equal to the removed Module.

---

# 8. Module identifier allocation history

A Program lineage maintains identifier allocation history independently of the set of Modules present in its current revision.

This history exists to guarantee that removed Module identities are never accidentally reused.

## MNIR-CORE-054 — Committed Module identifier history

A mutable Program lineage MUST maintain sufficient allocation history to determine which `ModuleId` values have previously become committed Module identities in that lineage.

This history MUST include Module identifiers that belonged to Modules which were later removed.

---

## MNIR-CORE-055 — Permanent retirement

Once a `ModuleId` has become part of a successfully committed Program revision, that identifier MUST remain unavailable for allocation to another Module identity in the same Program lineage.

Removal of the Module MUST NOT make its `ModuleId` available for reuse.

---

## MNIR-CORE-056 — Mutable lineage persistence requirement

Any persisted representation that is intended to restore a Program lineage for future mutation MUST preserve enough Module identifier allocation history to continue satisfying `MNIR-CORE-055`.

The concrete serialization representation of that history is outside the scope of version 0.1.

---

## MNIR-CORE-057 — Snapshot allocation history

An immutable snapshot is not required to expose Module identifier allocation history through its public API.

A snapshot that is restored as the mutable continuation of the same Program lineage MUST nevertheless restore sufficient allocation history to satisfy `MNIR-CORE-055`.

---

# 9. Identity during copy, fork, import, and merge

## MNIR-CORE-016 — Snapshot copy

Copying one Program revision as a snapshot of the same Program lineage MUST preserve:

- `ProgramId`,
- `RevisionId`,
- and all contained `ModuleId` values.

---

## MNIR-CORE-017 — Forked Module identifiers

When a Program is forked into a new Program lineage, the fork MAY preserve the raw `ModuleId` values of the source Program.

Because Module identity is scoped by Program lineage, the semantic identities remain distinct.

For example:

```text
(ProgramId(P1), ModuleId(M1))
```

and:

```text
(ProgramId(P2), ModuleId(M1))
```

represent different Module identities when `P1 != P2`.

---

## MNIR-CORE-018 — Cross-program import

Specification version 0.1 does not define semantic Module import between Programs.

An implementation claiming only version 0.1 conformance MUST NOT invent externally visible identity-preservation guarantees for cross-program Module import.

Import semantics are reserved for a later specification.

---

## MNIR-CORE-019 — Program merge

Specification version 0.1 does not define merging of independently evolving Program lineages.

An implementation MAY provide experimental merge tooling, but such behavior is outside MNIR 0.1 conformance.

---

# 10. Module collection

## MNIR-CORE-020 — Module collection semantics

The Program's Module collection MUST be treated as a collection of Module identities rather than as a sequence whose position defines identity.

The position of a Module in an implementation-specific container MUST NOT contribute to its semantic identity.

---

## MNIR-CORE-021 — Module lookup

A Module contained by a Program MUST be addressable by its `ModuleId`.

Module lookup MUST NOT depend solely on presentation metadata such as a preferred human-readable name.

---

# 11. Presentation metadata

Specification version 0.1 defines a minimal presentation metadata structure for Modules.

It consists of two optional values:

```text
preferred_name: optional text
documentation: optional text
```

These fields exist to demonstrate separation between semantic identity and human presentation.

---

## MNIR-CORE-022 — Presentation metadata separation

Module presentation metadata MUST NOT determine Module identity.

Changing presentation metadata MUST preserve the Module's `ModuleId`.

---

## MNIR-CORE-023 — Preferred name

A Module MAY have a `preferred_name`.

The preferred name is intended for human-readable tools.

Two Modules within the same Program MAY have equal preferred names in specification version 0.1.

Name uniqueness and name resolution are not defined by this specification.

---

## MNIR-CORE-024 — Documentation metadata

A Module MAY contain documentation text.

Documentation metadata MUST NOT affect Module identity.

---

## MNIR-CORE-025 — Missing presentation metadata

A Module MUST remain structurally valid when either or both presentation metadata fields are absent.

---

# 12. Revision model

## MNIR-CORE-026 — Program revision

Every committed Program state MUST have one `RevisionId`.

The `RevisionId` identifies one specific committed state within a Program lineage.

---

## MNIR-CORE-027 — Revision uniqueness

A `RevisionId` MUST NOT be reused for two different committed states within the same Program lineage.

---

## MNIR-CORE-028 — Revision identity domain

The uniqueness domain of `RevisionId` is the Program lineage.

The same raw `RevisionId` value MAY occur in another Program lineage without implying identity between those revisions.

---

## MNIR-CORE-029 — Revision opacity

Consumers MUST NOT infer chronological ordering from the raw representation of a `RevisionId`.

A future specification MAY introduce explicit ancestry or ordering mechanisms.

---

## MNIR-CORE-030 — New committed revision

Every successful mutation transaction commit MUST produce a new `RevisionId`.

This rule applies even when the transaction:

- changes only presentation metadata,
- produces semantically equivalent contents,
- or is a no-op with respect to observable Program contents.

This rule intentionally favors simple and deterministic revision semantics in version 0.1.

---

## MNIR-CORE-031 — Previous revision preservation

Creating a new committed revision MUST NOT retroactively change the contents represented by an already materialized snapshot of the previous revision.

Implementations MAY use mutable internal storage, copy-on-write storage, persistent data structures, or another implementation strategy as long as externally observable revision semantics satisfy this rule.

---

## MNIR-CORE-058 — Linear lineage

Specification version 0.1 defines each mutable Program lineage as a linear sequence of committed revisions.

Only the current head revision of a Program lineage MAY be used as the source of a new committed revision in that same lineage.

---

## MNIR-CORE-059 — Historical snapshot mutation

An older snapshot MUST NOT be used to create another committed revision in the same Program lineage.

To continue development from an older snapshot, the implementation MUST create a fork with a new `ProgramId`.

This rule prevents multiple independently evolving branches from sharing one Program identity in version 0.1.

---

# 13. Controlled mutation

## MNIR-CORE-032 — Controlled mutation

Committed Program state MUST NOT be modified through unrestricted external mutation that can bypass Program structural invariants.

Changes to committed state MUST occur through a controlled mutation mechanism.

---

## MNIR-CORE-033 — Mutation transaction target

A mutation transaction MUST begin from the current head revision of one Program lineage.

The transaction MUST retain the identity of that source revision for the lifetime of the transaction.

A transaction MUST NOT commit against a different Program lineage or against a different source revision than the one from which it began.

Behavior for concurrent transactions started from the same head revision is outside the scope of version 0.1.

---

## MNIR-CORE-034 — Initial mutation operations

The initial implementation MUST provide controlled mutation operations equivalent to:

```text
add_module
remove_module
set_module_preferred_name
set_module_documentation
```

The concrete Rust method names are implementation-defined.

`add_module` MUST allocate the new Module identity through the Program lineage's Module identifier allocation mechanism.

The standard version 0.1 mutation API MUST NOT require or permit a normal caller to select the raw `ModuleId` for a newly created Module.

---

## MNIR-CORE-060 — Mutation transaction states

A mutation transaction MUST have an implementation-visible lifecycle equivalent to the following logical states:

```text
Active
Failed
Committed
Discarded
```

A newly begun mutation transaction MUST enter the `Active` state.

Only an `Active` transaction MAY accept mutation operations or attempt commit.

A successful commit MUST transition the transaction to the `Committed` state.

Discarding an `Active` or `Failed` transaction MUST transition it to the `Discarded` state.

A `Committed` or `Discarded` transaction MUST be terminal.

A `Failed` transaction MUST NOT be committed.

---

## MNIR-CORE-061 — Operation failure poisons transaction

If any mutation operation fails while a transaction is Active, the transaction MUST enter the Failed state.

A Failed transaction MUST NOT accept additional mutation operations.

A Failed transaction MAY be discarded.

All uncommitted changes belonging to a Failed transaction MUST remain isolated from committed Program state.

---

## MNIR-CORE-062 — Sequential mutation semantics

Mutation operations within one Active transaction MUST be applied sequentially to the transaction's working state in the order in which the operations are issued.

Each operation observes the result of all previous successful operations in that transaction.

---

## MNIR-CORE-063 — Unknown Module mutation

A mutation operation that requires an existing Module MUST fail if the referenced `ModuleId` does not identify a Module in the transaction's current working state.

This rule applies at minimum to:

- remove Module,
- set preferred name,
- set documentation.

Such a failure MUST cause the transaction behavior defined by `MNIR-CORE-061`.

---

## MNIR-CORE-064 — Provisional Module identifiers

A Module identifier allocated during an Active mutation transaction is provisional until that transaction commits successfully.

A provisional identifier:

- MAY be used to address the new Module within that same transaction;
- MUST NOT be treated as a committed durable Module identity before commit;
- MUST NOT collide with any committed Module identifier in the Program lineage;
- MUST NOT collide with another provisional Module identifier in the same transaction.

---

## MNIR-CORE-065 — Provisional identifier commit

When a transaction commits successfully, every `ModuleId` allocated during that transaction MUST become part of the Program lineage's committed Module identifier allocation history.

This rule applies even if a Module was created and subsequently removed within the same successfully committed transaction.

---

## MNIR-CORE-066 — Provisional identifier after failed transaction

If a transaction does not commit successfully, its provisional Module identifiers do not become committed Module identities.

Specification version 0.1 does not require those provisional raw identifier values to remain permanently retired.

Consumers MUST NOT rely on the uniqueness or durability of a provisional identifier after its transaction has failed or been discarded.

---

The following table is informative; normative behavior is defined by `MNIR-CORE-061`, `MNIR-CORE-062`, and `MNIR-CORE-063`.

| Operation sequence          | Result                             |
| --------------------------- | ---------------------------------- |
| `add M; update M`           | valid                              |
| `add M; remove M`           | valid                              |
| `add M; remove M; update M` | transaction fails on update        |
| `update M; remove M`        | valid when M initially exists      |
| `remove M; update M`        | transaction fails on update        |
| `remove M; remove M`        | transaction fails on second remove |
| `update unknown M`          | transaction fails                  |
| `remove unknown M`          | transaction fails                  |

---

## MNIR-CORE-035 — Atomic commit

A mutation transaction commit MUST be atomic with respect to committed Program state.

A transaction either:

- commits one structurally valid new Program revision,

or:

- does not modify the committed source revision.

A failed transaction MUST NOT leave a partially committed Program state.

---

## MNIR-CORE-036 — Structural validation before commit

A mutation transaction MUST NOT commit a resulting Program state that violates the structural rules defined by the applicable specification version.

---

## MNIR-CORE-037 — Failed commit

When commit fails because the proposed result is structurally invalid, the transaction MUST NOT create a new committed revision.

The source revision MUST remain valid and unchanged.

---

## MNIR-CORE-038 — Successful commit

A successful commit MUST:

- produce a structurally valid Program state,
- preserve the Program's `ProgramId`,
- allocate a new `RevisionId`,
- and preserve or allocate Module identities according to the identity rules in this specification.

---

## MNIR-CORE-039 — No-op commit

The version 0.1 mutation API MUST permit an Active no-op transaction to commit successfully.

A successful no-op commit MUST produce a new `RevisionId`.

All other Program state defined by the no-op comparison remains unchanged.

---

# 14. Structural validity

Structural validity in this specification applies only to the concrete concepts defined in version 0.1.

It does not attempt to define structural requirements for future functions, types, expressions, or references.

---

## MNIR-CORE-040 — Structurally valid Program

A Program is structurally valid under this specification when all of the following are true:

1. it contains exactly one value of the `ProgramId` identifier category;
2. it contains exactly one value of the `RevisionId` identifier category;
3. every contained Module contains exactly one value of the `ModuleId` identifier category;
4. no two contained Modules have the same `ModuleId`;
5. every contained Module is owned by that Program;
6. every committed Module identifier satisfies the non-reuse requirements of the Program lineage.

Specification version 0.1 defines no reserved or intrinsically invalid raw identifier values.

Identifier construction APIs MAY impose representation-level restrictions, but such restrictions do not become MNIR semantics unless specified normatively.

---

## MNIR-CORE-041 — Structurally valid Module

A Module is structurally valid under this specification when:

1. it contains exactly one value of the `ModuleId` identifier category;
2. its optional presentation metadata can be represented by the implementation;
3. it is contained by exactly one owning Program within the committed Program state.

No additional Module fields are mandatory in version 0.1.

---

## MNIR-CORE-042 — Structural rejection

An operation that would create a Program state violating `MNIR-CORE-040` or `MNIR-CORE-041` MUST be rejected before commit completes.

---

# 15. Semantic validity

Specification version 0.1 does not define semantic program rules beyond the structural and identity semantics established by this document.

The distinction between structural validity and future semantic validity remains important, but no semantic verifier is introduced by this specification increment.

A later specification will define concrete semantic rules that can succeed or fail independently of structural validity.

---

# 16. Future verification relationship

This section is informative and introduces no normative verification requirements.

Future MNIR specifications are expected to distinguish between:

```text
MnirProgram
```

and an artifact or state demonstrating that a specific Program revision has passed a defined verification process.

Verification is expected to depend on more than a revision alone.

Potential verification context includes:

```text
ProgramId
RevisionId
MNIR specification version
verification profile
active rule set
security profile
```

The exact structure is intentionally deferred until concrete semantic verification rules exist.

Specification version 0.1 therefore does not require:

```text
VerifiedProgram
```

or any equivalent API type.

---

# 17. Future EasyH relationship

This section is informative and introduces no EasyH conformance requirements.

MNIR is intended to support human-readable projections.

EasyH is the planned first human-oriented language and projection for MNIR.

The architectural intent is:

```text
Human
  ↓
EasyH
  ↓
MNIR
```

and:

```text
AI / semantic tooling
  ↓
MNIR
  ↓
EasyH projection
  ↓
Human
```

Future general-purpose MNIR semantic constructs are expected to have complete human-readable EasyH representations.

That expectation becomes normative only when the corresponding EasyH specification defines the required representation.

Specification version 0.1 does not require an EasyH renderer.

---

# 18. Frontend independence

## MNIR-CORE-043 — Text-independent construction

It MUST be possible to construct the Program and Module structures defined by this specification through a semantic API without first generating textual source code.

---

## MNIR-CORE-044 — Frontend neutrality

The core Program and Module structures defined by this specification MUST NOT contain required fields whose meaning depends specifically on EasyH syntax.

Optional generic presentation metadata defined by this specification is permitted.

---

# 19. Identity examples

The examples in this section are informative.

## 19.1 Presentation update preserves identity

Before:

```text
ProgramId(P1)
RevisionId(R1)

ModuleId(M1)
preferred_name = "orders"
```

Mutation:

```text
set preferred_name of ModuleId(M1) to "sales_orders"
```

After commit:

```text
ProgramId(P1)
RevisionId(R2)

ModuleId(M1)
preferred_name = "sales_orders"
```

The Program identity and Module identity are preserved.

The revision identity changes.

---

## 19.2 Remove and create produces new Module identity

Before:

```text
ProgramId(P1)
RevisionId(R2)

ModuleId(M1)
```

Mutation:

```text
remove ModuleId(M1)
create new module
```

After commit:

```text
ProgramId(P1)
RevisionId(R3)

ModuleId(M2)
```

`M2` represents a new Module identity.

The implementation does not reuse `M1`.

---

## 19.3 Same raw Module ID in different Program lineages

The following may both exist:

```text
ProgramId(P1)
ModuleId(7)
```

and:

```text
ProgramId(P2)
ModuleId(7)
```

These are different semantic Module identities because the Module identity domain is the Program lineage.

---

## 19.4 Fork

Source:

```text
ProgramId(P1)
RevisionId(R10)
ModuleId(M1)
ModuleId(M2)
```

Fork:

```text
ProgramId(P2)
RevisionId(...)
ModuleId(M1)
ModuleId(M2)
```

The raw Module identifier values may be preserved.

The Module identities are nevertheless distinct because they belong to different Program lineages.

The initial revision identifier assigned to the fork is implementation-defined in version 0.1.

---

# 20. Mutation examples

The examples in this section are informative.

## 20.1 Successful add

```text
Revision R1
Modules:
    M1
```

Transaction:

```text
add module M2
```

Commit:

```text
Revision R2
Modules:
    M1
    M2
```

---

## 20.2 Failed unknown Module mutation

```text
Revision R2
Modules:
    M1
```

Transaction:

```text
add module M2
remove unknown ModuleId(M999)
```

The second operation fails and the transaction enters the Failed state.

The transaction cannot commit, and committed Revision R2 remains unchanged without M2.

---

## 20.3 Presentation-only transaction

```text
Revision R2

M1.preferred_name = "orders"
```

Transaction:

```text
M1.preferred_name = "sales_orders"
```

Commit:

```text
Revision R3
```

Version 0.1 assigns a new revision even though only presentation metadata changed.

---

# 21. Specification gaps

## MNIR-CORE-045 — Undefined semantics

When an implementation task requires externally observable MNIR semantics that are not defined by the applicable normative specification, the implementation MUST NOT silently establish those semantics as part of MNIR conformance.

The missing decision MUST be reported as a specification gap.

---

## MNIR-CORE-046 — Specification gap format

A specification gap report SHOULD use the following form:

```text
SPECIFICATION GAP

Location:
<specification section or rule>

Problem:
<what behavior is undefined or contradictory>

Required decision:
<what specification decision is needed>
```

---

# 22. Explicitly unresolved topics

The following topics are intentionally unresolved by specification version 0.1:

- concrete `ProgramId` encoding
- concrete `ModuleId` encoding
- concrete `RevisionId` encoding
- identifier bit width
- random versus sequential identifier generation
- persistent serialization encoding
- module imports
- module exports
- cross-program references
- cross-program identity transfer
- merge semantics
- merge conflict resolution
- concurrent mutation transactions
- distributed mutation
- semantic verification
- verification profiles
- verification artifacts
- type semantics
- functions
- expressions
- name resolution
- Module preferred-name uniqueness
- package semantics
- source locations
- comment preservation
- EasyH syntax
- EasyH rendering
- semantic equivalence
- execution behavior
- backend representation

Implementations may experiment with these areas, but such experiments do not define MNIR 0.1 semantics.

---

# 23. Initial implementation acceptance requirements

The first implementation increment for this document is intentionally narrow.

## AR-CORE-001 — Program construction

Demonstrate construction of an empty Program containing:

```text
ProgramId
RevisionId
zero Modules
```

and demonstrate that it satisfies the structural rules of this specification.

---

## AR-CORE-002 — Module creation

Demonstrate adding a Module through the controlled mutation mechanism.

After successful commit:

- the Program identity is preserved,
- the Module has a `ModuleId`,
- and the Program has a new `RevisionId`.

---

## AR-CORE-003 — Typed identifier separation

Demonstrate that `ProgramId`, `ModuleId`, and `RevisionId` are distinct API-level identifier categories.

Where technically practical in Rust, accidental interchange should fail at compile time.

---

## AR-CORE-004 — Program-scoped Module uniqueness

Demonstrate that two contained Modules cannot have the same `ModuleId` within one Program lineage.

---

## AR-CORE-005 — Module ID non-reuse

Demonstrate:

1. creation of a Module,
2. removal of that Module,
3. creation of another Module,

and verify that the new Module does not reuse the removed Module's identity.

---

## AR-CORE-006 — Presentation update preserves identity

Demonstrate changing:

```text
preferred_name
```

and:

```text
documentation
```

without changing:

```text
ProgramId
ModuleId
```

A successful commit still produces a new `RevisionId`.

---

## AR-CORE-007 — Atomic failed mutation

Demonstrate the following sequence:

1. begin a transaction from the current Program head;
2. successfully apply at least one mutation operation;
3. attempt to remove or update an unknown `ModuleId`;
4. observe operation failure;
5. attempt to commit or otherwise inspect transaction state.

Verify that:

- the transaction is Failed;
- commit is not permitted;
- no new committed revision is created;
- the earlier successful transaction-local change is not visible in committed Program state;
- and the source Program revision remains unchanged.

---

## AR-CORE-008 — Remove Module

Demonstrate removing an existing Module through controlled mutation and committing a new structurally valid revision.

---

## AR-CORE-009 — No-op revision behavior

Demonstrate committing an empty Active mutation transaction.

Verify that:

- commit succeeds;
- Program identity is unchanged;
- Module collection is unchanged;
- Module presentation metadata is unchanged;
- Module identifier allocation history is unchanged;
- and a new `RevisionId` is produced.

---

## AR-CORE-010 — No textual source dependency

Demonstrate all acceptance requirements using semantic Rust APIs without requiring:

- EasyH source,
- source parsing,
- or another textual frontend.

---

## AR-CORE-011 — Crate boundary

Demonstrate that `mnir-core` has no dependency on:

```text
easyh-render
```

and contains no EasyH-specific syntax model.

---

## AR-CORE-012 — Sequential operation behavior

Demonstrate at least:

```text
add module
set preferred name on the newly added module
commit
```

and verify that the update observes the Module created earlier in the same transaction.

Also demonstrate:

```text
remove existing module
set preferred name on the removed ModuleId
```

and verify that the second operation fails and the transaction becomes Failed.

---

## AR-CORE-013 — Committed Module ID retirement

Demonstrate:

1. create and commit a Module;
2. record its `ModuleId`;
3. remove and commit that Module;
4. create and commit another Module.

Verify that the newly created Module does not receive the removed Module's `ModuleId`.

---

## AR-CORE-014 — Provisional identity

Demonstrate that a Module created inside an Active transaction can be addressed by its provisional `ModuleId` within that transaction.

Verify that the identifier becomes a committed Module identity only after successful commit.

---

## AR-CORE-015 — Historical snapshots are not lineage mutation roots

Demonstrate that an older Program snapshot cannot be used to create a new committed revision in the same Program lineage once a newer head revision exists.

Continuing from that historical state requires creation of a fork with a new `ProgramId`.

---

# 24. Implementation constraints

## MNIR-CORE-047 — No speculative entities

The implementation of this specification MUST NOT introduce normative semantic entities such as:

- Function
- Parameter
- Variable
- Expression
- Type
- Effect
- Contract

solely in anticipation of future specification sections.

---

## MNIR-CORE-048 — No generic node abstraction requirement

This specification does not require a generic `Node`, `NodeId`, or universal reference abstraction.

The implementation MUST NOT introduce such an abstraction as a normative MNIR concept unless justified independently by a later specification or approved architecture decision.

Internal implementation abstractions that do not establish externally observable MNIR semantics are permitted.

---

## MNIR-CORE-049 — Safe Rust

The implementation MUST NOT require `unsafe` Rust to satisfy the requirements of this specification.

---

## MNIR-CORE-050 — Dependency direction

The `mnir-core` crate MUST remain independent of:

- `mnir-verify`,
- `easyh-render`,
- and `mnir-cli`.

Higher-level crates MAY depend on `mnir-core`.

---

## MNIR-CORE-051 — No semantic verifier in this increment

Implementation of this specification MUST NOT introduce a semantic `VerifiedProgram` concept merely to satisfy anticipated future verification requirements.

Structural validation required for controlled mutation is part of `mnir-core` and is not considered semantic program verification.

---

## MNIR-CORE-052 — No EasyH renderer requirement

Implementation of this specification MUST NOT require EasyH rendering to claim conformance with the Program Model 0.1 increment.

---

# 25. Implementation scope summary

The first implementation resulting from this specification is expected to remain approximately within the following conceptual scope:

```text
Mutable Program lineage
├── ProgramId
├── Head Program revision
│   ├── RevisionId
│   └── Modules
│       └── Module
│           ├── ModuleId
│           └── PresentationMetadata
│               ├── preferred_name
│               └── documentation
└── Committed ModuleId allocation history
```

with controlled mutation operations conceptually equivalent to:

```text
begin mutation
add module
remove module
set preferred name
set documentation
commit
discard
```

and lineage or snapshot operations conceptually equivalent to:

```text
create snapshot
fork from snapshot
```

This diagram is informative and does not prescribe exact Rust API names or storage representation.

---

# 26. Future specification sequence

A likely future specification sequence is:

```text
01 Program Model
02 Type System Foundations
03 Functions and Parameters
04 Expressions
05 Semantic Verification and Diagnostics
06 Effects
07 Contracts
08 Security Model
09 Policies
10 Patterns
11 EasyH
12 Serialization
13 Execution Backends
```

The exact sequence may change as design work progresses.

Later specifications may extend the Program and Module structures introduced here.

Such extensions are expected to preserve the identity and mutation foundations unless this specification is explicitly revised.

---

# 27. Foundational invariants

The following is an informative summary of the normative rules established by this document:

```text
Program is the aggregate root.

Program and model are not separate normative aggregates.

A Program lineage has one ProgramId.

Distinct Program lineages have distinct ProgramId values.

Modules are owned by one Program.

ModuleId uniqueness is scoped to a Program lineage.

ModuleId values are never reused for different Module identities
within the same Program lineage.

A mutable Program lineage retains committed ModuleId allocation history.

Provisional ModuleIds become durable identities only through commit.

Identifier categories are typed and semantically distinct.

Identifier raw values are opaque.

A snapshot preserves ProgramId, RevisionId, and ModuleIds.

A fork receives a new ProgramId.

A mutable Program lineage is linear, and only its head revision
may produce the next revision in that lineage.

Every successful commit produces a new RevisionId.

An Active no-op transaction may commit and receives a new RevisionId.

All mutation commits are atomic.

Failed structural mutations leave committed state unchanged.

Mutation operations are sequential, and an operation failure
places the transaction in the Failed state.

Presentation metadata does not determine Module identity.

Textual source code is not required to construct MNIR.

Semantic verification is intentionally not defined yet.

EasyH conformance is intentionally not required yet.

Undefined semantics are specification gaps, not implementation choices.
```
