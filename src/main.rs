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
mod my_pending_listings;
mod my_unsubmitted_listings;
mod new_listing;
mod pending_listings;
mod posts;
mod routes;
mod update_listing_images;
mod update_market_name;
mod update_shipping_options;
mod user;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(routes::stage())
}
