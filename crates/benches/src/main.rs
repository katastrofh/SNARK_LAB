#![forbid(unsafe_code)]

use ark_bls12_381::{Fr, G1Projective};
use ark_ec::PrimeGroup;
use multilinear::Multilinear;
use permcheck::{
    estimate_product_tree, estimate_rational_stream, product_fingerprint, rational_fingerprint,
};
use snark_lab_oracle::{
    canonical_ipa_srs_digest, commit_ipa_backend, encode_ipa_integrated_opening,
    expected_ipa_generator_count, open_ipa_backend, trim_ipa_integrated_keys, verify_ipa_backend,
    IpaCurveGeneratorBasis, IpaCurvePoint, IpaIntegratedProverKey, IpaIntegratedVerifierKey,
};
use snark_lab_transcript::MerlinTranscript;
use std::{fmt::Display, process, str::FromStr, time::Duration, time::Instant};

type BenchResult<T> = Result<T, String>;

const SUMCHECK_TRANSCRIPT_LABEL: &[u8] = b"snark-lab-benchmark/sumcheck/v1";
const IPA_TRANSCRIPT_LABEL: &[u8] = b"snark-lab-benchmark/ipa/v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BenchConfig {
    permcheck_log2: u32,
    ipa_variables: usize,
    samples: usize,
}

#[derive(Clone, Debug)]
struct Timed<T> {
    elapsed: Duration,
    value: T,
}

fn parse_or<T>(args: &[String], index: usize, default: T, name: &'static str) -> BenchResult<T>
where
    T: FromStr,
    T::Err: Display,
{
    match args.get(index) {
        Some(value) => value
            .parse::<T>()
            .map_err(|error| format!("invalid {name} value '{value}': {error}")),
        None => Ok(default),
    }
}

fn parse_config(args: &[String]) -> BenchResult<BenchConfig> {
    if args.len() > 3 {
        return Err(
            "usage: snark-lab-benches [permcheck_log2<=24] [ipa_variables<=12] [samples<=50]"
                .to_string(),
        );
    }

    let permcheck_log2 = parse_or(args, 0, 18_u32, "permcheck_log2")?;
    let ipa_variables = parse_or(args, 1, 8_usize, "ipa_variables")?;
    let samples = parse_or(args, 2, 3_usize, "samples")?;

    if permcheck_log2 > 24 {
        return Err("permcheck_log2 must be <= 24".to_string());
    }

    if ipa_variables > 12 {
        return Err("ipa_variables must be <= 12".to_string());
    }

    if !(1..=50).contains(&samples) {
        return Err("samples must be in 1..=50".to_string());
    }

    Ok(BenchConfig {
        permcheck_log2,
        ipa_variables,
        samples,
    })
}

fn time_min<T, F>(samples: usize, mut f: F) -> BenchResult<Timed<T>>
where
    F: FnMut() -> BenchResult<T>,
{
    if samples == 0 {
        return Err("samples must be nonzero".to_string());
    }

    let mut best_elapsed = None;
    let mut best_value = None;

    for _ in 0..samples {
        let start = Instant::now();
        let value = f()?;
        let elapsed = start.elapsed();

        if best_elapsed.is_none_or(|best| elapsed < best) {
            best_elapsed = Some(elapsed);
            best_value = Some(value);
        }
    }

    match (best_elapsed, best_value) {
        (Some(elapsed), Some(value)) => Ok(Timed { elapsed, value }),
        _ => Err("benchmark produced no samples".to_string()),
    }
}

fn human_bytes(bytes: usize) -> String {
    if bytes >= 1 << 30 {
        format!("{:.1}GiB", bytes as f64 / (1 << 30) as f64)
    } else if bytes >= 1 << 20 {
        format!("{:.1}MiB", bytes as f64 / (1 << 20) as f64)
    } else if bytes >= 1 << 10 {
        format!("{:.1}KiB", bytes as f64 / (1 << 10) as f64)
    } else {
        format!("{bytes}B")
    }
}

fn duration_us(duration: Duration) -> u128 {
    duration.as_micros()
}

fn digest_prefix(digest: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut out = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }

    out
}

