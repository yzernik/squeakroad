use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::UserAccount;
use crate::util;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::Redirect;
use rocket::serde::Serialize;
use rocket::State;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    mut db: Connection<Db>,
    user: Option<User>,
) -> Result<Redirect, Redirect> {
    let flash = flash.map(FlashMessage::into_inner);
    match user {
        Some(user) => {
            let maybe_user_account = UserAccount::single(&mut db, user.id()).await.ok();
            println!("maybe_user_account: {:?}", maybe_user_account);
            match maybe_user_account {
                Some(user_account) => Ok(Redirect::to(format!(
                    "/{}/{}",
                    "account_activation", user_account.public_id
                ))),
                None => Err(Redirect::to(uri!("/login"))),
            }
        }
        None => Err(Redirect::to(uri!("/login"))),
    }
}

pub fn activate_account_stage() -> AdHoc {
    AdHoc::on_ignite("Activate_Account Stage", |rocket| async {
        rocket.mount("/activate_account", routes![index])
    })
}
