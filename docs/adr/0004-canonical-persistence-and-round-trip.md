# ADR 0004: Canonical Persistence and Round-Trip

## Status

Accepted

## Context

MNIR now has a canonical in-memory Program model, persistent semantic entity
identities, immutable snapshots, identity-preserving forks, Domain Types,
Text and Bytes values, explicit control flow, and explicit effect sequencing.
The next architectural milestone is a persistent representation that can cross
process and machine boundaries without losing identity or making later
mutation unsafe.

The required flow is:

```text
MnirProgram
    ↓ canonical serialize
.mnir
    ↓ deserialize
MnirProgram'
```

ADR 0003 established persistent entity identity and required future
persistence to preserve the active allocation authority. Persistent Semantic
Identity 0.1 subsequently separated committed semantic revision state from
allocation-authority state: successful identity issuance can advance the
allocator even when no new semantic `RevisionId` is created. Domain Type
Foundations 0.1 added `TypeId`, Text and Bytes values, Domain Type ownership,
and further identity-bearing references that persistence must preserve.

Persistence therefore cannot be a dump of current Rust structs, a semantic
snapshot alone, or a reconstruction from the entities that remain present.
It must be an explicit, versioned, canonical checkpoint of one mutable Program
lineage.

This ADR records an architectural direction. It is informative and does not
define normative serialization behavior. The exact wire schema, validation
rules, error contracts, and conformance requirements belong in a follow-up
normative specification.

## Decision

### Architectural principle

> A normal mutable `.mnir` artifact represents one canonical checkpoint of a
> committed Program-lineage head together with the persistent lineage state
> required to continue that lineage safely.

The artifact is not a transaction log, revision-history database,
verification certificate, source file, or runtime image.

### Persisted checkpoint model

The checkpoint has three conceptually distinct categories:

```text
PersistedLineageCheckpoint
├── semantic revision state
│   ├── ProgramId
│   ├── RevisionId
│   └── committed Program contents
├── presentation state
│   └── presentation metadata
└── persistent lineage/allocation state
    ├── entity AllocationNamespaceId
    ├── entity AllocationCounterState
    └── revision-allocation continuation state
```

These categories are all persisted, but persistence does not change their
semantic classification. Presentation metadata does not become semantic
identity, and allocation state does not become semantic revision content.

The round-trip target is exact persisted-state equivalence:

```text
deserialize(serialize(P)) ≡ persisted-state P
```

Here, persisted-state equivalence means exact equality of every persisted
identity, value, reference, role, semantic sequence, presentation value, and
lineage/allocation-state value. It does not mean equal Rust allocation
addresses or equal implementation container layout.

Canonical byte stability is a separate requirement:

```text
serialize(deserialize(serialize(P))) == serialize(P)
```

Neither relation defines general behavioral or semantic equivalence between
independently constructed Programs.

### Semantic revision state and allocator persistence state

The persisted semantic revision is the current committed Program head. It is
identified by the checkpoint's exact `ProgramId` and `RevisionId` and contains
the complete committed Program contents.

The active entity allocation authority is persistent lineage state outside
that semantic revision. An issued entity identity permanently advances this
state even if the issuing transaction fails or is discarded. Such
allocator-only advancement changes the persisted checkpoint without creating
a new semantic `RevisionId`. This is intentional and preserves the state
separation established by Persistent Semantic Identity 0.1.

Two canonical checkpoints may therefore have the same `ProgramId`,
`RevisionId`, and semantic contents but different entity allocator states.
They represent different persistence checkpoints of the same semantic head.
Canonical bytes cover the complete checkpoint and consequently differ when a
persisted allocator state differs.

### Revision-allocation audit

The current mutable `MnirProgram` stores revision allocation continuation
state separately from the current `RevisionId`. In the present implementation
this is represented by `next_revision_raw: Option<u64>`: `Some(n)` is the next
available raw revision value and `None` is exhaustion.

A mutable-lineage checkpoint must preserve this continuation state exactly,
conceptually as:

```text
RevisionAllocationState =
    Available(next_revision_value)
    | Exhausted
```

It must not be reconstructed merely by numerically incrementing the persisted
`RevisionId`. `RevisionId` is opaque, and preserving the continuation cursor
avoids coupling the wire contract to the current allocation algorithm. This
state is persistent lineage metadata, not entity allocation-authority state
and not semantic Program contents.

