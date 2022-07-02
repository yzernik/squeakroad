use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AccountInfo, AdminSettings, Listing, ListingDisplay, Order, ShippingOption};
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
    base_context: BaseContext,
    flash: Option<(String, String)>,
    listing_display: Option<ListingDisplay>,
    selected_shipping_option: Option<ShippingOption>,
    quantity_in_stock: u32,
}

impl Context {
    // pub async fn err<M: std::fmt::Display>(
    //     db: Connection<Db>,
    //     msg: M,
    //     user: Option<User>,
    // ) -> Context {
    //     Context {
    //         flash: Some(("error".into(), msg.to_string())),
    //         listings: Listing::all(db).await.unwrap_or_default(),
    //         user: user,
    //     }
    // }

    pub async fn raw(
        mut db: Connection<Db>,
        listing_id: &str,
        shipping_option_id: Option<&str>,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let listing_display = ListingDisplay::single_by_public_id(&mut db, listing_id)
            .await
            .map_err(|_| "failed to get admin settings.")?;
        let quantity_sold =
            Order::quantity_of_listing_sold(&mut db, listing_display.listing.id.unwrap())
                .await
                .map_err(|_| "failed to get quantity sold.")?;
        let maybe_shipping_option = match shipping_option_id {
            Some(sid) => {
                let shipping_option = ShippingOption::single_by_public_id(&mut db, sid).await;
                shipping_option.ok()
            }
            None => None,
        };
        let account_info = match user {
            Some(ref u) => Some(
                AccountInfo::account_info_for_user(&mut db, u.id())
                    .await
                    .map_err(|_| "failed to get account info.")?,
            ),
            None => None,
        };
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;
        let quantity_in_stock = listing_display.listing.quantity - quantity_sold;

        Ok(Context {
            base_context,
            flash,
            listing_display: Some(listing_display),
            selected_shipping_option: maybe_shipping_option,
            quantity_in_stock,
        })
    }
}

#[put("/<id>/submit")]
async fn submit(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    println!("Handling submit endpoint for {:?}", id);
    //match Listing::mark_as_submitted(&mut db, id).await {
    match submit_listing(&mut db, id, user).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/listing", index(id, Some("")))),
            "Marked as submitted".to_string(),
        )),
        Err(e) => {
            error_!("Mark submitted({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(uri!("/listing", index(id, Some("")))),
                e,
            ))
        }
    }
}

async fn submit_listing(db: &mut Connection<Db>, id: &str, user: User) -> Result<(), String> {
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
    } else if listing.approved {
        Err("Listing is already approved.".to_string())
    } else if listing.removed {
        Err("Listing is already removed.".to_string())
    } else if shipping_options.len() == 0 {
        Err("At least one shipping option required.".to_string())
    } else {
        Listing::mark_as_submitted(db, id)
            .await
            .map_err(|_| "failed to update listing")?;
        Ok(())
    }
}

#[put("/<id>/approve")]
async fn approve(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: AdminUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match approve_listing(&mut db, id).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/listing", index(id, Some("")))),
            "Marked as approved".to_string(),
        )),
        Err(e) => {
            error_!("Mark approved({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(uri!("/listing", index(id, Some("")))),
                e,
            ))
        }
    }
}

async fn approve_listing(db: &mut Connection<Db>, id: &str) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get listing")?;
    if !listing.submitted {
        Err("Listing is not submitted.".to_string())
    } else if listing.reviewed {
        Err("Listing is already reviewed.".to_string())
    } else if listing.removed {
        Err("Listing is already removed.".to_string())
    } else {
        Listing::mark_as_approved(db, id)
            .await
            .map_err(|_| "failed to approve listing")?;
        Ok(())
    }
}

#[put("/<id>/reject")]
async fn reject(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: AdminUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match reject_listing(&mut db, id).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/listing", index(id, Some("")))),
            "Marked as rejected".to_string(),
        )),
        Err(e) => {
            error_!("Mark rejected({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(uri!("/listing", index(id, Some("")))),
                e,
            ))
        }
    }
}

async fn reject_listing(db: &mut Connection<Db>, id: &str) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get listing")?;
    if !listing.submitted {
        Err("Listing is not submitted.".to_string())
    } else if listing.reviewed {
        Err("Listing is already reviewed.".to_string())
    } else if listing.removed {
        Err("Listing is already removed.".to_string())
    } else {
        Listing::mark_as_rejected(db, id)
            .await
            .map_err(|_| "failed to reject listing")?;
        Ok(())
    }
}

#[put("/<id>/remove")]
async fn remove(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match remove_listing(&mut db, id, user, admin_user).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/listing", index(id, Some("")))),
            "Marked as removed".to_string(),
        )),
        Err(e) => {
            error_!("Mark removed({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(uri!("/listing", index(id, Some("")))),
                e,
            ))
        }
    }
}

async fn remove_listing(
    db: &mut Connection<Db>,
    id: &str,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get listing")?;
    if listing.user_id != user.id() && admin_user.is_none() {
        Err("Listing belongs to a different user.".to_string())
    } else if listing.removed {
        Err("Listing is already removed.".to_string())
    } else {
        Listing::mark_as_removed(db, id)
            .await
            .map_err(|_| "failed to remove listing")?;
        Ok(())
    }
}

#[get("/<id>?<shipping_option_id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    shipping_option_id: Option<&str>,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Template {
    println!("looking for listing...");
    println!("Shipping option id: {:?}", shipping_option_id);

    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "listing",
        Context::raw(db, id, shipping_option_id, flash, user, admin_user).await,
    )
}

pub fn listing_stage() -> AdHoc {
    AdHoc::on_ignite("Listing Stage", |rocket| async {
        rocket.mount("/listing", routes![index, submit, approve, reject, remove])
    })
}
