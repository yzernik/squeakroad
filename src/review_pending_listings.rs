use crate::base::BaseContext;
use crate::db::Db;
use crate::models::ListingCardDisplay;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

const PAGE_SIZE: u32 = 10;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    listing_cards: Vec<ListingCardDisplay>,
    page_num: u32,
}

impl Context {
    pub async fn raw(
        flash: Option<(String, String)>,
        mut db: Connection<Db>,
        maybe_page_num: Option<u32>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let page_num = maybe_page_num.unwrap_or(1);
        let listing_cards = ListingCardDisplay::all_pending(&mut db, PAGE_SIZE, page_num)
            .await
            .map_err(|_| "failed to get pending listings.")?;
        Ok(Context {
            base_context,
            flash,
            listing_cards,
            page_num,
        })
    }
}

#[get("/?<page_num>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    page_num: Option<u32>,
    user: Option<User>,
    admin_user: AdminUser,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(flash, db, page_num, user, Some(admin_user))
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("reviewpendinglistings", context))
}

pub fn review_pending_listings_stage() -> AdHoc {
    AdHoc::on_ignite("Pending Listings Stage", |rocket| async {
        rocket.mount("/review_pending_listings", routes![index])
        // .mount("/listing", routes![new])
    })
}
