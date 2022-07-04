use crate::config::Config;
use crate::lightning::get_lnd_client;
use crate::models::Order;
use sqlx::pool::PoolConnection;
use sqlx::Sqlite;

pub async fn handle_received_payments(
    config: Config,
    mut conn: PoolConnection<Sqlite>,
) -> Result<(), String> {
    let mut lighting_client = get_lnd_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .map_err(|_| "failed to get lightning client.")?;

    println!("Starting subscribe invoices...");
    let invoice_subscription = tonic_lnd::rpc::InvoiceSubscription {
        settle_index: 0,
        ..Default::default()
    };
    let update_stream_resp = lighting_client
        .subscribe_invoices(invoice_subscription)
        .await
        .map_err(|_| "Failed to call subscribe invoices.")?;
    let mut update_stream = update_stream_resp.into_inner();
    while let Ok(Some(invoice)) = update_stream.message().await {
        println!("Received invoice: {:?}", invoice);
        let invoice_hash = hex::encode(invoice.r_hash);
        println!("Invoice hash: {:?}", invoice_hash);

        Order::update_order_on_paid(&mut conn, &invoice_hash)
            .await
            .map_err(|_| "failed to update database with paid invoice.")?;
    }
    Ok(())
}
