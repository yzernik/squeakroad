use crate::db::Db;
use crate::models::{InitialListingInfo, Listing};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::uuid::Uuid;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
}

impl Context {
    pub async fn err<M: std::fmt::Display>(
        msg: M,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Context {
        Context {
            flash: Some(("error".into(), msg.to_string())),
            user,
            admin_user,
        }
    }

    pub async fn raw(
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Context {
        Context {
            flash,
            user,
            admin_user,
        }
    }
}

#[post("/", data = "<listing_form>")]
async fn new(
    listing_form: Form<InitialListingInfo>,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Flash<Redirect>, Template> {
    let listing_info = listing_form.into_inner();

    match create_listing(listing_info, &mut db, user.clone()).await {
        Ok(listing_id) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "listing", listing_id)),
            "Listing successfully added.",
        )),
        Err(e) => {
            error_!("DB insertion error: {}", e);
            Err(Template::render(
                "newlisting",
                Context::err(e, Some(user), admin_user).await,
            ))
        }
    }
}

async fn create_listing(
    listing_info: InitialListingInfo,
    db: &mut Connection<Db>,
    user: User,
) -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let listing = Listing {
        id: None,
        public_id: Uuid::new_v4().to_string(),
        user_id: user.id(),
        title: listing_info.title,
        description: listing_info.description,
        price_msat: listing_info.price_sat * 1000,
        submitted: false,
        approved: false,
        created_time_ms: now,
    };

    if listing.description.is_empty() {
        Err("Description cannot be empty.".to_string())
    } else if user.is_admin {
        Err("Admin user cannot create a listing.".to_string())
    } else {
        match Listing::insert(listing, db).await {
            Ok(listing_id) => match Listing::single(db, listing_id).await {
                Ok(new_listing) => Ok(new_listing.public_id.clone()),
                Err(e) => {
                    error_!("DB insertion error: {}", e);
                    Err("New listing could not be found after inserting.".to_string())
                }
            },
            Err(e) => {
                error_!("DB insertion error: {}", e);
                Err("Listing could not be inserted due an internal error.".to_string())
            }
        }
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

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    _db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "newlisting",
        Context::raw(flash, Some(user), admin_user).await,
    )
}

pub fn new_listing_stage() -> AdHoc {
    AdHoc::on_ignite("New Listing Stage", |rocket| async {
        rocket.mount("/new_listing", routes![index, new])
    })
}
