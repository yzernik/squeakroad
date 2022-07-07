use rocket_db_pools::{sqlx, Database};

#[derive(Database)]
#[database("squeakroad")]
pub struct Db(pub sqlx::SqlitePool);
