use crate::db::Db;
use crate::models::{ListingDisplay, ShippingOption};
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::serde::Serialize;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    listing_display: Option<ListingDisplay>,
    selected_shipping_option: Option<ShippingOption>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
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
        listing_id: i32,
        shipping_option_id: Option<i32>,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Context {
        let listing_display = ListingDisplay::single(&mut db, listing_id).await;
        let maybe_shipping_option = match shipping_option_id {
            Some(sid) => {
                let shipping_option = ShippingOption::single(&mut db, sid).await;
                shipping_option.ok()
            }
            None => None,
        };

        match listing_display {
            Ok(listing_display) => Context {
                flash,
                listing_display: Some(listing_display),
                selected_shipping_option: maybe_shipping_option,
                user,
                admin_user,
            },
            Err(e) => {
                error_!("DB Listing::all() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Fail to access database.".into())),
                    listing_display: None,
                    selected_shipping_option: None,
                    user: user,
                    admin_user: admin_user,
                }
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

#[get("/<id>?<shipping_option_id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: i32,
    shipping_option_id: Option<i32>,
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
        rocket.mount("/listing", routes![index])
    })
}
