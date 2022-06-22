#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod auth;
mod db;
mod listings;
mod models;
mod new_listing;
mod posts;
mod routes;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(routes::stage())
}
