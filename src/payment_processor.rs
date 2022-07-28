use crate::config::Config;
use crate::lightning::get_lnd_lightning_client;
use crate::models::{Order, UserAccount};
use crate::util;
use sqlx::pool::PoolConnection;
use sqlx::Sqlite;
use tonic_openssl_lnd::LndLightningClient;

pub async fn handle_received_payments(
    config: Config,
    mut conn: PoolConnection<Sqlite>,
) -> Result<(), String> {
    let mut lightning_client = get_lnd_lightning_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .map_err(|e| format!("failed to get lightning client: {:?}", e))?;

    let latest_settle_index = get_latest_settle_index(&mut conn, &mut lightning_client).await?;

    println!(
        "Starting subscribe invoices with latest settle index: {:?}",
        latest_settle_index
    );
    let invoice_subscription = tonic_openssl_lnd::lnrpc::InvoiceSubscription {
        settle_index: latest_settle_index,
        ..Default::default()
    };
    let mut update_stream = lightning_client
        .subscribe_invoices(invoice_subscription)
        .await
        .map_err(|_| "Failed to call subscribe invoices.")?
        .into_inner();
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
    let now = util::current_time_millis();

    let maybe_order = Order::single_by_invoice_hash(conn, invoice_hash).await.ok();
    if let Some(order) = maybe_order {
        Order::mark_as_paid(conn, order.id.unwrap(), now)
            .await
            .map_err(|_| "failed to mark order as paid.")?;
    }

    let maybe_user_account = UserAccount::single_by_invoice_hash(conn, invoice_hash)
        .await
        .ok();
    if let Some(user_account) = maybe_user_account {
        UserAccount::mark_as_paid(conn, user_account.id.unwrap(), now)
            .await
            .map_err(|_| "failed to mark user account as paid.")?;
    }

    Ok(())
}

async fn get_latest_settle_index(
    conn: &mut PoolConnection<Sqlite>,
    lightning_client: &mut LndLightningClient,
) -> Result<u64, String> {
    // Get latest paid invoice if exists.
    let latest_paid_order = Order::most_recent_paid_order(conn)
        .await
        .map_err(|_| "failed to latest paid order.")?;

    let settle_index: u64 = if let Some(latest_invoice_hash) = latest_paid_order {
        let latest_paid_order_invoice = lightning_client
            .lookup_invoice(tonic_openssl_lnd::lnrpc::PaymentHash {
                r_hash: util::from_hex(&latest_invoice_hash),
                ..Default::default()
            })
            .await
            .map_err(|e| format!("Failed to lookup invoice: {:?}", e))?
            .into_inner();
        latest_paid_order_invoice.settle_index
    } else {
        0
    };

    Ok(settle_index)
}
