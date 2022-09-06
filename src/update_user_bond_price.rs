use crate::base::BaseContext;
use crate::db::Db;
use crate::models::{AdminSettings, UserBondPriceInput};
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

#[post("/change", data = "<user_bond_price_form>")]
async fn update(
    user_bond_price_form: Form<UserBondPriceInput>,
    mut db: Connection<Db>,
    _user: User,
    _admin_user: AdminUser,
) -> Flash<Redirect> {
    let user_bond_price_input = user_bond_price_form.into_inner();
    let new_user_bond_price_basis_points = user_bond_price_input.user_bond_price_sat.unwrap_or(1);

    match change_user_bond_price(new_user_bond_price_basis_points, &mut db).await {
        Ok(_) => Flash::success(
            Redirect::to(uri!("/update_user_bond_price", index())),
            "User bond price successfully updated.",
        ),
        Err(e) => Flash::error(Redirect::to(uri!("/update_user_bond_price", index())), e),
    }
}

async fn change_user_bond_price(
    new_user_bond_price_basis_points: u64,
    db: &mut Connection<Db>,
) -> Result<(), String> {
    if new_user_bond_price_basis_points == 0 {
        return Err("Use bond price must be positive.".to_string());
    };

    AdminSettings::set_user_bond_price(db, new_user_bond_price_basis_points)
        .await
        .map_err(|_| "failed to update user bond price.")?;

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
    Ok(Template::render("updateuserbondprice", context))
}

pub fn update_user_bond_price_stage() -> AdHoc {
    AdHoc::on_ignite("Update User Bond Price Stage", |rocket| async {
        rocket.mount("/update_user_bond_price", routes![index, update])
    })
}
