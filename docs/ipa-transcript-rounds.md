# IPA Transcript Rounds

This document defines the Fiat-Shamir transcript schedule for future IPA opening proofs.

## What this adds

The oracle crate now exposes:

    IpaTranscriptRound
    IpaTranscriptError
    IpaRoundSide
    bind_ipa_opening_statement
    absorb_ipa_reduction_round
    expected_ipa_rounds
    validate_ipa_round_count

## Production boundary

This is not yet group arithmetic.

The purpose is to fix the transcript schedule before implementing the actual IPA prover and verifier.

## Bound statement

The opening statement binds:

    field modulus
    variable count
    commitment bytes
    opening point length
    opening point coordinates

## Bound round message

Each reduction round binds:

    round index
    left commitment bytes
    right commitment bytes

and derives:

    IPA round challenge

## Security rule

Do not add fake IPA verification.

The transcript schedule is only one part of the future verifier. A real backend must also verify the group relations for every reduction round.
