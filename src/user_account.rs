use crate::db::Db;
use crate::models::UserAccount;
use rocket::http::Status;
use rocket::outcome::try_outcome;
use rocket::request::{FromRequest, Outcome, Request};
use rocket_auth::User;
use rocket_db_pools::Connection;

#[derive(Debug)]
pub struct ActiveUser {
    //pub user: User,
    //pub user_account: UserAccount,
    pub user: User,
    pub user_account: UserAccount,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ActiveUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<ActiveUser, ()> {
        // This will unconditionally query the database!
        let user_outcome = request
            .guard::<User>()
            .await
            .map_failure(|(status, _)| (status, ()));
        let user = try_outcome!(user_outcome);
        let db_outcome = request
            .guard::<Connection<Db>>()
            .await
            .map_failure(|(status, _)| (status, ()));
        let mut db = try_outcome!(db_outcome);

        // TODO: Query the database for the user account.
        let maybe_user_account = UserAccount::single(&mut db, user.id()).await.ok();

        if let Some(user_account) = maybe_user_account {
            if user_account.paid && !user_account.disabled {
                Outcome::Success(ActiveUser { user, user_account })
            } else {
                Outcome::Failure((Status::Unauthorized, ()))
            }
        } else {
            Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}
