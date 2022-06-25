use crate::db::Db;
use crate::models::{Listing, ListingDisplay, ShippingOption, ShippingOptionInfo};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::uuid::Uuid;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    listing_display: Option<ListingDisplay>,
    user: User,
    admin_user: Option<AdminUser>,
}

impl Context {
    pub async fn err<M: std::fmt::Display>(
        _db: Connection<Db>,
        _listing_id: &str,
        msg: M,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Context {
        // TODO: get the listing display and put in context.
        Context {
            flash: Some(("error".into(), msg.to_string())),
            listing_display: None,
            user: user,
            admin_user,
        }
    }

    pub async fn raw(
        mut db: Connection<Db>,
        listing_id: &str,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Context {
        match ListingDisplay::single_by_public_id(&mut db, listing_id).await {
            Ok(listing_display) => {
                if listing_display.listing.user_id == user.id() {
                    println!("{:?}", listing_display.shipping_options);
                    Context {
                        flash,
                        listing_display: Some(listing_display),
                        user,
                        admin_user,
                    }
                } else {
                    error_!("Listing belongs to other user.");
                    Context {
                        flash: Some(("error".into(), "Listing belongs to other user.".into())),
                        listing_display: None,
                        user: user,
                        admin_user: admin_user,
                    }
                }
            }
            Err(e) => {
                error_!("DB Listing::single() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Fail to access database.".into())),
                    listing_display: None,
                    user: user,
                    admin_user: admin_user,
                }
            }
        }
    }
}

#[post("/<id>/add_shipping_option", data = "<shipping_option_form>")]
async fn new(
    id: &str,
    shipping_option_form: Form<ShippingOptionInfo>,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Flash<Redirect> {
    println!("listing_id: {:?}", id);

    let shipping_option_info = shipping_option_form.into_inner();
    let title = shipping_option_info.title;
    let description = shipping_option_info.description;
    let price_msat = shipping_option_info.price_sat * 1000;

    match add_shipping_option(
        id,
        title,
        description,
        price_msat,
        &mut db,
        user,
        admin_user,
    )
    .await
    {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/add_shipping_options", index(id))),
            "Shipping option successfully added.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/add_shipping_options", index(id))), e),
    }

    // Flash::error(
    //     Redirect::to(uri!("/add_shipping_options", index(id))),
    //     "not implemented".to_string(),
    // )
}

async fn add_shipping_option(
    id: &str,
    title: String,
    description: String,
    price_msat: u64,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get listing")?;
    let shipping_options = ShippingOption::all_for_listing(db, listing.id.unwrap())
        .await
        .map_err(|_| "failed to get shipping options for listing")?;

    if listing.user_id != user.id() {
        Err("Listing belongs to a different user.".to_string())
    } else if listing.submitted {
        Err("Listing is already submitted.".to_string())
    } else if shipping_options.len() >= 5 {
        Err("Maximum number of shipping options already exist.".to_string())
        // TODO: validate shipping option here.
    } else {
        let shipping_option = ShippingOption {
            id: None,
            public_id: Uuid::new_v4().to_string(),
            listing_id: listing.id.unwrap(),
            title: title,
            description: description,
            price_msat: price_msat,
        };

        ShippingOption::insert(shipping_option, db)
            .await
            .map_err(|_| "failed to save shipping option.")?;

        Ok(())
    }
}

#[delete("/<id>/add_shipping_option/<shipping_option_id>")]
async fn delete(
    id: &str,
    shipping_option_id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Template> {
    match delete_shipping_option(
        id,
        shipping_option_id,
        &mut db,
        user.clone(),
        admin_user.clone(),
    )
    .await
    {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/add_shipping_options", index(id))),
            "Shipping option was deleted.",
        )),
        Err(e) => {
            error_!("DB deletion({}) error: {}", id, e);
            Err(Template::render(
                "addshippingoptions",
                Context::err(
                    db,
                    id,
                    "Failed to delete shipping option.",
                    user,
                    admin_user,
                )
                .await,
            ))
        }
    }
}

async fn delete_shipping_option(
    listing_id: &str,
    shipping_option_id: &str,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let listing = Listing::single_by_public_id(&mut *db, listing_id)
        .await
        .map_err(|_| "failed to get listing")?;
    let shipping_option = ShippingOption::single_by_public_id(&mut *db, shipping_option_id)
        .await
        .map_err(|_| "failed to get shipping option")?;

    if shipping_option.listing_id != listing.id.unwrap() {
        Err("Invalid listing id given.".to_string())
    } else if listing.submitted {
        Err("Listing is already submitted.".to_string())
    } else if listing.user_id != user.id() {
        Err("Listing belongs to a different user.".to_string())
    } else {
        match ShippingOption::delete_with_public_id(shipping_option_id, &mut *db).await {
            Ok(num_deleted) => {
                if num_deleted > 0 {
                    Ok(())
                } else {
                    Err("No shipping options deleted.".to_string())
                }
            }
            Err(_) => Err("failed to delete shipping option.".to_string()),
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
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "addshippingoptions",
        Context::raw(db, id, flash, user, admin_user).await,
    )
}

pub fn add_shipping_options_stage() -> AdHoc {
    AdHoc::on_ignite("Add Shipping Options Stage", |rocket| async {
        rocket
            // .mount("/add_listing_images", routes![index, new])
            .mount("/add_shipping_options", routes![index, new, delete])
    })
}
