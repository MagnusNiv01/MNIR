# MNIR Specification — Canonical Serialization and Deserialization

**Document:** `12-canonical-serialization-and-deserialization.md`  
**Specification version:** 0.1  
**Persistence format:** 1.0  
**Status:** Draft normative specification

## 1. Purpose

This document defines the initial canonical persistence format for one
committed MNIR Program-lineage checkpoint.

It defines:

- the exact `.mnir` 1.0 file envelope;
- a restricted deterministic CBOR profile;
- stable field positions and semantic tags;
- exact identity, type, Expression, control-flow, and presentation encodings;
- canonical ordering;
- entity- and revision-allocation persistence;
- strict decoding and structural validation;
- resource-safety limits;
- immutable persistence-checkpoint capture and atomic reconstruction; and
- conformance and golden-vector requirements.

The required flow is:

```text
MnirProgram / committed lineage checkpoint
    ↓ canonical encode
canonical .mnir bytes
    ↓ decode and structural validation
mutable MnirProgram lineage
```

The target relations are:

```text
deserialize(serialize(P)) ≡ persisted-state P
```

and:

```text
serialize(deserialize(serialize(P))) == serialize(P)
```

This document follows accepted ADR 0004. It extends specifications `01`
through `11` only for persistence and does not redefine their MNIR semantics.

## 2. Normative language

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY**
are normative requirements.

Rules use stable identifiers of the form `MNIR-SER-*`. Acceptance requirements
use identifiers of the form `AR-SER-*`.

## 3. Authority and scope

### MNIR-SER-001 — Persistence authority

This document is authoritative for canonical `.mnir` persistence format 1.0.
Specifications `00` through `11` remain authoritative for Program semantics,
identity, structure, mutation, verification, and Domain Type behavior.

### MNIR-SER-002 — Persisted-state round-trip

A successful canonical encode/decode round-trip MUST preserve the complete
persisted checkpoint state exactly.

Persisted-state equality includes every persisted identity, field, reference,
role, ordered sequence, presentation value, entity-allocation state, and
revision-allocation state.

### MNIR-SER-003 — Canonical byte stability

Encoding the result of decoding valid canonical format-1.0 bytes MUST reproduce
the exact original bytes.

Exact byte equality is required only when complete persisted checkpoint state
is equal. Independently created behaviorally equivalent Programs carrying
different persistent identities are not required to encode identically.

### MNIR-SER-004 — Persistence is not behavioral equivalence

This specification MUST NOT be interpreted as defining general behavioral or
semantic equivalence between Programs.

## 4. Persisted lineage checkpoint

### MNIR-SER-005 — Checkpoint categories

A format-1.0 checkpoint MUST contain four conceptually distinct categories:

1. current committed semantic revision contents;
2. presentation state belonging to that committed head;
3. current persistent entity-allocation authority state; and
4. current revision-allocation continuation state.

Persistence MUST NOT cause presentation or allocator state to become semantic
Program identity or semantic execution behavior.

### MNIR-SER-006 — Current committed head

The checkpoint MUST represent exactly one current committed Program head and
MUST preserve its `ProgramId` and current `RevisionId` exactly.

Complete historical Program revisions and revision ancestry MUST NOT be
required in format 1.0.

### MNIR-SER-007 — Complete committed contents

The checkpoint MUST preserve every current committed Module, Domain Type,
Function, Parameter, Function body, Block, Expression, terminator, reference,
and semantic sequence defined by specifications `01` through `11`.

### MNIR-SER-008 — Presentation state

Every currently defined `preferred_name` and `documentation` value MUST be
persisted exactly, including absence versus a present empty string.

### MNIR-SER-009 — Entity allocator state

The checkpoint MUST preserve the current active `AllocationNamespaceId` and
complete `AllocationCounterState` exactly.

### MNIR-SER-010 — Revision allocator state

The checkpoint MUST preserve the revision-allocation continuation state needed
to create a fresh next `RevisionId` without requiring historical revisions.

### MNIR-SER-011 — Allocator-only checkpoint change

Entity allocator advancement without semantic commit MUST leave `RevisionId`
and semantic revision contents unchanged but MAY, and normally will, change
canonical checkpoint bytes.

### MNIR-SER-012 — Coherent persistence observation

Encoding MUST consume one immutable observation that coherently pairs the
current committed head, its presentation state, the current entity allocator,
and the current revision allocator.

### MNIR-SER-013 — Historical snapshot is insufficient by default

An arbitrary historical `ProgramSnapshot` MUST NOT be treated as a restorable
mutable-lineage checkpoint merely because it contains an allocator observation.
Its allocator observation may be stale relative to later issuance.

### MNIR-SER-014 — Transactions are excluded

Active, Failed, Committed, and Discarded transaction objects, transaction
working copies, and transaction-resume state MUST NOT be persisted.

If an uncommitted transaction has already issued IDs, a later checkpoint MUST
nevertheless include the resulting authoritative allocator advancement.

## 5. File envelope

### MNIR-SER-015 — Conventional extension

The conventional filename extension is `.mnir`. The extension is a tooling
convention and is not semantic identity.

### MNIR-SER-016 — Exact envelope

A format-1.0 file MUST contain, in order:

| Offset | Length | Meaning | Required bytes |
| ---: | ---: | --- | --- |
| 0 | 8 | MNIR magic | `89 4d 4e 49 52 0d 0a 1a` |
| 8 | 2 | unsigned major version, big-endian | `00 01` |
| 10 | 2 | unsigned minor version, big-endian | `00 00` |
| 12 | remaining | one canonical CBOR checkpoint payload | schema in section 10 |

The magic consists of a non-text-leading byte, ASCII `MNIR`, CR/LF, and the
control byte `1a`. No byte-order marker is permitted.

### MNIR-SER-017 — Independent format version

The envelope version is independent from MNIR release versions, Cargo package
versions, individual specification versions, and verification rule-set
versions.

### MNIR-SER-018 — Exact version acceptance

A format-1.0 decoder MUST accept only major `1`, minor `0`.

Any other well-formed version pair MUST fail as `UnsupportedFormatVersion`.
A truncated version or malformed envelope MUST fail as `MalformedEncoding`.

### MNIR-SER-019 — One payload and no trailing data

Exactly one CBOR item MUST begin at offset 12 and MUST consume all remaining
bytes. Missing payload or trailing bytes MUST fail as `MalformedEncoding`.

