# IPA SRS Loader

This branch adds canonical file encoding and fail-closed loading for IPA SRS material.

## Public API

The oracle crate exposes:

    encode_ipa_srs_file
    decode_ipa_srs_file
    read_ipa_srs_file
    IpaSrsFileError

## File format

The encoded file binds:

    format magic
    max variable count
    curve identifier
    SRS source metadata
    canonical basis digest
    polynomial generators
    evaluation generators
    blinding generator

All curve points are encoded using canonical compressed point bytes.

## Production rule

Loading returns only `IpaVerifiedSrs`.

That means decoded SRS material has already passed:

    curve-point decoding
    identity rejection
    duplicate rejection
    generator-count validation
    provenance validation
    canonical digest validation

## Boundary

This loader does not generate SRS material.

It only loads externally supplied or externally derived SRS material and validates it.
