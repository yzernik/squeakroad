use crate::db::Db;
use crate::models::UserAccount;
use rocket::fairing::AdHoc;
use rocket::response::Redirect;
use rocket_auth::User;
use rocket_db_pools::Connection;

#[get("/")]
async fn index(mut db: Connection<Db>, user: Option<User>) -> Result<Redirect, Redirect> {
    match user {
        Some(user) => {
            let maybe_user_account = UserAccount::single(&mut db, user.id()).await.ok();
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
