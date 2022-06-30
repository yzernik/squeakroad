use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub key: String,
    pub lnd_host: String,
    pub lnd_port: u32,
    pub lnd_tls_cert_path: String,
    pub lnd_macaroon_path: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            key: "default".into(),
            lnd_host: "localhost".into(),
            lnd_port: 10009,
            lnd_tls_cert_path: "~/.lnd/tls.cert".into(),
            lnd_macaroon_path: "~/.lnd/data/chain/bitcoin/testnet/admin.macaroon".into(),
        }
    }
}

impl Config {
    pub fn get_config() -> Figment {
        Figment::from(Serialized::defaults(Config::default()))
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("SQUEAKROAD_"))
    }
}
