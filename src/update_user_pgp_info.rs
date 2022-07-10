use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{PGPInfoInput, UserSettings};
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
    user_settings: UserSettings,
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
        let user_settings = UserSettings::single(&mut db, user.id(), UserSettings::default())
            .await
            .map_err(|_| "failed to get user settings.")?;

        Ok(Context {
            base_context,
            flash,
            user_settings,
        })
    }
}

#[post("/change", data = "<pgp_info_form>")]
async fn update(
    pgp_info_form: Form<PGPInfoInput>,
    mut db: Connection<Db>,
    user: User,
    _admin_user: Option<AdminUser>,
) -> Flash<Redirect> {
    let pgp_info = pgp_info_form.into_inner();

    match change_pgp_info(user, pgp_info, &mut db).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_user_pgp_info", index())),
            "PGP info successfully updated.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_user_pgp_info", index())), e),
    }
}

async fn change_pgp_info(
    user: User,
    pgp_info: PGPInfoInput,
    db: &mut Connection<Db>,
) -> Result<(), String> {
    let new_pgp_key_id = pgp_info.pgp_key_id;

    if new_pgp_key_id.len() != 64 {
        Err("PGP key id is not valid.".to_string())
    } else {
        let default_user_settings = UserSettings::default();
        UserSettings::set_pgp_key_id(
            db,
            user.id(),
            &new_pgp_key_id,
            default_user_settings.clone(),
        )
        .await
        .map_err(|_| "failed to update pgp key id.")?;

        Ok(())
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "updateuserpgpinfo",
        Context::raw(db, flash, user, admin_user).await,
    )
}

pub fn update_user_pgp_info_stage() -> AdHoc {
    AdHoc::on_ignite("Update User PGP Stage", |rocket| async {
        rocket.mount("/update_user_pgp_info", routes![index, update])
    })
}
