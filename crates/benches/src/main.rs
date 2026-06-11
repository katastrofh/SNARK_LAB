#![forbid(unsafe_code)]
use field::DefaultField as Fr;
use multilinear::Multilinear;
use permcheck::{
    estimate_product_tree, estimate_rational_stream, product_fingerprint, rational_fingerprint,
};
use snark_lab_transcript::MerlinTranscript;
use std::time::Instant;

fn human(bytes: usize) -> String {
    if bytes >= 1 << 30 {
        format!("{:.1} GiB", bytes as f64 / (1 << 30) as f64)
    } else if bytes >= 1 << 20 {
        format!("{:.1} MiB", bytes as f64 / (1 << 20) as f64)
    } else {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    }
}

fn main() {
    let power = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(18)
        .min(24);
    let elements = 1_usize << power;
    let values: Vec<Fr> = (0..elements)
        .map(|index| Fr::from((index as u64 * 17 + 3) % 32))
        .collect();
    let beta = Fr::from(41_u64);

    let start = Instant::now();
    let product = product_fingerprint(&values, beta);
    let product_time = start.elapsed();
    let start = Instant::now();
    let rational = rational_fingerprint(&values, beta);
    let rational_time = start.elapsed();
    let product_io = estimate_product_tree(elements, 32);
    let rational_io = estimate_rational_stream(elements, 32);

    println!("snark-lab PermCheck benchmark — 2^{power} = {elements} BLS12-381 scalar elements");
    println!(
        "product  fingerprint={product:?} runtime={product_time:?} modeled I/O={} peak={} elements",
        human(product_io.bytes_read + product_io.bytes_written),
        product_io.peak_field_elements
    );
    println!(
        "rational fingerprint={:?} runtime={rational_time:?} modeled I/O={} peak={} elements",
        rational.ok(),
        human(rational_io.bytes_read),
        rational_io.peak_field_elements
    );

    let polynomial =
        Multilinear::new((0..1024).map(|value| Fr::from(value as u64)).collect()).unwrap();
    let claim = polynomial.sum_hypercube();
    let start = Instant::now();
    let mut prover_transcript = MerlinTranscript::new(b"snark-lab-benchmark");
    let proof = sumcheck::prove(&polynomial, claim, &mut prover_transcript);
    let elapsed = start.elapsed();
    let mut verifier_transcript = MerlinTranscript::new(b"snark-lab-benchmark");
    println!(
        "sumcheck  variables={} rounds={} runtime={elapsed:?} verified={}",
        polynomial.variables(),
        proof.round_polynomials.len(),
        sumcheck::verify(&polynomial, claim, &proof, &mut verifier_transcript).is_ok()
    );
}
