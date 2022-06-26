use crate::db::Db;
use crate::models::{AdminSettings, MarketNameInput};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::uuid::Uuid;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

// const DEFAULT_ADMIN_SETTINGS: AdminSettings = AdminSettings {
//     id: None,
//     market_name: "default market name",
//     fee_rate_basis_points: 500,
// };

impl AdminSettings {
    pub fn get_default() -> AdminSettings {
        AdminSettings {
            id: None,
            market_name: "default market name".to_string(),
            fee_rate_basis_points: 500,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    admin_settings: Option<AdminSettings>,
    user: User,
    admin_user: Option<AdminUser>,
}

impl Context {
    pub async fn err<M: std::fmt::Display>(
        _db: Connection<Db>,
        msg: M,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Context {
        // TODO: get the listing display and put in context.
        Context {
            flash: Some(("error".into(), msg.to_string())),
            admin_settings: None,
            user: user,
            admin_user,
        }
    }

    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Context {
        let default_admin_settings = AdminSettings::get_default();
        match AdminSettings::single(&mut db, default_admin_settings).await {
            Ok(admin_settings) => Context {
                flash,
                admin_settings: Some(admin_settings),
                user,
                admin_user,
            },
            Err(e) => {
                error_!("DB AdminSettings::single() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Fail to access database.".into())),
                    admin_settings: None,
                    user: user,
                    admin_user: admin_user,
                }
            }
        }
    }
}

#[post("/change", data = "<market_name_form>")]
async fn update(
    market_name_form: Form<MarketNameInput>,
    mut db: Connection<Db>,
    user: User,
    admin_user: AdminUser,
) -> Flash<Redirect> {
    let market_name_input = market_name_form.into_inner();
    let new_market_name = market_name_input.market_name;

    match change_market_name(new_market_name, &mut db).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_market_name", index())),
            "Market name successfully added.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_market_name", index())), e),
    }

    // Flash::error(
    //     Redirect::to(uri!("/add_shipping_options", index(id))),
    //     "not implemented".to_string(),
    // )
}

async fn change_market_name(
    new_market_name: String,
    db: &mut Connection<Db>,
) -> Result<(), String> {
    if new_market_name.len() >= 64 {
        Err("Market name is too long.".to_string())
    } else {
        let default_admin_settings = AdminSettings::get_default();
        AdminSettings::set_market_name(db, &new_market_name, default_admin_settings)
            .await
            .map_err(|_| "failed to update market name.")?;

        Ok(())
    }
}

// #[delete("/<id>/add_shipping_option/<shipping_option_id>")]
// async fn delete(
//     id: &str,
//     shipping_option_id: &str,
//     mut db: Connection<Db>,
//     user: User,
//     admin_user: Option<AdminUser>,
// ) -> Result<Flash<Redirect>, Template> {
//     match delete_shipping_option(
//         id,
//         shipping_option_id,
//         &mut db,
//         user.clone(),
//         admin_user.clone(),
//     )
//     .await
//     {
//         Ok(_) => Ok(Flash::success(
//             Redirect::to(uri!("/add_shipping_options", index(id))),
//             "Shipping option was deleted.",
//         )),
//         Err(e) => {
//             error_!("DB deletion({}) error: {}", id, e);
//             Err(Template::render(
//                 "addshippingoptions",
//                 Context::err(
//                     db,
//                     id,
//                     "Failed to delete shipping option.",
//                     user,
//                     admin_user,
//                 )
//                 .await,
//             ))
//         }
//     }
// }

// async fn delete_shipping_option(
//     listing_id: &str,
//     shipping_option_id: &str,
//     db: &mut Connection<Db>,
//     user: User,
//     _admin_user: Option<AdminUser>,
// ) -> Result<(), String> {
//     let listing = Listing::single_by_public_id(&mut *db, listing_id)
//         .await
//         .map_err(|_| "failed to get listing")?;
//     let shipping_option = ShippingOption::single_by_public_id(&mut *db, shipping_option_id)
//         .await
//         .map_err(|_| "failed to get shipping option")?;

//     if shipping_option.listing_id != listing.id.unwrap() {
//         Err("Invalid listing id given.".to_string())
//     } else if listing.submitted {
//         Err("Listing is already submitted.".to_string())
//     } else if listing.user_id != user.id() {
//         Err("Listing belongs to a different user.".to_string())
//     } else {
//         match ShippingOption::delete_with_public_id(shipping_option_id, &mut *db).await {
//             Ok(num_deleted) => {
//                 if num_deleted > 0 {
//                     Ok(())
//                 } else {
//                     Err("No shipping options deleted.".to_string())
//                 }
//             }
//             Err(_) => Err("failed to delete shipping option.".to_string()),
//         }
//     }
// }

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: User,
    admin_user: AdminUser,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "updatemarketname",
        Context::raw(db, flash, user, Some(admin_user)).await,
    )
}

pub fn update_market_name_stage() -> AdHoc {
    AdHoc::on_ignite("Update Market Name Stage", |rocket| async {
        rocket
            // .mount("/add_listing_images", routes![index, new])
            .mount("/update_market_name", routes![index, update])
    })
}
