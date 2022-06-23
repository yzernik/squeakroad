use crate::db::Db;
use crate::models::FileUploadForm;
use crate::models::{Listing, ListingDisplay, ListingImage};
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
    flash: Option<(String, String)>,
    listing_display: Option<ListingDisplay>,
    user: User,
    admin_user: Option<AdminUser>,
}

impl Context {
    pub async fn err<M: std::fmt::Display>(
        _db: Connection<Db>,
        _listing_id: i32,
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
        listing_id: i32,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Context {
        match Listing::single_display(&mut db, listing_id).await {
            Ok(listing_display) => {
                if listing_display.listing.user_id == user.id() {
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

#[post("/<id>/add_photo", data = "<upload_image_form>")]
async fn new(
    id: i32,
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
            Redirect::to(uri!("/add_listing_photos", index(id))),
            "Listing image successfully added.",
        ),
        Err(_) => Flash::error(
            Redirect::to(uri!("/add_listing_photos", index(id))),
            "Listing could not be inserted due an internal error.",
        ),
    }
}

fn get_file_bytes(tmp_file: TempFile) -> Result<Vec<u8>, String> {
    let path = tmp_file.path().ok_or("Path not found.")?;
    let bytes = fs::read(&path).map_err(|_| "Unable to read bytes")?;
    Ok(bytes)
}

async fn upload_image(
    id: i32,
    tmp_file: TempFile<'_>,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let listing = Listing::single(db, id)
        .await
        .map_err(|_| "failed to get listing")?;
    let listing_images = ListingImage::all_for_listing(db, id)
        .await
        .map_err(|_| "failed to get listing")?;

    if listing.user_id != user.id() {
        Err("Listing belongs to a different user.".to_string())
    } else if listing.completed {
        Err("Listing is already completed.".to_string())
    } else if listing_images.len() >= 5 {
        Err("Maximum number of images already exist.".to_string())
    } else if tmp_file.len() >= 1000000 {
        Err("File is bigger than maximum allowed size.".to_string())
    } else {
        let image_bytes = get_file_bytes(tmp_file).map_err(|_| "failed to get bytes.")?;

        let listing_image = ListingImage {
            id: None,
            listing_id: id,
            image_data: image_bytes,
        };

        ListingImage::insert(listing_image, db)
            .await
            .map_err(|_| "failed to save image in db.")?;

        Ok(())
    }
}

// #[put("/<id>")]
// async fn toggle(id: i32, mut db: Connection<Db>, user: User) -> Result<Redirect, Template> {
//     match Task::toggle_with_id(id, &mut db).await {
//         Ok(_) => Ok(Redirect::to("/")),
//         Err(e) => {
//             error_!("DB toggle({}) error: {}", id, e);
//             Err(Template::render(
//                 "index",
//                 Context::err(db, "Failed to toggle task.", Some(user)).await,
//             ))
//         }
//     }
// }

#[delete("/<id>/add_photo/<image_id>")]
async fn delete(
    id: i32,
    image_id: i32,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Template> {
    match delete_image(id, image_id, &mut db, user.clone(), admin_user.clone()).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(uri!("/add_listing_photos", index(id))),
            "Listing image was deleted.",
        )),
        Err(e) => {
            error_!("DB deletion({}) error: {}", id, e);
            Err(Template::render(
                "addlistingphotos",
                Context::err(db, id, "Failed to delete listing image.", user, admin_user).await,
            ))
        }
    }

    // match ListingImage::delete_with_id(image_id, &mut db).await {
    //     Ok(_) => Ok(Flash::success(
    //         Redirect::to(uri!("/add_listing_photos", index(id))),
    //         "Listing image was deleted.",
    //     )),
    //     Err(e) => {
    //         error_!("DB deletion({}) error: {}", id, e);
    //         Err(Template::render(
    //             "addlistingphotos",
    //             Context::err(db, id, "Failed to delete listing image.", user, admin_user).await,
    //         ))
    //     }
    // }
}

async fn delete_image(
    listing_id: i32,
    image_id: i32,
    db: &mut Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Result<(), String> {
    let listing = Listing::single(&mut *db, listing_id)
        .await
        .map_err(|_| "failed to get listing")?;
    let listing_image = ListingImage::single(&mut *db, listing_id)
        .await
        .map_err(|_| "failed to get listing")?;

    if listing_image.listing_id != listing.id.unwrap() {
        Err("Invalid listing id given.".to_string())
    } else if listing.completed {
        Err("Listing is already completed.".to_string())
    } else if listing.user_id != user.id() {
        Err("Listing belongs to a different user.".to_string())
    } else {
        match ListingImage::delete_with_id(image_id, &mut *db).await {
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

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: i32,
    db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "addlistingphotos",
        Context::raw(db, id, flash, user, admin_user).await,
    )
}

pub fn add_listing_photos_stage() -> AdHoc {
    AdHoc::on_ignite("Add Listing Photos Stage", |rocket| async {
        rocket
            // .mount("/add_listing_photos", routes![index, new])
            .mount("/add_listing_photos", routes![index, new, delete])
    })
}
