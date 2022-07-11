use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{Listing, ListingDisplay, Order, OrderInfo, ShippingOption, UserSettings};
use crate::util;
use pgp::composed::{Deserializable, Message};
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
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    listing_display: Option<ListingDisplay>,
    selected_shipping_option: ShippingOption,
    quantity: i32,
    seller_user_settings: UserSettings,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        listing_id: &str,
        shipping_option_id: &str,
        quantity: i32,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let listing_display = ListingDisplay::single_by_public_id(&mut db, listing_id)
            .await
            .map_err(|_| "failed to get admin settings.")?;
        let shipping_option = ShippingOption::single_by_public_id(&mut db, shipping_option_id)
            .await
            .map_err(|_| "failed to get shipping option.")?;
        let seller_user_settings = UserSettings::single(
            &mut db,
            listing_display.listing.user_id,
            UserSettings::default(),
        )
        .await
        .map_err(|_| "failed to get visited user settings.")?;
        Ok(Context {
            base_context,
            flash,
            listing_display: Some(listing_display),
            selected_shipping_option: shipping_option,
            quantity,
            seller_user_settings,
        })
    }
}

#[post("/<id>/new", data = "<order_form>")]
async fn new(
    id: &str,
    order_form: Form<OrderInfo>,
    mut db: Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
    config: &State<Config>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let order_info = order_form.into_inner();

    match create_order(
        id,
        order_info.clone(),
        &mut db,
        user.clone(),
        config.inner().clone(),
    )
    .await
    {
        Ok(order_id) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "order", order_id)),
            "Order successfully created.",
        )),
        Err(e) => {
            error_!("DB insertion error: {}", e);
            Err(Flash::error(
                Redirect::to(uri!(
                    "/prepare_order",
                    index(id, Some(order_info.shipping_option_id), Some(1))
                )),
                e,
            ))
        }
    }
}

async fn create_order(
    listing_id: &str,
    order_info: OrderInfo,
    db: &mut Connection<Db>,
    user: User,
    config: Config,
) -> Result<String, String> {
    let listing = Listing::single_by_public_id(db, listing_id)
        .await
        .map_err(|_| "failed to get listing")?;
    let shipping_option = ShippingOption::single_by_public_id(db, &order_info.shipping_option_id)
        .await
        .map_err(|_| "failed to get shipping option.")?;
    let quantity_sold = Order::quantity_of_listing_sold(db, listing.id.unwrap())
        .await
        .map_err(|_| "failed to get quantity sold.")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let shipping_instructions = order_info.shipping_instructions;
    let quantity = order_info.quantity.unwrap_or(0);

    let price_per_unit_with_shipping_sat: u64 = listing.price_sat + shipping_option.price_sat;
    let amount_owed_sat: u64 = (quantity as u64) * price_per_unit_with_shipping_sat;
    // let market_fee_sat: u64 = (amount_owed_sat * (listing.fee_rate_basis_points as u64)) / 10000;
    let market_fee_sat: u64 = divide_round_up(
        amount_owed_sat * (listing.fee_rate_basis_points as u64),
        10000,
    );
    let seller_credit_sat: u64 = amount_owed_sat - market_fee_sat;
    let quantity_in_stock = listing.quantity - quantity_sold;

    let (message, _) =
        Message::from_string(&shipping_instructions).map_err(|_| "Invalid PGP message.")?;
    info!("message: {:?}", &message);

    if shipping_instructions.is_empty() {
        Err("Shipping instructions cannot be empty.".to_string())
    } else if shipping_instructions.len() > 4096 {
        Err("Shipping instructions length is too long.".to_string())
    } else if listing.user_id == user.id() {
        Err("Listing belongs to same user as buyer.".to_string())
    } else if !listing.approved {
        Err("Listing has not been approved by admin.".to_string())
    } else if listing.removed {
        Err("Listing has not been removed.".to_string())
    } else if shipping_option.listing_id != listing.id.unwrap() {
        Err("Shipping option not associated with listing.".to_string())
    } else if user.is_admin {
        Err("Admin user cannot create an order.".to_string())
    } else if quantity == 0 {
        Err("Quantity must be postive.".to_string())
    } else if quantity_in_stock < quantity {
        Err("Not enough items in stock.".to_string())
    } else {
        let mut lighting_client = lightning::get_lnd_client(
            config.lnd_host.clone(),
            config.lnd_port,
            config.lnd_tls_cert_path.clone(),
            config.lnd_macaroon_path.clone(),
        )
        .await
        .expect("failed to get lightning client");
        let invoice_resp = lighting_client
            // All calls require at least empty parameter
            .add_invoice(squeakroad_lnd_client::rpc::Invoice {
                value_msat: (amount_owed_sat as i64) * 1000,
                ..squeakroad_lnd_client::rpc::Invoice::default()
            })
            .await
            .expect("failed to get new invoice");
        // We only print it here, note that in real-life code you may want to call `.into_inner()` on
        // the response to get the message.
        let invoice = invoice_resp.into_inner();

        let order = Order {
            id: None,
            public_id: util::create_uuid(),
            quantity,
            buyer_user_id: user.id(),
            seller_user_id: listing.user_id,
            listing_id: listing.id.unwrap(),
            shipping_option_id: shipping_option.id.unwrap(),
            shipping_instructions: shipping_instructions.to_string(),
            amount_owed_sat,
            seller_credit_sat,
            paid: false,
            processing: false,
            acked: false,
            reviewed: false,
            invoice_hash: util::to_hex(&invoice.r_hash),
            invoice_payment_request: invoice.payment_request,
            review_rating: 0,
            review_text: "".to_string(),
            created_time_ms: now,
            payment_time_ms: 0,
            review_time_ms: 0,
        };

        match Order::insert(order, db).await {
            Ok(order_id) => match Order::single(db, order_id).await {
                Ok(new_order) => Ok(new_order.public_id),
                Err(e) => {
                    error_!("DB insertion error: {}", e);
                    Err("New order could not be found after inserting.".to_string())
                }
            },
            Err(e) => {
                error_!("DB insertion error: {}", e);
                Err("Order could not be inserted due an internal error.".to_string())
            }
        }
    }
}

fn divide_round_up(dividend: u64, divisor: u64) -> u64 {
    (dividend + divisor - 1) / divisor
}

#[get("/<id>?<shipping_option_id>&<quantity>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    shipping_option_id: Option<&str>,
    quantity: Option<i32>,
    db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Template {
    // TODO: Don't use unwrap.
    let sid = shipping_option_id.unwrap();
    let quantity = quantity.unwrap_or(0);

    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "prepareorder",
        Context::raw(db, id, sid, quantity, flash, user, admin_user).await,
    )
}

pub fn prepare_order_stage() -> AdHoc {
    AdHoc::on_ignite("Prepare Order Stage", |rocket| async {
        rocket.mount("/prepare_order", routes![index, new])
    })
}
