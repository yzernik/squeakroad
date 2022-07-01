use crate::config::Config;
use crate::db::Db;
use crate::lightning::get_lnd_client;
use crate::models::Order;
use rocket_db_pools::Connection;
use sqlx::pool::PoolConnection;
use sqlx::Sqlite;

pub async fn handle_received_payments(config: Config, mut conn: PoolConnection<Sqlite>) -> () {
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
        let invoice_hash = hex::encode(invoice.r_hash);
        println!("Invoice hash: {:?}", invoice_hash);

        // let order = Order::single_by_invoice_hash(&mut conn, &invoice_hash)
        //     .await
        //     .expect("failed to make order query.");
        // println!("Order: {:?}", order);
        Order::update_order_on_paid(&mut conn, &invoice_hash).await;
    }
}
