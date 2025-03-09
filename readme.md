# Betfair in Rust

The goal of the project is to allow user to trade with betfair with the benefit of the speed of rust and a stable simple interface to trade.

## Setup

Need to convert the cert and key files that were uploaded to betfair to pkcs12 format

```bash
openssl pkcs12 -export -out client.pfx -inkey client.key -in client.crt
```

Configuration are expected to be set in a `config.toml` file containing the following.

```toml
[betfair]
username = ""
password = ""
api_key = ""
pfx_path = "[absolute path]"
pfx_password = ""
```

