# IPA Proof Decoder Nightly Fuzz Smoke Evidence

## Status

smoke-complete

## Target

    ipa_proof_decode

## Command

    cd fuzz
    cargo +nightly fuzz run ipa_proof_decode -- -max_total_time=10

## Result

The smoke run completed without a crash.

## Run summary

- Runs: 5053927
- Seconds: 11
- Toolchain: rustc 1.98.0-nightly (b30f3df3b 2026-06-11)
- Commit: 0509cbf6ae09b043a024135b3a3d403ebfd09535
- Branch: nightly-fuzz-smoke-evidence
- Generated UTC: 2026-06-12T14:01:41Z

## Boundary

This is smoke evidence only.

It does not prove production security.

It does not replace:

- long fuzz campaign evidence
- external audit
- side-channel review
- production SRS evidence
