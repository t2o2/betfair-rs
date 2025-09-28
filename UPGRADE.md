# Upgrade Guide: v0.2.0 - reqwest 0.12

## Breaking Changes

### Certificate Format Change

**Old (v0.1.x):**
- Used PKCS#12 (.pfx) format
- Required both `pfx_path` and `pfx_password` in config

**New (v0.2.0):**
- Uses PEM format (combined certificate + private key)
- Only requires `pem_path` in config

### Configuration Updates

Update your `config.toml`:

```toml
# Old format
[betfair]
username = "your_username"
password = "your_password"
api_key = "your_api_key"
pfx_path = "/path/to/client.pfx"
pfx_password = "certificate_password"

# New format
[betfair]
username = "your_username"
password = "your_password"
api_key = "your_api_key"
pem_path = "/path/to/client.pem"
```

### Converting Certificates

If you have an existing PKCS#12 (.pfx) file, convert it to PEM:

```bash
openssl pkcs12 -in client.pfx -out client.pem -nodes
```

Or if you have separate certificate and key files:

```bash
cat client.crt client.key > client.pem
```

## Dependency Updates

- **reqwest**: 0.9 → 0.12
- **rustls**: Updated to 0.23
- **tokio-rustls**: 0.10 → 0.26
- Removed: `tokio-native-tls`, `webpki`, `pkcs8`, `pem`, `tokio-socks`
- Added: `rustls-pki-types`, `webpki-roots`

## Benefits

- **Security**: Modern TLS implementation with latest security patches
- **Performance**: Improved async/await support in reqwest 0.12
- **Simplicity**: PEM format is more portable and easier to manage
- **Maintenance**: Active upstream maintenance and bug fixes