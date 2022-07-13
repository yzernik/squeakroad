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
    .map_err(|e| format!("failed to get lightning client: {:?}", e))?;

    // Get latest paid invoice if exists.
    let latest_paid_order = Order::most_recent_paid_order(&mut conn)
        .await
        .map_err(|_| "failed to latest paid order.")?;

    let settle_index: u64 = if let Some(latest_invoice_hash) = latest_paid_order {
        let latest_paid_order_invoice = lighting_client
            .lookup_invoice(tonic_openssl_lnd::rpc::PaymentHash {
                r_hash: util::from_hex(&latest_invoice_hash),
                ..Default::default()
            })
            .await
            .map_err(|e| format!("Failed to lookup invoice: {:?}", e))?;
        latest_paid_order_invoice.into_inner().settle_index
    } else {
        0
    };

    println!("Starting subscribe invoices...");
    let invoice_subscription = tonic_openssl_lnd::rpc::InvoiceSubscription {
        settle_index,
        ..Default::default()
    };
    let update_stream_resp = lighting_client
        .subscribe_invoices(invoice_subscription)
        .await
        .map_err(|_| "Failed to call subscribe invoices.")?;
    let mut update_stream = update_stream_resp.into_inner();
    while let Ok(Some(invoice)) = update_stream.message().await {
        #[allow(deprecated)]
        if invoice.settled {
            println!("Handling settled invoice: {:?}", invoice);
            let invoice_hash = util::to_hex(&invoice.r_hash);
            handle_payment(&mut conn, &invoice_hash).await?;
        }
    }
    Ok(())
}

async fn handle_payment(
    conn: &mut PoolConnection<Sqlite>,
    invoice_hash: &str,
) -> Result<(), String> {
    let maybe_order = Order::single_by_invoice_hash(conn, invoice_hash).await.ok();
    if let Some(order) = maybe_order {
        let now = util::current_time_millis();
        Order::mark_as_paid(conn, order.id.unwrap(), now)
            .await
            .map_err(|_| "failed to mark order as paid.")?;
    }
    Ok(())
}
