use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{RocketAuthUser, Withdrawal};
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
    withdrawal: Withdrawal,
    withdrawal_user: RocketAuthUser,
    user: User,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        withdrawal_id: &str,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let withdrawal = Withdrawal::single_by_public_id(&mut db, withdrawal_id)
            .await
            .map_err(|_| "failed to get withdrawal.")?;
        let withdrawal_user = RocketAuthUser::single(&mut db, withdrawal.user_id)
            .await
            .map_err(|_| "failed to get withdrawal user.")?;
        Ok(Context {
            base_context,
            flash,
            withdrawal,
            withdrawal_user,
            user,
        })
    }
}

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    let context = match Context::raw(db, id, flash, active_user.user, admin_user).await {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("{}", e);
            panic!("failed to get context.")
        }
    };
    Template::render("withdrawal", context)
}

pub fn withdrawal_stage() -> AdHoc {
    AdHoc::on_ignite("Withdrawal Stage", |rocket| async {
        rocket.mount("/withdrawal", routes![index])
    })
}
