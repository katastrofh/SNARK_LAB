# Fuzz Campaign Runner Hardening

This document records fuzz campaign runner hardening.

## Problem

A smoke fuzz run may fail before campaign evidence is produced.

Failure logs are generated under:

    fuzz/campaigns/<timestamp>/

Generated corpora and artifacts may also appear under:

    fuzz/corpus/
    fuzz/artifacts/

These generated files are not committed by default.

## Policy

Failed fuzz runs are not evidence.

A fuzz campaign counts as evidence only when:

- all configured targets complete
- logs are archived
- manifest is generated
- SHA-256 manifest is generated
- failures are triaged
- regressions are added for crashes

## Hardening added

The runner now prints the last 80 lines of a failed target log into SUMMARY.md.

Generated fuzz corpus and artifact directories are ignored.
