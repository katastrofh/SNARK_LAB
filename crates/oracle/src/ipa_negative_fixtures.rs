use ark_bls12_381::{Fr, G1Projective};
use ark_ec::PrimeGroup;
use multilinear::Multilinear;
use snark_lab_transcript::MerlinTranscript;

use crate::ipa::IpaCommitment;
use crate::ipa_backend_codec::{decode_ipa_integrated_opening, encode_ipa_integrated_opening};
use crate::ipa_backend_integration::{
    commit_ipa_backend, open_ipa_backend, trim_ipa_integrated_keys, verify_ipa_backend,
    IpaBackendIntegrationError, IpaIntegratedCommitmentWitness, IpaIntegratedKeyPair,
    IpaIntegratedOpening, IpaIntegratedProverKey, IpaIntegratedVerifierKey,
};
use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint};
use crate::ipa_generators::expected_ipa_generator_count;

const TRANSCRIPT_LABEL: &[u8] = b"ipa-negative-fixtures-test/v1";

#[derive(Clone)]
struct Fixture {
    prover_key: IpaIntegratedProverKey<G1Projective>,
    verifier_key: IpaIntegratedVerifierKey<G1Projective>,
    witness: IpaIntegratedCommitmentWitness<Fr>,
    point: Vec<Fr>,
    opening: IpaIntegratedOpening<Fr>,
}

fn point_generator(seed: u64) -> IpaCurvePoint<G1Projective> {
    IpaCurvePoint::from_projective(G1Projective::generator() * Fr::from(seed)).unwrap()
}

fn basis(variables: usize) -> IpaCurveGeneratorBasis<G1Projective> {
    let count = expected_ipa_generator_count(variables).unwrap();

    IpaCurveGeneratorBasis::new(
        variables,
        (0..count)
            .map(|index| point_generator(index as u64 + 1))
            .collect(),
        (0..count)
            .map(|index| point_generator(index as u64 + 100))
            .collect(),
        point_generator(999),
    )
    .unwrap()
}

fn padding_polynomial(variables: usize) -> Vec<IpaCurvePoint<G1Projective>> {
    let original_len = expected_ipa_generator_count(variables).unwrap();
    let extended_len = expected_ipa_generator_count(variables + 1).unwrap();

    (0..(extended_len - original_len - 1))
        .map(|index| point_generator(index as u64 + 2_000))
        .collect()
}

fn padding_evaluation(variables: usize) -> Vec<IpaCurvePoint<G1Projective>> {
    let original_len = expected_ipa_generator_count(variables).unwrap();
    let extended_len = expected_ipa_generator_count(variables + 1).unwrap();

    (0..(extended_len - original_len))
        .map(|index| point_generator(index as u64 + 3_000))
        .collect()
}

fn keys(variables: usize) -> IpaIntegratedKeyPair<G1Projective> {
    trim_ipa_integrated_keys(
        basis(variables),
        point_generator(5_000),
        padding_polynomial(variables),
        padding_evaluation(variables),
        point_generator(9_000),
    )
    .unwrap()
}

fn polynomial(values: &[u64]) -> Multilinear<Fr> {
    Multilinear::new(values.iter().copied().map(Fr::from).collect()).unwrap()
}

fn fixture() -> Fixture {
    let polynomial = polynomial(&[2, 3, 5, 7]);
    let point = vec![Fr::from(3); polynomial.variables()];
    let (prover_key, verifier_key) = keys(polynomial.variables());

    let witness = commit_ipa_backend(&prover_key, &polynomial, Fr::from(9)).unwrap();

    let mut transcript = MerlinTranscript::new(TRANSCRIPT_LABEL);
    let opening =
        open_ipa_backend(&prover_key, &witness, &polynomial, &point, &mut transcript).unwrap();

    Fixture {
        prover_key,
        verifier_key,
        witness,
        point,
        opening,
    }
}

fn verify_with_label(
    verifier_key: &IpaIntegratedVerifierKey<G1Projective>,
    commitment: &IpaCommitment,
    point: &[Fr],
    opening: &IpaIntegratedOpening<Fr>,
    transcript_label: &'static [u8],
) -> Result<Fr, IpaBackendIntegrationError<Fr>> {
    let mut transcript = MerlinTranscript::new(transcript_label);

    verify_ipa_backend(verifier_key, commitment, point, opening, &mut transcript)
}

