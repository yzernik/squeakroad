#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod db;
mod sqlx;
mod task;
mod todo;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(sqlx::stage())
}
