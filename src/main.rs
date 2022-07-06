#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod about;
mod account;
mod admin;
mod auth;
mod base;
mod config;
mod db;
mod lightning;
mod listing;
mod listings;
mod market_liabilities;
mod models;
mod my_account_balance;
mod my_approved_listings;
mod my_completed_orders;
mod my_new_orders;
mod my_pending_listings;
mod my_received_orders;
mod my_refunded_orders;
mod my_rejected_listings;
mod my_unpaid_orders;
mod my_unread_messages;
mod my_unsubmitted_listings;
mod new_listing;
mod order;
mod payment_processor;
mod posts;
mod prepare_order;
mod review_pending_listings;
mod routes;
mod seller_history;
mod top_sellers;
mod update_fee_rate;
mod update_listing_images;
mod update_market_name;
mod update_shipping_options;
mod user;
mod withdraw;
mod withdrawal;

#[launch]
fn rocket() -> _ {
    let config_figment = config::Config::get_config();
    let config: config::Config = config_figment.extract().unwrap();
    println!("lnd_host: {:?}", config.lnd_host);
    println!("lnd_port: {:?}", config.lnd_port);
    println!("lnd_tls_cert_path: {:?}", config.lnd_tls_cert_path);
    println!("lnd_macaroon_path: {:?}", config.lnd_macaroon_path);

    // let mut lnd_client = tonic_lnd::connect("", "", "")
    //     .await
    //     .expect("failed to connect");

    rocket::build().attach(routes::stage(config))
}
