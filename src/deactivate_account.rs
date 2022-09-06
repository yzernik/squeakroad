use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{UserAccount, WithdrawalInfo};
use crate::user_account::ActiveUser;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::Flash;
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
}

impl Context {
    pub async fn raw(
        mut db: Connection<Db>,
        flash: Option<(String, String)>,
        user: User,
        user_account: UserAccount,
        admin_user: Option<AdminUser>,
        users: &Users,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, Some(user.clone()), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let maybe_account_user = users.get_by_id(user_account.user_id).await.ok();
        Ok(Context {
            base_context,
            flash,
            user_account,
            maybe_account_user,
            user,
            admin_user,
        })
    }
}

#[post("/new", data = "<withdrawal_form>")]
async fn new(
    withdrawal_form: Form<WithdrawalInfo>,
    mut db: Connection<Db>,
    active_user: ActiveUser,
    _admin_user: Option<AdminUser>,
    config: &State<Config>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let withdrawal_info = withdrawal_form.into_inner();
    match withdraw_account_deactivation_funds(
        withdrawal_info.clone(),
        active_user.user_account,
        &mut db,
        active_user.user.clone(),
        config.inner().clone(),
    )
    .await
    {
        Ok(_) => Ok(Flash::success(
            Redirect::to("/my_account_balance"),
            "Funds successfully withdrawn.",
        )),
        Err(e) => {
            error_!("Withdrawal error: {}", e);
            Err(Flash::error(
                Redirect::to(uri!("/deactivate_account", index())),
                e,
            ))
        }
    }
}

async fn withdraw_account_deactivation_funds(
    withdrawal_info: WithdrawalInfo,
    user_account: UserAccount,
    db: &mut Connection<Db>,
    _user: User,
    config: Config,
) -> Result<(), String> {
    if withdrawal_info.invoice_payment_request.is_empty() {
        return Err("Invoice payment request cannot be empty.".to_string());
    };

    let mut lightning_client = lightning::get_lnd_lightning_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .expect("failed to get lightning client");
    let decoded_pay_req = lightning_client
        .decode_pay_req(tonic_openssl_lnd::lnrpc::PayReqString {
            pay_req: withdrawal_info.invoice_payment_request.clone(),
        })
        .await
        .map_err(|_| "failed to decode payment request string.")?
        .into_inner();
    let amount_sat: u64 = decoded_pay_req.num_satoshis.try_into().unwrap();
    let invoice_payment_request = withdrawal_info.invoice_payment_request;
    let send_deactivation_funds_ret =
        send_account_deactivation_funds(invoice_payment_request, config);
    UserAccount::do_deactivation(amount_sat, user_account, db, send_deactivation_funds_ret)
        .await
        .map_err(|e| {
            error_!("Failed deactivation: {}", e);
            e
        })?;

    Ok(())
}

async fn send_account_deactivation_funds(
    invoice_payment_request: String,
    config: Config,
) -> Result<tonic_openssl_lnd::lnrpc::SendResponse, String> {
    let mut lightning_client = lightning::get_lnd_lightning_client(
        config.lnd_host.clone(),
        config.lnd_port,
        config.lnd_tls_cert_path.clone(),
        config.lnd_macaroon_path.clone(),
    )
    .await
    .expect("failed to get lightning client");
    let send_response = lightning_client
        .send_payment_sync(tonic_openssl_lnd::lnrpc::SendRequest {
            payment_request: invoice_payment_request,
            ..Default::default()
        })
        .await
        .map_err(|e| format!("failed to send payment: {:?}", e))?
        .into_inner();
    if send_response.payment_preimage.is_empty() {
        return Err(format!(
            "Send Payment failure: {:?}.",
            send_response.payment_error
        ));
    }
    Ok(send_response)
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
    users: &State<Users>,
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(
        db,
        flash,
        active_user.user,
        active_user.user_account,
        admin_user,
        users,
    )
    .await
    .map_err(|_| "failed to get template context.")?;

    Ok(Template::render("deactivateaccount", context))
}

pub fn deactivate_account_stage() -> AdHoc {
    AdHoc::on_ignite("Deactivate Account Stage", |rocket| async {
        rocket.mount("/deactivate_account", routes![index, new])
    })
}