Specification 12 must define the invariant between the persisted current
`RevisionId` and revision-allocation continuation state and must reject an
impossible combination. It need not persist complete historical revisions or
the set of all prior `RevisionId` values.

### Checkpoint coherence

Serialization may read an immutable observation, but the observation used for
normal persistence must coherently pair:

1. the current committed semantic head;
2. the current presentation state associated with that head;
3. the current authoritative entity allocator observation; and
4. the current revision-allocation continuation state.

An arbitrary historical `ProgramSnapshot` is not automatically a safe mutable
lineage checkpoint. Its allocator observation may be older than later
successful reservations in the same lineage. Restoring such stale authority
could reuse an issued identity.

The implementation direction is therefore a distinct immutable persistence
checkpoint or equivalent atomic capture operation. It may reuse immutable
semantic state internally, but it must capture current allocator and revision
continuation state coherently. Historical `ProgramSnapshot` objects remain
immutable and do not gain mutation authority.

### Persisted contents

The canonical checkpoint must preserve at least:

```text
ProgramId
RevisionId
RevisionAllocationState

AllocationNamespaceId
AllocationCounterState
    Available(next_counter)
    | Exhausted

ModuleId
TypeId
FunctionId
ParameterId
BlockId
ExpressionId

Module ownership
Domain Type ownership
Domain representations
Function signatures
Parameter order
Function bodies
entry Blocks
all Blocks
Expressions
Expression kinds and dependencies
Return terminators
Branch terminators
true/false Branch target roles
Calls and target references
Call argument order
EffectSequence order
Text values
Bytes values

presentation metadata
```

All persistent IDs and all references between them round-trip exactly.
Ordinary deserialization performs no remapping. A loaded Program continues the
same Program lineage and the same committed head rather than creating a fork.

### Explicit exclusions

Persistence format 0.1 does not store:

- Active, Failed, Committed, or Discarded mutation transaction objects;
- transaction working state or transaction-resume data;
- complete Program revision history or revision ancestry;
- Git history;
- verifier diagnostics;
- `VerifiedProgram` evidence;
- cached type derivation or cached verifier results;
- runtime execution state;
- EasyH source or projections;
- semantic diff data;
- package registry metadata;
- import trust, provenance, signatures, or authorization;
- runtime or Rust memory representation.

Git remains responsible for repository history. The `.mnir` artifact stores
only the current committed head checkpoint and the lineage state required for
safe continuation.

### Active transactions

Active or otherwise separately retained mutation transactions are never part
of persistence 0.1. Serialization operates on committed lineage state.

If identity reservation associated with an uncommitted transaction has
already advanced the authoritative entity allocator, a subsequently captured
checkpoint must include that advanced allocator state even though it excludes
the transaction and its working semantic state. Implementations may coordinate
or temporarily block checkpoint capture to obtain a coherent observation; no
transaction-resume semantics are introduced.

### Canonical determinism

Canonical determinism applies to the complete persisted checkpoint. Two
in-memory representations with the same complete persisted state must produce
exactly the same bytes regardless of:

- `HashMap` iteration or insertion order;
- memory addresses or allocation layout;
- process or machine;
- construction or load history; or
- incidental traversal order.

Programs that are behaviorally equivalent but carry different persistent IDs
do not have the same persisted identity state and are not required to produce
the same bytes. This ADR does not define behavioral equivalence.

### Canonical collection ordering

Semantically unordered, identity-based collections use deterministic wire
ordering. The selected architectural principle is:

```text
canonical order = typed persistent identity order
```

This applies to homogeneous identity collections including Modules, Domain
Types, Functions, Blocks whose collection order is non-semantic, and
identity-based Expression collections. Specification 12 must define the exact
binary comparison rule for each typed identity and must define nesting and
field order in the wire schema.

Canonical wire order is not semantic execution order, presentation order, or
entity creation order. APIs must not expose it as new MNIR semantics.

Collections whose order is already semantic preserve that exact order:

- Function Parameters;
- Call arguments;
- Block `EffectSequence` entries;
- the distinct true and false roles of Branch targets; and
- any later sequence explicitly defined as semantically ordered.

Sorting one of these semantic sequences for canonicalization would change the
Program and is prohibited.

### Presentation metadata

Presentation metadata is included and must round-trip exactly where present,
including the distinction between absence and an explicitly present empty
value where the future wire schema can represent that distinction.

Persistence does not make presentation metadata semantic identity. A
presentation-only committed update retains its existing revision behavior,
and the canonical format retains the architectural separation among semantic,
presentation, and allocation/persistence state.

