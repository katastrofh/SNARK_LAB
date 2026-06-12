use ark_bls12_381::{Fr, G1Projective};
use ark_ec::PrimeGroup;
use ark_ff::{UniformRand, Zero};
use multilinear::Multilinear;
use rand::{rngs::StdRng, SeedableRng};
use snark_lab_transcript::MerlinTranscript;

use crate::ipa::IpaCommitment;
use crate::ipa_backend_codec::{decode_ipa_integrated_opening, encode_ipa_integrated_opening};
use crate::ipa_backend_integration::{
    commit_ipa_backend, open_ipa_backend, trim_ipa_integrated_keys, verify_ipa_backend,
    IpaIntegratedCommitmentWitness, IpaIntegratedKeyPair, IpaIntegratedOpening,
    IpaIntegratedProverKey, IpaIntegratedVerifierKey,
};
use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint};
use crate::ipa_generators::expected_ipa_generator_count;

const ROUNDTRIP_TRANSCRIPT_LABEL: &[u8] = b"ipa-randomized-roundtrip-test/v1";

#[derive(Clone)]
struct RoundtripFixture {
    verifier_key: IpaIntegratedVerifierKey<G1Projective>,
    commitment: IpaCommitment,
    point: Vec<Fr>,
    opening: IpaIntegratedOpening<Fr>,
    expected_value: Fr,
}

fn test_point(seed: u64) -> IpaCurvePoint<G1Projective> {
    IpaCurvePoint::from_projective(G1Projective::generator() * Fr::from(seed)).unwrap()
}

fn generator_count(variables: usize) -> usize {
    expected_ipa_generator_count(variables).unwrap()
}

fn test_basis(variables: usize, case_index: usize) -> IpaCurveGeneratorBasis<G1Projective> {
    let count = generator_count(variables);
    let base = 100_000u64 + (variables as u64 * 10_000) + (case_index as u64 * 1_000);

    IpaCurveGeneratorBasis::new(
        variables,
        (0..count)
            .map(|index| test_point(base + index as u64 + 1))
            .collect(),
        (0..count)
            .map(|index| test_point(base + 2_000 + index as u64 + 1))
            .collect(),
        test_point(base + 4_000),
    )
    .unwrap()
}

fn padding_polynomial(variables: usize, case_index: usize) -> Vec<IpaCurvePoint<G1Projective>> {
    let original_len = generator_count(variables);
    let extended_len = generator_count(variables + 1);
    let padding_len = extended_len - original_len - 1;
    let base = 500_000u64 + (variables as u64 * 10_000) + (case_index as u64 * 1_000);

    (0..padding_len)
        .map(|index| test_point(base + index as u64 + 1))
        .collect()
}

fn padding_evaluation(variables: usize, case_index: usize) -> Vec<IpaCurvePoint<G1Projective>> {
    let original_len = generator_count(variables);
    let extended_len = generator_count(variables + 1);
    let padding_len = extended_len - original_len;
    let base = 700_000u64 + (variables as u64 * 10_000) + (case_index as u64 * 1_000);

    (0..padding_len)
        .map(|index| test_point(base + index as u64 + 1))
        .collect()
}

fn test_keys(variables: usize, case_index: usize) -> IpaIntegratedKeyPair<G1Projective> {
    let base = 900_000u64 + (variables as u64 * 10_000) + (case_index as u64 * 1_000);

    trim_ipa_integrated_keys(
        test_basis(variables, case_index),
        test_point(base + 1),
        padding_polynomial(variables, case_index),
        padding_evaluation(variables, case_index),
        test_point(base + 2),
    )
    .unwrap()
}

fn random_nonzero_field(rng: &mut StdRng) -> Fr {
    loop {
        let value = Fr::rand(rng);
        if !value.is_zero() {
            return value;
        }
    }
}

fn random_polynomial(rng: &mut StdRng, variables: usize) -> Multilinear<Fr> {
    let len = 1usize << variables;
    let evaluations = (0..len).map(|_| Fr::rand(rng)).collect();

    Multilinear::new(evaluations).unwrap()
}

fn random_opening_point(rng: &mut StdRng, variables: usize) -> Vec<Fr> {
    (0..variables).map(|_| Fr::rand(rng)).collect()
}

fn build_fixture(rng: &mut StdRng, variables: usize, case_index: usize) -> RoundtripFixture {
    let polynomial = random_polynomial(rng, variables);
    let point = random_opening_point(rng, variables);
    let blinding = random_nonzero_field(rng);
    let expected_value = polynomial.evaluate(&point).unwrap();

    let (prover_key, verifier_key) = test_keys(variables, case_index);
    let witness = commit_ipa_backend(&prover_key, &polynomial, blinding).unwrap();

    let mut prover_transcript = MerlinTranscript::new(ROUNDTRIP_TRANSCRIPT_LABEL);
    let opening = open_ipa_backend(
        &prover_key,
        &witness,
        &polynomial,
        &point,
        &mut prover_transcript,
    )
    .unwrap();

    RoundtripFixture {
        verifier_key,
        commitment: witness.commitment,
        point,
        opening,
        expected_value,
    }
}