#[test]
fn negative_fixture_rejects_wrong_commitment() {
    let fixture = fixture();
    let other_polynomial = polynomial(&[2, 3, 5, 8]);
    let other_witness =
        commit_ipa_backend(&fixture.prover_key, &other_polynomial, Fr::from(9)).unwrap();

    assert!(verify_with_label(
        &fixture.verifier_key,
        &other_witness.commitment,
        &fixture.point,
        &fixture.opening,
        TRANSCRIPT_LABEL,
    )
    .is_err());
}

#[test]
fn negative_fixture_rejects_wrong_opening_point() {
    let fixture = fixture();
    let wrong_point = vec![Fr::from(4), Fr::from(3)];

    assert!(verify_with_label(
        &fixture.verifier_key,
        &fixture.witness.commitment,
        &wrong_point,
        &fixture.opening,
        TRANSCRIPT_LABEL,
    )
    .is_err());
}

#[test]
fn negative_fixture_rejects_wrong_transcript_label() {
    let fixture = fixture();

    assert!(verify_with_label(
        &fixture.verifier_key,
        &fixture.witness.commitment,
        &fixture.point,
        &fixture.opening,
        b"ipa-negative-fixtures-wrong-transcript/v1",
    )
    .is_err());
}

#[test]
fn negative_fixture_rejects_tampered_claimed_value() {
    let fixture = fixture();
    let mut opening = fixture.opening.clone();
    opening.claimed_value += Fr::from(1);

    assert!(verify_with_label(
        &fixture.verifier_key,
        &fixture.witness.commitment,
        &fixture.point,
        &opening,
        TRANSCRIPT_LABEL,
    )
    .is_err());
}

#[test]
fn negative_fixture_rejects_tampered_final_scalar() {
    let fixture = fixture();
    let mut opening = fixture.opening.clone();
    opening.proof.final_polynomial_scalar += Fr::from(1);

    assert!(verify_with_label(
        &fixture.verifier_key,
        &fixture.witness.commitment,
        &fixture.point,
        &opening,
        TRANSCRIPT_LABEL,
    )
    .is_err());
}

#[test]
fn negative_fixture_rejects_tampered_round_commitment_bytes() {
    let fixture = fixture();
    let mut opening = fixture.opening.clone();

    opening.proof.rounds[0].left_commitment_bytes[0] ^= 1;

    assert!(verify_with_label(
        &fixture.verifier_key,
        &fixture.witness.commitment,
        &fixture.point,
        &opening,
        TRANSCRIPT_LABEL,
    )
    .is_err());
}

#[test]
fn negative_fixture_rejects_wrong_padding_generator_material() {
    let fixture = fixture();
    let mut verifier_key = fixture.verifier_key.clone();

    verifier_key.padding_polynomial_generators[0] = point_generator(7_777);

    assert!(verify_with_label(
        &verifier_key,
        &fixture.witness.commitment,
        &fixture.point,
        &fixture.opening,
        TRANSCRIPT_LABEL,
    )
    .is_err());
}

#[test]
fn negative_fixture_rejects_wrong_verifier_key_size() {
    let fixture = fixture();
    let (_wrong_prover_key, wrong_verifier_key) = keys(1);

    assert!(verify_with_label(
        &wrong_verifier_key,
        &fixture.witness.commitment,
        &fixture.point,
        &fixture.opening,
        TRANSCRIPT_LABEL,
    )
    .is_err());
}

#[test]
fn negative_fixture_rejects_corrupt_encoded_opening() {
    let fixture = fixture();
    let mut encoded = encode_ipa_integrated_opening(&fixture.opening).unwrap();

    let proof_start = b"SL-IPA-BACKEND-OPEN1".len() + 8 + 8;
    let tamper_index = proof_start + b"SL-IPA-PROOF1".len() + 8 + 8;
    encoded[tamper_index] ^= 1;

    if let Ok(decoded_opening) = decode_ipa_integrated_opening::<Fr>(&encoded) {
        assert!(verify_with_label(
            &fixture.verifier_key,
            &fixture.witness.commitment,
            &fixture.point,
            &decoded_opening,
            TRANSCRIPT_LABEL,
        )
        .is_err());
    }
}
