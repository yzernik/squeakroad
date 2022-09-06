use crate::base::BaseContext;
use crate::db::Db;
use crate::models::UserAccount;
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
    num_users: u64,
    user: Option<User>,
    admin_user: Option<AdminUser>,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let num_users = UserAccount::number_of_users(&mut db)
            .await
            .map_err(|_| "failed to get number of users.")?;
        Ok(Context {
            base_context,
            flash,
            num_users,
            user,
            admin_user,
        })
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: User,
    admin_user: AdminUser,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, flash, Some(user), Some(admin_user))
        .await
        .map_err(|_| "failed to get template context.")?;

    Ok(Template::render("admin", context))
}

pub fn admin_stage() -> AdHoc {
    AdHoc::on_ignite("Admin Stage", |rocket| async {
        rocket.mount("/admin", routes![index])
    })
}