fn verify_fixture(fixture: &RoundtripFixture) -> Fr {
    let mut verifier_transcript = MerlinTranscript::new(ROUNDTRIP_TRANSCRIPT_LABEL);

    verify_ipa_backend(
        &fixture.verifier_key,
        &fixture.commitment,
        &fixture.point,
        &fixture.opening,
        &mut verifier_transcript,
    )
    .unwrap()
}

fn commit_with_key(
    prover_key: &IpaIntegratedProverKey<G1Projective>,
    polynomial: &Multilinear<Fr>,
    blinding: Fr,
) -> IpaIntegratedCommitmentWitness<Fr> {
    commit_ipa_backend(prover_key, polynomial, blinding).unwrap()
}

#[test]
fn randomized_integrated_ipa_roundtrips_verify() {
    let mut rng = StdRng::seed_from_u64(0x5EED_1A0A_D7A1_0001);

    for variables in 0..=5 {
        for case_offset in 0..8 {
            let case_index = variables * 100 + case_offset;
            let fixture = build_fixture(&mut rng, variables, case_index);

            let encoded = encode_ipa_integrated_opening(&fixture.opening).unwrap();
            let decoded = decode_ipa_integrated_opening::<Fr>(&encoded).unwrap();

            let decoded_fixture = RoundtripFixture {
                verifier_key: fixture.verifier_key.clone(),
                commitment: fixture.commitment.clone(),
                point: fixture.point.clone(),
                opening: decoded,
                expected_value: fixture.expected_value,
            };

            let verified_value = verify_fixture(&decoded_fixture);

            assert_eq!(verified_value, fixture.expected_value);
            assert_eq!(
                decoded_fixture.opening.claimed_value,
                fixture.expected_value
            );
            assert_eq!(decoded_fixture.opening.proof.rounds.len(), variables + 1);
            assert!(!encoded.is_empty());
            assert!(!decoded_fixture.commitment.commitment_bytes.is_empty());
        }
    }
}

#[test]
fn randomized_integrated_ipa_rejects_tampered_final_scalar() {
    let mut rng = StdRng::seed_from_u64(0x0BAD_5CA1_AAAA_0002);

    for variables in 1..=5 {
        let mut fixture = build_fixture(&mut rng, variables, variables + 1_000);
        fixture.opening.proof.final_polynomial_scalar += random_nonzero_field(&mut rng);

        let mut verifier_transcript = MerlinTranscript::new(ROUNDTRIP_TRANSCRIPT_LABEL);

        assert!(verify_ipa_backend(
            &fixture.verifier_key,
            &fixture.commitment,
            &fixture.point,
            &fixture.opening,
            &mut verifier_transcript,
        )
        .is_err());
    }
}

#[test]
fn randomized_integrated_ipa_rejects_wrong_point() {
    let mut rng = StdRng::seed_from_u64(0x0BAD_901A_AAAA_0003);

    for variables in 1..=5 {
        let mut fixture = build_fixture(&mut rng, variables, variables + 2_000);
        fixture.point[0] += random_nonzero_field(&mut rng);

        let mut verifier_transcript = MerlinTranscript::new(ROUNDTRIP_TRANSCRIPT_LABEL);

        assert!(verify_ipa_backend(
            &fixture.verifier_key,
            &fixture.commitment,
            &fixture.point,
            &fixture.opening,
            &mut verifier_transcript,
        )
        .is_err());
    }
}

#[test]
fn randomized_integrated_ipa_rejects_wrong_commitment() {
    let mut rng = StdRng::seed_from_u64(0x0BAD_C0AA_17AA_0004);

    for variables in 1..=5 {
        let honest_polynomial = random_polynomial(&mut rng, variables);
        let other_polynomial = random_polynomial(&mut rng, variables);
        let point = random_opening_point(&mut rng, variables);
        let blinding = random_nonzero_field(&mut rng);

        let (prover_key, verifier_key) = test_keys(variables, variables + 3_000);
        let honest_witness = commit_with_key(&prover_key, &honest_polynomial, blinding);
        let other_witness = commit_with_key(&prover_key, &other_polynomial, blinding);

        let mut prover_transcript = MerlinTranscript::new(ROUNDTRIP_TRANSCRIPT_LABEL);
        let opening = open_ipa_backend(
            &prover_key,
            &honest_witness,
            &honest_polynomial,
            &point,
            &mut prover_transcript,
        )
        .unwrap();

        let mut verifier_transcript = MerlinTranscript::new(ROUNDTRIP_TRANSCRIPT_LABEL);

        assert!(verify_ipa_backend(
            &verifier_key,
            &other_witness.commitment,
            &point,
            &opening,
            &mut verifier_transcript,
        )
        .is_err());
    }
}

#[test]
fn randomized_integrated_ipa_rejects_wrong_transcript_label() {
    let mut rng = StdRng::seed_from_u64(0x0BAD_7AAA_5C21_0005);

    for variables in 1..=5 {
        let fixture = build_fixture(&mut rng, variables, variables + 4_000);
        let mut verifier_transcript = MerlinTranscript::new(b"ipa-randomized-wrong-label/v1");

        assert!(verify_ipa_backend(
            &fixture.verifier_key,
            &fixture.commitment,
            &fixture.point,
            &fixture.opening,
            &mut verifier_transcript,
        )
        .is_err());
    }
}
