use crate::db::Db;
use crate::models::{AccountInfo, AdminSettings, Order, OrderCard};
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
    flash: Option<(String, String)>,
    order_cards: Vec<OrderCard>,
    user: User,
    account_info: AccountInfo,
    admin_user: Option<AdminUser>,
    admin_settings: Option<AdminSettings>,
}

impl Context {
    pub async fn raw(
        flash: Option<(String, String)>,
        mut db: Connection<Db>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let order_cards = OrderCard::all_paid_for_user(&mut db, user.id)
            .await
            .map_err(|_| "failed to get paid orders.")?;
        let account_info = AccountInfo::account_info_for_user(&mut db, user.id())
            .await
            .map_err(|_| "failed to get account info.")?;
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;
        Ok(Context {
            flash,
            order_cards,
            user,
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
) -> Result<Template, NotFound<String>> {
    let flash = flash.map(FlashMessage::into_inner);
    Ok(Template::render(
        "mypaidorders",
        Context::raw(flash, db, user, admin_user).await,
    ))
}

pub fn my_paid_orders_stage() -> AdHoc {
    AdHoc::on_ignite("My Paid Orders Stage", |rocket| async {
        rocket.mount("/my_paid_orders", routes![index])
        // .mount("/listing", routes![new])
    })
}
