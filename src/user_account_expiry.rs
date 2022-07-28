use crate::config::Config;
use crate::lightning::get_lnd_invoices_client;
use crate::models::UserAccount;
use crate::util;
use sqlx::pool::PoolConnection;
use sqlx::Sqlite;
use tonic_openssl_lnd::LndInvoicesClient;

const USER_ACCOUNT_EXPIRY_INTERVAL_MS: u64 = 600000; // 10 minutes

pub async fn remove_expired_user_accounts(
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

    // Delete all users without a corresponding user account.
    UserAccount::delete_users_with_no_account(&mut conn)
        .await
        .map_err(|_| "failed to delete users with no account.")?;

    // Get all unactivated user accounts older than expiry time limit.
    let now = util::current_time_millis();
    let expiry_cutoff = now - USER_ACCOUNT_EXPIRY_INTERVAL_MS;
    let expired_user_accounts = UserAccount::all_older_than(&mut conn, expiry_cutoff)
        .await
        .map_err(|_| "failed to get expired user accounts.")?;

    for user_account in expired_user_accounts {
        remove_user_account(&mut conn, &user_account, &mut lightning_invoices_client)
            .await
            .expect("failed to remove user account.");
    }
    Ok(())
}

async fn remove_user_account(
    conn: &mut PoolConnection<Sqlite>,
    user_account: &UserAccount,
    lightning_invoices_client: &mut LndInvoicesClient,
) -> Result<(), String> {
    println!("deleting expired user account: {:?}", user_account);
    let cancel_user_account_invoice_ret = cancel_user_account_invoice(
        lightning_invoices_client,
        util::from_hex(&user_account.invoice_hash),
    );
    UserAccount::delete_expired_user_account(
        conn,
        user_account.user_id,
        cancel_user_account_invoice_ret,
    )
    .await
    .expect("failed to delete expired user account.");

    Ok(())
}

async fn cancel_user_account_invoice(
    lightning_invoices_client: &mut LndInvoicesClient,
    payment_hash: Vec<u8>,
) -> Result<tonic_openssl_lnd::invoicesrpc::CancelInvoiceResp, String> {
    let cancel_response = lightning_invoices_client
        .cancel_invoice(tonic_openssl_lnd::invoicesrpc::CancelInvoiceMsg { payment_hash })
        .await
        .expect("failed to cancel invoice")
        .into_inner();
    Ok(cancel_response)
}
