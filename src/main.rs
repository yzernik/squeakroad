#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod auth;
mod db;
mod posts;
mod routes;
mod task;
mod todo;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(routes::stage())
}
