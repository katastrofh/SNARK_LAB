# IPA CLI Demo

The CLI now exposes:

    snark-lab-cli ipa-demo

The command executes a real integrated IPA flow:

    commit
    open
    encode public opening
    decode public opening
    verify

## Production boundary

This demo uses deterministic fixture generators and an explicit nonzero blinding scalar so the command is reproducible.

The backend path itself does not fake verification and does not serialize prover witness material.

## Run

    cargo run -p snark-lab-cli -- ipa-demo

Expected output includes:

    ipa-demo: verified blinded IPA opening
    variables=2
    commitment_bytes=...
    encoded_opening_bytes=...
    decoded_rounds=3