### Text and Bytes

Text round-trips as the exact finite sequence of Unicode scalar values. The
persistence layer must not normalize Unicode, case-fold, or apply locale
transformations. Canonically equivalent but scalar-distinct Text values remain
distinct persisted values.

Bytes round-trips as the exact finite octet sequence, including empty values
and arbitrary octets. It is not converted through a textual character
encoding as part of the abstract value model.

The wire-level encoding of both forms belongs to specification 12.

### Structural validation on deserialize

Deserialization is a trust boundary. It must not construct a publicly usable
mutable Program merely because the input is syntactically decodable.

A successful load must produce structurally valid MNIR under the applicable
structural rules. Validation must cover at least:

- typed ID representation integrity;
- duplicate persistent IDs and impossible namespace/counter reuse within the
  artifact;
- exact ownership and containment;
- required reference resolution and reference ownership;
- Domain Type representation and `ValueType` references;
- entity allocation-authority consistency;
- revision-allocation continuation consistency;
- Function, Parameter, body, Block, and entry-Block invariants;
- Expression dependency ownership and acyclicity;
- Return and Branch structural invariants;
- CFG target roles, reachability, and acyclicity; and
- Call and `EffectSequence` structural invariants, including exact-once
  membership and dependency/order consistency.

Malformed or structurally invalid input is rejected without silent repair,
identity remapping, dropped fields, or partial mutable Program construction.

Structural validity remains distinct from semantic verification. A
structurally valid Program with arithmetic, comparison, Return, Branch, Call,
or Domain semantic diagnostics is loadable. Deserialization does not require
successful V0_5 verification.

### Allocator safety after deserialize

Entity allocation-authority state is restored exactly:

```text
Available(104) → Available(104)
Exhausted      → Exhausted
```

The next counter must never be inferred solely from the largest currently
present entity ID. Deleted entities and failed, discarded, or removed
provisional work may have consumed higher counters.

For `Available(n)`, the decoder must validate that the checkpoint does not
claim an entity issued by the active authority at counter `n` or above. For
`Exhausted`, the authority remains exhausted and does not rotate namespace.
Entities from foreign or inherited namespaces grant no allocation authority.

Loading a checkpoint must not create fresh `ProgramId`, `RevisionId`,
`AllocationNamespaceId`, or entity IDs. Continued mutation uses only the
restored active authority and revision-allocation continuation state.

### Verification evidence

Normal persistence stores Program state, not verification certification.
`VerifiedProgram`, diagnostics, and verification caches are excluded.

After deserialization a caller may verify the loaded immutable revision under
an explicitly selected rule set. Previously produced verification evidence is
not assumed valid merely because the Program that produced it was persisted.

### Failure model

The architecture distinguishes at least:

```text
unsupported format or version
malformed encoding
structurally invalid MNIR
```

from:

```text
structurally valid but semantically invalid MNIR
```

The first three fail loading. The final category loads successfully and may
subsequently produce verifier diagnostics. Exact Rust error types and payloads
belong to specification 12 and implementation.

### Wire-format isolation

The external format is defined by a stable explicit schema, never by:

- Rust struct layout;
- Rust enum discriminant values or declaration order;
- derived serialization of internal structs;
- `HashMap` iteration;
- compiler ABI;
- debug output; or
- pointer or allocation layout.

Stable explicit schema and tag values are required for all persisted variants,
including `IntrinsicType`, `ValueType`, `ExpressionKind`, and `Terminator`.
`VerificationRuleSet` is not Program checkpoint state and is therefore not
persisted in format 0.1.

The selected crate direction is:

```text
mnir-format ───→ mnir-core
mnir-core  -X→ mnir-format
```

`mnir-format` will own the envelope, wire schema, canonical encoder, bounded
decoder, format errors, and conversion to and from controlled `mnir-core`
persistence boundaries. `mnir-core` remains the owner of semantic types and
structural invariants and must not depend on the external format layer.

No crate is created by this ADR.

### Format identity and versioning

The persistence format has its own explicit identity and version, independent
of:

- the MNIR product release version;
- Cargo package versions;
- individual normative specification versions; and
- verifier rule-set versions.

The direction is an explicit `major.minor` format version or an equivalent
pair in the file envelope. Specification 12 defines its exact encoding and the
initial supported value.

Initial compatibility is strict:

