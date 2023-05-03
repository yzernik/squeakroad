use tonic_openssl_lnd::connect;
use tonic_openssl_lnd::LndClient;

pub async fn get_lnd_client(
    lnd_host: String,
    lnd_port: u32,
    lnd_tls_cert_path: String,
    lnd_macaroon_path: String,
) -> Result<LndClient, String> {
    // TODO: don't use unwrap.
    let client = connect(lnd_host, lnd_port, lnd_tls_cert_path, lnd_macaroon_path)
        .await
        .map_err(|e| format!("Failed to get lightning lnd client: {:?}", e))?;
    Ok(client)
}

pub async fn cancel_invoice(
    lnd_client: &mut LndClient,
    payment_hash: Vec<u8>,
) -> Result<(), String> {
    let maybe_invoice = lnd_client
        .invoices()
        .lookup_invoice_v2(tonic_openssl_lnd::invoicesrpc::LookupInvoiceMsg {
            invoice_ref: Some(
                tonic_openssl_lnd::invoicesrpc::lookup_invoice_msg::InvoiceRef::PaymentHash(
                    payment_hash.clone(),
                ),
            ),
            ..Default::default()
        })
        .await
        .ok();

    if maybe_invoice.is_none() {
        return Ok(());
    }

    lnd_client
        .invoices()
        .cancel_invoice(tonic_openssl_lnd::invoicesrpc::CancelInvoiceMsg { payment_hash })
        .await
        .map_err(|_| "failed to cancel invoice")?;

    Ok(())
}
