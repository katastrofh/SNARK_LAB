# Public Test Vectors

This branch adds committed public test vectors.

## Added

- `test-vectors/ipa-demo-v1.txt`
- `test-vectors/README.md`
- `scripts/generate-test-vectors.sh`
- `scripts/check-test-vectors.sh`

## Production gate

The local production gate now checks that the committed IPA demo vector matches the current implementation.

## Boundary

The vector is a deterministic regression artifact.

It is not production SRS material, not a trusted setup ceremony output, and not an audit result.
