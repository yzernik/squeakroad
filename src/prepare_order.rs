use crate::db::Db;
use crate::models::{AdminSettings, Listing, ListingDisplay, Order, ShippingOption};
use rocket::fairing::AdHoc;
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

async fn create_order(
    quantity: u32,
    listing_id: &str,
    shipping_option_id: &str,
    shipping_instructions: &str,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, listing_id)
        .await
        .map_err(|_| "failed to get listing")?;
    let shipping_option = ShippingOption::single_by_public_id(db, shipping_option_id)
        .await
        .map_err(|_| "failed to get shipping option.")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let price_per_unit_with_shipping_sat: u64 = listing.price_sat + shipping_option.price_sat;
    let amount_owed_sat: u64 = ((quantity as u64) * price_per_unit_with_shipping_sat).into();
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
            quantity: quantity,
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

        Order::insert(order, db)
            .await
            .map_err(|_| "failed to save order.")?;

        Ok(())
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
        rocket.mount("/prepare_order", routes![index])
    })
}