- unsupported major versions are rejected;
- unknown semantic tags are rejected;
- fields or constructs whose meaning cannot be preserved are not ignored;
- lossy best-effort loading is not supported.

Minor-version forward compatibility is deferred. Until a later specification
defines safe extension rules, a decoder accepts only format versions and
schema constructs it explicitly supports.

### Selected wire-encoding direction

The selected architectural direction is a narrowly profiled deterministic
CBOR encoding carrying an explicit MNIR wire schema.

CBOR is selected as a carrier because it can represent exact unsigned 64-bit
values, byte strings, Unicode text strings, arrays, maps, and explicitly
tagged structures without depending on Rust layout. The future MNIR profile
will represent 128-bit `ProgramId` and `AllocationNamespaceId` values as exact
fixed-width bytes rather than normalized text. Deterministic encoding rules
can eliminate alternative byte representations for the same schema value,
while definite lengths and bounded decoding remain compatible with streaming
implementations.

CBOR by itself is not the MNIR schema and does not guarantee canonical MNIR
bytes. Specification 12 must define:

- the exact deterministic CBOR profile;
- the file envelope and format version fields;
- stable explicit MNIR field and variant tags;
- integer and fixed-width identity encodings;
- definite-length and map/array ordering requirements;
- duplicate-key rejection;
- permitted and prohibited CBOR features; and
- canonical validation of input encodings.

In particular, implementation-derived enum ordering, generic reflected struct
serialization, alternate integer widths, indefinite-length encodings, and
unconstrained map ordering must not create multiple canonical encodings.

No CBOR library or other external dependency is authorized by this ADR. A
future implementation task that needs one requires a separate explicit
dependency decision under `AGENTS.md` unless an authoritative task mandates
it.

### Encoding alternatives considered

#### Explicit custom deterministic binary encoding

Not selected as the initial direction. It offers complete control and could
satisfy all requirements, but it would require MNIR to design, implement, and
audit low-level integer, length, collection, text, extension, and streaming
rules that a suitable standard data model already supplies. The explicit MNIR
schema is still required with CBOR, but the binary primitives need not be
invented again.

#### Deterministic CBOR with an explicit MNIR profile

Selected. It provides suitable primitive representations and a documented
deterministic foundation while preserving MNIR control over identities, tags,
ordering, validation, and versioning. Its flexibility is also its main risk,
so the normative profile must be narrow and exact rather than accepting every
valid CBOR representation.

#### Canonical text format

Not selected as the canonical machine representation. Text could be made
canonical, but it adds escaping, whitespace, numeric spelling, binary-data,
and Unicode-normalization hazards without making the semantic model inherently
more reviewable. Human readability is better supplied by deterministic
projections and Git tooling.

### Resource limits and hostile input

Persistence parsing must be designed for hostile input. A decoder must not
trust encoded lengths, nesting, counts, or references before checking them.
The architecture requires:

- checked length and integer arithmetic;
- bounds before allocation;
- bounded nesting or an iterative decoding strategy;
- bounded aggregate entities, strings, bytes, and references according to an
  explicit resource policy;
- rejection of duplicate or conflicting fields;
- no panics, stack exhaustion, or uncontrolled allocation for expected bad
  input; and
- failure without exposing partially initialized mutable lineage state.

Exact limits and configurability belong to specification 12 or its
implementation contract. This trust boundary does not introduce package
provenance, signatures, or authorization semantics.

### Filesystem update mechanics

Canonical bytes and their decoding are format concerns. Temporary files,
`fsync`, atomic rename, crash-safe replacement, file permissions, and recovery
are storage/CLI concerns.

Later tooling should provide an explicit crash-safe save policy, but filesystem
APIs and replacement mechanics do not belong in `mnir-core` and are not MNIR
semantic state. A serializer may produce bytes or write a caller-provided
stream without defining repository update behavior.

### Git and human projection

The conventional extension is:

```text
.mnir
```

The extension is a tooling convention, not semantic identity. Canonical
machine-oriented persistence is suitable for Git blob storage and is not
required to be directly text-diffable.

The intended progression is:

```text
canonical .mnir persistence
    ↓
Git storage
    ↓
textconv human projection
    ↓
semantic diff
    ↓
semantic merge later
```

Text projection does not become canonical state. Semantic diff and merge are
outside persistence 0.1.

### Round-trip acceptance direction

Specification 12 must require acceptance evidence for exact persisted-state
round-trip and canonical byte stability covering at least:

