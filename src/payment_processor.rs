use crate::config::Config;
use crate::lightning::get_lnd_client;
use crate::models::Order;
use crate::util;
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

    // Get latest paid invoice if exists.
    let latest_paid_order = Order::most_recent_paid_order(&mut conn)
        .await
        .map_err(|_| "failed to latest paid order.")?;

    let settle_index: u64 = if let Some(latest_invoice_hash) = latest_paid_order {
        let latest_paid_order_invoice = lighting_client
            .lookup_invoice(tonic_lnd::rpc::PaymentHash {
                r_hash: hex::decode(latest_invoice_hash).unwrap(),
                ..Default::default()
            })
            .await
            .map_err(|_| "Failed to lookup invoice.")?;
        latest_paid_order_invoice.into_inner().settle_index
    } else {
        0
    };

    println!("Starting subscribe invoices...");
    let invoice_subscription = tonic_lnd::rpc::InvoiceSubscription {
        settle_index,
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
        let now = util::current_time_millis();

        Order::update_order_on_paid(&mut conn, &invoice_hash, now)
            .await
            .map_err(|_| "failed to update database with paid invoice.")?;
    }
    Ok(())
}
