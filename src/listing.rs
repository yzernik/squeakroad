use crate::db::Db;
use crate::models::{AdminSettings, Listing, ListingDisplay, ShippingOption};
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::status::NotFound;
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
        listing_id: &str,
        shipping_option_id: Option<&str>,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let listing_display = ListingDisplay::single_by_public_id(&mut db, listing_id)
            .await
            .map_err(|_| "failed to get admin settings.")?;
        let maybe_shipping_option = match shipping_option_id {
            Some(sid) => {
                let shipping_option = ShippingOption::single_by_public_id(&mut db, sid).await;
                shipping_option.ok()
            }
            None => None,
        };
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;

        Ok(Context {
            flash,
            listing_display: Some(listing_display),
            selected_shipping_option: maybe_shipping_option,
            user,
            admin_user,
            admin_settings: Some(admin_settings),
        })
    }
}

#[put("/<id>/submit")]
async fn submit(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<String, NotFound<String>> {
    // match Task::toggle_with_id(id, &mut db).await {
    //     Ok(_) => Ok(Redirect::to("/")),
    //     Err(e) => {
    //         error_!("DB toggle({}) error: {}", id, e);
    //         Err(Template::render(
    //             "index",
    //             Context::err(db, "Failed to toggle task.", Some(user)).await,
    //         ))
    //     }
    // }

    println!("Handling submit endpoint for {:?}", id);

    Ok("foo".to_string())
}

async fn submit_listing(id: &str, db: &mut Connection<Db>, user: User) -> Result<(), String> {
    let listing = Listing::single_by_public_id(db, id)
        .await
        .map_err(|_| "failed to get listing")?;

    if listing.approved {
        Err("Listing is already approved.".to_string())
    } else if !listing.submitted {
        Err("Listing has not been submitted.".to_string())
    } else {
        // let shipping_option = ShippingOption {
        //     id: None,
        //     public_id: Uuid::new_v4().to_string(),
        //     listing_id: listing.id.unwrap(),
        //     title: title,
        //     description: description,
        //     price_sat: price_sat,
        // };

        // ShippingOption::insert(shipping_option, db)
        //     .await
        //     .map_err(|_| "failed to save shipping option.")?;

        Ok(())
    }
}

#[put("/<id>/approve")]
async fn approve(
    id: &str,
    mut db: Connection<Db>,
    user: User,
    admin_user: AdminUser,
) -> Result<String, NotFound<String>> {
    // match Task::toggle_with_id(id, &mut db).await {
    //     Ok(_) => Ok(Redirect::to("/")),
    //     Err(e) => {
    //         error_!("DB toggle({}) error: {}", id, e);
    //         Err(Template::render(
    //             "index",
    //             Context::err(db, "Failed to toggle task.", Some(user)).await,
    //         ))
    //     }
    // }

    println!("Handling approve endpoint for {:?}", id);

    Ok("foo".to_string())
}

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
    id: &str,
    shipping_option_id: Option<&str>,
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
        rocket.mount("/listing", routes![index, submit, approve])
    })
}
