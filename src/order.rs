use crate::db::Db;
use crate::models::{AdminSettings, Listing, ListingDisplay, Order, OrderInfo, ShippingOption};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket::serde::uuid::Uuid;
use rocket::serde::Serialize;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    order: Order,
    listing: Listing,
    user: User,
    admin_user: Option<AdminUser>,
    admin_settings: Option<AdminSettings>,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        order_id: &str,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let order = Order::single_by_public_id(&mut db, order_id)
            .await
            .map_err(|_| "failed to get order.")?;
        let listing = Listing::single(&mut db, order.listing_id)
            .await
            .map_err(|_| "failed to get listing display.")?;
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;
        println!("found order: {:?}", order);

        Ok(Context {
            flash,
            order,
            listing,
            user,
            admin_user,
            admin_settings: Some(admin_settings),
        })
    }
}

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Template {
    println!("looking for order...");

    let flash = flash.map(FlashMessage::into_inner);
    let context = match Context::raw(db, id, flash, user, admin_user).await {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("{}", e);
            panic!("failed to get context.")
        }
    };
    Template::render("order", context)
}

pub fn order_stage() -> AdHoc {
    AdHoc::on_ignite("Order Stage", |rocket| async {
        rocket.mount("/order", routes![index])
    })
}