### MNIR-SER-020 — No built-in checksum or compression

Format 1.0 MUST contain no checksum, signature, MAC, or built-in compression.
External storage may compress bytes. Such compression is outside canonical
`.mnir` bytes and provides no MNIR trust semantics.

## 6. Restricted deterministic CBOR profile

### MNIR-SER-021 — Permitted CBOR major types

The payload MAY use only:

- major type 0: unsigned integers;
- major type 1: negative integers;
- major type 2: byte strings;
- major type 3: text strings;
- major type 4: arrays; and
- the simple values `false` and `true`.

CBOR maps, semantic tags, floating-point values, `null`, `undefined`, and every
other simple value MUST NOT appear.

### MNIR-SER-022 — Definite length only

Byte strings, text strings, and arrays MUST use definite lengths. Indefinite
lengths, break codes, and chunked string representations MUST fail as
`NonCanonicalEncoding`.

### MNIR-SER-023 — Shortest integer and length encoding

Every integer value and every string or array length MUST use the shortest
valid CBOR additional-information representation:

- values `0..=23` are encoded directly;
- values through `u8::MAX` use additional information 24 and one byte;
- values through `u16::MAX` use additional information 25 and two bytes;
- values through `u32::MAX` use additional information 26 and four bytes;
- larger supported values use additional information 27 and eight bytes.

The initial byte is `(major_type << 5) | additional_information`. Every
two-, four-, or eight-byte argument following the initial byte MUST be encoded
unsigned most-significant byte first (network/big-endian order).

Additional-information values 28 through 30 are invalid. A longer encoding
for a value that has a shorter encoding MUST fail as `NonCanonicalEncoding`.

### MNIR-SER-024 — Signed integer encoding

Non-negative signed literal values MUST use major type 0. A negative value `n`
MUST use major type 1 with encoded unsigned argument `-1 - n`, using the
shortest representation required by `MNIR-SER-023`.

### MNIR-SER-025 — No maps or CBOR semantic tags

Format 1.0 uses fixed-position arrays and MUST NOT contain CBOR maps or major
type 6 semantic tags. Stable MNIR variant tags are ordinary canonical unsigned
integers in fixed array positions.

### MNIR-SER-026 — Text validity

Every CBOR text string MUST be shortest-length, definite, valid UTF-8 encoding
of a Unicode scalar-value sequence. Overlong UTF-8, surrogate code points,
invalid scalar encodings, and malformed UTF-8 MUST fail as
`MalformedEncoding`.

No Unicode normalization, case folding, locale transformation, or byte-order
mark insertion/removal is permitted.

### MNIR-SER-027 — Canonical-only decoder

The canonical `.mnir` decoder MUST reject a well-formed CBOR value that
violates any format-1.0 canonical profile rule as `NonCanonicalEncoding`. It
MUST NOT accept non-canonical input merely because re-encoding could normalize
it.

## 7. Common schema forms

### MNIR-SER-028 — Fixed-position arrays

Every composite schema value MUST be a definite-length array with the exact
length and field positions specified by this document. No field is omitted
unless its schema is an explicit option.

### MNIR-SER-029 — Stable numeric tags

Every variant tag defined in this document is stable for persistence format
1.0 and MUST NOT be derived from Rust enum ordering, discriminants, names,
debug output, or a generic serializer's layout.

### MNIR-SER-030 — Unknown schema data

Unknown variant tags, wrong array lengths, extra fields, missing fields, and
values of the wrong CBOR major type MUST fail as `InvalidWireSchema`.

### MNIR-SER-031 — Option encoding

An optional value `T` MUST be encoded as:

| Meaning | Schema |
| --- | --- |
| absent | `[0]` |
| present | `[1, T]` |

Every other option tag or array length MUST fail as `InvalidWireSchema`.

## 8. Identifier encoding

### MNIR-SER-032 — 128-bit identity encoding

`ProgramId` and `AllocationNamespaceId` MUST each encode as a CBOR byte string
of exactly 16 bytes. Byte index 0 is the most-significant/network-order byte
for canonical comparison and display-independent wire interpretation.

No UUID text, hexadecimal text, integer alternative, or CBOR bignum tag is
permitted.

### MNIR-SER-033 — RevisionId encoding

`RevisionId` MUST encode as one canonical unsigned CBOR integer in the range
`0..=u64::MAX`. Its wire integer is opaque MNIR identity; consumers MUST NOT
infer semantic chronology from it.

### MNIR-SER-034 — Typed entity ID encoding

Each `ModuleId`, `TypeId`, `FunctionId`, `ParameterId`, `BlockId`, and
`ExpressionId` MUST encode as:

```text
[namespace_id, counter]
```

where `namespace_id` is the exact 16-byte encoding from `MNIR-SER-032` and
`counter` is a canonical unsigned integer in `0..=u64::MAX`. This preserves
Persistent Semantic Identity 0.1's implementation-defined first valid counter.

### MNIR-SER-035 — Typed category from schema position

The typed ID category is determined by the enclosing schema field. An ID MUST
NOT carry a textual or redundant numeric category tag. Decoders MUST retain
the statically required category and MUST NOT accept one category in another
category's schema position.

### MNIR-SER-036 — Exact reference encoding

Every semantic reference MUST use the complete typed ID encoding, including
all 16 namespace bytes and the counter. Ordinary decode MUST NOT remap an ID or
rewrite a reference.

### MNIR-SER-037 — Typed identity comparison order

Canonical order within one homogeneous typed-ID collection is lexicographic:

1. compare the 16 namespace bytes as unsigned bytes from index 0 through 15;
2. if namespaces are equal, compare counters as mathematical unsigned
   integers.

The first unequal component determines order. This wire order does not create
MNIR semantic entity order.

## 9. Allocation continuation encoding

### MNIR-SER-038 — Revision allocation state

Revision allocation state MUST encode as exactly one of:

| State | Schema | Tag |
| --- | --- | ---: |
| next revision available | `[0, next_revision]` | 0 |
| exhausted | `[1]` | 1 |

`next_revision` is a canonical unsigned integer in `1..=u64::MAX`.

### MNIR-SER-039 — Revision continuation invariant

For format 1.0, revision state MUST satisfy exactly one of:

- current `RevisionId < u64::MAX` and state is
  `Available(current_revision + 1)`; or
