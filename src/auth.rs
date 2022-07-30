use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{AdminSettings, UserAccount};
use crate::util;
use rocket::fairing::AdHoc;
use rocket::State;
use rocket::{form::*, get, post, response::Redirect, routes};
use rocket_auth::Users;
use rocket_auth::{Auth, Login, Signup, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use serde_json::json;
use std::result::Result;

#[catch(401)]
fn not_authorized() -> Redirect {
    Redirect::to(uri!("/activate_account"))
}

#[get("/login")]
async fn get_login(mut db: Connection<Db>, user: Option<User>) -> Result<Template, String> {
    let base_context = BaseContext::raw(&mut db, user.clone(), None)
        .await
        .map_err(|_| "failed to get base template.")?;
    Ok(Template::render(
        "login",
        json!({ "base_context": base_context }),
    ))
}

#[post("/login", data = "<form>")]
async fn post_login(auth: Auth<'_>, form: Form<Login>) -> Result<Redirect, String> {
    auth.login(&form).await.map_err(|_| "failed to login.")?;
    Ok(Redirect::to("/"))
}

#[get("/signup")]
async fn get_signup(mut db: Connection<Db>, user: Option<User>) -> Result<Template, String> {
    let base_context = BaseContext::raw(&mut db, user.clone(), None)
        .await
        .map_err(|_| "failed to get base template.")?;
    Ok(Template::render(
        "signup",
        json!({ "base_context": base_context }),
    ))
}

#[post("/signup", data = "<form>")]
async fn post_signup(
    auth: Auth<'_>,
    form: Form<Signup>,
    mut db: Connection<Db>,
    config: &State<Config>,
    users: &State<Users>,
) -> Result<Redirect, String> {
    auth.signup(&form)
        .await
        .map_err(|_| "failed to signup user.")?;
    auth.login(&form.clone().into())
        .await
        .map_err(|_| "failed to signup user.")?;

    // Get the new user and create a market account
    let signup: Signup = form.into_inner();
    let user = users
        .get_by_email(&signup.email.to_lowercase())
        .await
        .unwrap();
    create_user_account(&mut db, user, config.inner().clone())
        .await
        .map_err(|_| "failed to create new user account.")?;

    Ok(Redirect::to("/activate_account"))
}

#[get("/logout")]
async fn logout(auth: Auth<'_>, mut db: Connection<Db>) -> Result<Template, String> {
    auth.logout().map_err(|_| "failed to logout.")?;
    let base_context = BaseContext::raw(&mut db, None, None)
        .await
        .map_err(|_| "failed to get base template.")?;
    Ok(Template::render(
        "logout",
        json!({ "base_context": base_context }),
    ))
}

async fn create_user_account(
    db: &mut Connection<Db>,
    user: User,
    config: Config,
) -> Result<(), String> {
    let now = util::current_time_millis();

    let admin_settings = AdminSettings::single(db)
        .await
        .map_err(|_| "failed to update market name.")?;
    let amount_owed_sat: u64 = admin_settings.user_bond_price_sat;

    let mut lightning_client = lightning::get_lnd_lightning_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .expect("failed to get lightning client");
    let invoice = lightning_client
        .add_invoice(tonic_openssl_lnd::lnrpc::Invoice {
            value_msat: (amount_owed_sat as i64) * 1000,
            ..Default::default()
        })
        .await
        .expect("failed to get new invoice")
        .into_inner();

    let user_account = UserAccount {
        id: None,
        public_id: util::create_uuid(),
        user_id: user.id(),
        amount_owed_sat,
        paid: false,
        disabled: false,
        invoice_hash: util::to_hex(&invoice.r_hash),
        invoice_payment_request: invoice.payment_request,
        created_time_ms: now,
        payment_time_ms: 0,
    };

    UserAccount::insert(user_account, db)
        .await
        .map_err(|_| "failed to insert user account.")?;

    Ok(())
}

pub fn auth_stage() -> AdHoc {
    AdHoc::on_ignite("Auth Stage", |rocket| async {
        rocket.register("/", catchers![not_authorized]).mount(
            "/",
            routes![get_login, post_signup, get_signup, post_login, logout,],
        )
    })
}
