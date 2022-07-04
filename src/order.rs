use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Listing, Order, OrderMessageInput, ShippingOption};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::response::Redirect;
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
    admin_user: Option<AdminUser>,
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
            admin_user,
        })
    }
}

#[post("/<id>/new_message", data = "<order_message_form>")]
async fn new_message(
    id: &str,
    order_message_form: Form<OrderMessageInput>,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Template> {
    let order_message_info = order_message_form.into_inner();

    println!("order_message_info: {:?}", order_message_info);

    // match create_listing(listing_info, &mut db, user.clone()).await {
    //     Ok(listing_id) => Ok(Flash::success(
    //         Redirect::to(format!("/{}/{}", "listing", listing_id)),
    //         "Listing successfully added.",
    //     )),
    //     Err(e) => {
    //         error_!("DB insertion error: {}", e);
    //         Err(Template::render(
    //             "newlisting",
    //             Context::err(db, e, Some(user), admin_user).await,
    //         ))
    //     }
    // }

    Ok(Flash::success(
        Redirect::to(format!("/{}/{}", "order", id)),
        "Order Message Successfully Sent.",
    ))
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
        rocket.mount("/order", routes![index, new_message])
    })
}
