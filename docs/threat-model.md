# Threat model and production-readiness gates

## Adversary

Assume a malicious prover controls every witness value and proof message, can replay or mutate serialized proofs, and chooses inputs adaptively before transcript challenges are derived. Browser JSON and CLI file paths are untrusted input. The verifier and deployment host are trusted to run the reviewed binary and preserve security headers.

## Guarantees implemented now

- Rust protocol arithmetic is generic over Arkworks prime fields and defaults to BLS12-381 `Fr`.
- Merlin transcripts use protocol domains, field-modulus binding, fixed-width field encodings, statement binding, and messages-before-challenges ordering.
- Sumcheck rejects inconsistent rounds and mismatched final transparent-oracle evaluations.
- Zerocheck binds the constraint table before deriving its mixing point.
- PermCheck binds tagged columns before deriving `β, γ` and reports denominator poles explicitly.
- Educational JSON parsing rejects unknown fields, noncanonical F₉₇ values, oversized files, oversized tables, malformed rounds, and invalid final checks.
- The visualizer has no telemetry or remote font dependency and ships a restrictive CSP/header policy.

## Not guaranteed yet

- succinct verification or hiding of the oracle table;
- zero knowledge;
- authenticated polynomial openings;
- production proof serialization;
- trusted-setup safety;
- side-channel resistance;
- audit-level assurance.

## IP and data handling

All browser calculations remain local. The app makes no analytics or backend requests. Exported transcripts contain the complete educational oracle table, so users must not load proprietary or secret witness material into the browser lab. The source is dual-licensed under MIT or Apache-2.0; third-party dependencies retain their own licenses.
