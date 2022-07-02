use tonic_lnd::rpc::lightning_client::LightningClient;
use tonic_lnd::tonic::codegen::InterceptedService;
use tonic_lnd::tonic::transport::Channel;
use tonic_lnd::MacaroonInterceptor;

pub async fn get_lnd_client(
    lnd_host: String,
    lnd_port: u32,
    lnd_tls_cert_path: String,
    lnd_macaroon_path: String,
) -> Result<LightningClient<InterceptedService<Channel, MacaroonInterceptor>>, String> {
    let lnd_address = format!("http://{}:{}", lnd_host, lnd_port);
    println!("lnd_address: {:?}", lnd_address);
    let client = tonic_lnd::connect(lnd_address, lnd_tls_cert_path, lnd_macaroon_path)
        .await
        .map_err(|_| "failed to get LND client.")?;
    Ok(client)
}
