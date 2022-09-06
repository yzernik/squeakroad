use crate::base::BaseContext;
use crate::db::Db;
use crate::image_util;
use crate::models::FileUploadForm;
use crate::models::{Listing, ListingDisplay, ListingImage};
use crate::user_account::ActiveUser;
use crate::util;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::fs;

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
            Err("Listing belongs to other user".into())
        }
    }
}

#[post("/<id>/add_image", data = "<upload_image_form>")]
async fn new(
    id: &str,
    upload_image_form: Form<FileUploadForm<'_>>,
    mut db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Flash<Redirect> {
    let image_info = upload_image_form.into_inner();
    let file = image_info.file;

    match upload_image(id, file, &mut db, active_user.user, admin_user).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_listing_images", index(id))),
            "Listing image successfully added.",
        ),
        Err(e) => {
            error!("Failed to save listing image.: {}", e);
            Flash::error(Redirect::to(uri!("/update_listing_images", index(id))), e)
        }
    }
}

fn get_file_bytes(tmp_file: TempFile) -> Result<Vec<u8>, String> {
    let path = tmp_file.path().ok_or("Path not found.")?;
    let bytes = fs::read(&path).map_err(|_| "Unable to read bytes")?;
    Ok(bytes)
}

async fn upload_image(
    id: &str,
    tmp_file: TempFile<'_>,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get listing")?;
    let listing_images = ListingImage::all_for_listing(db, listing.id.unwrap())
        .await
        .map_err(|_| "failed to get listing")?;

    if listing.user_id != user.id() {
        return Err("Listing belongs to a different user.".to_string());
    };
    if listing.submitted {
        return Err("Listing is already submitted.".to_string());
    };
    if listing_images.len() >= 5 {
        return Err("Maximum number of images already exist.".to_string());
    };
    if tmp_file.len() == 0 {
        return Err("File is empty.".to_string());
    };

    let image_bytes = get_file_bytes(tmp_file).map_err(|_| "failed to get bytes.")?;
    let cleared_metadata_image_bytes = image_util::get_stripped_image_bytes(&image_bytes)
        .map_err(|_| "failed to clear image metadata.")?;

    let listing_image = ListingImage {
        id: None,
        public_id: util::create_uuid(),
        listing_id: listing.id.unwrap(),
        image_data: cleared_metadata_image_bytes,
        is_primary: false,
    };

    ListingImage::insert(listing_image, db).await.map_err(|e| {
        error!("failed to save image in db: {}", e);
        "failed to save image in db."
    })?;

    Ok(())
}

#[delete("/<id>/add_image/<image_id>")]
async fn delete(
    id: &str,
    image_id: &str,
    mut db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match delete_image_with_public_id(
        id,
        image_id,
        &mut db,
        active_user.user.clone(),
        admin_user.clone(),
    )
    .await
    {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/update_listing_images", index(id))),
            "Listing image was deleted.",
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

async fn delete_image_with_public_id(
    listing_id: &str,
    image_id: &str,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let listing = Listing::single_by_public_id(&mut *db, listing_id)
        .await
        .map_err(|_| "failed to get listing")?;
    let listing_image = ListingImage::single_by_public_id(&mut *db, image_id)
        .await
        .map_err(|_| "failed to get listing")?;

    if listing_image.listing_id != listing.id.unwrap() {
        return Err("Invalid listing id given.".to_string());
    };
    if listing.submitted {
        return Err("Listing is already submitted.".to_string());
    };
    if listing.user_id != user.id() {
        return Err("Listing belongs to a different user.".to_string());
    };

    ListingImage::delete_with_public_id(image_id, &mut *db)
        .await
        .map_err(|_| "failed to delete image.".to_string())?;

    Ok(())
}

#[put("/<id>/set_primary/<image_id>")]
async fn set_primary(
    id: &str,
    image_id: &str,
    mut db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match mark_as_primary(
        id,
        image_id,
        &mut db,
        active_user.user.clone(),
        admin_user.clone(),
    )
    .await
    {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/update_listing_images", index(id))),
            "Image was marked as primary.",
        )),
        Err(e) => {
            error_!("DB update({}) error: {}", id, e);
            Err(Flash::error(
                Redirect::to(uri!("/update_listing_images", index(id))),
                "Failed to mark image as primary.",
            ))
        }
    }
}

async fn mark_as_primary(
    listing_id: &str,
    image_id: &str,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let listing = Listing::single_by_public_id(&mut *db, listing_id)
        .await
        .map_err(|_| "failed to get listing")?;
    let listing_image = ListingImage::single_by_public_id(&mut *db, image_id)
        .await
        .map_err(|_| "failed to get listing")?;

    if listing_image.listing_id != listing.id.unwrap() {
        return Err("Invalid listing id given.".to_string());
    };
    if listing.submitted {
        return Err("Listing is already submitted.".to_string());
    };
    if listing.user_id != user.id() {
        return Err("Listing belongs to a different user.".to_string());
    };

    ListingImage::mark_image_as_primary_by_public_id(&mut *db, listing.id.unwrap(), image_id)
        .await
        .map_err(|_| "failed to mark image as primary.".to_string())?;

    Ok(())
}

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, id, flash, active_user.user, admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("updatelistingimages", context))
}

pub fn update_listing_images_stage() -> AdHoc {
    AdHoc::on_ignite("Add Listing Images Stage", |rocket| async {
        rocket
            // .mount("/update_listing_images", routes![index, new])
            .mount(
                "/update_listing_images",
                routes![index, new, delete, set_primary],
            )
    })
}
