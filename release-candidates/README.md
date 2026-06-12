# Release Candidate Evidence Summaries

This directory stores small, reviewable summaries of generated deployment evidence packs.

Raw evidence packs are generated under:

    deployment/evidence/

Those raw packs are ignored by default because they may contain large logs.

A release-candidate summary should include:

- commit under evidence
- branch
- git cleanliness
- production gate result
- public vector result
- SRS manifest result
- fuzz target compile result
- artifact digest file reference
- production-security conclusion

## Important boundary

A release-candidate evidence summary does not itself mean production-secure.

Production-secure status additionally requires:

- external audit
- side-channel review
- production SRS artifact and digest
- production ceremony transcript
- final release approval
