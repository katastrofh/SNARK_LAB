#![no_main]

use ark_bls12_381::Fr;
use libfuzzer_sys::fuzz_target;
use snark_lab_oracle::ipa_serialization::decode_ipa_opening_proof;

fuzz_target!(|data: &[u8]| {
    let _ = decode_ipa_opening_proof::<Fr>(data);
});