fn curve_point(seed: u64) -> BenchResult<IpaCurvePoint<G1Projective>> {
    IpaCurvePoint::from_projective(G1Projective::generator() * Fr::from(seed))
        .map_err(|error| format!("invalid benchmark generator seed {seed}: {error:?}"))
}

fn generator_count(variables: usize) -> BenchResult<usize> {
    expected_ipa_generator_count(variables).map_err(|error| {
        format!("invalid IPA generator count for {variables} variables: {error:?}")
    })
}

fn benchmark_basis(variables: usize) -> BenchResult<IpaCurveGeneratorBasis<G1Projective>> {
    let count = generator_count(variables)?;

    IpaCurveGeneratorBasis::new(
        variables,
        (0..count)
            .map(|index| curve_point(index as u64 + 1))
            .collect::<BenchResult<Vec<_>>>()?,
        (0..count)
            .map(|index| curve_point(index as u64 + 100))
            .collect::<BenchResult<Vec<_>>>()?,
        curve_point(999)?,
    )
    .map_err(|error| format!("invalid benchmark IPA basis: {error:?}"))
}

fn padding_polynomial(variables: usize) -> BenchResult<Vec<IpaCurvePoint<G1Projective>>> {
    let original_len = generator_count(variables)?;
    let extended_len = generator_count(
        variables
            .checked_add(1)
            .ok_or_else(|| "IPA variable count overflow".to_string())?,
    )?;

    let padding_len = extended_len
        .checked_sub(original_len + 1)
        .ok_or_else(|| "IPA polynomial padding underflow".to_string())?;

    (0..padding_len)
        .map(|index| curve_point(index as u64 + 2_000))
        .collect()
}

fn padding_evaluation(variables: usize) -> BenchResult<Vec<IpaCurvePoint<G1Projective>>> {
    let original_len = generator_count(variables)?;
    let extended_len = generator_count(
        variables
            .checked_add(1)
            .ok_or_else(|| "IPA variable count overflow".to_string())?,
    )?;

    let padding_len = extended_len
        .checked_sub(original_len)
        .ok_or_else(|| "IPA evaluation padding underflow".to_string())?;

    (0..padding_len)
        .map(|index| curve_point(index as u64 + 3_000))
        .collect()
}

fn trim_keys(
    basis: IpaCurveGeneratorBasis<G1Projective>,
    variables: usize,
) -> BenchResult<(
    IpaIntegratedProverKey<G1Projective>,
    IpaIntegratedVerifierKey<G1Projective>,
)> {
    trim_ipa_integrated_keys(
        basis,
        curve_point(5_000)?,
        padding_polynomial(variables)?,
        padding_evaluation(variables)?,
        curve_point(9_000)?,
    )
    .map_err(|error| format!("IPA key trim failed: {error:?}"))
}

fn benchmark_polynomial(variables: usize) -> BenchResult<Multilinear<Fr>> {
    let len = 1usize
        .checked_shl(variables as u32)
        .ok_or_else(|| "polynomial length overflow".to_string())?;

    let evaluations = (0..len)
        .map(|index| Fr::from((index as u64).wrapping_mul(17).wrapping_add(3)))
        .collect();

    Multilinear::new(evaluations)
        .map_err(|error| format!("invalid benchmark polynomial: {error:?}"))
}

