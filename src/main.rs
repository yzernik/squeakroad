#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod account;
mod admin;
mod auth;
mod db;
mod listing;
mod listings;
mod models;
mod my_approved_listings;
mod my_pending_listings;
mod my_rejected_listings;
mod my_unsubmitted_listings;
mod new_listing;
mod order;
mod posts;
mod prepare_order;
mod review_pending_listings;
mod routes;
mod update_fee_rate;
mod update_listing_images;
mod update_market_name;
mod update_shipping_options;
mod user;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(routes::stage())
}
