# squeakroad

[![GitHub release](https://img.shields.io/github/release/yzernik/squeakroad.svg)](https://github.com/yzernik/squeakroad/releases)
[![GitHub CI workflow](https://github.com/yzernik/squeakroad/actions/workflows/ci.yaml/badge.svg)](https://github.com/yzernik/squeakroad/actions/workflows/ci.yaml)

## Installation

### Requirements
* an LND node
* Rust and Cargo

### Step 1. Create the configuration
> Create a **config.toml** file and fill in the relevant sections to connect to your LND node:

```
db_url="db.sqlite"
admin_username="admin"
admin_password="pass"
lnd_host="localhost"
lnd_port=10009
lnd_tls_cert_path="~/.lnd/tls.cert"
lnd_macaroon_path="~/.lnd/data/chain/bitcoin/mainnet/admin.macaroon"
```

### Step 2. Start squeakroad:

```
cargo run
```

Go to http://localhost:8000/ and use the username/password in **config.toml** to log in.

## Test

```
cargo test
```

## Database Migrations

Use [sqlx-cli](https://crates.io/crates/sqlx-cli/).

## Telegram

[Join our Telegram group!](https://t.me/squeakroad)

## License

Distributed under the MIT License. See [LICENSE file](LICENSE).
