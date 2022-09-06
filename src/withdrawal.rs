use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{RocketAuthUser, Withdrawal};
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
    withdrawal: Withdrawal,
    maybe_withdrawal_user: Option<RocketAuthUser>,
    user: Option<User>,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        withdrawal_id: &str,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let withdrawal = Withdrawal::single_by_public_id(&mut db, withdrawal_id)
            .await
            .map_err(|_| "failed to get withdrawal.")?;
        let maybe_withdrawal_user = RocketAuthUser::single(&mut db, withdrawal.user_id)
            .await
            .ok();
        Ok(Context {
            base_context,
            flash,
            withdrawal,
            maybe_withdrawal_user,
            user,
        })
    }
}

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, id, flash, user, admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("withdrawal", context))
}

pub fn withdrawal_stage() -> AdHoc {
    AdHoc::on_ignite("Withdrawal Stage", |rocket| async {
        rocket.mount("/withdrawal", routes![index])
    })
}
