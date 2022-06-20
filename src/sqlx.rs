use rocket::{form::*, get, post, response::Redirect, routes, State};
use rocket_auth::{Auth, Error, Login, Signup, User, Users};
use rocket_dyn_templates::Template;
use serde_json::json;
use sqlx::*;

use std::result::Result;
use std::*;

use rocket::fairing::{self, AdHoc};
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{futures, Build, Rocket};

use rocket_db_pools::{sqlx, Connection, Database};

use futures::{future::TryFutureExt, stream::TryStreamExt};

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::SqlitePool);

type MyResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[get("/login")]
fn get_login() -> Template {
    Template::render("login", json!({}))
}

#[post("/login", data = "<form>")]
async fn post_login(auth: Auth<'_>, form: Form<Login>) -> Result<Redirect, Error> {
    let result = auth.login(&form).await;
    println!("login attempt: {:?}", result);
    result?;
    Ok(Redirect::to("/"))
}

#[get("/signup")]
async fn get_signup() -> Template {
    Template::render("signup", json!({}))
}

#[post("/signup", data = "<form>")]
async fn post_signup(auth: Auth<'_>, form: Form<Signup>) -> Result<Redirect, Error> {
    auth.signup(&form).await?;
    auth.login(&form.into()).await?;

    Ok(Redirect::to("/"))
}

#[get("/")]
async fn index(user: Option<User>) -> Template {
    Template::render("index", json!({ "user": user }))
}

#[get("/logout")]
fn logout(auth: Auth<'_>) -> Result<Template, Error> {
    auth.logout()?;
    Ok(Template::render("logout", json!({})))
}

#[get("/delete")]
async fn delete_auth(auth: Auth<'_>) -> Result<Template, Error> {
    auth.delete().await?;
    Ok(Template::render("deleted", json!({})))
}

#[get("/show_all_users")]
async fn show_all_users(mut db: Connection<Db>, user: Option<User>) -> Result<Template, Error> {
    let users: Vec<User> = query_as("select * from users;").fetch_all(&mut *db).await?;
    println!("{:?}", users);
    Ok(Template::render(
        "users",
        json!({"users": users, "user": user}),
    ))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Post {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    title: String,
    text: String,
}

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

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/sqlx/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

async fn create_users_table(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => {
            let users: Users = db.0.clone().into();
            match users.create_table().await {
                Ok(_) => Ok(rocket.manage(users)),
                Err(e) => {
                    error!("Failed to create users table: {}", e);
                    Err(rocket)
                }
            }
        }
        None => Err(rocket),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(Db::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .attach(AdHoc::try_on_ignite(
                "SQLx Create Users table",
                create_users_table,
            ))
            .mount(
                "/",
                routes![
                    index,
                    get_login,
                    post_signup,
                    get_signup,
                    post_login,
                    logout,
                    delete_auth,
                    show_all_users,
                ],
            )
            .mount("/sqlx", routes![list, create, read, delete, destroy])
            .attach(Template::fairing())
    })
}
