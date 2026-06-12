# IPA Round Commitments

This document defines the concrete `L` and `R` commitments for one IPA reduction round.

## Formula

For vectors split into left and right halves:

    a = (a_L, a_R)
    b = (b_L, b_R)
    G = (G_L, G_R)
    H = (H_L, H_R)

The round commitments are:

    L = <a_L, G_R> + <b_R, H_L> + <a_L, b_R> U
    R = <a_R, G_L> + <b_L, H_R> + <a_R, b_L> U

where `U` is an explicit inner-product generator.

## What this adds

The oracle crate now exposes:

    IpaRoundCommitments
    IpaRoundCommitmentError
    compute_ipa_round_commitments

## Production boundary

This is a real algebraic component of an IPA opening proof.

It is still not a full verifier. The next steps are to integrate these commitments with the transcript challenge schedule, fold vectors and generators, and check the final scalar and commitment relation.

## Security rule

The inner-product generator must be independently generated or derived with proper domain separation.

Do not reuse the hiding/blinding generator as `U`.
