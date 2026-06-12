#![no_main]

use ark_bls12_381::G1Projective;
use libfuzzer_sys::fuzz_target;
use snark_lab_oracle::decode_ipa_srs_file;

fuzz_target!(|data: &[u8]| {
    let _ = decode_ipa_srs_file::<G1Projective>(data);
});
