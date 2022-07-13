use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{Listing, Order, ReviewInput, RocketAuthUser, ShippingOption};
use crate::util;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket::serde::Serialize;
use rocket::State;
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
    seller_user: RocketAuthUser,
    user: Option<User>,
    admin_user: Option<AdminUser>,
    qr_svg_base64: String,
    lightning_node_pubkey: String,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        order_id: &str,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
        config: &Config,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
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
        let seller_user = RocketAuthUser::single(&mut db, listing.user_id)
            .await
            .map_err(|_| "failed to get order messages.")?;
        let qr_svg_bytes = util::generate_qr(&order.invoice_payment_request);
        let qr_svg_base64 = util::to_base64(&qr_svg_bytes);
        let lightning_node_pubkey = get_lightning_node_pubkey(config)
            .await
            .unwrap_or_else(|_| "".to_string());
        Ok(Context {
            base_context,
            flash,
            order,
            listing,
            shipping_option,
            seller_user,
            user,
            admin_user,
            qr_svg_base64,
            lightning_node_pubkey,
        })
    }
}

async fn get_lightning_node_pubkey(config: &Config) -> Result<String, String> {
    let mut lighting_client = lightning::get_lnd_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .expect("failed to get lightning client");
    let get_info_resp = lighting_client
        // All calls require at least empty parameter
        .get_info(tonic_openssl_lnd::rpc::GetInfoRequest {})
        .await
        .expect("failed to get lightning node info");
    let info = get_info_resp.into_inner();
    Ok(info.identity_pubkey)
}

#[put("/<id>/ship")]
async fn ship(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match mark_order_as_shipped(id, &mut db, user.clone(), admin_user.clone()).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "order", id)),
            "Order marked as shipped.",
        )),
        Err(e) => {
            error_!("DB update({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(format!("/{}/{}", "order", id)),
                "Failed to mark order as shipped.",
            ))
        }
    }
}

async fn mark_order_as_shipped(
    order_id: &str,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let order = Order::single_by_public_id(db, order_id)
        .await
        .map_err(|_| "failed to get order.")?;

    if order.seller_user_id != user.id() {
        Err("User is not the order seller.".to_string())
    } else if order.shipped {
        Err("order is already shipped.".to_string())
    } else if order.canceled_by_seller || order.canceled_by_buyer {
        Err("order is already canceled.".to_string())
    } else {
        match Order::mark_as_shipped(&mut *db, order.id.unwrap()).await {
            Ok(_) => Ok(()),
            Err(_) => Err("failed to mark order as shipped.".to_string()),
        }
    }
}

#[put("/<id>/seller_cancel")]
async fn seller_cancel(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match mark_order_as_canceled_by_seller(id, &mut db, user.clone(), admin_user.clone()).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "order", id)),
            "Order marked as canceled by seller.",
        )),
        Err(e) => {
            error_!("DB update({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(format!("/{}/{}", "order", id)),
                "Failed to mark order as canceled by seller.",
            ))
        }
    }
}

async fn mark_order_as_canceled_by_seller(
    order_id: &str,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let order = Order::single_by_public_id(db, order_id)
        .await
        .map_err(|_| "failed to get order.")?;

    if order.seller_user_id != user.id() {
        Err("User is not the order seller.".to_string())
    } else if order.shipped {
        Err("order is already shipped.".to_string())
    } else if order.canceled_by_seller || order.canceled_by_buyer {
        Err("order is already canceled.".to_string())
    } else {
        match Order::mark_as_canceled_by_seller(&mut *db, order.id.unwrap()).await {
            Ok(_) => Ok(()),
            Err(_) => Err("failed to mark order as canceled by seller.".to_string()),
        }
    }
}

#[put("/<id>/buyer_cancel")]
async fn buyer_cancel(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match mark_order_as_canceled_by_buyer(id, &mut db, user.clone(), admin_user.clone()).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "order", id)),
            "Order marked as canceled by buyer.",
        )),
        Err(e) => {
            error_!("DB update({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(format!("/{}/{}", "order", id)),
                "Failed to mark order as canceled by buyer.",
            ))
        }
    }
}

async fn mark_order_as_canceled_by_buyer(
    order_id: &str,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let order = Order::single_by_public_id(db, order_id)
        .await
        .map_err(|_| "failed to get order.")?;

    if order.buyer_user_id != user.id() {
        Err("User is not the order buyer.".to_string())
    } else if order.shipped {
        Err("order is already shipped.".to_string())
    } else if order.canceled_by_seller || order.canceled_by_buyer {
        Err("order is already canceled.".to_string())
    } else {
        match Order::mark_as_canceled_by_buyer(&mut *db, order.id.unwrap()).await {
            Ok(_) => Ok(()),
            Err(_) => Err("failed to mark order as canceled by buyer.".to_string()),
        }
    }
}

#[post("/<id>/new_review", data = "<order_review_form>")]
async fn new_review(
    id: &str,
    order_review_form: Form<ReviewInput>,
    mut db: Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let order_review_info = order_review_form.into_inner();
    match create_order_review(id, order_review_info, &mut db, user.clone()).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "order", id)),
            "Review Successfully Posted.",
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

async fn create_order_review(
    order_id: &str,
    order_review_info: ReviewInput,
    db: &mut Connection<Db>,
    user: User,
) -> Result<(), String> {
    let now = util::current_time_millis();
    let order = Order::single_by_public_id(db, order_id)
        .await
        .map_err(|_| "failed to get order")?;
    let review_rating = order_review_info.review_rating.unwrap_or(0);
    let review_text = order_review_info.review_text;

    if !order.shipped {
        Err("Cannot post review for order that is not shipped.".to_string())
    } else if user.id() != order.buyer_user_id {
        Err("User is not the buyer.".to_string())
    } else if !(1..=5).contains(&review_rating) {
        Err("Review rating must be between 1 and 5.".to_string())
    } else if review_text.len() > 4096 {
        Err("Review text is too long.".to_string())
    } else {
        let new_review_time_ms = if order.review_time_ms > 0 {
            order.review_time_ms
        } else {
            now
        };

        match Order::set_order_review(
            db,
            order_id,
            review_rating,
            &review_text,
            new_review_time_ms,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error_!("DB insertion error: {}", e);
                Err("Order Review could not be inserted due an internal error.".to_string())
            }
        }
    }
}

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
    config: &State<Config>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    let context = match Context::raw(db, id, flash, user, admin_user, config).await {
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
        rocket.mount(
            "/order",
            routes![index, ship, seller_cancel, buyer_cancel, new_review],
        )
    })
}
