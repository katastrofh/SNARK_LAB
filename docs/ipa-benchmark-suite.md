# IPA Benchmark Suite

This branch expands the benchmark binary to cover the integrated IPA path.

## Command

    cargo run --release -p snark-lab-benches -- [permcheck_log2] [ipa_variables] [samples]

Defaults:

    permcheck_log2 = 18
    ipa_variables = 8
    samples = 3

Limits:

    permcheck_log2 <= 24
    ipa_variables <= 12
    samples in 1..=50

## Benchmarks

The binary reports best-of-N timing in microseconds for:

    PermCheck product fingerprint
    PermCheck rational fingerprint
    Sumcheck proving
    IPA SRS canonical digest computation
    IPA key trimming
    IPA commit
    IPA open
    IPA opening encoding
    IPA opening decoding
    IPA verify

## Production boundary

The IPA benchmark uses synthetic fixture generator material only to measure code paths.

It is not production SRS generation.

The benchmark output explicitly prints:

    synthetic_generator_fixture=true
    not_production_srs=true

## Measurement boundary

Runtime timings are measured with `std::time::Instant`.

Logical I/O values for PermCheck remain modeled estimates, not hardware counters.
