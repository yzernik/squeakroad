use crate::db::Db;
use crate::models::{AdminSettings, ListingCardDisplay};
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

// impl AdminSettings {
//     pub fn get_default() -> AdminSettings {
//         AdminSettings {
//             id: None,
//             market_name: "default market name".to_string(),
//             fee_rate_basis_points: 500,
//         }
//     }
// }

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    listing_cards: Vec<ListingCardDisplay>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
    admin_settings: Option<AdminSettings>,
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
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Context {
        match get_context_fields(&mut db).await {
            Ok((listing_cards, admin_settings)) => Context {
                flash,
                listing_cards: listing_cards,
                user,
                admin_user,
                admin_settings: Some(admin_settings),
            },
            Err(e) => {
                error_!("DB Listing::all() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Fail to access database.".into())),
                    listing_cards: vec![],
                    user: user,
                    admin_user: admin_user,
                    admin_settings: None,
                }
            }
        }

        // match ListingCardDisplay::all(&mut db).await {
        //     Ok(listing_cards) => Context {
        //         flash,
        //         listing_cards: listing_cards,
        //         user,
        //         admin_user,
        //     },
        //     Err(e) => {
        //         error_!("DB Listing::all() error: {}", e);
        //         Context {
        //             flash: Some(("error".into(), "Fail to access database.".into())),
        //             listing_cards: vec![],
        //             user: user,
        //             admin_user: admin_user,
        //         }
        //     }
        // }
    }
}

async fn get_context_fields(
    db: &mut Connection<Db>,
) -> Result<(Vec<ListingCardDisplay>, AdminSettings), String> {
    let listing_cards = ListingCardDisplay::all(db)
        .await
        .map_err(|_| "failed to update market name.")?;

    let admin_settings = AdminSettings::single(db, AdminSettings::get_default())
        .await
        .map_err(|_| "failed to update market name.")?;

    Ok((listing_cards, admin_settings))
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
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "listingsindex",
        Context::raw(db, flash, user, admin_user).await,
    )
}

pub fn listings_stage() -> AdHoc {
    AdHoc::on_ignite("Listings Stage", |rocket| async {
        rocket.mount("/", routes![index])
        // .mount("/listing", routes![new])
    })
}
