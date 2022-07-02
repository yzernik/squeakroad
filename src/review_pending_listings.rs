use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AdminSettings, ListingCardDisplay};
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::status::NotFound;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    listing_cards: Vec<ListingCardDisplay>,
}

impl Context {
    pub async fn raw(
        flash: Option<(String, String)>,
        mut db: Connection<Db>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let listing_cards = ListingCardDisplay::all_pending(&mut db)
            .await
            .map_err(|_| "failed to get pending listings.")?;
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;
        println!("Pending listings: {:?}", listing_cards);
        Ok(Context {
            base_context,
            flash,
            listing_cards,
        })
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: AdminUser,
) -> Result<Template, NotFound<String>> {
    let flash = flash.map(FlashMessage::into_inner);
    Ok(Template::render(
        "reviewpendinglistings",
        Context::raw(flash, db, user, Some(admin_user)).await,
    ))
}

pub fn review_pending_listings_stage() -> AdHoc {
    AdHoc::on_ignite("Pending Listings Stage", |rocket| async {
        rocket.mount("/review_pending_listings", routes![index])
        // .mount("/listing", routes![new])
    })
}
