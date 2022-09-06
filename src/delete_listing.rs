use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Listing, ListingDisplay};
use rocket::fairing::AdHoc;
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

        if listing_display.listing.user_id != user.id() && admin_user.is_none() {
            return Err("User does not have permission to delete listing.".to_string());
        };

        Ok(Context {
            base_context,
            flash,
            listing_display: Some(listing_display),
        })
    }
}

#[delete("/<id>")]
async fn delete(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match delete_listing(id, &mut db, user.clone(), admin_user.clone()).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/")),
            "Listing was deleted.",
        )),
        Err(e) => {
            error_!("DB deletion({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(uri!("/update_listing_images", index(id))),
                "Failed to delete listing image.",
            ))
        }
    }
}

async fn delete_listing(
    listing_id: &str,
    db: &mut Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let listing = Listing::single_by_public_id(&mut *db, listing_id)
        .await
        .map_err(|_| "failed to get listing")?;

    if listing.user_id != user.id() && admin_user.is_none() {
        return Err("User does not have permission to delete listing.".to_string());
    };

    Listing::delete(listing.id.unwrap(), &mut *db)
        .await
        .map_err(|_| "failed to delete listing.".to_string())?;

    Ok(())
}

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, id, flash, user, admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("deletelisting", context))
}

pub fn delete_listing_stage() -> AdHoc {
    AdHoc::on_ignite("Delete Listing Stage", |rocket| async {
        rocket.mount("/delete_listing", routes![index, delete])
    })
}
