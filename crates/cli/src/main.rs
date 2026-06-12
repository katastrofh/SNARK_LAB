#![forbid(unsafe_code)]

use ark_bls12_381::{Fr, G1Projective};
use ark_ec::PrimeGroup;
use multilinear::Multilinear;
use snark_lab_interchange::{parse_and_verify, Protocol};
use snark_lab_oracle::{
    commit_ipa_backend, decode_ipa_integrated_opening, encode_ipa_integrated_opening,
    expected_ipa_generator_count, open_ipa_backend, trim_ipa_integrated_keys, verify_ipa_backend,
    IpaCurveGeneratorBasis, IpaCurvePoint,
};
use snark_lab_transcript::MerlinTranscript;
use std::{env, fs, process};

type DemoResult<T> = Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct IpaDemoReport {
    variables: usize,
    claimed_value: Fr,
    commitment_bytes: usize,
    encoded_opening_bytes: usize,
    decoded_rounds: usize,
}

fn usage() -> ! {
    eprintln!("usage: snark-lab-cli verify-transcript <path.json>");
    eprintln!("       snark-lab-cli ipa-demo");
    process::exit(2);
}

fn generator_count(variables: usize) -> DemoResult<usize> {
    expected_ipa_generator_count(variables)
        .map_err(|error| format!("invalid generator count for {variables} variables: {error:?}"))
}

fn point(seed: u64) -> DemoResult<IpaCurvePoint<G1Projective>> {
    IpaCurvePoint::from_projective(G1Projective::generator() * Fr::from(seed))
        .map_err(|error| format!("invalid demo generator seed {seed}: {error:?}"))
}

fn demo_basis(variables: usize) -> DemoResult<IpaCurveGeneratorBasis<G1Projective>> {
    let count = generator_count(variables)?;

    let polynomial_generators = (0..count)
        .map(|index| point(index as u64 + 1))
        .collect::<DemoResult<Vec<_>>>()?;

    let evaluation_generators = (0..count)
        .map(|index| point(index as u64 + 100))
        .collect::<DemoResult<Vec<_>>>()?;

    IpaCurveGeneratorBasis::new(
        variables,
        polynomial_generators,
        evaluation_generators,
        point(999)?,
    )
    .map_err(|error| format!("invalid demo generator basis: {error:?}"))
}

fn padding_polynomial_generators(variables: usize) -> DemoResult<Vec<IpaCurvePoint<G1Projective>>> {
    let original_len = generator_count(variables)?;
    let extended_variables = variables
        .checked_add(1)
        .ok_or_else(|| "variable count overflow".to_string())?;
    let extended_len = generator_count(extended_variables)?;
    let padding_len = extended_len
        .checked_sub(original_len + 1)
        .ok_or_else(|| "polynomial padding underflow".to_string())?;

    (0..padding_len)
        .map(|index| point(index as u64 + 2_000))
        .collect()
}

fn padding_evaluation_generators(variables: usize) -> DemoResult<Vec<IpaCurvePoint<G1Projective>>> {
    let original_len = generator_count(variables)?;
    let extended_variables = variables
        .checked_add(1)
        .ok_or_else(|| "variable count overflow".to_string())?;
    let extended_len = generator_count(extended_variables)?;
    let padding_len = extended_len
        .checked_sub(original_len)
        .ok_or_else(|| "evaluation padding underflow".to_string())?;

    (0..padding_len)
        .map(|index| point(index as u64 + 3_000))
        .collect()
}