- current `RevisionId == u64::MAX` and state is `Exhausted`.

This is a persistence/allocation invariant and does not grant semantic
chronology to general `RevisionId` consumers. Any other combination MUST fail
as `StructurallyInvalidProgram`.

### MNIR-SER-040 — Fresh post-load revision

The next successful commit after load MUST issue the restored available
revision value and advance it by one, or transition to exhausted after issuing
`u64::MAX`. An exhausted revision allocator MUST reject a further commit
without reusing a revision ID.

### MNIR-SER-041 — Entity allocation authority

Entity allocation authority MUST encode as:

```text
[namespace_id, counter_state]
```

where `counter_state` is exactly one of:

| State | Schema | Tag |
| --- | --- | ---: |
| available | `[0, next_counter]` | 0 |
| exhausted | `[1]` | 1 |

`next_counter` is in `0..=u64::MAX`.

### MNIR-SER-042 — Exact allocator restoration

Decode MUST restore `namespace_id` and counter state exactly. `Available(n)`
MUST remain `Available(n)`, and `Exhausted` MUST remain `Exhausted`.

### MNIR-SER-043 — No counter reconstruction

The decoder MUST NOT derive `next_counter` from the largest currently present
entity ID. Deleted or failed/discarded issuance may have consumed absent higher
counters.

### MNIR-SER-044 — Allocation-authority validity

For `Available(n)`, every contained entity minted in the active namespace MUST
have counter less than `n`. No namespace/counter pair may occur on more than
one contained persistent entity, even across typed categories. `Exhausted`
MUST grant no further issuance. Entities from other namespaces MUST NOT grant
allocation authority.

A violation MUST fail as `StructurallyInvalidProgram`.

## 10. Top-level checkpoint schema

### MNIR-SER-045 — Checkpoint payload

The CBOR payload MUST be the five-element array:

| Position | Field | Schema |
| ---: | --- | --- |
| 0 | Program identity | `ProgramId` |
| 1 | current revision | `RevisionId` |
| 2 | revision allocator | revision allocation state |
| 3 | entity allocator | allocation authority |
| 4 | Modules | canonical Module array |

The Program model currently defines no Program-level presentation fields, so
format 1.0 contains no Program presentation field.

### MNIR-SER-046 — Module schema

A Module MUST encode as:

```text
[module_id, presentation, domain_types, functions]
```

`domain_types` and `functions` are arrays sorted according to
`MNIR-SER-037` by their respective IDs.

### MNIR-SER-047 — Domain Type schema

A Domain Type MUST encode as:

```text
[type_id, intrinsic_representation, presentation]
```

Its owning Module is the Module that contains it. No duplicate owner field is
encoded.

### MNIR-SER-048 — Function schema

A Function MUST encode as:

```text
[function_id, return_value_type, parameters, body, presentation]
```

`parameters` preserves exact semantic Parameter order and MUST NOT be sorted.

### MNIR-SER-049 — Parameter schema

A Parameter MUST encode as:

```text
[parameter_id, value_type, presentation]
```

Its owning Function and position are determined by its exact position in the
containing Function's Parameter array.

### MNIR-SER-050 — Presentation schema

Presentation metadata MUST encode as:

```text
[preferred_name_option, documentation_option]
```

Each option uses `MNIR-SER-031`, and each present value is a CBOR text string.

## 11. Type schema

### MNIR-SER-051 — IntrinsicType tags

`IntrinsicType` MUST encode as one canonical unsigned integer:

| Tag | Intrinsic type |
| ---: | --- |
| 0 | `Int32` |
| 1 | `Int64` |
| 2 | `Bool` |
| 3 | `Unit` |
| 4 | `Text` |
| 5 | `Bytes` |

### MNIR-SER-052 — ValueType tags

`ValueType` MUST encode as:

| Alternative | Schema | Tag |
| --- | --- | ---: |
| intrinsic | `[0, intrinsic_type]` | 0 |
| Domain | `[1, type_id]` | 1 |

No representation type is duplicated in a Domain `ValueType`; it remains a
live property of the referenced Domain Type.

## 12. Function bodies and Blocks

### MNIR-SER-053 — Function body option

Function body state MUST encode as:

| State | Schema |
| --- | --- |
| absent | `[0]` |
| present | `[1, [entry_block_id, blocks]]` |

`blocks` is sorted by `BlockId`. A present body MUST explicitly name its entry
Block and MUST NOT infer entry identity from collection position.

### MNIR-SER-054 — Block schema

A Block MUST encode as:

```text
[block_id, expressions, effect_sequence, terminator]
```

`expressions` is sorted by `ExpressionId`. `effect_sequence` is an array of
complete `ExpressionId` values in exact semantic order. A committed Block has
one terminator and therefore uses no terminator option.

### MNIR-SER-055 — Flat reference structure

Blocks and Expressions MUST be encoded as flat identity collections with ID
references. Expression dependencies and CFG successors MUST NOT be recursively
inlined.

## 13. Expression schema

### MNIR-SER-056 — Expression record

Every Expression MUST encode as:

```text
[expression_id, expression_kind]
```

The kind array contains the stable tag in position 0 followed by exactly the
fields in the following table.

### MNIR-SER-057 — Complete Expression kind table

| Tag | Kind | Exact kind schema |
| ---: | --- | --- |
| 0 | `Int32Literal` | `[0, signed_value]` |
| 1 | `Int64Literal` | `[1, signed_value]` |
| 2 | `BoolLiteral` | `[2, boolean]` |
| 3 | `UnitLiteral` | `[3]` |
| 4 | `TextLiteral` | `[4, text]` |
| 5 | `BytesLiteral` | `[5, bytes]` |
| 6 | `ParameterReference` | `[6, parameter_id]` |
| 7 | `Add` | `[7, left_expression_id, right_expression_id]` |
| 8 | `Subtract` | `[8, left_expression_id, right_expression_id]` |
| 9 | `Multiply` | `[9, left_expression_id, right_expression_id]` |
| 10 | `Divide` | `[10, left_expression_id, right_expression_id]` |
| 11 | `Remainder` | `[11, left_expression_id, right_expression_id]` |
| 12 | `Equal` | `[12, left_expression_id, right_expression_id]` |
| 13 | `NotEqual` | `[13, left_expression_id, right_expression_id]` |
| 14 | `LessThan` | `[14, left_expression_id, right_expression_id]` |
| 15 | `LessThanOrEqual` | `[15, left_expression_id, right_expression_id]` |
| 16 | `GreaterThan` | `[16, left_expression_id, right_expression_id]` |
| 17 | `GreaterThanOrEqual` | `[17, left_expression_id, right_expression_id]` |
| 18 | `Call` | `[18, target_function_id, arguments]` |
| 19 | `DomainConstruct` | `[19, type_id, source_expression_id]` |
| 20 | `DomainProject` | `[20, source_expression_id]` |

