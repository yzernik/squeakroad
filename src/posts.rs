use rocket::{get, post, routes};

use std::*;

use rocket::fairing::AdHoc;
use rocket::futures;
use rocket::response::status::Created;
use rocket::serde::json::Json;

use rocket_db_pools::{sqlx, Connection};

use futures::{future::TryFutureExt, stream::TryStreamExt};

use crate::auth::MyResult;
use crate::db::Db;
use crate::models::Post;

#[post("/", data = "<post>")]
async fn create(mut db: Connection<Db>, post: Json<Post>) -> MyResult<Created<Json<Post>>> {
    // There is no support for `RETURNING`.
    sqlx::query!(
        "INSERT INTO posts (title, text) VALUES (?, ?)",
        post.title,
        post.text
    )
    .execute(&mut *db)
    .await?;

    Ok(Created::new("/").body(post))
}

#[get("/")]
async fn list(mut db: Connection<Db>) -> MyResult<Json<Vec<i64>>> {
    let ids = sqlx::query!("SELECT id FROM posts")
        .fetch(&mut *db)
        .map_ok(|record| record.id)
        .try_collect::<Vec<_>>()
        .await?;

    Ok(Json(ids))
}

#[get("/<id>")]
async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<Post>> {
    sqlx::query!("SELECT id, title, text FROM posts WHERE id = ?", id)
        .fetch_one(&mut *db)
        .map_ok(|r| {
            Json(Post {
                id: Some(r.id),
                title: r.title,
                text: r.text,
            })
        })
        .await
        .ok()
}

#[delete("/<id>")]
async fn delete(mut db: Connection<Db>, id: i64) -> MyResult<Option<()>> {
    let result = sqlx::query!("DELETE FROM posts WHERE id = ?", id)
        .execute(&mut *db)
        .await?;

    Ok((result.rows_affected() == 1).then(|| ()))
}

#[delete("/")]
async fn destroy(mut db: Connection<Db>) -> MyResult<()> {
    sqlx::query!("DELETE FROM posts").execute(&mut *db).await?;

    Ok(())
}

pub fn posts_stage() -> AdHoc {
    AdHoc::on_ignite("Posts Stage", |rocket| async {
        rocket.mount("/posts", routes![list, create, read, delete, destroy])
    })
}
