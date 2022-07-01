use crate::config::Config;
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

pub async fn handle_received_payments(config: Config) -> Result<(), String> {
    let mut lighting_client = get_lnd_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .expect("failed to get lightning client");

    println!("Starting subscribe invoices...");
    let invoice_subscription = tonic_lnd::rpc::InvoiceSubscription {
        settle_index: 0,
        ..Default::default()
    };
    let mut update_stream = lighting_client
        .subscribe_invoices(invoice_subscription)
        .await
        .expect("Failed to call subscribe invoices")
        .into_inner();
    while let Some(invoice) = update_stream
        .message()
        .await
        .expect("failed to receive update")
    {
        println!("Received invoice: {:?}", invoice);
    }

    Ok(())
}