No cached derived type, representation, effect classification, source order,
or verifier result is encoded.

### MNIR-SER-058 — Integer literal ranges

An `Int32Literal` value MUST be in `-2^31..=2^31-1`. An `Int64Literal` value
MUST be in `-2^63..=2^63-1`. Values use `MNIR-SER-024`; out-of-range values
MUST fail as `InvalidWireSchema`.

### MNIR-SER-059 — Bool and Unit literals

A Bool literal payload MUST be the CBOR simple value `false` (`f4`) or `true`
(`f5`). Unit has no payload beyond kind tag 3.

### MNIR-SER-060 — Text literals

A Text literal MUST be one CBOR text string satisfying `MNIR-SER-026`. Exact
Unicode scalar values MUST round-trip without normalization or segmentation.

### MNIR-SER-061 — Bytes literals

A Bytes literal MUST be one definite-length CBOR byte string. Every octet and
its position MUST round-trip exactly.

### MNIR-SER-062 — Call arguments

The Call `arguments` field is an array of complete `ExpressionId` values in
exact semantic argument order. It MUST NOT be sorted. Effectfulness is derived
from Call kind under current semantics and MUST NOT be separately encoded.

### MNIR-SER-063 — Domain Expression references

`DomainConstruct` MUST preserve exact `TypeId` and source `ExpressionId`.
`DomainProject` MUST preserve exact source `ExpressionId`. Neither encodes a
cached Domain representation or derived `ValueType`.

## 14. Terminator schema

### MNIR-SER-064 — Terminator tags

A terminator MUST encode as:

| Tag | Terminator | Exact schema |
| ---: | --- | --- |
| 0 | `Return` | `[0, expression_id]` |
| 1 | `Branch` | `[1, condition_expression_id, true_block_id, false_block_id]` |

### MNIR-SER-065 — Branch roles

True and false Branch target positions are semantic roles. They MUST be
preserved exactly and MUST NOT be sorted even when false target identity
compares before true target identity.

## 15. Canonical collection ordering

### MNIR-SER-066 — Unordered collection sorting

The following arrays MUST be strictly increasing under `MNIR-SER-037`:

- top-level Modules by `ModuleId`;
- Domain Types within a Module by `TypeId`;
- Functions within a Module by `FunctionId`;
- Blocks within a present body by `BlockId`; and
- Expressions within a Block by `ExpressionId`.

### MNIR-SER-067 — Duplicate and unsorted rejection

An identity collection containing equal adjacent IDs, duplicate IDs anywhere,
or non-increasing order MUST fail canonical decode. Non-increasing but otherwise
structurally meaningful input fails as `NonCanonicalEncoding`; duplicate
identity fails as `StructurallyInvalidProgram`.

### MNIR-SER-068 — Semantic sequences

Parameter arrays, Call arguments, and `EffectSequence` entries MUST retain
their exact semantic order. They MUST NOT be sorted for canonical encoding.

### MNIR-SER-069 — No order inference

Canonical collection position MUST NOT create execution order, creation order,
presentation order, identity, or other MNIR semantics for collections whose
order is otherwise non-semantic.

## 16. Decode pipeline and structural boundary

### MNIR-SER-070 — Decode stages

Decode MUST conceptually proceed through these stages:

```text
bytes
↓ envelope validation
↓ CBOR syntax and canonical-profile validation
↓ fixed wire-schema decoding
↓ candidate persisted checkpoint
↓ identity, allocator, reference, and ownership validation
↓ complete MNIR structural validation
↓ atomic mutable-lineage reconstruction
```

### MNIR-SER-071 — Atomic reconstruction

No partially decoded or partially reconstructed mutable `MnirProgram` MAY
become externally observable after any failure.

### MNIR-SER-072 — Identity and ownership validation

Decode MUST reject duplicate or invalid IDs, impossible namespace/counter
reuse, duplicate ownership, invalid containment, unresolved structurally
required references, wrong typed-ID categories, and cross-owner references
prohibited by specifications `01` through `11`.

### MNIR-SER-073 — Body and CFG validation

Decode MUST reject invalid body presence, missing or invalid entry Block,
missing required terminators, unreachable Blocks, invalid Branch targets,
cross-body targets, and cyclic CFGs.

### MNIR-SER-074 — Expression validation

Decode MUST reject unresolved Parameter, Expression, Function, or Type
references where structural resolution is required; foreign-Block Expression
dependencies; and cyclic shared Expression dependency graphs.

### MNIR-SER-075 — EffectSequence validation

Decode MUST reject non-Call sequence entries, foreign-Block entries,
duplicates, missing Calls, Calls occurring other than exactly once, and
dependency/effect-order conflicts.

### MNIR-SER-076 — Complete structural validation

After wire-specific validation, the candidate MUST satisfy the complete
structural validity rules applicable under specifications `01` through `11`.
The decoder MUST NOT silently repair, drop, replace, normalize, or remap state.

### MNIR-SER-077 — Semantic invalidity remains loadable

A structurally valid Program MUST decode successfully even when V0_5 would
diagnose arithmetic or comparison incompatibility, Return mismatch, Branch
type invalidity, Call argument invalidity, or Domain construction/projection
invalidity.

### MNIR-SER-078 — Verification is separate

Normal decode MUST NOT run semantic verification and MUST NOT require a
`VerifiedProgram`. Previously generated evidence and diagnostics are not
persisted and do not certify the loaded Program.

## 17. Resource-safety profile

### MNIR-SER-079 — Maximum canonical file size

The complete canonical file, including the 12-byte envelope, MUST NOT exceed
`1,073,741,824` bytes (`2^30`). A larger input MUST fail as
`ResourceLimitExceeded` before proportional allocation.

### MNIR-SER-080 — String and byte limits

