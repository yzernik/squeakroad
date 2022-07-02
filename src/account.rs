use crate::db::Db;
use crate::models::{AccountInfo, AdminSettings, Order};
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
    user: User,
    amount_earned: u64,
    amount_refunded: u64,
    admin_user: Option<AdminUser>,
    account_info: AccountInfo,
    admin_settings: Option<AdminSettings>,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let amount_earned = Order::amount_earned_sat(&mut db, user.id())
            .await
            .map_err(|_| "failed to get amount earned.")?;
        let amount_refunded = Order::amount_refunded_sat(&mut db, user.id())
            .await
            .map_err(|_| "failed to get amount refunded.")?;
        let account_info = AccountInfo::account_info_for_user(&mut db, user.id())
            .await
            .map_err(|_| "failed to get account info.")?;
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;
        Ok(Context {
            flash,
            user,
            amount_earned,
            amount_refunded,
            account_info,
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
    admin_user: Option<AdminUser>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("account", Context::raw(db, flash, user, admin_user).await)
}

pub fn account_stage() -> AdHoc {
    AdHoc::on_ignite("Account Stage", |rocket| async {
        rocket.mount("/account", routes![index])
        // .mount("/listing", routes![new])
    })
}
