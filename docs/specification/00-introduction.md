# Introduction

## Status

MNIR is experimental and at an early design stage. This document establishes only the project principles and terminology already agreed upon. It does not define detailed language or verification semantics.

## Principles

- MNIR is intended to become the canonical semantic intermediate representation for programs.
- The canonical representation is semantic rather than a human-oriented textual source form.
- Future human-oriented languages, including EasyH, may compile to or be rendered from MNIR.
- Future AI systems may manipulate MNIR without generating textual source code.
- Security, correctness, explicit semantics, established software patterns, policies, contracts, and machine-verifiable invariants are intended to be fundamental concerns.
- The normative specification is authoritative. Implementation follows the specification and must not invent missing semantics.

## Terminology

### MNIR

**Magnus Nivinger Intermediate Representation**, the project described by this specification.

### Canonical representation

The authoritative program representation. Human-readable projections are not canonical merely because they express or display a program.

### Semantic representation

A representation intended to capture specified program meaning independently of a particular human-oriented source syntax. Detailed semantic structures have not yet been specified.

### EasyH

A future human-oriented presentation/frontend language. EasyH is not the canonical representation. No EasyH syntax, parser, or rendering semantics are specified yet.

### Verifier

A future component that will check MNIR against normative invariants and rules. No verification behavior is specified yet.

### Normative specification

The authoritative documents that define required MNIR behavior. Normative rules use **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** and will use stable rule identifiers.

### Presentation metadata

Future information intended to influence a human-readable presentation without replacing the canonical semantic representation. Its structure and behavior are not yet specified.

### Specification gap

A missing, ambiguous, or contradictory normative decision required for implementation. A specification gap must be reported and must not be resolved automatically by implementation code.

## Development sequence

```text
Design decision
      ↓
Normative MNIR specification
      ↓
Acceptance criteria
      ↓
Implementation
      ↓
Tests
      ↓
Verification
      ↓
Next specification increment
```
