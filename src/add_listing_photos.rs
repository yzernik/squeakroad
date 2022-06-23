use crate::db::Db;
use crate::models::FileUploadForm;
use crate::models::{Listing, ListingImage};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::json::{json, Value};
use rocket::serde::{json, Deserialize, Serialize};
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::fs;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    listing: Option<Listing>,
    user: User,
    admin_user: Option<AdminUser>,
}

impl Context {
    pub async fn err<M: std::fmt::Display>(
        db: Connection<Db>,
        listing_id: i32,
        msg: M,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Context {
        Context {
            flash: Some(("error".into(), msg.to_string())),
            listing: None,
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
        match Listing::single(&mut db, listing_id).await {
            Ok(Some(lstng)) => {
                let listing = Some(lstng.clone());

                if lstng.user_id == user.id() {
                    Context {
                        flash,
                        listing,
                        user,
                        admin_user,
                    }
                } else {
                    error_!("Listing belongs to other user.");
                    Context {
                        flash: Some(("error".into(), "Listing belongs to other user.".into())),
                        listing: None,
                        user: user,
                        admin_user: admin_user,
                    }
                }
            }
            Ok(None) => {
                error_!("Listing not found.");
                Context {
                    flash: Some(("error".into(), "Listing not found.".into())),
                    listing: None,
                    user: user,
                    admin_user: admin_user,
                }
            }
            Err(e) => {
                error_!("DB Listing::single() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Fail to access database.".into())),
                    listing: None,
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
    db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Template> {
    // TODO: Change return type to Flash<Redirect>.
    // Same as "new" method in listings.rs

    println!("listing_id: {:?}", id);

    let image_info = upload_image_form.into_inner();
    let file = image_info.file;

    if let Some(image_bytes) = get_file_bytes(file) {
        println!("got bytes: {:?}", image_bytes);

        let listing_image = ListingImage {
            id: None,
            listing_id: id,
            image_data: image_bytes,
        };

        if let Err(e) = ListingImage::insert(listing_image, db).await {
            error_!("DB insertion error: {}", e);
            Ok(Flash::error(
                Redirect::to(uri!("/add_listing_photos", index(id))),
                "Listing image could not be inserted due an internal error.",
            ))
        } else {
            Ok(Flash::success(
                Redirect::to(uri!("/add_listing_photos", index(id))),
                "Listing image successfully added.",
            ))
        }

        // Ok(Flash::error(
        //     Redirect::to("/"),
        //     "Listing could not be inserted due an internal error.",
        // ))
    } else {
        error_!("DB deletion({}) error: {}", id, "Some error string");
        Err(Template::render(
            "index",
            Context::err(db, id, "Failed to delete task.", user, admin_user).await,
        ))
    }
}

fn get_file_bytes(tmp_file: TempFile) -> Option<Vec<u8>> {
    println!("path: {:?}", tmp_file.path());
    println!("content_type: {:?}", tmp_file.content_type());

    // match tmp_file {
    //     TempFile::File { len, .. } => {
    //         println!("matched a file.")
    //     }
    //     TempFile::Buffered { content } => {
    //         println!("matched a buffered");
    //         println!("content.len: {:?}", content.len() as u64);
    //         println!("content: {:?}", content)
    //     }
    // }

    if let Some(path) = tmp_file.path() {
        println!("found path.");
        let content = fs::read(&path);
        content.ok()
    } else {
        None
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

// #[delete("/<id>")]
// async fn delete(id: i32, mut db: Connection<Db>, user: User) -> Result<Flash<Redirect>, Template> {
//     match Task::delete_with_id(id, &mut db).await {
//         Ok(_) => Ok(Flash::success(Redirect::to("/"), "Listing was deleted.")),
//         Err(e) => {
//             error_!("DB deletion({}) error: {}", id, e);
//             Err(Template::render(
//                 "index",
//                 Context::err(db, "Failed to delete task.", Some(user)).await,
//             ))
//         }
//     }
// }

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
            .mount("/add_listing_photos", routes![index, new])
    })
}
