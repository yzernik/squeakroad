use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AdminSettings, FeeRateInput};
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

#[post("/change", data = "<fee_rate_form>")]
async fn update(
    fee_rate_form: Form<FeeRateInput>,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Flash<Redirect> {
    let fee_rate_input = fee_rate_form.into_inner();
    let new_fee_rate_basis_points = fee_rate_input.fee_rate_basis_points.unwrap_or(0);

    match change_fee_rate(new_fee_rate_basis_points, &mut db).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_fee_rate", index())),
            "Fee rate successfully updated.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_fee_rate", index())), e),
    }
}

async fn change_fee_rate(
    new_fee_rate_basis_points: i32,
    db: &mut Connection<Db>,
) -> Result<(), String> {
    if new_fee_rate_basis_points < 0 {
        return Err("Fee rate cannot be negative.".to_string());
    };
    if new_fee_rate_basis_points > 10000 {
        return Err("Fee rate basis points cannot be > 10000.".to_string());
    };

    AdminSettings::set_fee_rate(db, new_fee_rate_basis_points)
        .await
        .map_err(|_| "failed to update fee rate.")?;

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
    Ok(Template::render("updatefeerate", context))
}

pub fn update_fee_rate_stage() -> AdHoc {
    AdHoc::on_ignite("Update Fee Rate Stage", |rocket| async {
        rocket
            // .mount("/update_listing_images", routes![index, new])
            .mount("/update_fee_rate", routes![index, update])
    })
}
