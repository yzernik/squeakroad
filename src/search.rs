use crate::base::BaseContext;
use crate::db::Db;
use crate::models::ListingCardDisplay;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::serde::Serialize;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

const PAGE_SIZE: u32 = 10;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    search_text: String,
    listing_cards: Vec<ListingCardDisplay>,
    page_num: u32,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        search_text: String,
        flash: Option<(String, String)>,
        maybe_page_num: Option<u32>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let page_num = maybe_page_num.unwrap_or(1);
        let listing_cards = ListingCardDisplay::all_active_for_search_text(
            &mut db,
            &search_text,
            PAGE_SIZE,
            page_num,
        )
        .await
        .map_err(|_| "failed to get approved listings.")?;
        Ok(Context {
            base_context,
            flash,
            search_text,
            listing_cards,
            page_num,
        })
    }
}

#[get("/?<search_text>&<page_num>")]
async fn index(
    search_text: &str,
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    page_num: Option<u32>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(
        db,
        search_text.to_string(),
        flash,
        page_num,
        user,
        admin_user,
    )
    .await
    .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("search", context))
}

pub fn search_stage() -> AdHoc {
    AdHoc::on_ignite("Search Stage", |rocket| async {
        rocket.mount("/search", routes![index])
    })
}
