use field::Fp;
use multilinear::Multilinear;
use permcheck::{
    estimate_product_tree, estimate_rational_stream, product_fingerprint, rational_fingerprint,
};
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
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(18)
        .min(24);
    let n = 1usize << power;
    let values: Vec<Fp> = (0..n).map(|i| Fp::from((i as u64 * 17 + 3) % 32)).collect();
    let beta = Fp::from(41);

    let start = Instant::now();
    let product = product_fingerprint(&values, beta);
    let product_time = start.elapsed();
    let start = Instant::now();
    let rational = rational_fingerprint(&values, beta);
    let rational_time = start.elapsed();
    let product_io = estimate_product_tree(n, 32);
    let rational_io = estimate_rational_stream(n, 32);

    println!("snark-lab PermCheck benchmark — 2^{power} = {n} field elements");
    println!("product  fingerprint={product:>3} runtime={product_time:?} modeled I/O={} peak={} elements", human(product_io.bytes_read + product_io.bytes_written), product_io.peak_field_elements);
    println!(
        "rational fingerprint={:?} runtime={rational_time:?} modeled I/O={} peak={} elements",
        rational.ok(),
        human(rational_io.bytes_read),
        rational_io.peak_field_elements
    );

    let small = Multilinear::new((0..1024).map(|i| Fp::from(i as u64)).collect()).unwrap();
    let start = Instant::now();
    let proof = sumcheck::prove(&small, small.sum_hypercube());
    let elapsed = start.elapsed();
    println!(
        "sumcheck  variables={} rounds={} runtime={elapsed:?} verified={}",
        small.variables(),
        proof.rounds.len(),
        sumcheck::verify(&small, &proof).is_ok()
    );
}
