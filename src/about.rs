use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::AdminSettings;
use rocket::fairing::AdHoc;
use rocket::request::FlashMessage;
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
    admin_settings: AdminSettings,
    lightning_node_pubkey: String,
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
        config: &Config,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let admin_settings = AdminSettings::single(&mut db)
            .await
            .map_err(|_| "failed to update market name.")?;
        let lightning_node_pubkey = get_lightning_node_pubkey(config)
            .await
            .unwrap_or_else(|_| "".to_string());
        Ok(Context {
            base_context,
            flash,
            admin_settings,
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
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, flash, user, admin_user, config.inner())
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("about", context))
}

pub fn about_stage() -> AdHoc {
    AdHoc::on_ignite("About Stage", |rocket| async {
        rocket.mount("/about", routes![index])
        // .mount("/listing", routes![new])
    })
}
