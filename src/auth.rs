use rocket::{form::*, get, post, response::Redirect, routes};
use rocket_auth::{Auth, Error, Login, Signup, User};
use rocket_dyn_templates::Template;
use serde_json::json;
use sqlx::*;

use std::result::Result;
use std::*;

use rocket::fairing::AdHoc;

use rocket_db_pools::{sqlx, Connection};

use crate::db::Db;

pub type MyResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[catch(401)]
fn not_authorized() -> Redirect {
    Redirect::to(uri!(get_login))
}

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

pub fn auth_stage() -> AdHoc {
    AdHoc::on_ignite("Auth Stage", |rocket| async {
        rocket.register("/", catchers![not_authorized]).mount(
            "/",
            routes![
                get_login,
                post_signup,
                get_signup,
                post_login,
                logout,
                delete_auth,
                show_all_users,
            ],
        )
    })
}
