# fleet-crypto

Certificates, tokens and secrets for Fleet.

- `generate_certs`: self-signed certificate and private key (PEM bytes) for the receiver
- `generate_secure_token` / `validate_token`: random token of exactly `TOKEN_BYTE_SIZE` bytes, and a check for it
- `Secret`: bytes that never show up in `Display` or `Debug` output
