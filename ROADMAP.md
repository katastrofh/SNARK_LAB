# SNARK_LAB Roadmap

This roadmap separates completed release-candidate work from future work.

## Current release-candidate track

Completed:

- Sumcheck, Zerocheck, and PermCheck protocol components
- IPA polynomial commitment integration
- IPA proof serialization and negative fixtures
- SRS provenance and placeholder-policy checks
- public test vectors
- reference implementation comparison tests
- fuzz target builds
- fuzz smoke evidence
- fuzz crash regression suite
- release artifact tooling
- GitHub Release publication evidence
- browser visualizer and screenshots
- production-readiness gate for repository evidence

## Near-term engineering work

Planned:

- add more independent reference checks
- expand fuzz corpus seeds
- add more malformed proof regression fixtures
- improve benchmark result summaries
- add visualizer walkthrough video or GIF assets
- add more CLI examples for protocol demos
- add protocol trace export for educational notebooks

## Research work

Possible directions:

- rational PermCheck experiments
- streaming-friendly permutation checks
- memory and I/O benchmark comparisons
- alternative PCS backends
- verifier-cost analysis
- Scribe/HyperPlonk-style reduction comparisons

## Security-review work

Before any production-security claim, the project needs:

- external cryptographic review
- side-channel review
- dependency audit
- longer fuzz campaigns
- production SRS ceremony artifact review
- threat-model review by independent reviewers
- release reproducibility review

## Non-goals

This roadmap does not target:

- custody deployment
- mainnet deployment
- production prover service operation
- claims of audited security without an audit
- replacing mature proof-system libraries
