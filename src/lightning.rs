use tonic_openssl_lnd::connect_invoices;
use tonic_openssl_lnd::connect_lightning;
use tonic_openssl_lnd::LndInvoicesClient;
use tonic_openssl_lnd::LndLightningClient;

pub async fn get_lnd_lightning_client(
    lnd_host: String,
    lnd_port: u32,
    lnd_tls_cert_path: String,
    lnd_macaroon_path: String,
) -> Result<LndLightningClient, String> {
    // TODO: don't use unwrap.
    let client = connect_lightning(lnd_host, lnd_port, lnd_tls_cert_path, lnd_macaroon_path)
        .await
        .unwrap();
    Ok(client)
}

pub async fn get_lnd_invoices_client(
    lnd_host: String,
    lnd_port: u32,
    lnd_tls_cert_path: String,
    lnd_macaroon_path: String,
) -> Result<LndInvoicesClient, String> {
    // TODO: don't use unwrap.
    let client = connect_invoices(lnd_host, lnd_port, lnd_tls_cert_path, lnd_macaroon_path)
        .await
        .unwrap();
    Ok(client)
}
