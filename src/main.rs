#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod auth;
mod db;
mod listings;
mod models;
mod posts;
mod routes;
mod todo;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(routes::stage())
}
