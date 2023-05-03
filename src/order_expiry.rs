use crate::config::Config;
use crate::lightning::cancel_invoice;
use crate::lightning::get_lnd_client;
use crate::models::Payment;
use crate::models::UserAccount;
use crate::util;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use tonic_openssl_lnd::LndClient;

const PAYMENT_EXPIRY_INTERVAL_MS: u64 = 600000; // 10 minutes

pub async fn remove_expired_orders(
    config: Config,
    mut conn: PoolConnection<Postgres>,
) -> Result<(), String> {
    let mut lnd_client = get_lnd_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .map_err(|e| format!("failed to get lightning client: {:?}", e))?;

    // Get all orders older than expiry time limit.
    let now = util::current_time_millis();
    // let expiry_cutoff = now - ORDER_EXPIRY_INTERVAL_MS;
    let expired_payments = Payment::all_older_than(&mut conn, now - PAYMENT_EXPIRY_INTERVAL_MS)
        .await
        .map_err(|_| "failed to expired orders.")?;

    // Delete all users without a corresponding user account.
    UserAccount::delete_users_with_no_account(&mut conn)
        .await
        .map_err(|_| "failed to delete users with no account.")?;

    // Delete expired payments
    for payment in expired_payments {
        remove_payment(&mut conn, &payment, &mut lnd_client)
            .await
            .ok();
    }
    Ok(())
}

async fn remove_payment(
    conn: &mut PoolConnection<Postgres>,
    payment: &Payment,
    lnd_client: &mut LndClient,
) -> Result<(), String> {
    println!("deleting expired payment: {:?}", payment);
    let cancel_payment_invoice_ret =
        cancel_invoice(lnd_client, util::from_hex(&payment.invoice_hash));
    Payment::delete_expired(conn, payment.id.unwrap(), cancel_payment_invoice_ret)
        .await
        .map_err(|e| error!("e: {:?}", e))
        .ok();
    Ok(())
}
