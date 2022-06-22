use crate::db::Db;
use crate::models::Listing;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    listing: Option<Listing>,
    user: User,
    admin_user: Option<AdminUser>,
}

impl Context {
    // pub async fn err<M: std::fmt::Display>(msg: M, user: Option<User>) -> Context {
    //     Context {
    //         flash: Some(("error".into(), msg.to_string())),
    //         user: user,
    //     }
    // }

    pub async fn raw(
        db: Connection<Db>,
        listing_id: i32,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Context {
        match Listing::single(db, listing_id).await {
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

// #[post("/add_photo", data = "<listing_form>")]
// async fn new(
//     listing_form: Form<InitialListingInfo>,
//     db: Connection<Db>,
//     user: User,
// ) -> Flash<Redirect> {
//     let listing_info = listing_form.into_inner();
//     let now = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_millis() as u64;

//     let listing = Listing {
//         id: None,
//         user_id: user.id(),
//         title: listing_info.title,
//         description: listing_info.description,
//         price_msat: listing_info.price_msat,
//         completed: false,
//         approved: false,
//         created_time_ms: now,
//     };

//     if listing.description.is_empty() {
//         Flash::error(Redirect::to("/"), "Description cannot be empty.")
//     } else if let Err(e) = Listing::insert(listing, db).await {
//         error_!("DB insertion error: {}", e);
//         Flash::error(
//             Redirect::to("/"),
//             "Listing could not be inserted due an internal error.",
//         )
//     } else {
//         Flash::success(Redirect::to("/"), "Listing successfully added.")
//     }
// }

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
            .mount("/add_listing_photos", routes![index])
    })
}
