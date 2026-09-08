# Architecture decision records

This directory contains architecture decision records (ADRs) for significant repository and implementation choices. An ADR records context, the chosen decision, and its consequences so that the architecture remains understandable over time.

ADRs do not define MNIR language semantics unless the normative specification explicitly incorporates a decision.

## Decisions

- [`0001-value-identity-and-sequencing.md`](0001-value-identity-and-sequencing.md) — accepts `ExpressionId` as local computed-value identity, preserves pure Expression DAG semantics, separates frontend names from semantic references, and defers the explicit sequencing model for future effects.
- [`0002-explicit-effect-sequencing.md`](0002-explicit-effect-sequencing.md) — selects a separate Block-local ordered `EffectSequence` for future observable effects while retaining the unordered pure Expression dependency DAG and unified `ExpressionId` identity.
