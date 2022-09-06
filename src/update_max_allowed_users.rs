use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AdminSettings, MaxAllowedUsersInput};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    admin_settings: AdminSettings,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let admin_settings = AdminSettings::single(&mut db)
            .await
            .map_err(|_| "failed to get admin settings.")?;
        Ok(Context {
            base_context,
            flash,
            admin_settings,
        })
    }
}

#[post("/change", data = "<max_allowed_users_form>")]
async fn update(
    max_allowed_users_form: Form<MaxAllowedUsersInput>,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Flash<Redirect> {
    let max_allowed_users_input = max_allowed_users_form.into_inner();
    let new_max_allowed_users = max_allowed_users_input.max_allowed_users.unwrap_or(0);

    match change_max_allowed_users(new_max_allowed_users, &mut db).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_max_allowed_users", index())),
            "max allowed users successfully updated.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_max_allowed_users", index())), e),
    }
}

async fn change_max_allowed_users(
    new_max_allowed_users: u64,
    db: &mut Connection<Db>,
) -> Result<(), String> {
    AdminSettings::set_max_allowed_users(db, new_max_allowed_users)
        .await
        .map_err(|_| "failed to update max allowed users.")?;

    Ok(())
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: User,
    admin_user: AdminUser,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, flash, user, Some(admin_user))
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("updatemaxallowedusers", context))
}

pub fn update_max_allowed_users_stage() -> AdHoc {
    AdHoc::on_ignite("Update Max Allowed Users Stage", |rocket| async {
        rocket
            // .mount("/update_listing_images", routes![index, new])
            .mount("/update_max_allowed_users", routes![index, update])
    })
}
