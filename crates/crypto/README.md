# fleet-crypto

Certificates, secrets, and the token and password hashing used by Fleet.

- `generate_certs`: self-signed certificate and private key (PEM bytes) for the receiver
- `Secret`: bytes that never show up in `Display` or `Debug` output
- `generate_secure_token` / `hash_token`: a random token of `TOKEN_BYTE_SIZE` bytes, and its SHA-256 hash of `TOKEN_HASH_BYTE_SIZE` bytes
- `TOKEN_HASH_BYTE_SIZE` / `validate_token_hash`: the fixed size of the token hash an agent sends with every packet, and a check for it
- `hash_password` / `verify_password`: Argon2id with a random salt; the hash is a PHC string as bytes
- `constant_time_eq`: compares two byte strings in time that depends only on their length
