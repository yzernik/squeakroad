use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{OrderMessage, OrderMessageCard};
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
    order_message_cards: Vec<OrderMessageCard>,
    page_num: u32,
}

impl Context {
    pub async fn raw(
        flash: Option<(String, String)>,
        mut db: Connection<Db>,
        maybe_page_num: Option<u32>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let page_num = maybe_page_num.unwrap_or(1);
        let order_message_cards =
            OrderMessage::all_unread_for_recipient(&mut db, user.id, PAGE_SIZE, page_num)
                .await
                .map_err(|_| "failed to get unread messages.")?;
        Ok(Context {
            base_context,
            flash,
            order_message_cards,
            page_num,
        })
    }
}

#[get("/?<page_num>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    page_num: Option<u32>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Result<Template, NotFound<String>> {
    let flash = flash.map(FlashMessage::into_inner);
    Ok(Template::render(
        "myunreadmessages",
        Context::raw(flash, db, page_num, user, admin_user).await,
    ))
}

pub fn my_unread_messages_stage() -> AdHoc {
    AdHoc::on_ignite("My Unread Messages Stage", |rocket| async {
        rocket.mount("/my_unread_messages", routes![index])
        // .mount("/listing", routes![new])
    })
}
