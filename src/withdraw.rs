use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{AccountInfo, Withdrawal, WithdrawalInfo};
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
    user: User,
    _admin_user: Option<AdminUser>,
    config: &State<Config>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let withdrawal_info = withdrawal_form.into_inner();
    match withdraw(
        withdrawal_info.clone(),
        &mut db,
        user.clone(),
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
) -> Result<String, String> {
    if withdrawal_info.invoice_payment_request.is_empty() {
        Err("Invoice payment request cannot be empty.".to_string())
    } else if user.is_admin {
        Err("Admin user cannot withdraw funds.".to_string())
    } else {
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
        let account_info = AccountInfo::account_info_for_user(db, user.id)
            .await
            .map_err(|_| "failed to get account info.")?;
        let account_balance_sat_u64: u64 = account_info.account_balance_sat.try_into().unwrap();
        if amount_sat > account_balance_sat_u64 {
            Err("Insufficient funds in account.".to_string())
        } else if user.is_admin {
            Err("Admin user cannot withdraw funds.".to_string())
        } else {
            let send_response = lightning_client
                .send_payment_sync(tonic_openssl_lnd::lnrpc::SendRequest {
                    payment_request: withdrawal_info.invoice_payment_request.clone(),
                    ..Default::default()
                })
                .await
                .map_err(|e| format!("failed to send payment: {:?}", e))?
                .into_inner();
            let now = util::current_time_millis();
            let withdrawal = Withdrawal {
                id: None,
                public_id: util::create_uuid(),
                user_id: user.id(),
                amount_sat,
                invoice_hash: util::to_hex(&send_response.payment_hash),
                invoice_payment_request: withdrawal_info.invoice_payment_request,
                created_time_ms: now,
            };

            match Withdrawal::insert(withdrawal, db).await {
                Ok(_) => Ok("Inserted.".to_string()),
                Err(e) => {
                    error_!("DB insertion error: {}", e);
                    Err("Order could not be inserted due an internal error.".to_string())
                }
            }
        }
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: User,
    admin_user: Option<AdminUser>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("withdraw", Context::raw(db, flash, user, admin_user).await)
}

pub fn withdraw_stage() -> AdHoc {
    AdHoc::on_ignite("Withdraw Stage", |rocket| async {
        rocket.mount("/withdraw", routes![index, new])
    })
}