fn bench_permcheck(config: BenchConfig) -> BenchResult<()> {
    let elements = 1usize
        .checked_shl(config.permcheck_log2)
        .ok_or_else(|| "permcheck element count overflow".to_string())?;

    let values: Vec<Fr> = (0..elements)
        .map(|index| Fr::from((index as u64 * 17 + 3) % 32))
        .collect();
    let beta = Fr::from(41_u64);

    let product = time_min(config.samples, || Ok(product_fingerprint(&values, beta)))?;
    let rational = time_min(config.samples, || {
        rational_fingerprint(&values, beta)
            .map_err(|error| format!("rational fingerprint failed: {error:?}"))
    })?;

    let product_io = estimate_product_tree(elements, 32);
    let rational_io = estimate_rational_stream(elements, 32);

    println!(
        "bench=permcheck_product elements={} samples={} best_us={} modeled_io={} peak_field_elements={} fingerprint={:?}",
        elements,
        config.samples,
        duration_us(product.elapsed),
        human_bytes(product_io.bytes_read + product_io.bytes_written),
        product_io.peak_field_elements,
        product.value
    );

    println!(
        "bench=permcheck_rational elements={} samples={} best_us={} modeled_io={} peak_field_elements={} fingerprint={:?}",
        elements,
        config.samples,
        duration_us(rational.elapsed),
        human_bytes(rational_io.bytes_read),
        rational_io.peak_field_elements,
        rational.value
    );

    Ok(())
}

fn bench_sumcheck(config: BenchConfig) -> BenchResult<()> {
    let polynomial = benchmark_polynomial(10)?;
    let claim = polynomial.sum_hypercube();

    let proof = time_min(config.samples, || {
        let mut transcript = MerlinTranscript::new(SUMCHECK_TRANSCRIPT_LABEL);
        Ok(sumcheck::prove(&polynomial, claim, &mut transcript))
    })?;

    let mut verifier_transcript = MerlinTranscript::new(SUMCHECK_TRANSCRIPT_LABEL);
    let verified =
        sumcheck::verify(&polynomial, claim, &proof.value, &mut verifier_transcript).is_ok();

    println!(
        "bench=sumcheck_prove variables={} rounds={} samples={} best_us={} verified={}",
        polynomial.variables(),
        proof.value.round_polynomials.len(),
        config.samples,
        duration_us(proof.elapsed),
        verified
    );

    Ok(())
}

fn bench_ipa(config: BenchConfig) -> BenchResult<()> {
    let variables = config.ipa_variables;
    let polynomial = benchmark_polynomial(variables)?;
    let opening_point = vec![Fr::from(3); variables];
    let expected_value = polynomial
        .evaluate(&opening_point)
        .map_err(|error| format!("benchmark polynomial evaluation failed: {error:?}"))?;

    let basis = benchmark_basis(variables)?;

    let srs_digest = time_min(config.samples, || {
        canonical_ipa_srs_digest(&basis)
            .map_err(|error| format!("IPA SRS digest computation failed: {error:?}"))
    })?;

    let key_material = time_min(config.samples, || trim_keys(basis.clone(), variables))?;

    let (prover_key, verifier_key) = key_material.value;

    let commit = time_min(config.samples, || {
        commit_ipa_backend(&prover_key, &polynomial, Fr::from(9))
            .map_err(|error| format!("IPA commit failed: {error:?}"))
    })?;

    let open = time_min(config.samples, || {
        let mut transcript = MerlinTranscript::new(IPA_TRANSCRIPT_LABEL);
        open_ipa_backend(
            &prover_key,
            &commit.value,
            &polynomial,
            &opening_point,
            &mut transcript,
        )
        .map_err(|error| format!("IPA open failed: {error:?}"))
    })?;

    if open.value.claimed_value != expected_value {
        return Err(format!(
            "IPA opening claim mismatch: expected {expected_value:?}, got {:?}",
            open.value.claimed_value
        ));
    }

    let encoded = time_min(config.samples, || {
        encode_ipa_integrated_opening(&open.value)
            .map_err(|error| format!("IPA opening encoding failed: {error:?}"))
    })?;

    let decoded = time_min(config.samples, || {
        snark_lab_oracle::decode_ipa_integrated_opening::<Fr>(&encoded.value)
            .map_err(|error| format!("IPA opening decoding failed: {error:?}"))
    })?;

    let verified = time_min(config.samples, || {
        let mut transcript = MerlinTranscript::new(IPA_TRANSCRIPT_LABEL);
        verify_ipa_backend(
            &verifier_key,
            &commit.value.commitment,
            &opening_point,
            &decoded.value,
            &mut transcript,
        )
        .map_err(|error| format!("IPA verify failed: {error:?}"))
    })?;

    if verified.value != expected_value {
        return Err(format!(
            "IPA verified value mismatch: expected {expected_value:?}, got {:?}",
            verified.value
        ));
    }

    println!(
        "bench=ipa_srs_digest variables={} generators={} samples={} best_us={} digest_prefix={}",
        variables,
        basis.polynomial_generators.len() + basis.evaluation_generators.len() + 1,
        config.samples,
        duration_us(srs_digest.elapsed),
        digest_prefix(&srs_digest.value)
    );

    println!(
        "bench=ipa_key_trim variables={} samples={} best_us={} synthetic_generator_fixture=true",
        variables,
        config.samples,
        duration_us(key_material.elapsed)
    );

    println!(
        "bench=ipa_commit variables={} evaluations={} samples={} best_us={} commitment_bytes={}",
        variables,
        polynomial.evaluations().len(),
        config.samples,
        duration_us(commit.elapsed),
        commit.value.commitment.commitment_bytes.len()
    );

    println!(
        "bench=ipa_open variables={} rounds={} samples={} best_us={} claimed_value={:?}",
        variables,
        open.value.proof.rounds.len(),
        config.samples,
        duration_us(open.elapsed),
        open.value.claimed_value
    );

    println!(
        "bench=ipa_opening_encode variables={} samples={} best_us={} encoded_bytes={}",
        variables,
        config.samples,
        duration_us(encoded.elapsed),
        encoded.value.len()
    );

    println!(
        "bench=ipa_opening_decode variables={} samples={} best_us={} decoded_rounds={}",
        variables,
        config.samples,
        duration_us(decoded.elapsed),
        decoded.value.proof.rounds.len()
    );

    println!(
        "bench=ipa_verify variables={} samples={} best_us={} verified_value={:?}",
        variables,
        config.samples,
        duration_us(verified.elapsed),
        verified.value
    );

    Ok(())
}

