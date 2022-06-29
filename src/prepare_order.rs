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
    listing_display: Option<ListingDisplay>,
    selected_shipping_option: ShippingOption,
    quantity: i32,
    user: User,
    admin_user: Option<AdminUser>,
    admin_settings: Option<AdminSettings>,
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
        let listing_display = ListingDisplay::single_by_public_id(&mut db, listing_id)
            .await
            .map_err(|_| "failed to get admin settings.")?;
        let shipping_option = ShippingOption::single_by_public_id(&mut db, shipping_option_id)
            .await
            .map_err(|_| "failed to get shipping option.")?;
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;

        Ok(Context {
            flash,
            listing_display: Some(listing_display),
            selected_shipping_option: shipping_option,
            quantity: quantity,
            user,
            admin_user,
            admin_settings: Some(admin_settings),
        })
    }
}

#[post("/<id>/new", data = "<order_form>")]
async fn new(
    id: &str,
    order_form: Form<OrderInfo>,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let order_info = order_form.into_inner();

    match create_order(id, order_info, &mut db, user.clone()).await {
        Ok(order_id) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "order", order_id)),
            "Order successfully created.",
        )),
        Err(e) => {
            error_!("DB insertion error: {}", e);
            Err(Flash::error(
                Redirect::to(uri!("/prepare_order", index(id, Some(""), Some(1)))),
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
) -> Result<String, String> {
    let listing = Listing::single_by_public_id(db, listing_id)
        .await
        .map_err(|_| "failed to get listing")?;
    let shipping_option = ShippingOption::single_by_public_id(db, &order_info.shipping_option_id)
        .await
        .map_err(|_| "failed to get shipping option.")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let shipping_instructions = order_info.shipping_instructions;

    let price_per_unit_with_shipping_sat: u64 = listing.price_sat + shipping_option.price_sat;
    let amount_owed_sat: u64 =
        ((order_info.quantity as u64) * price_per_unit_with_shipping_sat).into();
    let market_fee_sat: u64 = (amount_owed_sat * (listing.fee_rate_basis_points as u64)) / 10000;
    let seller_credit_sat: u64 = amount_owed_sat - market_fee_sat;

    if shipping_instructions.is_empty() {
        Err("Shipping instructions cannot be empty.".to_string())
    } else if shipping_instructions.len() > 1024 {
        Err("Shipping instructions length is too long.".to_string())
    } else if listing.user_id == user.id() {
        Err("Listing belongs to same user as buyer.".to_string())
    } else if shipping_option.listing_id != listing.id.unwrap() {
        Err("Shipping option not associated with listing.".to_string())
    } else {
        let order = Order {
            id: None,
            public_id: Uuid::new_v4().to_string(),
            quantity: order_info.quantity,
            user_id: user.id(),
            listing_id: listing.id.unwrap(),
            shipping_option_id: shipping_option.id.unwrap(),
            shipping_instructions: shipping_instructions.to_string(),
            amount_owed_sat: amount_owed_sat,
            seller_credit_sat: seller_credit_sat,
            paid: false,
            completed: false,
            created_time_ms: now,
        };

        match Order::insert(order, db).await {
            Ok(order_id) => match Order::single(db, order_id).await {
                Ok(new_order) => Ok(new_order.public_id.clone()),
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
    println!("looking for listing...");
    println!("Shipping option id: {:?}", shipping_option_id);

    // TODO: Don't use unwrap.
    let sid = shipping_option_id.unwrap();
    let quantity = quantity.unwrap();

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