# Public Test Vectors

These vectors are committed reference outputs for SNARK_LAB.

They are intended for:

- regression testing
- release reproducibility
- public artifact review
- downstream compatibility checks

They are not production SRS material and must not be interpreted as audited cryptographic deployment fixtures.

## Current vectors

- `ipa-demo-v1.txt`

## Regeneration

Run:

    scripts/generate-test-vectors.sh

Then verify:

    scripts/check-test-vectors.sh

## Boundary

The IPA demo vector uses deterministic fixture material for reproducible testing.

It is not a production trusted setup, not a ceremony output, and not safe as production SRS material.
