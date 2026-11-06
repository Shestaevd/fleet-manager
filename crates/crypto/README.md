# fleet-crypto

Certificates and secrets for Fleet.

- `generate_certs`: self-signed certificate and private key (PEM bytes) for the receiver
- `Secret`: bytes that never show up in `Display` or `Debug` output
- `generate_secure_token` / `hash_token`: a random token of `TOKEN_BYTE_SIZE` bytes, and its SHA-256 hash of `TOKEN_HASH_BYTE_SIZE` bytes
- `TOKEN_HASH_BYTE_SIZE` / `validate_token_hash`: the fixed size of the token hash an agent sends with every packet, and a check for it