One Text or Bytes value, including presentation strings, MUST NOT exceed
`268,435,456` encoded content bytes (`2^28`). The limit is measured on UTF-8
bytes for Text and octets for Bytes.

### MNIR-SER-081 — Collection count limit

No schema array used as a variable-length collection may contain more than
`16,777,216` elements (`2^24`). Fixed schema arrays retain their exact smaller
lengths.

### MNIR-SER-082 — Nesting depth limit

CBOR nesting depth, counting the top-level payload array as depth 1, MUST NOT
exceed 16. The flat ID-reference schema is defined to remain within this bound.

### MNIR-SER-083 — Checked resource processing

Decoders MUST check file size, declared lengths, element counts, nesting, and
all size arithmetic before allocation or indexing. Integer overflow, length
multiplication overflow, unbounded recursion, and allocation based solely on
an unchecked declared length are prohibited.

### MNIR-SER-084 — Interoperability requirement

A conforming format-1.0 decoder MUST support every structurally valid canonical
checkpoint within the format limits. It MUST NOT impose a smaller undocumented
format limit. Operational failure such as unavailable memory MAY be reported
separately but MUST NOT be represented as successful decode.

## 18. Persistence snapshot and reconstruction APIs

### MNIR-SER-085 — Persistence snapshot purpose

The implementation MUST provide an immutable persistence observation,
conceptually `PersistenceSnapshot`, that captures the coherent state required
by `MNIR-SER-012`.

The exact Rust type and method names are implementation-defined.

### MNIR-SER-086 — Capture is read-only

Creating a persistence observation MUST NOT allocate an entity ID, advance an
allocator, create a `RevisionId`, alter semantic or presentation state, or
make a historical snapshot mutable.

### MNIR-SER-087 — Encoder input

Canonical encoding MUST consume a coherent persistence observation or an
equivalent immutable view. Encoding an arbitrary historical
`ProgramSnapshot` without current lineage allocator and revision continuation
state is non-conforming.

### MNIR-SER-088 — Controlled reconstruction

Decode MUST use a controlled, validated reconstruction boundary capable of
installing exact persisted IDs, references, `ProgramId`, `RevisionId`, entity
authority, and revision continuation without normal entity-creation APIs.

### MNIR-SER-089 — Reconstruction encapsulation

The reconstruction boundary MUST validate the complete candidate atomically
and MUST NOT expose unchecked field setters, arbitrary ID injection, remapping,
or an unrestricted mutation path to ordinary consumers.

### MNIR-SER-090 — No identity creation during load

Successful load MUST create no fresh Program, namespace, entity, or revision
identity and MUST consume no identity-generation entropy.

## 19. Layer and dependency boundaries

### MNIR-SER-091 — mnir-format ownership

The implementation layer for envelope, wire schema, deterministic CBOR,
canonical encode/decode, persistence errors, and reconstruction orchestration
MUST reside in a separate `mnir-format` crate.

### MNIR-SER-092 — Dependency direction

The dependency direction MUST be:

```text
mnir-format → mnir-core
```

`mnir-core` MUST NOT depend on `mnir-format`. `mnir-core` continues to own
semantic models, structural invariants, and the controlled persistence
observation/reconstruction boundary.

### MNIR-SER-093 — No mandated external implementation crate

This specification requires deterministic CBOR capability but does not mandate
or authorize any external Rust package. Dependency selection remains governed
by `AGENTS.md` and an explicit implementation decision.

## 20. Error model

### MNIR-SER-094 — Decode error categories

Decode MUST expose machine-readable categories equivalent to:

```text
UnsupportedFormatVersion
MalformedEncoding
NonCanonicalEncoding
ResourceLimitExceeded
InvalidWireSchema
StructurallyInvalidProgram
```

Exact Rust enum and field names are implementation-defined.

### MNIR-SER-095 — Decode category precedence

When multiple defects exist, category selection MUST follow the first failing
stage in `MNIR-SER-070`:

1. envelope resource and syntax validation;
2. version validation;
3. CBOR syntax and canonical-profile validation;
4. wire-schema validation;
5. identity/allocator/reference/ownership validation;
6. complete structural validation.

Detailed error ordering within one stage is implementation-defined.

### MNIR-SER-096 — Structural details

`StructurallyInvalidProgram` MAY carry or wrap an existing `StructuralError` or
an equivalent persistence-safe detail. Such detail MUST NOT change whether the
candidate is accepted.

### MNIR-SER-097 — Pure encoding

Every valid current format-1.0 persistence observation within resource limits
MUST be representable by the pure canonical encoder. The pure bytes encoder
MAY fail only for invalid checkpoint state, a resource-limit violation, or a
semantic construct not supported by format 1.0.

### MNIR-SER-098 — Filesystem errors are separate

Writing bytes to a file or stream MAY fail with I/O errors, but such errors are
not canonical encoding errors and MUST NOT alter the canonical byte definition.
Temporary files, `fsync`, file locking, permissions, and atomic rename are
outside this specification.

## 21. Explicit exclusions

### MNIR-SER-099 — Excluded state

Format 1.0 MUST NOT persist:

- transaction objects or working state;
- historical revisions, revision ancestry, or Git history;
- diagnostics, `VerifiedProgram`, verifier caches, or type caches;
- runtime values, execution state, or runtime memory representation;
- EasyH source or projection;
- semantic diff or merge state;
- package registry metadata;
- trust, provenance, signatures, or publisher identity; or
- filesystem metadata.

### MNIR-SER-100 — Git boundary

Canonical bytes are intended for Git blob storage. Textual readability,
`textconv`, semantic diff, and semantic merge are future tooling and MUST NOT
alter format-1.0 bytes or Program semantics.

## 22. Normative byte examples

Spaces and line breaks in hexadecimal examples are visual separators only and
are not bytes.

### MNIR-SER-101 — Typed ID example

For namespace bytes `10 11 ... 1f` and counter 24, the exact typed-ID CBOR is:

```text
82
50 101112131415161718191a1b1c1d1e1f
18 18
```

The bytes are identical for the value portion of any entity-ID category; the
schema position supplies the typed category.

### MNIR-SER-102 — Allocator examples

For the same namespace, `Available(24)` is:

```text
82 50 101112131415161718191a1b1c1d1e1f 82 00 18 18
```

`Exhausted` is:

