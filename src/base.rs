use crate::db::Db;
use crate::models::{AccountInfo, AdminSettings};
use rocket::serde::Serialize;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct BaseContext {
    pub user: Option<User>,
    pub account_info: Option<AccountInfo>,
    pub admin_user: Option<AdminUser>,
    pub admin_settings: Option<AdminSettings>,
}

impl BaseContext {
    pub async fn raw(
        db: &mut Connection<Db>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<BaseContext, String> {
        let account_info = match user {
            Some(ref u) => Some(
                AccountInfo::account_info_for_user(db, u.id())
                    .await
                    .map_err(|_| "failed to get account info.")?,
            ),
            None => None,
        };
        let admin_settings = AdminSettings::single(db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to get admin settings.")?;
        Ok(BaseContext {
            user,
            account_info,
            admin_user,
            admin_settings: Some(admin_settings),
        })
    }
}
