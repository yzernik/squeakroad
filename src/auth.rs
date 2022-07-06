use crate::base::BaseContext;
use crate::db::Db;
use rocket::fairing::AdHoc;
use rocket::{form::*, get, post, response::Redirect, routes};
use rocket_auth::{Auth, Error, Login, Signup, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use serde_json::json;
use std::result::Result;

#[catch(401)]
fn not_authorized() -> Redirect {
    Redirect::to(uri!(get_login))
}

#[get("/login")]
async fn get_login(mut db: Connection<Db>, user: Option<User>) -> Result<Template, String> {
    let base_context = BaseContext::raw(&mut db, user.clone(), None)
        .await
        .map_err(|_| "failed to get base template.")?;
    Ok(Template::render(
        "login",
        json!({ "base_context": base_context }),
    ))
}

#[post("/login", data = "<form>")]
async fn post_login(auth: Auth<'_>, form: Form<Login>) -> Result<Redirect, String> {
    auth.login(&form).await.map_err(|_| "failed to login.")?;
    Ok(Redirect::to("/"))
}

#[get("/signup")]
async fn get_signup(mut db: Connection<Db>, user: Option<User>) -> Result<Template, String> {
    let base_context = BaseContext::raw(&mut db, user.clone(), None)
        .await
        .map_err(|_| "failed to get base template.")?;
    Ok(Template::render(
        "signup",
        json!({ "base_context": base_context }),
    ))
}

#[post("/signup", data = "<form>")]
async fn post_signup(auth: Auth<'_>, form: Form<Signup>) -> Result<Redirect, Error> {
    auth.signup(&form).await?;
    auth.login(&form.into()).await?;
    Ok(Redirect::to("/"))
}

#[get("/logout")]
async fn logout(auth: Auth<'_>, mut db: Connection<Db>) -> Result<Template, String> {
    auth.logout().map_err(|_| "failed to logout.")?;
    let base_context = BaseContext::raw(&mut db, None, None)
        .await
        .map_err(|_| "failed to get base template.")?;
    Ok(Template::render(
        "logout",
        json!({ "base_context": base_context }),
    ))
}

// #[get("/delete")]
// async fn delete_auth(auth: Auth<'_>) -> Result<Template, Error> {
//     auth.delete().await?;
//     Ok(Template::render("deleted", json!({})))
// }

// #[get("/show_all_users")]
// async fn show_all_users(mut db: Connection<Db>, user: Option<User>) -> Result<Template, Error> {
//     let users: Vec<User> = query_as("select * from users;").fetch_all(&mut *db).await?;
//     let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default()).await?;
//     println!("{:?}", users);
//     Ok(Template::render(
//         "users",
//         json!({"users": users, "user": user, "admin_settings": admin_settings}),
//     ))
// }

pub fn auth_stage() -> AdHoc {
    AdHoc::on_ignite("Auth Stage", |rocket| async {
        rocket.register("/", catchers![not_authorized]).mount(
            "/",
            routes![get_login, post_signup, get_signup, post_login, logout,],
        )
    })
}
