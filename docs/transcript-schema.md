# Educational browser transcript schema (version 1)

This schema is isolated in `crates/interchange`; it is not the Fiat–Shamir transcript used by the Rust protocol core. The visualizer exports deterministic JSON that the Rust CLI can verify without trusting the browser. Version 1 intentionally describes the lab's educational protocol over `F_97`; it does **not** claim cryptographic Fiat–Shamir security or hide the oracle table.

## Envelope

```json
{
  "version": 1,
  "protocol": "sumcheck",
  "field": { "modulus": 97 },
  "claim": {
    "num_variables": 3,
    "claimed_sum": 31,
    "oracle_evaluations": [3, 1, 4, 1, 5, 9, 2, 6]
  },
  "rounds": [
    {
      "round": 0,
      "g_at_zero": 14,
      "g_at_one": 17,
      "challenge": 9
    }
  ],
  "final": {
    "point": [9, 11, 19],
    "oracle_evaluation": 42
  }
}
```

The numbers above illustrate the shape; use [`examples/transcripts/sumcheck-valid.json`](../examples/transcripts/sumcheck-valid.json) for a complete, verified transcript.

## Fields and validation

- `version` must be `1`.
- `protocol` is `sumcheck` or `zerocheck`.
- `field.modulus` must be `97`.
- Every field element must be a canonical integer in `[0, 96]`. The verifier rejects out-of-range values rather than silently reducing them.
- `claim.oracle_evaluations` is a non-empty, power-of-two table in the little-endian Boolean-cube order used by the browser lab.
- `claim.num_variables` must equal `log2(oracle_evaluations.length)`.
- Each round index must equal its zero-based position. The verifier checks the round sum, deterministic challenge, and folded claim.
- `final.point` must exactly equal the ordered round challenges. The verifier evaluates the supplied oracle table at that point and compares it with `final.oracle_evaluation`.

## Zerocheck specialization

A Zerocheck claim adds:

```json
"mixing_point": [5, 11, 19]
```

For Zerocheck, `oracle_evaluations` contains the **unweighted constraint table**, `claimed_sum` must be zero, and the verifier independently constructs `f(x) · eq(mixing_point, x)` before replaying Sumcheck. A nonzero constraint therefore cannot be hidden by exporting a fabricated final value.

## Compatibility policy

Readers must reject unknown versions. Future changes that alter field encoding, challenge derivation, or protocol semantics will increment `version`; additive documentation changes do not require a new version.
