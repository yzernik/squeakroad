use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{AccountInfo, Withdrawal, WithdrawalInfo};
use crate::user_account::ActiveUser;
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

const MAX_WITHDRAWALS_PER_USER_PER_DAY: u32 = 5;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    account_balance_sat: i64,
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
        let account_info = AccountInfo::account_info_for_user(&mut db, user.id)
            .await
            .map_err(|_| "failed to get account info.")?;
        let account_balance_sat = account_info.account_balance_sat;
        Ok(Context {
            base_context,
            flash,
            account_balance_sat,
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
    match withdraw(
        withdrawal_info.clone(),
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
            Err(Flash::error(Redirect::to(uri!("/withdraw", index())), e))
        }
    }
}

async fn withdraw(
    withdrawal_info: WithdrawalInfo,
    db: &mut Connection<Db>,
    user: User,
    config: Config,
) -> Result<(), String> {
    let now = util::current_time_millis();
    let one_day_in_ms = 24 * 60 * 60 * 1000;

    if withdrawal_info.invoice_payment_request.is_empty() {
        return Err("Invoice payment request cannot be empty.".to_string());
    };
    if user.is_admin {
        return Err("Admin user cannot withdraw funds.".to_string());
    }

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
    let withdrawal = Withdrawal {
        id: None,
        public_id: util::create_uuid(),
        user_id: user.id(),
        amount_sat,
        invoice_hash: "".to_string(),
        invoice_payment_request: invoice_payment_request.clone(),
        created_time_ms: now,
    };
    let send_withdrawal_funds_ret = send_withdrawal_funds(invoice_payment_request, config);
    Withdrawal::do_withdrawal(
        withdrawal,
        db,
        send_withdrawal_funds_ret,
        MAX_WITHDRAWALS_PER_USER_PER_DAY,
        now - one_day_in_ms,
    )
    .await
    .map_err(|e| {
        error_!("Failed withdrawal: {}", e);
        e
    })?;

    Ok(())
}

async fn send_withdrawal_funds(
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
) -> Result<Template, String> {
    let flash = flash.map(FlashMessage::into_inner);
    let context = Context::raw(db, flash, active_user.user, admin_user)
        .await
        .map_err(|_| "failed to get template context.")?;
    Ok(Template::render("withdraw", context))
}

pub fn withdraw_stage() -> AdHoc {
    AdHoc::on_ignite("Withdraw Stage", |rocket| async {
        rocket.mount("/withdraw", routes![index, new])
    })
}
