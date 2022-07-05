use crate::base::BaseContext;
use crate::db::Db;
use crate::models::AdminSettings;
use crate::models::{InitialListingInfo, Listing};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::uuid::Uuid;
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    base_context: BaseContext,
    flash: Option<(String, String)>,
    admin_settings: AdminSettings,
}

impl Context {
    pub async fn raw(
        flash: Option<(String, String)>,
        mut db: Connection<Db>,
        user: Option<User>,
        admin_user: Option<AdminUser>,
    ) -> Result<Context, String> {
        let base_context = BaseContext::raw(&mut db, user.clone(), admin_user.clone())
            .await
            .map_err(|_| "failed to get base template.")?;
        let admin_settings = AdminSettings::single(&mut db, AdminSettings::get_default())
            .await
            .map_err(|_| "failed to update market name.")?;
        Ok(Context {
            base_context,
            flash,
            admin_settings,
        })
    }
}

#[post("/", data = "<listing_form>")]
async fn new(
    listing_form: Form<InitialListingInfo>,
    mut db: Connection<Db>,
    user: User,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let listing_info = listing_form.into_inner();

    match create_listing(listing_info, &mut db, user.clone()).await {
        Ok(listing_id) => Ok(Flash::success(
            Redirect::to(format!("/{}/{}", "listing", listing_id)),
            "Listing successfully added.",
        )),
        Err(e) => {
            error_!("DB insertion error: {}", e);
            Err(Flash::error(Redirect::to(uri!("/new_listing", index())), e))
        }
    }
}

async fn create_listing(
    listing_info: InitialListingInfo,
    db: &mut Connection<Db>,
    user: User,
) -> Result<String, String> {
    let admin_settings = AdminSettings::single(db, AdminSettings::get_default())
        .await
        .map_err(|_| "failed to update market name.")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    if listing_info.title.is_empty() {
        Err("Title cannot be empty.".to_string())
    } else if listing_info.description.is_empty() {
        Err("Description cannot be empty.".to_string())
    } else if listing_info.title.len() > 64 {
        Err("Title length is too long.".to_string())
    } else if listing_info.description.len() > 4096 {
        Err("Description length is too long.".to_string())
    } else if listing_info.quantity == 0 {
        Err("Quantity must be a positive number.".to_string())
    } else if listing_info.price_sat == 0 {
        Err("Price must be a positive number.".to_string())
    } else if user.is_admin {
        Err("Admin user cannot create a listing.".to_string())
    } else {
        let listing = Listing {
            id: None,
            public_id: Uuid::new_v4().to_string(),
            user_id: user.id(),
            title: listing_info.title,
            description: listing_info.description,
            price_sat: listing_info.price_sat,
            quantity: listing_info.quantity,
            fee_rate_basis_points: admin_settings.fee_rate_basis_points,
            submitted: false,
            reviewed: false,
            approved: false,
            removed: false,
            created_time_ms: now,
        };
        match Listing::insert(listing, db).await {
            Ok(listing_id) => match Listing::single(db, listing_id).await {
                Ok(new_listing) => Ok(new_listing.public_id),
                Err(e) => {
                    error_!("DB insertion error: {}", e);
                    Err("New listing could not be found after inserting.".to_string())
                }
            },
            Err(e) => {
                error_!("DB insertion error: {}", e);
                Err("Listing could not be inserted due an internal error.".to_string())
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
    Template::render(
        "newlisting",
        Context::raw(flash, db, Some(user), admin_user).await,
    )
}

pub fn new_listing_stage() -> AdHoc {
    AdHoc::on_ignite("New Listing Stage", |rocket| async {
        rocket.mount("/new_listing", routes![index, new])
    })
}
