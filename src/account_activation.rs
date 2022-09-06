use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::UserAccount;
use crate::util;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
use rocket::response::Redirect;
use rocket::serde::Serialize;
use rocket::State;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_auth::Users;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    user_account: UserAccount,
    maybe_account_user: Option<User>,
    user: User,
    admin_user: Option<AdminUser>,
    qr_svg_base64: String,
    lightning_node_pubkey: String,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        id: &str,
        user: User,
        admin_user: Option<AdminUser>,
        config: &Config,
        users: &Users,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let user_account = UserAccount::single_by_public_id(&mut db, id)
            .await
            .map_err(|_| "failed to get user account.")?;
        let maybe_account_user = users.get_by_id(user_account.user_id).await.ok();
        let qr_svg_bytes = util::generate_qr(&user_account.invoice_payment_request);
        let qr_svg_base64 = util::to_base64(&qr_svg_bytes);
        let lightning_node_pubkey = get_lightning_node_pubkey(config)
            .await
            .unwrap_or_else(|_| "".to_string());
        Ok(Context {
            base_context,
            flash,
            user_account,
            maybe_account_user,
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

#[get("/<id>")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    id: &str,
    db: Connection<Db>,
    user: Option<User>,
    admin_user: Option<AdminUser>,
    config: &State<Config>,
    users: &State<Users>,
) -> Result<Template, Redirect> {
    let flash = flash.map(FlashMessage::into_inner);
    match user {
        Some(user) => {
            let context = Context::raw(db, flash, id, user, admin_user, config, users)
                .await
                .map_err(|_| Redirect::to(uri!("/login")))?;
            Ok(Template::render("accountactivation", context))
        }
        None => Err(Redirect::to(uri!("/login"))),
    }
}

pub fn account_activation_stage() -> AdHoc {
    AdHoc::on_ignite("Account Activation Stage", |rocket| async {
        rocket.mount("/account_activation", routes![index])
    })
}
