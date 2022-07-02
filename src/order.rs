use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Listing, Order, ShippingOption};
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
    order: Order,
    listing: Listing,
    shipping_option: ShippingOption,
    user: User,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        order_id: &str,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let order = Order::single_by_public_id(&mut db, order_id)
            .await
            .map_err(|_| "failed to get order.")?;
        let listing = Listing::single(&mut db, order.listing_id)
            .await
            .map_err(|_| "failed to get listing.")?;
        let shipping_option = ShippingOption::single(&mut db, order.shipping_option_id)
            .await
            .map_err(|_| "failed to get shipping option.")?;
        println!("found order: {:?}", order);
        Ok(Context {
            base_context,
            flash,
            order,
            listing,
            shipping_option,
            user,
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