```text
82 50 101112131415161718191a1b1c1d1e1f 81 01
```

### MNIR-SER-103 — Presentation, Text, and Bytes examples

Presentation with both fields absent is:

```text
82 81 00 81 00
```

Text containing scalar sequence `U+0041 U+00E9` (`Aé`) is:

```text
63 41 c3 a9
```

Bytes containing octets `00 ff` is:

```text
42 00 ff
```

### MNIR-SER-104 — One Module example

A Module with ID `(namespace 10..1f, counter 1)`, absent presentation fields,
no Domain Types, and no Functions is exactly:

```text
84
82 50 101112131415161718191a1b1c1d1e1f 01
82 81 00 81 00
80
80
```

### MNIR-SER-105 — Bodyless Function example

A bodyless Unit-returning Function with ID `(namespace 10..1f, counter 2)`, no
Parameters, and absent presentation fields is exactly:

```text
85
82 50 101112131415161718191a1b1c1d1e1f 02
82 00 03
80
81 00
82 81 00 81 00
```

### MNIR-SER-106 — Complete minimal golden fixture

The normative minimal checkpoint has:

```text
ProgramId = 000102030405060708090a0b0c0d0e0f
RevisionId = 1
RevisionAllocationState = Available(2)
AllocationNamespaceId = 101112131415161718191a1b1c1d1e1f
AllocationCounterState = Available(1)
Modules = []
```

Its complete 56-byte `.mnir` file is exactly:

```text
89 4d 4e 49 52 0d 0a 1a 00 01 00 00
85
50 000102030405060708090a0b0c0d0e0f
01
82 00 02
82 50 101112131415161718191a1b1c1d1e1f 82 00 01
80
```

Contiguous lowercase hexadecimal:

```text
894d4e49520d0a1a000100008550000102030405060708090a0b0c0d0e0f018200028250101112131415161718191a1b1c1d1e1f82000180
```

## 23. Golden vectors

### MNIR-SER-107 — Specification-controlled vectors

Conforming implementations MUST test specification-controlled golden vectors
that cover:

```text
persisted state → exact canonical bytes
exact canonical bytes → exact persisted state
decoded state → exact same canonical bytes
```

The fixture in `MNIR-SER-106` is the first mandatory vector. Later repository
fixture files MUST reproduce it exactly and MUST add the acceptance coverage
required below without changing its bytes.

## 24. Normative compatibility audit

### MNIR-SER-108 — Earlier semantics remain unchanged

Persistence MUST preserve, not reinterpret, the state and relationships
defined by specifications `01` through `11`. Wire ordering, tags, and envelope
values MUST NOT become MNIR execution, type, ownership, verification, or
presentation semantics.

Every normative rule and acceptance requirement in specifications `01`
through `11` was reviewed. The ranges below mean every existing rule carrying
that prefix and a number within the stated range, including rules previously
superseded or revised by later specifications. Earlier acceptance requirements
remain applicable; this document adds persistence evidence without replacing
their semantic evidence.

The complete audit has the following outcome:

| Specification | Persistence-relevant rules and required preservation |
| --- | --- |
| `01-program-model.md` | All existing `MNIR-CORE-001`–`066`: aggregate ownership, Program/snapshot/fork identity, unordered Module collection, presentation, revisions, controlled mutation, structural validity, historical allocation clauses, and frontend independence. PSI supersession remains authoritative where applicable. |
| `02-type-system-foundations.md` | All existing `MNIR-TYPE-001`–`032`: intrinsic identity/value domains, target independence, closed representation, no Program registration, and no inferred Domain/security meaning. Domain Type Foundations supplies the six-type extension. |
| `03-functions-and-parameters.md` | All existing `MNIR-FUNC-001`–`083`: identity, ownership, exact Parameter order, types, presentation, allocation/provisional state, removal, structure, signature shape, snapshots/forks, and non-semantic Function iteration. PSI supersession governs allocation history and normal-fork ID preservation. |
| `04-expressions-and-basic-function-bodies.md` | All existing `MNIR-EXPR-001`–`101`: body presence, Block/Expression identity and ownership, literals, Parameter references, Return, type inspection boundaries, allocation/removal, snapshots/forks, and non-semantic Expression order. Conditional Control Flow supersedes the single-Block clauses. |
| `05-arithmetic-expressions.md` | All existing `MNIR-ARITH-001`–`095`: five exact operator variants, operand roles, shared dependency DAG, structural/semantic separation, identity/allocation preservation, snapshots/forks, and non-semantic collection order. Runtime fault/evaluation state is not persisted. |
| `06-semantic-verification-and-diagnostics.md` | All existing `MNIR-VERIFY-001`–`089`: immutable revision binding, diagnostics, applicability, deterministic traversal, and structural/semantic separation. Diagnostics and `VerifiedProgram` are intentionally excluded and verification is rerun after load when requested. |
| `07-comparison-expressions.md` | All existing `MNIR-CMP-001`–`106`: six exact operator variants, operand roles, shared dependencies, Bool result semantics, identity/allocation, snapshots/forks, diagnostics, and historical verifier applicability. |
| `08-conditional-control-flow.md` | All existing `MNIR-CFG-001`–`128`: multiple Blocks, explicit entry, Return/Branch terminators, true/false roles, reachability, acyclicity, structural mutation, allocation/cascades, snapshots/forks, and verifier separation. |
| `09-function-calls-and-sequencing-foundations.md` | All existing `MNIR-CALL-001`–`138`: Call identity and fields, ordered arguments, exact effect sequencing, dependency/order consistency, target references, structural validation, allocation, snapshots/forks, diagnostics, and verifier separation. |
| `10-persistent-semantic-identity.md` | All existing `MNIR-PSI-001`–`104`, especially `014`, `020`, `022`–`026`, `031`–`060`, `063`–`084`: typed namespace/counter identity, Program/namespace persistence, one authority, durable issuance, semantic/allocator separation, snapshot freshness, exact normal-fork preservation, reference identity, structural allocator invariants, and minimum persistence contents. |
| `11-domain-type-foundations.md` | All existing `MNIR-DOMAIN-001`–`120`: Text/Bytes, `ValueType`, `TypeId`, Domain ownership/representation/presentation, Domain literals and Expressions, shared DAG integration, signature/Return/Branch/Call typing, removal/repair, snapshot/fork preservation, PSI extension, diagnostics, and implementation boundaries. V0_5 diagnostics remain non-persisted. |

