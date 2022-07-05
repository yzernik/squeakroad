use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Order, OrderCard, RocketAuthUser};
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
    visited_user: RocketAuthUser,
    amount_sold_sat: u64,
    amount_sold_with_reviews_sat: u64,
    received_orders: Vec<OrderCard>,
}

impl Context {
    pub async fn raw(
        flash: Option<(String, String)>,
        mut db: Connection<Db>,
        visited_user_username: &str,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let visited_user =
            RocketAuthUser::single_by_username(&mut db, visited_user_username.to_string())
                .await
                .map_err(|_| "failed to get visited user.")?;
        let received_orders = OrderCard::all_received_for_user(&mut db, visited_user.id.unwrap())
            .await
            .map_err(|_| "failed to get received orders for user.")?;
        let amount_sold_sat = Order::amount_sold_sat_for_user(&mut db, visited_user.id.unwrap())
            .await
            .map_err(|_| "failed to get amount sold for user.")?;
        let amount_sold_with_reviews_sat =
            Order::amount_sold_with_reviews_sat_for_user(&mut db, visited_user.id.unwrap())
                .await
                .map_err(|_| "failed to get amount sold with reviews for user.")?;
        Ok(Context {
            base_context,
            flash,
            visited_user,
            amount_sold_sat,
            amount_sold_with_reviews_sat,
            received_orders,
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
) -> Result<Template, NotFound<String>> {
    let flash = flash.map(FlashMessage::into_inner);
    Ok(Template::render(
        "sellerrating",
        Context::raw(flash, db, username, user, admin_user).await,
    ))
}

pub fn seller_rating_stage() -> AdHoc {
    AdHoc::on_ignite("Seller Rating Stage", |rocket| async {
        rocket.mount("/seller_rating", routes![index])
        // .mount("/listing", routes![new])
    })
}
