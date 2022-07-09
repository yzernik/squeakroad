use agora_lnd_client::get_client;
use agora_lnd_client::LndClient;

// pub async fn get_lnd_client(
//     lnd_host: String,
//     lnd_port: u32,
//     lnd_tls_cert_path: String,
//     lnd_macaroon_path: String,
// ) -> Result<LightningClient<InterceptedService<Channel, MacaroonInterceptor>>, String> {
//     let lnd_address = format!("http://{}:{}", lnd_host, lnd_port);
//     let client = tonic_lnd::connect(lnd_address, lnd_tls_cert_path, lnd_macaroon_path)
//         .await
//         .map_err(|e| format!("failed to get LND client: {:?}", e))?;
//     Ok(client)
// }

pub async fn get_lnd_client(
    lnd_host: String,
    lnd_port: u32,
    lnd_tls_cert_path: String,
    lnd_macaroon_path: String,
) -> Result<LndClient, String> {
    // let lnd_address = format!("http://{}:{}", lnd_host, lnd_port);

    // TODO: don't use unwrap.
    let client = get_client(lnd_host, lnd_port, lnd_tls_cert_path, lnd_macaroon_path)
        .await
        .unwrap();
    Ok(client)
}