### MNIR-SER-109 — Fork checkpoints

A checkpoint of a fork MUST preserve the fork's `ProgramId`, inherited entity
IDs and references, fork allocation authority, current revision, and revision
continuation exactly. Decode MUST NOT recreate the fork operation or remap
inherited identities.

## 25. Explicitly deferred topics

### MNIR-SER-110 — Deferred topics are not format 1.0

Format 1.0 does not define:

- historical revision storage or ancestry;
- transaction resumption;
- import, package, trust, provenance, signature, or encryption formats;
- built-in compression;
- verification evidence persistence;
- semantic diff or merge;
- EasyH or another human-readable canonical form;
- filesystem durability or atomic replacement;
- textconv configuration; or
- future semantic constructs not present in specifications `01` through `11`.

### MNIR-SER-111 — Undefined persistence behavior is a specification gap

If implementation requires externally observable encoding, decoding,
validation, error, or resource behavior not defined here, it MUST be reported
as a `SPECIFICATION GAP` rather than invented.

## 26. Acceptance requirements

### AR-SER-001 — Minimal golden file

Decode the exact 56-byte vector in `MNIR-SER-106`, demonstrate every listed
field, re-encode it byte-identically, and confirm structural validity.

### AR-SER-002 — Envelope and version rejection

Reject wrong/truncated magic, truncated version, missing payload, trailing
bytes, unsupported major, and unsupported minor with the specified categories.

### AR-SER-003 — Canonical CBOR strictness

Reject non-shortest integers or lengths, indefinite items, maps, CBOR tags,
floats, null/undefined, invalid UTF-8, and disallowed simple values. Evidence
MUST distinguish malformed from well-formed non-canonical input.

### AR-SER-004 — Unknown schema rejection

Reject unknown type, ValueType, Expression, terminator, counter-state, option,
and structural schema tags, wrong field types, and wrong fixed array lengths.

### AR-SER-005 — Identifier byte compatibility

Demonstrate exact Program/namespace 16-byte preservation, `u64` RevisionId,
all six typed entity-ID categories, counter boundaries 0, 1, 23, 24, 255, 256,
`u32::MAX + 1`, and `u64::MAX`, and the comparison order in
`MNIR-SER-037`.

### AR-SER-006 — Minimal and multiple Modules

Round-trip empty and multi-Module Programs, preserving Module IDs and
presentation. Construct equal persisted state with different insertion order
and demonstrate identical bytes.

### AR-SER-007 — Domain Types

Round-trip multiple Domain Types with all six intrinsic representations,
`TypeId`, ownership, presentation, and cross-Module Domain references exactly.

### AR-SER-008 — Functions and Parameters

Round-trip bodyless Functions, every intrinsic and Domain return/Parameter
`ValueType`, exact Parameter identities/order, duplicate presentation names,
and absent, empty, ASCII, and non-ASCII presentation strings.

### AR-SER-009 — Body and Block forms

Round-trip body absence, a single-Block body, and a multi-Block body while
preserving explicit entry identity, canonical Block collection ordering, and
all terminators.

### AR-SER-010 — Literal forms

Round-trip signed Int32/Int64 minima, maxima, `-1`, and zero; both Bool values;
Unit; empty and non-ASCII/scalar-distinct Text; and empty/arbitrary Bytes.

### AR-SER-011 — Arithmetic operators

Round-trip Add, Subtract, Multiply, Divide, and Remainder with exact left/right
references and canonical Expression collection order.

### AR-SER-012 — Comparison operators

Round-trip Equal, NotEqual, LessThan, LessThanOrEqual, GreaterThan, and
GreaterThanOrEqual with exact left/right references.

### AR-SER-013 — ParameterReference

Round-trip ParameterReference with its exact `ParameterId` and owning-body
structural relationship.

### AR-SER-014 — Calls and argument order

Round-trip Calls with exact target `FunctionId`, zero and multiple arguments,
duplicate argument references where valid, and exact argument order.

### AR-SER-015 — EffectSequence order

Round-trip multiple Calls whose `EffectSequence` order differs from canonical
Expression-ID order. Prove sequence order is retained and not sorted.

### AR-SER-016 — Domain Expressions

Round-trip DomainConstruct and DomainProject with exact Type/source references
and demonstrate that no derived type or representation cache is persisted.

### AR-SER-017 — Branch role preservation

Round-trip a Branch whose false target sorts before its true target and prove
condition, true role, and false role remain exact.

### AR-SER-018 — Canonical collection independence

Using controlled test-only reconstruction if needed, encode two checkpoints
with identical complete persisted state but different Module, Domain Type,
Function, Block, and Expression insertion histories. Bytes MUST be identical
without changing IDs.

### AR-SER-019 — Presentation-only distinction

Encode checkpoints differing only in persisted presentation metadata. Prove
semantic entity IDs remain equal and canonical bytes differ.

### AR-SER-020 — Available allocator round-trip

Round-trip multiple `Available(next_counter)` values including 0, 1, 24, and
`u64::MAX`. Allocate afterward and prove exact restored issuance and advance or
transition to `Exhausted`.

### AR-SER-021 — Allocator-only persistence

Starting from revision `R` and `Available(100)`, issue counters 100 through 103
without semantic commit and capture `Available(104)`. Prove `RevisionId` and
semantic contents are unchanged, checkpoint bytes differ, the second decode
restores 104, and later allocation reuses none of 100 through 103.

### AR-SER-022 — Exhausted allocator

Round-trip `Exhausted`, prove it remains exhausted, performs no namespace
rotation, and rejects later entity allocation according to PSI.

### AR-SER-023 — Revision continuation

Round-trip nontrivial current revision and revision cursor, prove both are
preserved, commit once, and prove the fresh expected revision is issued and
the historical revision is not reused. Also round-trip revision exhaustion and
prove another commit is rejected.

### AR-SER-024 — Snapshot/checkpoint immutability

Capture a persistence observation, later advance semantic and allocator state,
and prove the earlier observation and its canonical bytes remain unchanged.

### AR-SER-025 — Fork checkpoint

Round-trip a fork containing inherited and fork-minted identities. Prove exact
Program/entity/reference preservation, restored fork authority, no remapping,
and independent subsequent allocation.

### AR-SER-026 — Structurally invalid duplicate ID

