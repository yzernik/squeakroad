use rocket_db_pools::{sqlx, Database};

#[derive(Database)]
#[database("sqlx")]
pub struct Db(pub sqlx::SqlitePool);
