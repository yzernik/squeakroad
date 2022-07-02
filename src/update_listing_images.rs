use crate::base::BaseContext;
use crate::db::Db;
use crate::models::FileUploadForm;
use crate::models::{Listing, ListingDisplay, ListingImage};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::uuid::Uuid;
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
    user: User,
    admin_user: Option<AdminUser>,
) -> Flash<Redirect> {
    println!("listing_id: {:?}", id);

    let image_info = upload_image_form.into_inner();
    let file = image_info.file;

    match upload_image(id, file, &mut db, user, admin_user).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_listing_images", index(id))),
            "Listing image successfully added.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_listing_images", index(id))), e),
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
        Err("Listing belongs to a different user.".to_string())
    } else if listing.submitted {
        Err("Listing is already submitted.".to_string())
    } else if listing_images.len() >= 5 {
        Err("Maximum number of images already exist.".to_string())
    } else if tmp_file.len() == 0 {
        Err("File is empty.".to_string())
    } else if tmp_file.len() >= 1000000 {
        Err("File is bigger than maximum allowed size.".to_string())
    } else {
        let image_bytes = get_file_bytes(tmp_file).map_err(|_| "failed to get bytes.")?;

        let listing_image = ListingImage {
            id: None,
            public_id: Uuid::new_v4().to_string(),
            listing_id: listing.id.unwrap(),
            image_data: image_bytes,
            is_primary: false,
        };

        ListingImage::insert(listing_image, db)
            .await
            .map_err(|_| "failed to save image in db.")?;

        Ok(())
    }
}

#[delete("/<id>/add_image/<image_id>")]
async fn delete(
    id: &str,
    image_id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match delete_image_with_public_id(id, image_id, &mut db, user.clone(), admin_user.clone()).await
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

    // match ListingImage::delete_with_id(image_id, &mut db).await {
    //     Ok(_) => Ok(Flash::success(
    //         Redirect::to(uri!("/update_listing_images", index(id))),
    //         "Listing image was deleted.",
    //     )),
    //     Err(e) => {
    //         error_!("DB deletion({}) error: {}", id, e);
    //         Err(Template::render(
    //             "updatelistingimages",
    //             Context::err(db, id, "Failed to delete listing image.", user, admin_user).await,
    //         ))
    //     }
    // }
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
        Err("Invalid listing id given.".to_string())
    } else if listing.submitted {
        Err("Listing is already submitted.".to_string())
    } else if listing.user_id != user.id() {
        Err("Listing belongs to a different user.".to_string())
    } else {
        match ListingImage::delete_with_public_id(image_id, &mut *db).await {
            Ok(num_deleted) => {
                if num_deleted > 0 {
                    Ok(())
                } else {
                    Err("No images deleted.".to_string())
                }
            }
            Err(_) => Err("failed to delete image.".to_string()),
        }
    }
}

#[put("/<id>/set_primary/<image_id>")]
async fn set_primary(
    id: &str,
    image_id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match mark_as_primary(id, image_id, &mut db, user.clone(), admin_user.clone()).await {
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
        Err("Invalid listing id given.".to_string())
    } else if listing.submitted {
        Err("Listing is already submitted.".to_string())
    } else if listing.user_id != user.id() {
        Err("Listing belongs to a different user.".to_string())
    } else {
        match ListingImage::mark_image_as_primary_by_public_id(
            &mut *db,
            listing.id.unwrap(),
            image_id,
        )
        .await
        {
            Ok(num_marked) => {
                if num_marked > 0 {
                    Ok(())
                } else {
                    Err("No images marked as primary.".to_string())
                }
            }
            Err(_) => Err("failed to mark image as primary.".to_string()),
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
        "updatelistingimages",
        Context::raw(db, id, flash, user, admin_user).await,
    )
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
