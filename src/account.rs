use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AccountInfo, UserAccount};
use crate::user_account::ActiveUser;
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
    user: User,
    account_info: AccountInfo,
    user_account: UserAccount,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let account_info = AccountInfo::account_info_for_user(&mut db, user.id())
            .await
            .map_err(|_| "failed to get account info.")?;
        let user_account = UserAccount::single(&mut db, user.id())
            .await
            .map_err(|_| "failed to get user account.")?;
        Ok(Context {
            base_context,
            flash,
            user,
            account_info,
            user_account,
        })
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, flash, active_user.user, admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("account", context))
}

pub fn account_stage() -> AdHoc {
    AdHoc::on_ignite("Account Stage", |rocket| async {
        rocket.mount("/account", routes![index])
        // .mount("/listing", routes![new])
    })
}