fn run() -> BenchResult<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let config = parse_config(&args)?;

    println!("snark-lab benchmark suite");
    println!(
        "config permcheck_log2={} ipa_variables={} samples={}",
        config.permcheck_log2, config.ipa_variables, config.samples
    );
    println!("boundary ipa_generator_material=synthetic-benchmark-fixture not_production_srs=true");
    println!("timing best_of_samples=true unit=microseconds");

    bench_permcheck(config)?;
    bench_sumcheck(config)?;
    bench_ipa(config)?;

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_uses_defaults() {
        let config = parse_config(&[]).unwrap();

        assert_eq!(
            config,
            BenchConfig {
                permcheck_log2: 18,
                ipa_variables: 8,
                samples: 3,
            }
        );
    }

    #[test]
    fn parse_config_accepts_three_positionals() {
        let config = parse_config(&["20".to_string(), "9".to_string(), "4".to_string()]).unwrap();

        assert_eq!(
            config,
            BenchConfig {
                permcheck_log2: 20,
                ipa_variables: 9,
                samples: 4,
            }
        );
    }

    #[test]
    fn parse_config_rejects_excessive_ipa_variables() {
        let error = parse_config(&["18".to_string(), "13".to_string()]).unwrap_err();

        assert_eq!(error, "ipa_variables must be <= 12");
    }

    #[test]
    fn parse_config_rejects_zero_samples() {
        let error =
            parse_config(&["18".to_string(), "8".to_string(), "0".to_string()]).unwrap_err();

        assert_eq!(error, "samples must be in 1..=50");
    }

    #[test]
    fn human_bytes_formats_units() {
        assert_eq!(human_bytes(512), "512B");
        assert_eq!(human_bytes(2048), "2.0KiB");
        assert_eq!(human_bytes(2 << 20), "2.0MiB");
    }

    #[test]
    fn digest_prefix_has_16_hex_chars() {
        let digest = [0xab; 32];

        assert_eq!(digest_prefix(&digest), "abababababababab");
    }
}
