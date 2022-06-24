#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod add_listing_images;
mod add_shipping_options;
mod admin;
mod auth;
mod db;
mod listing;
mod listings;
mod models;
mod new_listing;
mod posts;
mod routes;
mod user;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(routes::stage())
}
