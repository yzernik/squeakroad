use crate::config::Config;
use crate::db::Db;
use crate::payment_processor;
use rocket::fairing::{self, AdHoc};
use rocket::fs::{relative, FileServer};
use rocket::{Build, Rocket};
use rocket_auth::Error::SqlxError;
use rocket_auth::Users;
use rocket_db_pools::{sqlx, Database};
use rocket_dyn_templates::Template;

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

async fn create_users_table(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => {
            let users: Users = db.0.clone().into();
            match users.create_table().await {
                Ok(_) => Ok(rocket.manage(users)),
                Err(e) => {
                    error!("Failed to create users table: {}", e);
                    Err(rocket)
                }
            }
        }
        None => Err(rocket),
    }
}

async fn create_admin_user(rocket: Rocket<Build>, config: Config) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => {
            let users: Users = db.0.clone().into();
            // TODO: Delete all existing admins users here.
            let username = config.admin_username;
            let password = config.admin_password;
            match users.create_user(&username, &password, true).await {
                Ok(_) => Ok(rocket),
                Err(e) => match e {
                    SqlxError(_) => Ok(rocket),
                    _ => {
                        error!("Failed to create admin user: {}", e);
                        Err(rocket)
                    }
                },
            }
        }
        None => Err(rocket),
    }
}

pub fn stage(config: Config) -> AdHoc {
    let config_clone = config.clone();
    let config_clone_2 = config.clone();

    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(AdHoc::try_on_ignite("Manage config", |rocket| {
                Box::pin(async move { Ok(rocket.manage(config)) })
            }))
            .attach(Db::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .attach(AdHoc::try_on_ignite(
                "SQLx Create Users table",
                create_users_table,
            ))
            .attach(AdHoc::try_on_ignite("SQLx Create Admin User", |r| {
                create_admin_user(r, config_clone_2)
            }))
            .attach(AdHoc::on_liftoff("DB polling", |rocket| {
                // Copied from: https://stackoverflow.com/a/72457117/1639564
                Box::pin(async move {
                    let pool = match Db::fetch(rocket) {
                        Some(pool) => pool.0.clone(), // clone the wrapped pool
                        None => panic!("failed to get db for background task."),
                    };
                    rocket::tokio::spawn(async move {
                        let mut interval = rocket::tokio::time::interval(
                            rocket::tokio::time::Duration::from_secs(10),
                        );
                        loop {
                            if let Ok(conn) = pool.acquire().await {
                                match payment_processor::handle_received_payments(
                                    config_clone.clone(),
                                    conn,
                                )
                                .await
                                {
                                    Ok(_) => println!(
                                        "payment processor task `completed` (shouldn't happen)."
                                    ),
                                    Err(e) => println!("payment processor task failed: {:?}", e),
                                }
                            }
                            println!("Subscription failed. Trying again in {:?} seconds.", 10);
                            interval.tick().await;
                        }
                    });
                })
            }))
            .attach(Template::fairing())
            .mount("/", FileServer::from(relative!("static")))
            .attach(crate::about::about_stage())
            .attach(crate::auth::auth_stage())
            .attach(crate::admin::admin_stage())
            .attach(crate::market_liabilities::market_liabilities_stage())
            .attach(crate::listings::listings_stage())
            .attach(crate::listing::listing_stage())
            .attach(crate::new_listing::new_listing_stage())
            .attach(crate::update_listing_images::update_listing_images_stage())
            .attach(crate::update_shipping_options::update_shipping_options_stage())
            .attach(crate::user::user_stage())
            .attach(crate::user_profile::user_profile_stage())
            .attach(crate::update_market_name::update_market_name_stage())
            .attach(crate::update_fee_rate::update_fee_rate_stage())
            .attach(crate::update_squeaknode_info::update_squeaknode_info_stage())
            .attach(crate::update_pgp_info::update_pgp_info_stage())
            .attach(crate::update_user_squeaknode_info::update_user_squeaknode_info_stage())
            .attach(crate::update_user_pgp_info::update_user_pgp_info_stage())
            .attach(crate::review_pending_listings::review_pending_listings_stage())
            .attach(crate::account::account_stage())
            .attach(crate::my_unsubmitted_listings::my_unsubmitted_listings_stage())
            .attach(crate::my_pending_listings::my_pending_listings_stage())
            .attach(crate::my_approved_listings::my_approved_listings_stage())
            .attach(crate::my_rejected_listings::my_rejected_listings_stage())
            .attach(crate::my_unpaid_orders::my_unpaid_orders_stage())
            .attach(crate::my_paid_orders::my_paid_orders_stage())
            .attach(crate::my_account_balance::my_account_balance_stage())
            .attach(crate::my_pending_orders::my_pending_orders_stage())
            .attach(crate::prepare_order::prepare_order_stage())
            .attach(crate::order::order_stage())
            .attach(crate::withdraw::withdraw_stage())
            .attach(crate::withdrawal::withdrawal_stage())
            .attach(crate::seller_history::seller_history_stage())
            .attach(crate::top_sellers::top_sellers_stage())
            .attach(crate::search::search_stage())
    })
}
