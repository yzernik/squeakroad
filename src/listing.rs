use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Listing, ListingDisplay, ShippingOption};
use crate::user_account::ActiveUser;
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
    listing_display: ListingDisplay,
    user: Option<User>,
    admin_user: Option<AdminUser>,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        listing_id: &str,
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

        // Do not show listing if it is not active (unless user is seller or admin).
        if !(user.as_ref().map(|u| u.id()) == Some(listing_display.listing.user_id)
            || admin_user.is_some())
        {
            if !listing_display.listing.approved {
                return Err("Listing is not approved.".to_string());
            }
            if listing_display.listing.deactivated_by_seller {
                return Err("Listing has been deactivated by seller.".to_string());
            }
            if listing_display.listing.deactivated_by_admin {
                return Err("Listing has been deactivated by admin.".to_string());
            }
        };

        Ok(Context {
            base_context,
            flash,
            listing_display,
            user,
            admin_user,
        })
    }
}

#[put("/<id>/submit")]
async fn submit(
    id: &str,
    mut db: Connection<Db>,
    active_user: ActiveUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match submit_listing(&mut db, id, active_user.user).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/listing", index(id))),
            "Marked as submitted".to_string(),
        )),
        Err(e) => {
            error_!("Mark submitted({}) error: {}", id, e);
            Err(Flash::error(Redirect::to(uri!("/listing", index(id))), e))
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
        return Err("Listing belongs to a different user.".to_string());
    };
    if listing.submitted {
        return Err("Listing is already submitted.".to_string());
    };
    if listing.approved {
        return Err("Listing is already approved.".to_string());
    };
    if shipping_options.is_empty() {
        return Err("At least one shipping option required.".to_string());
    };

    Listing::mark_as_submitted(db, id)
        .await
        .map_err(|_| "failed to update listing")?;
    Ok(())
}

#[put("/<id>/approve")]
async fn approve(
    id: &str,
    mut db: Connection<Db>,
    _active_user: ActiveUser,
    _admin_user: AdminUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match approve_listing(&mut db, id).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/listing", index(id))),
            "Marked as approved".to_string(),
        )),
        Err(e) => {
            error_!("Mark approved({}) error: {}", id, e);
            Err(Flash::error(Redirect::to(uri!("/listing", index(id))), e))
        }
    }
}

async fn approve_listing(db: &mut Connection<Db>, id: &str) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get listing")?;
    if !listing.submitted {
        return Err("Listing is not submitted.".to_string());
    };
    if listing.reviewed {
        return Err("Listing is already reviewed.".to_string());
    };

    Listing::mark_as_approved(db, id)
        .await
        .map_err(|_| "failed to approve listing")?;
    Ok(())
}

#[put("/<id>/reject")]
async fn reject(
    id: &str,
    mut db: Connection<Db>,
    _active_user: ActiveUser,
    _admin_user: AdminUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match reject_listing(&mut db, id).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/listing", index(id))),
            "Marked as rejected".to_string(),
        )),
        Err(e) => {
            error_!("Mark rejected({}) error: {}", id, e);
            Err(Flash::error(Redirect::to(uri!("/listing", index(id,))), e))
        }
    }
}

async fn reject_listing(db: &mut Connection<Db>, id: &str) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get listing")?;
    if !listing.submitted {
        return Err("Listing is not submitted.".to_string());
    };
    if listing.reviewed {
        return Err("Listing is already reviewed.".to_string());
    };

    Listing::mark_as_rejected(db, id)
        .await
        .map_err(|_| "failed to reject listing")?;
    Ok(())
}

#[put("/<id>/deactivate_as_admin")]
async fn admin_deactivate(
    id: &str,
    mut db: Connection<Db>,
    _active_user: ActiveUser,
    _admin_user: AdminUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match deactivate_listing_as_admin(&mut db, id).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/listing", index(id))),
            "Marked as deactivated by admin".to_string(),
        )),
        Err(e) => {
            error_!("Mark as deactivated error({}) error: {}", id, e);
            Err(Flash::error(Redirect::to(uri!("/listing", index(id))), e))
        }
    }
}

#[put("/<id>/deactivate")]
async fn deactivate(
    id: &str,
    mut db: Connection<Db>,
    active_user: ActiveUser,
    _admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match deactivate_listing_as_seller(&mut db, id, active_user.user).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/listing", index(id))),
            "Marked as deactivated by seller".to_string(),
        )),
        Err(e) => {
            error_!("Mark as deactivated error({}) error: {}", id, e);
            Err(Flash::error(Redirect::to(uri!("/listing", index(id))), e))
        }
    }
}

async fn deactivate_listing_as_seller(
    db: &mut Connection<Db>,
    id: &str,
    user: User,
) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get listing")?;
    if listing.user_id != user.id() {
        return Err("Listing belongs to a different user.".to_string());
    };
    if listing.deactivated_by_seller || listing.deactivated_by_admin {
        return Err("Listing is already deactivated.".to_string());
    };

    Listing::mark_as_deactivated_by_seller(db, id)
        .await
        .map_err(|_| "failed to deactivate listing by seller")?;
    Ok(())
}

async fn deactivate_listing_as_admin(db: &mut Connection<Db>, id: &str) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get listing")?;
    if listing.deactivated_by_seller || listing.deactivated_by_admin {
        return Err("Listing is already deactivated.".to_string());
    };

    Listing::mark_as_deactivated_by_admin(db, id)
        .await
        .map_err(|_| "failed to deactivate listing by admin")?;
    Ok(())
}

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "listing",
        Context::raw(db, id, flash, user, admin_user).await,
    )
}

pub fn listing_stage() -> AdHoc {
    AdHoc::on_ignite("Listing Stage", |rocket| async {
        rocket.mount(
            "/listing",
            routes![index, submit, approve, reject, deactivate, admin_deactivate],
        )
    })
}
