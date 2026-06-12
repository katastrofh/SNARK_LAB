# Audit Request

## Project

SNARK_LAB

## Repository

https://github.com/katastrofh/SNARK_LAB

## Requested review type

- cryptographic protocol review
- Rust implementation review
- parser/codec review
- SRS ceremony review
- side-channel boundary review
- deployment evidence review

## Target commit

Commit:

## Scope

See:

- audits/scope.md
- SECURITY.md
- docs/threat-model.md
- docs/security-proof-sketch.md
- docs/side-channel-boundary-notes.md
- docs/production-srs-ceremony-spec.md
- docs/deployment-evidence-pack.md

## Key questions

1. Can the verifier accept invalid proofs?
2. Are Fiat-Shamir transcript bindings complete?
3. Are IPA folding equations implemented correctly?
4. Are proof and SRS codecs canonical and rejection-safe?
5. Are malformed byte inputs rejected safely?
6. Are public test vectors and reference comparisons meaningful?
7. Is the SRS ceremony specification sufficient for production?
8. What side-channel risks remain before deployment?
9. What release evidence is missing for production-secure claims?

## Expected deliverables

- findings report
- severity classification
- remediation recommendations
- final review status
- production-readiness conclusion
