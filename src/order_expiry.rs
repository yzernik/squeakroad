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
    let cancel_order_invoice_ret = cancel_order_invoice(
        lightning_invoices_client,
        util::from_hex(&order.invoice_hash),
    );
    Order::delete_expired_order(conn, order.id.unwrap(), cancel_order_invoice_ret)
        .await
        .ok();
    Ok(())
}

async fn cancel_order_invoice(
    lightning_invoices_client: &mut LndInvoicesClient,
    payment_hash: Vec<u8>,
) -> Result<tonic_openssl_lnd::invoicesrpc::CancelInvoiceResp, String> {
    let cancel_response = lightning_invoices_client
        .cancel_invoice(tonic_openssl_lnd::invoicesrpc::CancelInvoiceMsg {
            payment_hash: payment_hash,
        })
        .await
        .expect("failed to cancel invoice")
        .into_inner();
    Ok(cancel_response)
}
