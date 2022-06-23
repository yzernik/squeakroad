use rocket_db_pools::{sqlx, Database};

#[derive(Database)]
#[database("sqlite")]
pub struct Db(pub sqlx::SqlitePool);
