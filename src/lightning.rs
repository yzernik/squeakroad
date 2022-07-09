use squeakroad_lnd_client::get_client;
use squeakroad_lnd_client::LndClient;

pub async fn get_lnd_client(
    lnd_host: String,
    lnd_port: u32,
    lnd_tls_cert_path: String,
    lnd_macaroon_path: String,
) -> Result<LndClient, String> {
    // TODO: don't use unwrap.
    let client = get_client(lnd_host, lnd_port, lnd_tls_cert_path, lnd_macaroon_path)
        .await
        .unwrap();
    Ok(client)
}
