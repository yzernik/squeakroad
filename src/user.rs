use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AdminSettings, ListingCardDisplay, RocketAuthUser};
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
    base_context: BaseContext,
    flash: Option<(String, String)>,
    visited_user: Option<RocketAuthUser>,
    listing_cards: Vec<ListingCardDisplay>,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        username: String,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let visited_user = RocketAuthUser::single_by_username(&mut db, username)
            .await
            .map_err(|_| "failed to get visited user.")?;
        let listing_cards =
            ListingCardDisplay::all_approved_for_user(&mut db, visited_user.id.unwrap())
                .await
                .map_err(|_| "failed to get approved listings.")?;
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;

        Ok(Context {
            base_context,
            flash,
            visited_user: Some(visited_user),
            listing_cards: listing_cards,
        })
    }
}

#[get("/<username>")]
async fn index(
    username: &str,
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "user",
        Context::raw(db, username.to_string(), flash, user, admin_user).await,
    )
}

pub fn user_stage() -> AdHoc {
    AdHoc::on_ignite("User Stage", |rocket| async {
        rocket.mount("/user", routes![index])
    })
}
