use ark_bls12_381::Fr;
use snark_lab_oracle::ipa_serialization::{decode_ipa_opening_proof, IpaProofCodecError};

const IPA_PROOF_DECODE_CAPACITY_OVERFLOW_20260612: &[u8] = &[
    83, 76, 45, 73, 80, 65, 45, 80, 82, 79, 79, 70, 49, 255, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255,
    255, 255, 255, 162, 162, 162, 162, 162, 162, 162, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 10,
];

#[test]
fn ipa_proof_decode_capacity_overflow_regression_returns_error_not_panic() {
    let result = std::panic::catch_unwind(|| {
        decode_ipa_opening_proof::<Fr>(IPA_PROOF_DECODE_CAPACITY_OVERFLOW_20260612)
    });

    assert!(
        result.is_ok(),
        "decoder must not panic on fuzz regression input"
    );

    match result.unwrap() {
        Err(IpaProofCodecError::LengthOverflow) => {}
        other => panic!("expected LengthOverflow, got {other:?}"),
    }
}
