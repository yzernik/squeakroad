use crate::base::BaseContext;
use crate::db::Db;
use crate::models::OrderCard;
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
    order_cards: Vec<OrderCard>,
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
        let order_cards = OrderCard::all_unpaid_for_user(&mut db, user.id)
            .await
            .map_err(|_| "failed to get unpaid orders.")?;
        Ok(Context {
            base_context,
            flash,
            order_cards,
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
        "myunpaidorders",
        Context::raw(flash, db, user, admin_user).await,
    ))
}

pub fn my_unpaid_orders_stage() -> AdHoc {
    AdHoc::on_ignite("My Unpaid Orders Stage", |rocket| async {
        rocket.mount("/my_unpaid_orders", routes![index])
        // .mount("/listing", routes![new])
    })
}
