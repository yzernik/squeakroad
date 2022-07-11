use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{Order, SellerInfo};
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::status::NotFound;
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
    seller_infos: Vec<SellerInfo>,
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
        let seller_infos_for_all = Order::seller_info_for_all_users(&mut db)
            .await
            .map_err(|_| "failed to get seller infos for top users.")?;
        // Use db query to take only top N sellers.
        let seller_infos = seller_infos_for_all
            .iter()
            .take(PAGE_SIZE.try_into().unwrap())
            .cloned()
            .collect::<Vec<_>>();
        Ok(Context {
            base_context,
            flash,
            seller_infos,
        })
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
) -> Result<Template, NotFound<String>> {
    let flash = flash.map(FlashMessage::into_inner);
    Ok(Template::render(
        "topsellers",
        Context::raw(flash, db, user, admin_user).await,
    ))
}

pub fn top_sellers_stage() -> AdHoc {
    AdHoc::on_ignite("Top Sellers Stage", |rocket| async {
        rocket.mount("/top_sellers", routes![index])
        // .mount("/listing", routes![new])
    })
}