fn build_ipa_demo_report() -> DemoResult<IpaDemoReport> {
    let polynomial = Multilinear::new(vec![Fr::from(2), Fr::from(3), Fr::from(5), Fr::from(7)])
        .map_err(|error| format!("invalid demo polynomial: {error:?}"))?;
    let variables = polynomial.variables();
    let opening_point = vec![Fr::from(3); variables];
    let expected_value = polynomial
        .evaluate(&opening_point)
        .map_err(|error| format!("failed to evaluate demo polynomial: {error:?}"))?;

    let (prover_key, verifier_key) = trim_ipa_integrated_keys(
        demo_basis(variables)?,
        point(5_000)?,
        padding_polynomial_generators(variables)?,
        padding_evaluation_generators(variables)?,
        point(9_000)?,
    )
    .map_err(|error| format!("failed to build IPA demo keys: {error:?}"))?;

    let witness = commit_ipa_backend(&prover_key, &polynomial, Fr::from(9))
        .map_err(|error| format!("IPA demo commit failed: {error:?}"))?;

    let mut prover_transcript = MerlinTranscript::new(b"snark-lab-cli-ipa-demo/v1");
    let opening = open_ipa_backend(
        &prover_key,
        &witness,
        &polynomial,
        &opening_point,
        &mut prover_transcript,
    )
    .map_err(|error| format!("IPA demo open failed: {error:?}"))?;

    let encoded_opening = encode_ipa_integrated_opening(&opening)
        .map_err(|error| format!("IPA demo opening encoding failed: {error:?}"))?;
    let decoded_opening = decode_ipa_integrated_opening::<Fr>(&encoded_opening)
        .map_err(|error| format!("IPA demo opening decoding failed: {error:?}"))?;

    let mut verifier_transcript = MerlinTranscript::new(b"snark-lab-cli-ipa-demo/v1");
    let verified_value = verify_ipa_backend(
        &verifier_key,
        &witness.commitment,
        &opening_point,
        &decoded_opening,
        &mut verifier_transcript,
    )
    .map_err(|error| format!("IPA demo verify failed: {error:?}"))?;

    if verified_value != expected_value {
        return Err(format!(
            "verified value mismatch: expected {expected_value:?}, got {verified_value:?}"
        ));
    }

    Ok(IpaDemoReport {
        variables,
        claimed_value: verified_value,
        commitment_bytes: witness.commitment.commitment_bytes.len(),
        encoded_opening_bytes: encoded_opening.len(),
        decoded_rounds: decoded_opening.proof.rounds.len(),
    })
}

fn run_ipa_demo() -> DemoResult<()> {
    let report = build_ipa_demo_report()?;

    println!("ipa-demo: verified blinded IPA opening");
    println!("variables={}", report.variables);
    println!("claimed_value={:?}", report.claimed_value);
    println!("commitment_bytes={}", report.commitment_bytes);
    println!("encoded_opening_bytes={}", report.encoded_opening_bytes);
    println!("decoded_rounds={}", report.decoded_rounds);

    Ok(())
}

fn run_verify_transcript(mut args: impl Iterator<Item = String>) {
    let path = args.next().unwrap_or_else(|| usage());
    if args.next().is_some() {
        usage();
    }

    let json = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("error: could not read {path}: {error}");
        process::exit(2);
    });

    match parse_and_verify(&json) {
        Ok(transcript) => {
            let protocol = match transcript.protocol {
                Protocol::Sumcheck => "sumcheck",
                Protocol::Zerocheck => "zerocheck",
            };
            println!(
                "accepted: {protocol} transcript verified over F_{}",
                transcript.field.modulus
            );
        }
        Err(error) => {
            eprintln!("rejected: {error}");
            process::exit(1);
        }
    }
}

fn main() {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("verify-transcript") => run_verify_transcript(args),
        Some("ipa-demo") => {
            if args.next().is_some() {
                usage();
            }

            if let Err(error) = run_ipa_demo() {
                eprintln!("error: {error}");
                process::exit(1);
            }
        }
        _ => usage(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipa_demo_builds_and_verifies_public_opening() {
        let report = build_ipa_demo_report().unwrap();

        assert_eq!(report.variables, 2);
        assert_eq!(report.decoded_rounds, 3);
        assert!(report.commitment_bytes > 0);
        assert!(report.encoded_opening_bytes > report.commitment_bytes);
    }
}
