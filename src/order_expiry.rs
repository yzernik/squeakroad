use crate::config::Config;
use crate::lightning::get_lnd_invoices_client;
use crate::models::Order;
use crate::util;
use sqlx::pool::PoolConnection;
use sqlx::Sqlite;
use tonic_openssl_lnd::LndInvoicesClient;

const ORDER_EXPIRY_INTERVAL_MS: u64 = 86400000;

pub async fn remove_expired_orders(
    config: Config,
    mut conn: PoolConnection<Sqlite>,
) -> Result<(), String> {
    let mut lightning_invoices_client = get_lnd_invoices_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .map_err(|e| format!("failed to get lightning client: {:?}", e))?;

    // Get all orders older than expiry time limit.
    let now = util::current_time_millis();
    let expiry_cutoff = now - ORDER_EXPIRY_INTERVAL_MS;
    let expired_orders = Order::all_older_than(&mut conn, expiry_cutoff)
        .await
        .map_err(|_| "failed to expired orders.")?;

    for order in expired_orders {
        remove_order(&mut conn, &order, &mut lightning_invoices_client)
            .await
            .ok();
    }
    Ok(())
}

async fn remove_order(
    conn: &mut PoolConnection<Sqlite>,
    order: &Order,
    lightning_invoices_client: &mut LndInvoicesClient,
) -> Result<(), String> {
    println!("deleting expired order: {:?}", order);
    lightning_invoices_client
        .cancel_invoice(tonic_openssl_lnd::invoicesrpc::CancelInvoiceMsg {
            payment_hash: util::from_hex(&order.invoice_hash),
        })
        .await
        .expect("failed to cancel invoice");

    Order::delete_expired_order(conn, order.id.unwrap())
        .await
        .ok();

    Ok(())
}
