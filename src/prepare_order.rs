use crate::db::Db;
use crate::models::{AdminSettings, Listing, ListingDisplay, ShippingOption};
use rocket::fairing::AdHoc;
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
