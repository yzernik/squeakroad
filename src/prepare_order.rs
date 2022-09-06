use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{Listing, ListingDisplay, Order, OrderInfo, ShippingOption, UserSettings};
use crate::user_account::ActiveUser;
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

const MAX_UNPAID_ORDERS: u32 = 100;

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
        let seller_user_settings = UserSettings::single(&mut db, listing_display.listing.user_id)
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
    active_user: ActiveUser,
    _admin_user: Option<AdminUser>,
    config: &State<Config>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let order_info = order_form.into_inner();

    match create_order(
        id,
        order_info.clone(),
        &mut db,
        active_user.user.clone(),
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
                    index(id, order_info.shipping_option_id, 1)
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
    let now = util::current_time_millis();
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

    let (message, _) =
        Message::from_string(&shipping_instructions).map_err(|_| "Invalid PGP message.")?;
    info!("message: {:?}", &message);

    if shipping_instructions.is_empty() {
        return Err("Shipping instructions cannot be empty.".to_string());
    };
    if shipping_instructions.len() > 4096 {
        return Err("Shipping instructions length is too long.".to_string());
    };
    if listing.user_id == user.id() {
        return Err("Listing belongs to same user as buyer.".to_string());
    };
    if !listing.approved {
        return Err("Listing has not been approved by admin.".to_string());
    };
    if listing.deactivated_by_seller || listing.deactivated_by_admin {
        return Err("Listing has been deactivated.".to_string());
    };
    if shipping_option.listing_id != listing.id.unwrap() {
        return Err("Shipping option not associated with listing.".to_string());
    };
    if user.is_admin {
        return Err("Admin user cannot create an order.".to_string());
    };
    if quantity == 0 {
        return Err("Quantity must be postive.".to_string());
    };

    let mut lightning_client = lightning::get_lnd_lightning_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .expect("failed to get lightning client");
    let invoice = lightning_client
        .add_invoice(tonic_openssl_lnd::lnrpc::Invoice {
            value_msat: (amount_owed_sat as i64) * 1000,
            ..Default::default()
        })
        .await
        .expect("failed to get new invoice")
        .into_inner();

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
        shipped: false,
        canceled_by_seller: false,
        canceled_by_buyer: false,
        reviewed: false,
        invoice_hash: util::to_hex(&invoice.r_hash),
        invoice_payment_request: invoice.payment_request,
        review_rating: 0,
        review_text: "".to_string(),
        created_time_ms: now,
        payment_time_ms: 0,
        review_time_ms: 0,
    };

    match Order::insert(order, MAX_UNPAID_ORDERS, db).await {
        Ok(order_id) => match Order::single(db, order_id).await {
            Ok(new_order) => Ok(new_order.public_id),
            Err(e) => {
                error_!("DB insertion error: {}", e);
                Err("New order could not be found after inserting.".to_string())
            }
        },
        Err(e) => {
            error_!("DB insertion error: {}", e);
            Err(e)
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
    shipping_option_id: &str,
    quantity: usize,
    db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(
        db,
        id,
        shipping_option_id,
        quantity.try_into().unwrap(),
        flash,
        active_user.user,
        admin_user,
    )
    .await
    .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("prepareorder", context))
}

pub fn prepare_order_stage() -> AdHoc {
    AdHoc::on_ignite("Prepare Order Stage", |rocket| async {
        rocket.mount("/prepare_order", routes![index, new])
    })
}
