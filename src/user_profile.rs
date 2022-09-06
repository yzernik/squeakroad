use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{RocketAuthUser, UserSettings};
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::serde::Serialize;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    visited_user: RocketAuthUser,
    visited_user_settings: UserSettings,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        username: String,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let visited_user = RocketAuthUser::single_by_username(&mut db, username)
            .await
            .map_err(|_| "failed to get visited user.")?;
        let visited_user_settings = UserSettings::single(&mut db, visited_user.id.unwrap())
            .await
            .map_err(|_| "failed to get visited user settings.")?;
        Ok(Context {
            base_context,
            flash,
            visited_user,
            visited_user_settings,
        })
    }
}

#[get("/<username>")]
async fn index(
    username: &str,
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, username.to_string(), flash, user, admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("userprofile", context))
}

pub fn user_profile_stage() -> AdHoc {
    AdHoc::on_ignite("User Profile Stage", |rocket| async {
        rocket.mount("/user_profile", routes![index])
    })
}
