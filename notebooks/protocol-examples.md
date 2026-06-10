# Protocol examples notebook

Rust examples use the BLS12-381 scalar field and Merlin. Browser examples use educational F₉₇.

## Fiat–Shamir Sumcheck

```rust
use ark_bls12_381::Fr;
use multilinear::Multilinear;
use snark_lab_transcript::MerlinTranscript;

let polynomial = Multilinear::new(
    [1_u64, 2, 3, 4].into_iter().map(Fr::from).collect()
)?;
let claim = polynomial.sum_hypercube();

let mut prover_transcript = MerlinTranscript::new(b"my-application");
let proof = sumcheck::prove(&polynomial, claim, &mut prover_transcript);

let mut verifier_transcript = MerlinTranscript::new(b"my-application");
let challenges = sumcheck::verify(
    &polynomial,
    claim,
    &proof,
    &mut verifier_transcript,
)?;
```

Each round polynomial is appended before its challenge is derived. Prover and verifier start from the same application label and statement, so they reconstruct identical challenges without placing challenges in the proof.

## Zerocheck

Bind a constraint table, derive the random equality-mixing point from the transcript, and prove the weighted zero sum. Changing one constraint changes the mixing point as well as invalidating the zero claim.

## Tagged PermCheck

Bind two columns of `(value, tag)` pairs, derive `β,γ`, and compare either product or rational fingerprints. A reordered collection of identical pairs passes; a changed value or tag fails except with the expected large-field soundness probability.

## I/O experiment

```bash
cargo run --release -p snark-lab-benches -- 20
```

The runtime uses BLS12-381 scalar arithmetic. Reported bytes remain a documented logical traffic model.
