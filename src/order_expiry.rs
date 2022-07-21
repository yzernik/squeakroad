use crate::config::Config;
use crate::lightning::get_lnd_client;
use crate::models::Order;
use crate::util;
use sqlx::pool::PoolConnection;
use sqlx::Sqlite;
use tonic_openssl_lnd::LndClient;

// const ORDER_EXPIRY_INTERVAL_MS: u64 = 86400000;
const ORDER_EXPIRY_INTERVAL_MS: u64 = 100;

pub async fn remove_expired_orders(
    config: Config,
    mut conn: PoolConnection<Sqlite>,
) -> Result<(), String> {
    let lightning_client = get_lnd_client(
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

    println!("Expired orders: {:?}", expired_orders);
    for order in expired_orders {
        println!("expired order: {:?}", order);
        remove_order(&mut conn, &order, &lightning_client);
    }
    Ok(())
}

async fn remove_order(
    conn: &mut PoolConnection<Sqlite>,
    order: &Order,
    lightning_client: &LndClient,
) -> Result<(), String> {
    let mut cancel_resp = lightning_client
        .cancel_invoice(tonic_openssl_lnd::rpc::CancelInvoiceMsg {
            payment_hash: util::from_hex(&order.invoice_hash),
            ..Default::default()
        })
        .await
        .expect("failed to cancel invoice")
        .into_inner();

    Ok(())
}
