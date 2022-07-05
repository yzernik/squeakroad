use crate::base::BaseContext;
use crate::config::Config;
use crate::db::Db;
use crate::lightning;
use crate::models::{AccountInfo, Withdrawal, WithdrawalInfo};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket::serde::uuid::Uuid;
use rocket::serde::Serialize;
use rocket::State;
use rocket_auth::AdminUser;
use rocket_auth::User;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

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
    println!("config: {:?}", config);

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
    println!("config: {:?}", config);

    if withdrawal_info.invoice_payment_request.is_empty() {
        Err("Invoice payment request cannot be empty.".to_string())
    } else if user.is_admin {
        Err("Admin user cannot withdraw funds.".to_string())
    } else {
        let mut lighting_client = lightning::get_lnd_client(
            config.lnd_host.clone(),
            config.lnd_port,
            config.lnd_tls_cert_path.clone(),
            config.lnd_macaroon_path.clone(),
        )
        .await
        .expect("failed to get lightning client");
        let decoded_pay_req_resp = lighting_client
            .decode_pay_req(tonic_lnd::rpc::PayReqString {
                pay_req: withdrawal_info.invoice_payment_request.clone(),
            })
            .await
            .map_err(|_| "failed to decode payment request string.")?;
        // We only print it here, note that in real-life code you may want to call `.into_inner()` on
        // the response to get the message.
        println!("{:#?}", decoded_pay_req_resp);
        // let invoice = invoice_resp.into_inner();
        let decoded_pay_req = decoded_pay_req_resp.into_inner();
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
            let send_payment_resp = lighting_client
                .send_payment_sync(tonic_lnd::rpc::SendRequest {
                    payment_request: withdrawal_info.invoice_payment_request.clone(),
                    ..tonic_lnd::rpc::SendRequest::default()
                })
                .await
                .map_err(|e| format!("failed to send payment: {:?}", e))?;
            let send_response = send_payment_resp.into_inner();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let withdrawal = Withdrawal {
                id: None,
                public_id: Uuid::new_v4().to_string(),
                user_id: user.id(),
                amount_sat,
                invoice_hash: hex::encode(send_response.payment_hash),
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

        // let order = Order {
        //     id: None,
        //     public_id: Uuid::new_v4().to_string(),
        //     quantity: order_info.quantity,
        //     user_id: user.id(),
        //     listing_id: listing.id.unwrap(),
        //     shipping_option_id: shipping_option.id.unwrap(),
        //     shipping_instructions: shipping_instructions.to_string(),
        //     amount_owed_sat: amount_owed_sat,
        //     seller_credit_sat: seller_credit_sat,
        //     paid: false,
        //     completed: false,
        //     invoice_hash: hex::encode(invoice.r_hash),
        //     invoice_payment_request: invoice.payment_request,
        //     created_time_ms: now,
        // };

        // match Order::insert(order, db).await {
        //     Ok(order_id) => match Order::single(db, order_id).await {
        //         Ok(new_order) => Ok(new_order.public_id.clone()),
        //         Err(e) => {
        //             error_!("DB insertion error: {}", e);
        //             Err("New order could not be found after inserting.".to_string())
        //         }
        //     },
        //     Err(e) => {
        //         error_!("DB insertion error: {}", e);
        //         Err("Order could not be inserted due an internal error.".to_string())
        //     }
        // }
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
