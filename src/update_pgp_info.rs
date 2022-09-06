use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AdminSettings, PGPInfoInput};
use pgp::Deserializable;
use pgp::SignedPublicKey;
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

#[post("/change", data = "<pgp_info_form>")]
async fn update(
    pgp_info_form: Form<PGPInfoInput>,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Flash<Redirect> {
    let pgp_info = pgp_info_form.into_inner();

    match change_pgp_info(pgp_info, &mut db).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_pgp_info", index())),
            "PGP info successfully updated.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_pgp_info", index())), e),
    }
}

async fn change_pgp_info(pgp_info: PGPInfoInput, db: &mut Connection<Db>) -> Result<(), String> {
    let new_pgp_key = pgp_info.pgp_key;
    let (key, _headers) =
        SignedPublicKey::from_string(&new_pgp_key).map_err(|_| "Invalid PGP key input.")?;
    let validated_pgp_key = key.to_armored_string(None).unwrap();
    AdminSettings::set_pgp_key(db, &validated_pgp_key)
        .await
        .map_err(|_| "failed to update PGP key id.")?;
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
    Ok(Template::render("updatepgpinfo", context))
}

pub fn update_pgp_info_stage() -> AdHoc {
    AdHoc::on_ignite("Update PGP Stage", |rocket| async {
        rocket.mount("/update_pgp_info", routes![index, update])
    })
}
