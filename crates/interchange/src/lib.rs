#![forbid(unsafe_code)]
//! Educational F_97 JSON interchange used by the browser visualizer.
//! The production-sized Rust protocol core lives in the Sumcheck, Zerocheck,
//! PermCheck, and transcript crates.

use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

pub const SCHEMA_VERSION: u32 = 1;
pub const EDUCATIONAL_MODULUS: u64 = 97;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Sumcheck,
    Zerocheck,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldSpec {
    pub modulus: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub num_variables: usize,
    pub claimed_sum: u64,
    pub oracle_evaluations: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mixing_point: Option<Vec<u64>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranscriptRound {
    pub round: usize,
    pub g_at_zero: u64,
    pub g_at_one: u64,
    pub challenge: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalCheck {
    pub point: Vec<u64>,
    pub oracle_evaluation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transcript {
    pub version: u32,
    pub protocol: Protocol,
    pub field: FieldSpec,
    pub claim: Claim,
    pub rounds: Vec<TranscriptRound>,
    #[serde(rename = "final")]
    pub final_check: FinalCheck,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TranscriptError {
    UnsupportedVersion(u32),
    UnsupportedModulus(u64),
    NonCanonicalFieldElement { path: String, value: u64 },
    InvalidEvaluationCount,
    VariableCountMismatch,
    InvalidMixingPoint,
    ZerocheckClaimMustBeZero,
    WrongRoundCount,
    WrongRoundIndex { position: usize, declared: usize },
    RoundDoesNotMatchClaim { round: usize },
    FinalPointMismatch,
    FinalEvaluationMismatch,
}

impl fmt::Display for TranscriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(value) => write!(f, "unsupported transcript version {value}"),
            Self::UnsupportedModulus(value) => write!(f, "unsupported educational modulus {value}"),
            Self::NonCanonicalFieldElement { path, value } => {
                write!(f, "non-canonical field element at {path}: {value}")
            }
            Self::InvalidEvaluationCount => {
                write!(f, "oracle evaluation count must be a non-zero power of two")
            }
            Self::VariableCountMismatch => write!(f, "variable count does not match oracle table"),
            Self::InvalidMixingPoint => write!(f, "invalid Zerocheck mixing point"),
            Self::ZerocheckClaimMustBeZero => write!(f, "Zerocheck claimed sum must be zero"),
            Self::WrongRoundCount => write!(f, "wrong round count"),
            Self::WrongRoundIndex { position, declared } => write!(
                f,
                "round index mismatch at position {position}: found {declared}"
            ),
            Self::RoundDoesNotMatchClaim { round } => {
                write!(f, "round {round} consistency or challenge check failed")
            }
            Self::FinalPointMismatch => write!(f, "final point does not match round challenges"),
            Self::FinalEvaluationMismatch => write!(f, "final oracle evaluation check failed"),
        }
    }
}

impl Error for TranscriptError {}

fn add(left: u64, right: u64) -> u64 {
    (left + right) % EDUCATIONAL_MODULUS
}
fn sub(left: u64, right: u64) -> u64 {
    (EDUCATIONAL_MODULUS + left - right) % EDUCATIONAL_MODULUS
}
fn mul(left: u64, right: u64) -> u64 {
    (left * right) % EDUCATIONAL_MODULUS
}
fn fold(left: u64, right: u64, challenge: u64) -> u64 {
    add(left, mul(challenge, sub(right, left)))
}

fn canonical(value: u64, path: impl Into<String>) -> Result<u64, TranscriptError> {
    if value >= EDUCATIONAL_MODULUS {
        Err(TranscriptError::NonCanonicalFieldElement {
            path: path.into(),
            value,
        })
    } else {
        Ok(value)
    }
}

fn equality_evaluations(point: &[u64]) -> Vec<u64> {
    let mut values = vec![1];
    for &coordinate in point {
        values = values
            .into_iter()
            .flat_map(|value| [mul(value, sub(1, coordinate)), mul(value, coordinate)])
            .collect();
    }
    values
}

fn evaluate(mut table: Vec<u64>, point: &[u64]) -> u64 {
    for &challenge in point {
        table = table
            .chunks_exact(2)
            .map(|pair| fold(pair[0], pair[1], challenge))
            .collect();
    }
    table[0]
}

fn challenge(round: usize, claim: u64, g0: u64, g1: u64) -> u64 {
    (claim * 17 + g0 * 31 + g1 * 43 + round as u64 * 13 + 7) % EDUCATIONAL_MODULUS
}

pub fn verify_transcript(transcript: &Transcript) -> Result<(), TranscriptError> {
    if transcript.version != SCHEMA_VERSION {
        return Err(TranscriptError::UnsupportedVersion(transcript.version));
    }
    if transcript.field.modulus != EDUCATIONAL_MODULUS {
        return Err(TranscriptError::UnsupportedModulus(
            transcript.field.modulus,
        ));
    }
    let length = transcript.claim.oracle_evaluations.len();
    if length == 0 || !length.is_power_of_two() {
        return Err(TranscriptError::InvalidEvaluationCount);
    }
    let variables = length.ilog2() as usize;
    if transcript.claim.num_variables != variables {
        return Err(TranscriptError::VariableCountMismatch);
    }
    if transcript.rounds.len() != variables {
        return Err(TranscriptError::WrongRoundCount);
    }

    let mut table = transcript
        .claim
        .oracle_evaluations
        .iter()
        .enumerate()
        .map(|(index, &value)| canonical(value, format!("claim.oracle_evaluations[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    table = match transcript.protocol {
        Protocol::Sumcheck => {
            if transcript.claim.mixing_point.is_some() {
                return Err(TranscriptError::InvalidMixingPoint);
            }
            table
        }
        Protocol::Zerocheck => {
            if transcript.claim.claimed_sum != 0 {
                return Err(TranscriptError::ZerocheckClaimMustBeZero);
            }
            let point = transcript
                .claim
                .mixing_point
                .as_ref()
                .ok_or(TranscriptError::InvalidMixingPoint)?;
            if point.len() != variables {
                return Err(TranscriptError::InvalidMixingPoint);
            }
            let point = point
                .iter()
                .enumerate()
                .map(|(index, &value)| canonical(value, format!("claim.mixing_point[{index}]")))
                .collect::<Result<Vec<_>, _>>()?;
            table
                .into_iter()
                .zip(equality_evaluations(&point))
                .map(|(value, equality)| mul(value, equality))
                .collect()
        }
    };

    let mut claim = canonical(transcript.claim.claimed_sum, "claim.claimed_sum")?;
    let mut point = Vec::with_capacity(variables);
    for (position, round) in transcript.rounds.iter().enumerate() {
        if round.round != position {
            return Err(TranscriptError::WrongRoundIndex {
                position,
                declared: round.round,
            });
        }
        let g0 = canonical(round.g_at_zero, format!("rounds[{position}].g_at_zero"))?;
        let g1 = canonical(round.g_at_one, format!("rounds[{position}].g_at_one"))?;
        let round_challenge = canonical(round.challenge, format!("rounds[{position}].challenge"))?;
        if add(g0, g1) != claim || round_challenge != challenge(position, claim, g0, g1) {
            return Err(TranscriptError::RoundDoesNotMatchClaim { round: position });
        }
        claim = fold(g0, g1, round_challenge);
        point.push(round_challenge);
    }
    if transcript.final_check.point != point {
        return Err(TranscriptError::FinalPointMismatch);
    }
    let final_evaluation = canonical(
        transcript.final_check.oracle_evaluation,
        "final.oracle_evaluation",
    )?;
    if claim != final_evaluation || evaluate(table, &point) != final_evaluation {
        return Err(TranscriptError::FinalEvaluationMismatch);
    }
    Ok(())
}

#[derive(Debug)]
pub enum ParseOrVerifyError {
    Json(serde_json::Error),
    Transcript(TranscriptError),
}
impl fmt::Display for ParseOrVerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(f, "invalid transcript JSON: {error}"),
            Self::Transcript(error) => error.fmt(f),
        }
    }
}
impl Error for ParseOrVerifyError {}

pub fn parse_and_verify(json: &str) -> Result<Transcript, ParseOrVerifyError> {
    let transcript = serde_json::from_str(json).map_err(ParseOrVerifyError::Json)?;
    verify_transcript(&transcript).map_err(ParseOrVerifyError::Transcript)?;
    Ok(transcript)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(name: &str) -> String {
        std::fs::read_to_string(format!(
            "{}/../../examples/transcripts/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap()
    }
    #[test]
    fn accepts_valid_fixtures() {
        parse_and_verify(&fixture("sumcheck-valid.json")).unwrap();
        parse_and_verify(&fixture("zerocheck-valid.json")).unwrap();
    }
    #[test]
    fn rejects_tampered_fixtures() {
        for name in [
            "sumcheck-bad-round.json",
            "sumcheck-bad-final-oracle.json",
            "zerocheck-violation.json",
        ] {
            assert!(parse_and_verify(&fixture(name)).is_err(), "{name} accepted");
        }
    }
}
