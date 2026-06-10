//! Fiat–Shamir transcripts for non-interactive public-coin protocols.

use ark_ff::{BigInteger, PrimeField};

/// Transcript operations needed by the protocol crates.
pub trait ProofTranscript<F: PrimeField> {
    fn append_domain_separator(&mut self, label: &'static [u8]);
    fn append_field_element(&mut self, label: &'static [u8], element: &F);
    fn append_bytes(&mut self, label: &'static [u8], bytes: &[u8]);
    fn append_u64(&mut self, label: &'static [u8], value: u64);

    fn append_field_modulus(&mut self) {
        self.append_bytes(b"field-modulus", &F::MODULUS.to_bytes_le());
    }

    fn challenge_scalar(&mut self, label: &'static [u8]) -> F;

    fn challenge_vector(&mut self, label: &'static [u8], length: usize) -> Vec<F> {
        (0..length)
            .map(|index| {
                self.append_u64(b"challenge-index", index as u64);
                self.challenge_scalar(label)
            })
            .collect()
    }
}

/// Merlin/STROBE-backed Fiat–Shamir transcript with framed messages.
#[derive(Clone)]
pub struct MerlinTranscript {
    inner: merlin::Transcript,
}

impl MerlinTranscript {
    pub fn new(application_label: &'static [u8]) -> Self {
        Self {
            inner: merlin::Transcript::new(application_label),
        }
    }
}

impl<F: PrimeField> ProofTranscript<F> for MerlinTranscript {
    fn append_domain_separator(&mut self, label: &'static [u8]) {
        self.inner.append_message(b"dom-sep", label);
    }

    fn append_field_element(&mut self, label: &'static [u8], element: &F) {
        let bytes = element.into_bigint().to_bytes_le();
        self.inner.append_message(label, &bytes);
    }

    fn append_bytes(&mut self, label: &'static [u8], bytes: &[u8]) {
        self.inner.append_message(label, bytes);
    }

    fn append_u64(&mut self, label: &'static [u8], value: u64) {
        self.inner.append_u64(label, value);
    }

    fn challenge_scalar(&mut self, label: &'static [u8]) -> F {
        // A 512-bit XOF output gives negligible reduction bias for common SNARK fields.
        let mut bytes = [0_u8; 64];
        self.inner.challenge_bytes(label, &mut bytes);
        F::from_le_bytes_mod_order(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    fn challenge(message: &[u8]) -> Fr {
        let mut transcript = MerlinTranscript::new(b"snark-lab-test");
        <MerlinTranscript as ProofTranscript<Fr>>::append_domain_separator(
            &mut transcript,
            b"test-v1",
        );
        <MerlinTranscript as ProofTranscript<Fr>>::append_bytes(
            &mut transcript,
            b"message",
            message,
        );
        transcript.challenge_scalar(b"challenge")
    }

    #[test]
    fn same_transcript_has_same_challenge() {
        assert_eq!(challenge(b"statement"), challenge(b"statement"));
    }

    #[test]
    fn changed_message_changes_challenge() {
        assert_ne!(challenge(b"statement-a"), challenge(b"statement-b"));
    }
}
