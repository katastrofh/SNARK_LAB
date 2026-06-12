#![forbid(unsafe_code)]

use ark_bls12_381::{Fr, G1Projective};
use ark_ec::PrimeGroup;
use snark_lab_oracle::{
    canonical_ipa_srs_digest, encode_ipa_srs_file, expected_ipa_generator_count,
    validate_ipa_srs_provenance, IpaCurveGeneratorBasis, IpaCurvePoint, IpaSrsProvenance,
    IpaSrsSource, IpaVerifiedSrs,
};
use std::{
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn point(seed: u64) -> IpaCurvePoint<G1Projective> {
    IpaCurvePoint::from_projective(G1Projective::generator() * Fr::from(seed)).unwrap()
}

fn basis(variables: usize) -> IpaCurveGeneratorBasis<G1Projective> {
    let count = expected_ipa_generator_count(variables).unwrap();

    IpaCurveGeneratorBasis::new(
        variables,
        (0..count).map(|index| point(index as u64 + 1)).collect(),
        (0..count).map(|index| point(index as u64 + 100)).collect(),
        point(999),
    )
    .unwrap()
}

fn valid_verified_srs() -> IpaVerifiedSrs<G1Projective> {
    let basis = basis(2);

    let provenance = IpaSrsProvenance {
        curve_id: "BLS12-381-G1".to_string(),
        max_variables: basis.variables,
        source: IpaSrsSource::ExternalTrustedSetup {
            name: "cli-integration-test-srs".to_string(),
            uri: "file:///tmp/snark-lab-cli-integration-test.srs".to_string(),
            artifact_sha256: [7u8; 32],
        },
        canonical_basis_sha256: canonical_ipa_srs_digest(&basis).unwrap(),
    };

    validate_ipa_srs_provenance(basis, provenance).unwrap()
}

fn valid_srs_bytes() -> Vec<u8> {
    encode_ipa_srs_file(&valid_verified_srs()).unwrap()
}

fn unique_temp_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!(
        "snark-lab-{label}-{}-{nanos}.srs",
        std::process::id()
    ))
}

fn write_temp_srs(label: &str, bytes: &[u8]) -> PathBuf {
    let path = unique_temp_path(label);
    fs::write(&path, bytes).unwrap();
    path
}

fn remove_temp_file(path: PathBuf) {
    let _ = fs::remove_file(path);
}

fn run_cli<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_snark-lab-cli"))
        .args(args)
        .output()
        .expect("snark-lab-cli should execute")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

#[test]
fn cli_validates_valid_srs_with_explicit_curve() {
    let path = write_temp_srs("valid-explicit", &valid_srs_bytes());
    let path_arg = path.to_string_lossy().to_string();

    let output = run_cli([
        "ipa-srs-validate".to_string(),
        "--curve".to_string(),
        "bls12-381-g1".to_string(),
        path_arg,
    ]);

    assert_success(&output);

    let out = stdout(&output);
    assert!(out.contains("ipa-srs-validate: accepted production IPA SRS"));
    assert!(out.contains("curve=bls12-381-g1"));
    assert!(out.contains("curve_id=BLS12-381-G1"));
    assert!(out.contains("max_variables=2"));
    assert!(out.contains("source=external-trusted-setup"));
    assert!(out.contains("polynomial_generators=4"));
    assert!(out.contains("evaluation_generators=4"));
    assert!(out.contains("blinding_generator=present"));

    remove_temp_file(path);
}

#[test]
fn cli_validates_valid_srs_with_default_curve() {
    let path = write_temp_srs("valid-default", &valid_srs_bytes());
    let path_arg = path.to_string_lossy().to_string();

    let output = run_cli(["ipa-srs-validate".to_string(), path_arg]);

    assert_success(&output);
    assert!(stdout(&output).contains("ipa-srs-validate: accepted production IPA SRS"));

    remove_temp_file(path);
}

#[test]
fn cli_rejects_unsupported_curve_before_file_read() {
    let output = run_cli([
        "ipa-srs-validate",
        "--curve",
        "unsupported-curve",
        "/this/file/should/not/be/read.srs",
    ]);

    assert_failure(&output);
    assert!(stderr(&output).contains("unsupported IPA SRS curve"));
}

#[test]
fn cli_rejects_missing_srs_file() {
    let output = run_cli([
        "ipa-srs-validate",
        "--curve",
        "bls12-381-g1",
        "/definitely/missing/snark-lab.srs",
    ]);

    assert_failure(&output);
    assert!(stderr(&output).contains("IPA SRS validation failed"));
}

#[test]
fn cli_rejects_srs_with_wrong_magic() {
    let mut bytes = valid_srs_bytes();
    bytes[0] = b'X';

    let path = write_temp_srs("wrong-magic", &bytes);
    let path_arg = path.to_string_lossy().to_string();

    let output = run_cli([
        "ipa-srs-validate".to_string(),
        "--curve".to_string(),
        "bls12-381-g1".to_string(),
        path_arg,
    ]);

    assert_failure(&output);
    assert!(stderr(&output).contains("InvalidMagic"));

    remove_temp_file(path);
}

#[test]
fn cli_rejects_truncated_srs_file() {
    let mut bytes = valid_srs_bytes();
    bytes.pop();

    let path = write_temp_srs("truncated", &bytes);
    let path_arg = path.to_string_lossy().to_string();

    let output = run_cli([
        "ipa-srs-validate".to_string(),
        "--curve".to_string(),
        "bls12-381-g1".to_string(),
        path_arg,
    ]);

    assert_failure(&output);
    assert!(stderr(&output).contains("Truncated"));

    remove_temp_file(path);
}

#[test]
fn cli_rejects_srs_with_trailing_bytes() {
    let mut bytes = valid_srs_bytes();
    bytes.push(0);

    let path = write_temp_srs("trailing", &bytes);
    let path_arg = path.to_string_lossy().to_string();

    let output = run_cli([
        "ipa-srs-validate".to_string(),
        "--curve".to_string(),
        "bls12-381-g1".to_string(),
        path_arg,
    ]);

    assert_failure(&output);
    assert!(stderr(&output).contains("TrailingBytes"));

    remove_temp_file(path);
}
