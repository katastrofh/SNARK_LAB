# Audit Triage Policy

## Severity definitions

### Critical

An issue that can produce false proof acceptance, forge commitments, bypass verification, corrupt SRS validation, or compromise secret witness material in a realistic deployment.

### High

An issue that can break soundness under plausible conditions, bypass parser rejection, allow malformed proof confusion, or create severe denial of service in expected deployment.

### Medium

An issue that weakens assumptions, causes incorrect rejection/acceptance under narrow conditions, or creates meaningful reliability/security risk.

### Low

An issue that affects hardening, documentation, diagnostics, or non-critical misuse resistance.

### Informational

A note that improves clarity, maintainability, or future auditability.

## Fix policy

- Critical: must be fixed before any release candidate.
- High: must be fixed before production-secure claims.
- Medium: must be fixed or explicitly accepted before production-secure claims.
- Low: may be deferred with rationale.
- Informational: may be tracked as improvement work.

## Disclosure

Security-sensitive findings should not be publicly disclosed until maintainers have time to remediate.

## Regression requirements

Every fixed security finding should include at least one of:

- unit test
- integration test
- negative fixture
- fuzz regression corpus entry
- reference comparison test
- documented proof argument

## Production release rule

No production-secure release may be cut with open critical or high findings.
