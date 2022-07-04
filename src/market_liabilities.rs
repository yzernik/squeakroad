use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AccountBalanceChange, AccountInfo};
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::status::NotFound;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    account_balance_sat: i64,
    account_balance_changes: Vec<AccountBalanceChange>,
}

impl Context {
    pub async fn raw(
        flash: Option<(String, String)>,
        mut db: Connection<Db>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let account_balance_changes = AccountInfo::all_account_balance_changes(&mut db)
            .await
            .map_err(|_| "failed to get account balance changes.")?;
        let account_balance_sat = 0;
        Ok(Context {
            base_context,
            flash,
            account_balance_sat,
            account_balance_changes,
        })
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Template, NotFound<String>> {
    let flash = flash.map(FlashMessage::into_inner);
    Ok(Template::render(
        "marketliabilities",
        Context::raw(flash, db, user, admin_user).await,
    ))
}

pub fn market_liabilities_stage() -> AdHoc {
    AdHoc::on_ignite("Market Liabilies Stage", |rocket| async {
        rocket.mount("/market_liabilities", routes![index])
        // .mount("/listing", routes![new])
    })
}
