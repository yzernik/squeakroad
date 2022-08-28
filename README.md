# squeakroad

[![GitHub release](https://img.shields.io/github/release/yzernik/squeakroad.svg)](https://github.com/yzernik/squeakroad/releases)
[![GitHub CI workflow](https://github.com/yzernik/squeakroad/actions/workflows/ci.yaml/badge.svg)](https://github.com/yzernik/squeakroad/actions/workflows/ci.yaml)

Open source darknet market with lightning network payments and withdrawals.

## Installation

### Requirements
* an LND node
* Rust and Cargo
* openssl `apt install libssl-dev`
* gexiv2 `apt install libgexiv2-dev`
* compiler dependencies `apt install libprotobuf-dev protobuf-compiler cmake`

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

`cargo install sqlx-cli`

`cargo sqlx migrate --source db/migrations add <YOUR_MIGRATION_NAME>`

Then put your SQL changes in the new file.

`cargo sqlx migrate --source db/migrations run`

After running migrations, generate the schema for compile-time type-checking:

`cargo sqlx prepare --database-url sqlite3://db.sqlite`

Optional: create a `.env` with `DATABASE_URL=sqlite3://db.sqlite` to avoid passing `--database-url`

## Telegram

[Join our Telegram group!](https://t.me/squeakroad)

## License

Distributed under the MIT License. See [LICENSE file](LICENSE).
