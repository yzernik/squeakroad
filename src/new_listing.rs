use crate::base::BaseContext;
use crate::db::Db;
use crate::models::AdminSettings;
use crate::models::{InitialListingInfo, Listing};
use crate::user_account::ActiveUser;
use crate::util;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket_auth::{AdminUser, User};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

const MAX_LISTINGS_PER_USER_PER_DAY: u32 = 10;

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
        let admin_settings = AdminSettings::single(&mut db)
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
    active_user: ActiveUser,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let listing_info = listing_form.into_inner();

    match create_listing(listing_info, &mut db, active_user.user.clone()).await {
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
    let admin_settings = AdminSettings::single(db)
        .await
        .map_err(|_| "failed to update market name.")?;
    let now = util::current_time_millis();
    let one_day_in_ms = 24 * 60 * 60 * 1000;
    let recent_listing_count =
        Listing::count_for_user_since_time_ms(db, user.id(), now - one_day_in_ms)
            .await
            .map_err(|_| "failed to get number of recent listings.")?;

    let price_sat = listing_info.price_sat.unwrap_or(0);

    if listing_info.title.is_empty() {
        return Err("Title cannot be empty.".to_string());
    };
    if listing_info.description.is_empty() {
        return Err("Description cannot be empty.".to_string());
    };
    if listing_info.title.len() > 64 {
        return Err("Title length is too long.".to_string());
    };
    if listing_info.description.len() > 4096 {
        return Err("Description length is too long.".to_string());
    };
    if price_sat == 0 {
        return Err("Price must be a positive number.".to_string());
    };
    if recent_listing_count >= MAX_LISTINGS_PER_USER_PER_DAY {
        return Err(format!(
            "More than {:?} listings in a single day not allowed.",
            MAX_LISTINGS_PER_USER_PER_DAY,
        ));
    };
    if user.is_admin {
        return Err("Admin user cannot create a listing.".to_string());
    };

    let listing = Listing {
        id: None,
        public_id: util::create_uuid(),
        user_id: user.id(),
        title: listing_info.title,
        description: listing_info.description,
        price_sat,
        fee_rate_basis_points: admin_settings.fee_rate_basis_points,
        submitted: false,
        reviewed: false,
        approved: false,
        deactivated_by_seller: false,
        deactivated_by_admin: false,
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

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    active_user: ActiveUser,
    admin_user: Option<AdminUser>,
) -> Template {
    println!("active_user: {:?}", active_user);

    let flash = flash.map(FlashMessage::into_inner);
    Template::render(
        "newlisting",
        Context::raw(flash, db, Some(active_user.user), admin_user).await,
    )
}

pub fn new_listing_stage() -> AdHoc {
    AdHoc::on_ignite("New Listing Stage", |rocket| async {
        rocket.mount("/new_listing", routes![index, new])
    })
}