Reject a canonical-schema candidate containing duplicate persistent identity,
including one namespace/counter pair reused across typed categories.

### AR-SER-027 — Structurally invalid references

Independently reject unresolved Call target, unresolved TypeId, invalid Branch
target, foreign-Block Expression reference, and invalid entry Block.

### AR-SER-028 — Structurally invalid graphs

Independently reject an Expression dependency cycle, unreachable/cyclic CFG,
EffectSequence missing a Call, duplicate sequence entry, foreign entry, and
dependency/order conflict.

### AR-SER-029 — Atomic decode failure

For every rejection family, prove that no partial mutable lineage, new ID,
allocator advancement, or externally observable reconstructed state escapes.

### AR-SER-030 — Semantic-invalid round-trip

Round-trip byte-identically a structurally valid Program containing arithmetic
operand mismatch, Return mismatch, invalid DomainConstruct representation, and
invalid Call argument types. Decode MUST succeed and V0_5 MUST subsequently
produce the expected diagnostics.

### AR-SER-031 — Resource file-size limit

Accept boundary-valid input at the supported maximum through a bounded/streamed
fixture strategy and reject declared or actual data above `2^30` without
proportional allocation or overflow.

### AR-SER-032 — Resource field limits

Test Text/Bytes and collection counts at representative canonical boundaries,
reject values above `2^28` content bytes or `2^24` elements, reject depth 17,
and exercise overflow-prone declared lengths without panic or uncontrolled
allocation.

### AR-SER-033 — Flat dependency decoding

Round-trip a deep shared Expression DAG while proving parser nesting remains
bounded by wire structure rather than graph depth.

### AR-SER-034 — Pure encoder boundary

Demonstrate that every valid current checkpoint within format limits encodes,
that filesystem errors are absent from pure encode-to-bytes errors, and that
future unsupported semantics cannot be silently dropped.

### AR-SER-035 — Controlled reconstruction

Prove decode restores all exact IDs and authorities without invoking normal ID
creation, consuming entropy, advancing counters, or exposing unchecked public
field mutation.

### AR-SER-036 — Crate direction

Demonstrate `mnir-format → mnir-core`, no `mnir-core → mnir-format`, no
serialization implementation in other semantic crates, and no dependency on
EasyH or verifier execution for persistence.

### AR-SER-037 — Golden-vector suite

Maintain specification-controlled vectors for state-to-bytes, bytes-to-state,
and byte-identical re-encoding across every schema family and boundary encoding.

### AR-SER-038 — Existing conformance preservation

Run all Program Model through Domain Type Foundations and V0_1 through V0_5
tests unchanged in semantic behavior after persistence implementation.

### AR-SER-039 — Scope boundary

Demonstrate that implementation adds no active-transaction persistence,
history store, verification cache, EasyH, package/trust format, semantic diff,
merge, compression, signature, checksum-as-authenticity, or filesystem update
protocol.

### AR-SER-040 — Stale checkpoint authority

Persist a checkpoint, issue additional entity IDs, and persist a later
checkpoint. Prove the earlier checkpoint remains decodable as an immutable
historical observation but cannot be activated as the same mutable lineage's
current authority when the implementation knows of the later issuance.

### AR-SER-041 — Independent restoration requires fork

Decode one authoritative checkpoint and create two intended independent lines
of evolution. Prove they do not both issue through the restored namespace:
one remains the same-lineage continuation and the other performs a normal fork
before allocation, receiving a fresh `ProgramId` and allocation authority.

## 27. Implementation constraints

### MNIR-SER-112 — Safe implementation

Implementation MUST use safe Rust unless a later explicit specification or ADR
requires otherwise. Expected malformed input MUST produce typed errors rather
than panics.

### MNIR-SER-113 — No implementation-layout coupling

Implementation MUST NOT use Rust memory layout, enum discriminants, derived
Debug output, `HashMap` iteration, or generic derived struct layout as the
wire contract.

### MNIR-SER-114 — Dependency governance

No external dependency is mandated by this specification. Any future direct
dependency or feature decision MUST follow `AGENTS.md` unless explicitly
authorized by an authoritative implementation task.

### MNIR-SER-115 — No implementation in specification task

Defining this document requires no serializer, decoder, crate, fixture file,
Cargo change, CLI, Git integration, or filesystem API.

### MNIR-SER-116 — Checkpoint does not duplicate authority

Copying or decoding checkpoint bytes MUST NOT be interpreted as minting,
cloning, transferring, or delegating allocation authority. The restored
authority is the continuation authority of the same Program lineage and has
the single-authority meaning defined by Persistent Semantic Identity 0.1.

### MNIR-SER-117 — Current-authority precondition

A checkpoint MAY be activated as the mutable continuation of its Program
lineage only when its allocator observations are current relative to every
successful issuance known to that lineage. If an implementation knows that a
newer issuance or checkpoint exists, it MUST reject same-lineage mutable
activation of the stale checkpoint.

A stale checkpoint MAY remain an immutable observation or serve as the source
of a normal fork, whose fresh `ProgramId` and allocation authority prevent
reuse in the original namespace.

### MNIR-SER-118 — Persistence durability boundary

Canonical serialization records allocator state at checkpoint capture; it
does not by itself make later in-memory issuance durable. An implementation
claiming safe same-lineage continuation across process restart MUST ensure
that successful issuance and the allocator-state transition preventing reuse
are durably reflected according to its storage model before that issuance is
considered persistently successful, as required by `MNIR-PSI-034`.

The exact temporary-file, synchronization, locking, and atomic-replacement
mechanics remain outside format 1.0.

### MNIR-SER-119 — Independent evolution uses normal fork

Two copies of one checkpoint MUST NOT independently allocate through the same
restored authority. Before a restored copy evolves independently of another
continuation, it MUST undergo the normal fork operation and receive a fresh
`ProgramId` and allocation authority. Format 1.0 does not claim distributed
detection of unauthorized duplicated authority across isolated processes.

## 28. Foundational invariants

The persistence foundation is:

```text
one committed lineage checkpoint
    = exact ProgramId and RevisionId
    + exact committed contents and presentation
    + exact entity allocation authority
    + exact revision continuation

same complete persisted state
    → same canonical format-1.0 bytes

canonical decode
    → structurally valid mutable continuation
    → no remapping, repair, verification, or identity creation
```
