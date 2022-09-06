use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{ListingCardDisplay, Order, RocketAuthUser, UserAccount};
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::response::Redirect;
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
    visited_user: RocketAuthUser,
    visited_user_account: UserAccount,
    weighted_average_rating: f32,
    listing_cards: Vec<ListingCardDisplay>,
    admin_user: Option<AdminUser>,
    page_num: u32,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        username: String,
        flash: Option<(String, String)>,
        maybe_page_num: Option<u32>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let visited_user = RocketAuthUser::single_by_username(&mut db, username)
            .await
            .map_err(|_| "failed to get visited user.")?;
        let visited_user_account = UserAccount::single(&mut db, visited_user.id.unwrap())
            .await
            .map_err(|_| "failed to get user account.")?;
        let page_num = maybe_page_num.unwrap_or(1);
        let listing_cards = ListingCardDisplay::all_active_for_user(
            &mut db,
            visited_user.id.unwrap(),
            PAGE_SIZE,
            page_num,
        )
        .await
        .map_err(|_| "failed to get approved listings.")?;
        let seller_info = Order::seller_info_for_user(&mut db, visited_user.id.unwrap())
            .await
            .map_err(|_| "failed to get weighted average rating for user.")?;
        let weighted_average_rating = seller_info.weighted_average_rating;
        Ok(Context {
            base_context,
            flash,
            visited_user,
            visited_user_account,
            weighted_average_rating,
            listing_cards,
            admin_user,
            page_num,
        })
    }
}

#[put("/<username>/disable")]
async fn disable(
    username: &str,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match disable_user(&mut db, username).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "user", username)),
            "User disabled by admin".to_string(),
        )),
        Err(e) => {
            error_!("Mark as disabled error({}) error: {}", username, e);
            Err(Flash::error(
                Redirect::to(format!("/{}/{}", "user", username)),
                e,
            ))
        }
    }
}

async fn disable_user(db: &mut Connection<Db>, username: &str) -> Result<(), String> {
    let rocket_auth_user = RocketAuthUser::single_by_username(db, username.to_string())
        .await
        .map_err(|_| "failed to get user")?;
    UserAccount::mark_as_disabled(db, rocket_auth_user.id.unwrap())
        .await
        .map_err(|_| "failed to disable user account.")?;
    Ok(())
}

#[put("/<username>/enable")]
async fn enable(
    username: &str,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match enable_user(&mut db, username).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "user", username)),
            "User enabled by admin".to_string(),
        )),
        Err(e) => {
            error_!("Mark as enabled error({}) error: {}", username, e);
            Err(Flash::error(
                Redirect::to(format!("/{}/{}", "user", username)),
                e,
            ))
        }
    }
}

async fn enable_user(db: &mut Connection<Db>, username: &str) -> Result<(), String> {
    let rocket_auth_user = RocketAuthUser::single_by_username(db, username.to_string())
        .await
        .map_err(|_| "failed to get user")?;
    UserAccount::mark_as_enabled(db, rocket_auth_user.id.unwrap())
        .await
        .map_err(|_| "failed to enable user account.")?;
    Ok(())
}

#[get("/<username>?<page_num>")]
async fn index(
    username: &str,
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    page_num: Option<u32>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, username.to_string(), flash, page_num, user, admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("user", context))
}

pub fn user_stage() -> AdHoc {
    AdHoc::on_ignite("User Stage", |rocket| async {
        rocket.mount("/user", routes![index, disable, enable])
    })
}
