use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{Listing, Order, ReviewInput, RocketAuthUser, ShippingOption, UserAccount};
use crate::util;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket::serde::Serialize;
use rocket::State;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    user_account: UserAccount,
    user: User,
    admin_user: Option<AdminUser>,
    qr_svg_base64: String,
    lightning_node_pubkey: String,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: User,
        admin_user: Option<AdminUser>,
        config: &Config,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let user_account = UserAccount::single(&mut db, user.id())
            .await
            .map_err(|_| "failed to get user account.")?;
        let qr_svg_bytes = util::generate_qr(&user_account.invoice_payment_request);
        let qr_svg_base64 = util::to_base64(&qr_svg_bytes);
        let lightning_node_pubkey = get_lightning_node_pubkey(config)
            .await
            .unwrap_or_else(|_| "".to_string());
        Ok(Context {
            base_context,
            flash,
            user_account,
            user,
            admin_user,
            qr_svg_base64,
            lightning_node_pubkey,
        })
    }
}

async fn get_lightning_node_pubkey(config: &Config) -> Result<String, String> {
    let mut lightning_client = lightning::get_lnd_lightning_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .expect("failed to get lightning client");
    let get_info_resp = lightning_client
        // All calls require at least empty parameter
        .get_info(tonic_openssl_lnd::lnrpc::GetInfoRequest {})
        .await
        .expect("failed to get lightning node info")
        .into_inner();
    Ok(get_info_resp.identity_pubkey)
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
    config: &State<Config>,
) -> Result<Template, Redirect> {
    let flash = flash.map(FlashMessage::into_inner);
    match user {
        Some(user) => Ok(Template::render(
            "activateaccount",
            Context::raw(db, flash, user, admin_user, config).await,
        )),
        None => Err(Redirect::to(uri!("/login"))),
    }
}

pub fn activate_account_stage() -> AdHoc {
    AdHoc::on_ignite("Activate_Account Stage", |rocket| async {
        rocket.mount("/activate_account", routes![index])
    })
}
