use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Listing, ListingDisplay, ShippingOption, ShippingOptionInfo};
use crate::util;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    listing_display: Option<ListingDisplay>,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        listing_id: &str,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let listing_display = ListingDisplay::single_by_public_id(&mut db, listing_id)
            .await
            .map_err(|_| "failed to get listing display.")?;
        if listing_display.listing.user_id == user.id() {
            Ok(Context {
                base_context,
                flash,
                listing_display: Some(listing_display),
            })
        } else {
            error_!("Listing belongs to other user.");
            Ok(Context {
                base_context,
                flash: Some(("error".into(), "Listing belongs to other user.".into())),
                listing_display: None,
            })
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
    let shipping_option_info = shipping_option_form.into_inner();
    let title = shipping_option_info.title;
    let description = shipping_option_info.description;
    let price_sat = shipping_option_info.price_sat.unwrap_or(0);

    match add_shipping_option(id, title, description, price_sat, &mut db, user, admin_user).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_shipping_options", index(id))),
            "Shipping option successfully added.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_shipping_options", index(id))), e),
    }

    // Flash::error(
    //     Redirect::to(uri!("/update_shipping_options", index(id))),
    //     "not implemented".to_string(),
    // )
}

async fn add_shipping_option(
    id: &str,
    title: String,
    description: String,
    price_sat: u64,
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

    if title.is_empty() {
        Err("Title cannot be empty.".to_string())
    } else if description.is_empty() {
        Err("Description cannot be empty.".to_string())
    } else if title.len() > 64 {
        Err("Title length is too long.".to_string())
    } else if description.len() > 4096 {
        Err("Description length is too long.".to_string())
    } else if listing.user_id != user.id() {
        Err("Listing belongs to a different user.".to_string())
    } else if listing.submitted {
        Err("Listing is already submitted.".to_string())
    } else if shipping_options.len() >= 5 {
        Err("Maximum number of shipping options already exist.".to_string())
    } else {
        let shipping_option = ShippingOption {
            id: None,
            public_id: util::create_uuid(),
            listing_id: listing.id.unwrap(),
            title,
            description,
            price_sat,
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
) -> Result<Flash<Redirect>, Flash<Redirect>> {
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
            Redirect::to(uri!("/update_shipping_options", index(id))),
            "Shipping option was deleted.",
        )),
        Err(e) => {
            error_!("DB deletion({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(uri!("/update_shipping_options", index(id))),
                "Failed to delete shipping option.",
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
        "updateshippingoptions",
        Context::raw(db, id, flash, user, admin_user).await,
    )
}

pub fn update_shipping_options_stage() -> AdHoc {
    AdHoc::on_ignite("Add Shipping Options Stage", |rocket| async {
        rocket
            // .mount("/update_listing_images", routes![index, new])
            .mount("/update_shipping_options", routes![index, new, delete])
    })
}
