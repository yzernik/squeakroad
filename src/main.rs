#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod about;
mod account;
mod account_activation;
mod activate_account;
mod active_users;
mod admin;
mod auth;
mod base;
mod config;
mod db;
mod deactivate_account;
mod deactivated_listings;
mod delete_listing;
mod disabled_users;
mod image_util;
mod lightning;
mod listing;
mod listings;
mod market_liabilities;
mod models;
mod my_account_balance;
mod my_active_listings;
mod my_deactivated_listings;
mod my_paid_orders;
mod my_pending_listings;
mod my_processing_orders;
mod my_rejected_listings;
mod my_unpaid_orders;
mod my_unsubmitted_listings;
mod new_listing;
mod order;
mod order_expiry;
mod payment_processor;
mod prepare_order;
mod review_pending_listings;
mod routes;
mod search;
mod seller_history;
mod top_sellers;
mod update_fee_rate;
mod update_listing_images;
mod update_market_name;
mod update_max_allowed_users;
mod update_pgp_info;
mod update_shipping_options;
mod update_squeaknode_info;
mod update_user_bond_price;
mod update_user_pgp_info;
mod update_user_squeaknode_info;
mod user;
mod user_account;
mod user_account_expiry;
mod user_profile;
mod util;
mod withdraw;
mod withdrawal;

#[launch]
fn rocket() -> _ {
    let config_figment = config::Config::get_config();
    let config: config::Config = config_figment.extract().unwrap();
    println!("Starting with config: {:?}", config);

    let figment = rocket::Config::figment().merge((
        "databases.squeakroad",
        rocket_db_pools::Config {
            url: config.clone().db_url,
            min_connections: None,
            max_connections: 1024,
            connect_timeout: 3,
            idle_timeout: None,
        },
    ));

    rocket::custom(figment).attach(routes::stage(config))
}
