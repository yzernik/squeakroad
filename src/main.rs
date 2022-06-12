#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod sqlx;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(sqlx::stage())
}
