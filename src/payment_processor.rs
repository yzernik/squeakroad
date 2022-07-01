use crate::config::Config;
use crate::lightning::get_lnd_client;

pub async fn handle_received_payments(config: Config) -> Result<(), String> {
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

        // let listing_display = ListingDisplay::single_by_public_id(&mut db, listing_id)
        //     .await
        //     .map_err(|_| "failed to get admin settings.")?;
    }

    Ok(())
}
