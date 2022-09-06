use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Order, OrderCard, RocketAuthUser};
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

const PAGE_SIZE: u32 = 10;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    visited_user: RocketAuthUser,
    amount_sold_sat: u64,
    weighted_average_rating: f32,
    order_cards: Vec<OrderCard>,
    page_num: u32,
}

impl Context {
    pub async fn raw(
        flash: Option<(String, String)>,
        mut db: Connection<Db>,
        maybe_page_num: Option<u32>,
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
        let page_num = maybe_page_num.unwrap_or(1);
        let order_cards = OrderCard::all_received_for_user(
            &mut db,
            visited_user.id.unwrap(),
            PAGE_SIZE,
            page_num,
        )
        .await
        .map_err(|_| "failed to get received orders for user.")?;
        let seller_info = Order::seller_info_for_user(&mut db, visited_user.id.unwrap())
            .await
            .map_err(|_| "failed to get weighted average rating for user.")?;
        let weighted_average_rating = seller_info.weighted_average_rating;
        let amount_sold_sat = seller_info.total_amount_sold_sat;
        Ok(Context {
            base_context,
            flash,
            visited_user,
            amount_sold_sat,
            weighted_average_rating,
            order_cards,
            page_num,
        })
    }
}

#[get("/<username>?<page_num>")]
async fn index(
    username: &str,
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    page_num: Option<u32>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(flash, db, page_num, username, user, admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("sellerhistory", context))
}

pub fn seller_history_stage() -> AdHoc {
    AdHoc::on_ignite("Seller History Stage", |rocket| async {
        rocket.mount("/seller_history", routes![index])
        // .mount("/listing", routes![new])
    })
}
