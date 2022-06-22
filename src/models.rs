use crate::db::Db;
use crate::rocket::futures::TryFutureExt;
use crate::rocket::futures::TryStreamExt;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::{sqlx, Connection};
use sqlx::Acquire;
use std::result::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Post {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub title: String,
    pub text: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Task {
    pub id: Option<i32>,
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, FromForm)]
pub struct Todo {
    pub description: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Listing {
    pub id: Option<i32>,
    pub user_id: i32,
    pub title: String,
    pub description: String,
    pub price_msat: u64,
    pub completed: bool,
    pub approved: bool,
    pub created_time_s: u64,
}

#[derive(Debug, FromForm)]
pub struct InitialListingInfo {
    pub title: String,
    pub description: String,
    pub price_msat: u64,
}

// #[derive(Debug, FromForm)]
// pub struct InitialListingInfo {
//     pub title: String,
//     pub description: String,
//     pub price_msat: u64,
//     pub created_time_s: u64,
// }

impl Task {
    pub async fn all(mut db: Connection<Db>) -> Result<Vec<Task>, sqlx::Error> {
        let tasks = sqlx::query!("select * from tasks;")
            .fetch(&mut *db)
            .map_ok(|r| Task {
                id: Some(r.id.try_into().unwrap()),
                description: r.description,
                completed: r.completed,
            })
            .try_collect::<Vec<_>>()
            .await?;

        println!("{}", tasks.len());
        println!("{:?}", tasks);

        Ok(tasks)
    }

    /// Returns the number of affected rows: 1.
    pub async fn insert(todo: Todo, mut db: Connection<Db>) -> Result<usize, sqlx::Error> {
        let insert_result = sqlx::query!(
            "INSERT INTO tasks (description, completed) VALUES (?, ?)",
            todo.description,
            false,
        )
        .execute(&mut *db)
        .await?;

        println!("{:?}", insert_result);

        Ok(insert_result.rows_affected() as _)
    }

    /// Returns the number of affected rows: 1.
    pub async fn toggle_with_id(id: i32, db: &mut Connection<Db>) -> Result<usize, sqlx::Error> {
        let mut tx = db.begin().await?;

        let get_task_completed = sqlx::query!("select * from tasks WHERE id = ?;", id)
            .fetch_one(&mut tx)
            .map_ok(|r| r.completed)
            // .try_collect::<bool>()
            .await?;

        let new_completed = !get_task_completed;

        let update_result = sqlx::query!(
            "UPDATE tasks SET completed = ? WHERE id = ?",
            // !task.completed,
            new_completed,
            id,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        println!("{:?}", update_result);

        Ok(update_result.rows_affected() as _)
    }

    /// Returns the number of affected rows: 1.
    pub async fn delete_with_id(id: i32, db: &mut Connection<Db>) -> Result<usize, sqlx::Error> {
        let delete_result = sqlx::query!("DELETE FROM tasks WHERE id = ?", id)
            .execute(&mut **db)
            .await?;

        Ok(delete_result.rows_affected() as _)
    }

    // /// Returns the number of affected rows.
    // #[cfg(test)]
    // pub async fn delete_all(conn: &DbConn) -> QueryResult<usize> {
    //     conn.run(|c| diesel::delete(all_tasks).execute(c)).await
    // }
}

impl Listing {
    pub async fn all(mut db: Connection<Db>) -> Result<Vec<Listing>, sqlx::Error> {
        let listings = sqlx::query!("select * from listings;")
            .fetch(&mut *db)
            .map_ok(|r| Listing {
                id: Some(r.id.try_into().unwrap()),
                user_id: r.user_id as _,
                title: r.title,
                description: r.description,
                price_msat: r.price_msat as _,
                completed: r.completed,
                approved: r.approved,
                created_time_s: r.created_time_s as _,
            })
            .try_collect::<Vec<_>>()
            .await?;

        println!("{}", listings.len());
        println!("{:?}", listings);

        Ok(listings)
    }

    /// Returns the number of affected rows: 1.
    pub async fn insert(listing: Listing, mut db: Connection<Db>) -> Result<usize, sqlx::Error> {
        let price_msat: i64 = listing.price_msat as _;
        let created_time_s: i64 = listing.created_time_s as _;

        let insert_result = sqlx::query!(
            "INSERT INTO listings (user_id, title, description, price_msat, completed, approved, created_time_s) VALUES (?, ?, ?, ?, ?, ?, ?)",
            listing.user_id,
            listing.title,
            listing.description,
            price_msat,
            listing.completed,
            listing.approved,
            created_time_s,
        )
        .execute(&mut *db)
        .await?;

        println!("{:?}", insert_result);

        Ok(insert_result.rows_affected() as _)
    }
}
