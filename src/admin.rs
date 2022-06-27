use crate::db::Db;
use crate::models::AdminSettings;
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
    flash: Option<(String, String)>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
    admin_settings: Option<AdminSettings>,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;
        Ok(Context {
            flash,
            user,
            admin_user,
            admin_settings: Some(admin_settings),
        })
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: User,
    admin_user: AdminUser,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "admin",
        Context::raw(db, flash, Some(user), Some(admin_user)).await,
    )
}

pub fn admin_stage() -> AdHoc {
    AdHoc::on_ignite("Admin Stage", |rocket| async {
        rocket.mount("/admin", routes![index])
        // .mount("/listing", routes![new])
    })
}