- an empty/minimal Program;
- multiple Modules;
- Domain Types and representations;
- Text and Bytes, including empty and non-ASCII/scalar-distinct values;
- Functions, return types, Parameters, and Parameter order;
- single- and multiple-Block bodies and entry identity;
- every arithmetic and comparison Expression kind;
- Branch target roles and CFG structure;
- Calls, Call argument order, and target identity;
- `EffectSequence` order;
- presentation metadata, including absence and present values;
- entity allocation authority in both `Available` and `Exhausted` states;
- revision-allocation continuation in available and exhausted states;
- snapshots and forks where applicable to checkpoint capture;
- construction-history and `HashMap`-order independence; and
- structurally valid but semantically invalid Programs.

Tests must distinguish same complete persisted identity state from merely
behaviorally similar state. Persistent-ID differences do not satisfy the
precondition for canonical-byte equality.

## Rationale

A committed-head checkpoint is the smallest useful persistence unit that can
restore a mutable MNIR lineage without embedding a history database. Including
allocator state closes the identity-reuse gap that would arise from serializing
only currently present entities. Including revision continuation state avoids
deriving opaque revision allocation from the head ID.

Deterministic CBOR plus an explicit MNIR schema balances control and
standardization. The schema remains authoritative; CBOR supplies only a binary
data model and deterministic primitive encoding. Isolating it in
`mnir-format` keeps external wire concerns out of `mnir-core` while permitting
core-owned structural validation and controlled reconstruction.

Strict decoding, exact identity preservation, and separation from semantic
verification allow persistence to remain lossless and safe without turning
the loader into a semantic repair or certification step.

## Consequences

- A `.mnir` artifact represents one current mutable-lineage checkpoint, not
  complete revision history.
- Allocator-only advancement can change canonical bytes without changing
  `RevisionId` or semantic Program contents.
- Program, revision, namespace, Type, entity, reference, presentation, and
  ordered semantic data round-trip exactly.
- Safe restoration requires current allocator state and cannot use a stale
  historical snapshot observation.
- Revision allocation continuation becomes explicit persisted lineage state.
- Unordered in-memory collections require identity-based canonical ordering.
- Semantic sequences retain their existing order and roles.
- Deserialization rejects malformed and structurally invalid state but accepts
  structurally valid semantic invalidity.
- Verification evidence is recomputed rather than persisted as certification.
- The wire format is isolated in a future `mnir-format` crate depending on
  `mnir-core`.
- A deterministic CBOR profile is the selected carrier, subject to exact
  normative definition and a later dependency decision.
- Git history, textconv, semantic diff, and semantic merge remain separate
  layers.

## Follow-up normative specification

The next normative milestone is tentatively:

```text
Canonical Serialization and Deserialization 0.1
```

as specification 12. It must define:

- the exact file envelope and `.mnir` format identity;
- the initial independent format version;
- the deterministic CBOR profile;
- stable field and semantic-variant tags;
- exact encodings for every persisted identity and construct;
- canonical field and collection ordering;
- complete encode and decode rules;
- checkpoint coherence and revision-continuation representation;
- structural and allocator validation;
- unsupported-version, malformed-encoding, and structural error contracts;
- resource-safety requirements and limits/policy;
- construction-history independence; and
- the complete round-trip and canonical-byte acceptance suite.

The specification must not add behavioral equivalence, persistence of active
transactions, revision history, verifier evidence, import trust, packages,
semantic diff, merge, EasyH, execution state, or filesystem save mechanics
merely to complete persistence 0.1.

## Deferred questions

- What exact magic/envelope bytes and initial format version are used?
- What exact deterministic CBOR subset and well-formedness checks apply?
- What numeric values are assigned to stable fields and semantic tags?
- What exact binary comparison rule orders typed identities?
- What public persistence-checkpoint API safely captures semantic head,
  allocator state, and revision continuation?
- What controlled reconstruction API lets `mnir-format` create a valid mutable
  `MnirProgram` without weakening normal mutation encapsulation?
- Which resource limits are fixed by conformance and which are caller
  configurable?
- Which external CBOR implementation, if any, satisfies the dependency,
  canonicality, streaming, and hostile-input requirements?
- What crash-safe file replacement policy will later CLI/storage tooling use?
- What textconv projection and Git attributes will later tooling provide?

These are required specification or implementation decisions, but none blocks
writing specification 12 because the required decision boundaries and
acceptance scope are now explicit.
