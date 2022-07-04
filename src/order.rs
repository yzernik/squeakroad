use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Listing, Order, OrderMessage, OrderMessageInput, ShippingOption};
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
    base_context: BaseContext,
    flash: Option<(String, String)>,
    order: Order,
    listing: Listing,
    shipping_option: ShippingOption,
    order_messages: Vec<OrderMessage>,
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
        let order_messages = OrderMessage::all_for_order(&mut db, order.id.unwrap())
            .await
            .map_err(|_| "failed to get order messages.")?;
        println!("found order: {:?}", order);
        Ok(Context {
            base_context,
            flash,
            order,
            listing,
            shipping_option,
            order_messages,
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
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let order_message_info = order_message_form.into_inner();

    println!("order_message_info: {:?}", order_message_info);

    match create_order_message(id, order_message_info, &mut db, user.clone()).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "order", id)),
            "Order Message Successfully Sent.",
        )),
        Err(e) => {
            error_!("DB insertion error: {}", e);
            Err(Flash::error(
                Redirect::to(format!("/{}/{}", "order", id)),
                e,
            ))
        }
    }
}

async fn create_order_message(
    order_id: &str,
    order_message_info: OrderMessageInput,
    db: &mut Connection<Db>,
    user: User,
) -> Result<(), String> {
    let order = Order::single_by_public_id(db, order_id)
        .await
        .map_err(|_| "failed to get order")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    println!("user.id(): {:?}", user.id());
    println!("order.seller_user_id: {:?}", order.seller_user_id);
    println!("order.buyer_user_id: {:?}", order.buyer_user_id);

    if user.id() != order.seller_user_id && user.id() != order.buyer_user_id {
        Err("User is not the seller or the buyer.".to_string())
    } else if order_message_info.text.is_empty() {
        Err("Message text cannot be empty.".to_string())
    } else if order_message_info.text.len() > 1024 {
        Err("Message text is too long.".to_string())
    } else {
        let recipient_id = if user.id() == order.seller_user_id {
            order.buyer_user_id
        } else {
            order.seller_user_id
        };
        let order_message = OrderMessage {
            id: None,
            public_id: Uuid::new_v4().to_string(),
            order_id: order.id.unwrap(),
            author_id: user.id(),
            recipient_id: recipient_id,
            text: order_message_info.text,
            viewed: false,
            created_time_ms: now,
        };
        match OrderMessage::insert(order_message, db).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error_!("DB insertion error: {}", e);
                Err("Order Message could not be inserted due an internal error.".to_string())
            }
        }
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
        rocket.mount("/order", routes![index, new_message])
    })
}
