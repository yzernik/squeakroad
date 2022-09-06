use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AdminSettings, SqueaknodeInfoInput};
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

#[post("/change", data = "<squeaknode_info_form>")]
async fn update(
    squeaknode_info_form: Form<SqueaknodeInfoInput>,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Flash<Redirect> {
    let squeaknode_info = squeaknode_info_form.into_inner();

    match change_squeaknode_info(squeaknode_info, &mut db).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_squeaknode_info", index())),
            "Squeaknode info successfully updated.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_squeaknode_info", index())), e),
    }
}

async fn change_squeaknode_info(
    squeaknode_info: SqueaknodeInfoInput,
    db: &mut Connection<Db>,
) -> Result<(), String> {
    let new_squeaknode_pubkey = squeaknode_info.squeaknode_pubkey;
    let new_squeaknode_address = squeaknode_info.squeaknode_address;

    if new_squeaknode_pubkey.len() != 64 {
        return Err("Pubkey is not valid.".to_string());
    };
    if new_squeaknode_address.len() > 128 {
        return Err("Address is too long.".to_string());
    };

    AdminSettings::set_squeaknode_pubkey(db, &new_squeaknode_pubkey)
        .await
        .map_err(|_| "failed to update squeaknode pubkey.")?;
    AdminSettings::set_squeaknode_address(db, &new_squeaknode_address)
        .await
        .map_err(|_| "failed to update squeaknode address.")?;

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
    Ok(Template::render("updatesqueaknodeinfo", context))
}

pub fn update_squeaknode_info_stage() -> AdHoc {
    AdHoc::on_ignite("Update Squeaknode Stage", |rocket| async {
        rocket.mount("/update_squeaknode_info", routes![index, update])
    })
}
