use crate::db::Db;
use crate::rocket::futures::TryFutureExt;
use crate::rocket::futures::TryStreamExt;
use crate::util;
use rocket::fs::TempFile;
use rocket::serde::Serialize;
use rocket_db_pools::{sqlx, Connection};
use sqlx::pool::PoolConnection;
use sqlx::Acquire;
use sqlx::Row;
use sqlx::Sqlite;
use std::future::Future;
use std::result::Result;

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Listing {
    pub id: Option<i32>,
    pub public_id: String,
    pub user_id: i32,
    pub title: String,
    pub description: String,
    pub price_sat: u64,
    pub fee_rate_basis_points: u32,
    pub submitted: bool,
    pub reviewed: bool,
    pub approved: bool,
    pub deactivated_by_seller: bool,
    pub deactivated_by_admin: bool,
    pub created_time_ms: u64,
}

#[derive(Debug, FromForm)]
pub struct InitialListingInfo {
    pub title: String,
    pub description: String,
    pub price_sat: Option<u64>,
}

#[derive(FromForm)]
pub struct FileUploadForm<'f> {
    pub file: TempFile<'f>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ListingImage {
    pub id: Option<i32>,
    pub public_id: String,
    pub listing_id: i32,
    pub image_data: Vec<u8>,
    pub is_primary: bool,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ListingDisplay {
    pub listing: Listing,
    pub images: Vec<ListingImageDisplay>,
    pub shipping_options: Vec<ShippingOption>,
    pub user: Option<RocketAuthUser>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ListingCardDisplay {
    pub listing: Listing,
    pub image: Option<ListingImageDisplay>,
    pub user: RocketAuthUser,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ListingCard {
    pub listing: Listing,
    pub image: Option<ListingImage>,
    pub user: RocketAuthUser,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ListingImageDisplay {
    pub id: Option<i32>,
    pub public_id: String,
    pub listing_id: i32,
    pub image_data_base64: String,
    pub is_primary: bool,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ShippingOption {
    pub id: Option<i32>,
    pub public_id: String,
    pub listing_id: i32,
    pub title: String,
    pub description: String,
    pub price_sat: u64,
}

#[derive(Debug, FromForm)]
pub struct ShippingOptionInfo {
    pub title: String,
    pub description: String,
    pub price_sat: Option<u64>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct RocketAuthUser {
    pub id: Option<i32>,
    pub username: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AdminSettings {
    pub id: Option<i32>,
    pub market_name: String,
    pub fee_rate_basis_points: u32,
    pub user_bond_price_sat: u64,
    pub pgp_key: String,
    pub squeaknode_pubkey: String,
    pub squeaknode_address: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UserSettings {
    pub id: Option<i32>,
    pub pgp_key: String,
    pub squeaknode_pubkey: String,
    pub squeaknode_address: String,
}

#[derive(Debug, FromForm)]
pub struct MarketNameInput {
    pub market_name: String,
}

#[derive(Debug, FromForm)]
pub struct PGPInfoInput {
    pub pgp_key: String,
}

#[derive(Debug, FromForm)]
pub struct SqueaknodeInfoInput {
    pub squeaknode_pubkey: String,
    pub squeaknode_address: String,
}

#[derive(Debug, FromForm)]
pub struct FeeRateInput {
    pub fee_rate_basis_points: Option<i32>,
}

#[derive(Debug, FromForm)]
pub struct UserBondPriceInput {
    pub user_bond_price_sat: Option<u64>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Order {
    pub id: Option<i32>,
    pub public_id: String,
    pub quantity: u32,
    pub buyer_user_id: i32,
    pub seller_user_id: i32,
    pub listing_id: i32,
    pub shipping_option_id: i32,
    pub shipping_instructions: String,
    pub amount_owed_sat: u64,
    pub seller_credit_sat: u64,
    pub paid: bool,
    pub shipped: bool,
    pub canceled_by_seller: bool,
    pub canceled_by_buyer: bool,
    pub reviewed: bool,
    pub invoice_hash: String,
    pub invoice_payment_request: String,
    pub review_rating: u32,
    pub review_text: String,
    pub created_time_ms: u64,
    pub payment_time_ms: u64,
    pub review_time_ms: u64,
}

#[derive(Debug, FromForm, Clone)]
pub struct OrderInfo {
    pub quantity: Option<u32>,
    pub shipping_option_id: String,
    pub shipping_instructions: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct OrderCard {
    pub order: Order,
    pub listing: Option<Listing>,
    pub image: Option<ListingImage>,
    pub user: Option<RocketAuthUser>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AccountInfo {
    pub account_balance_sat: i64,
    pub num_unshipped_orders: u32,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AdminInfo {
    pub num_pending_listings: u32,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AccountBalanceChange {
    pub amount_change_sat: i64,
    pub event_type: String,
    pub event_id: String,
    pub event_time_ms: u64,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Withdrawal {
    pub id: Option<i32>,
    pub public_id: String,
    pub user_id: i32,
    pub amount_sat: u64,
    pub invoice_hash: String,
    pub invoice_payment_request: String,
    pub created_time_ms: u64,
}

#[derive(Debug, FromForm, Clone)]
pub struct WithdrawalInfo {
    pub invoice_payment_request: String,
}

#[derive(Debug, FromForm, Clone)]
pub struct ReviewInput {
    pub review_rating: Option<u32>,
    pub review_text: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SellerInfo {
    pub username: String,
    pub total_amount_sold_sat: u64,
    pub weighted_average_rating: f32,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UserAccount {
    pub id: Option<i32>,
    pub public_id: String,
    pub user_id: i32,
    pub amount_owed_sat: u64,
    pub paid: bool,
    pub disabled: bool,
    pub invoice_hash: String,
    pub invoice_payment_request: String,
    pub created_time_ms: u64,
    pub payment_time_ms: u64,
}

impl Default for AdminSettings {
    fn default() -> AdminSettings {
        AdminSettings {
            id: None,
            market_name: "Squeak Road".to_string(),
            fee_rate_basis_points: 500,
            user_bond_price_sat: 1,
            pgp_key: "".to_string(),
            squeaknode_pubkey: "".to_string(),
            squeaknode_address: "".to_string(),
        }
    }
}

impl Default for UserSettings {
    fn default() -> UserSettings {
        UserSettings {
            id: None,
            pgp_key: "".to_string(),
            squeaknode_pubkey: "".to_string(),
            squeaknode_address: "".to_string(),
        }
    }
}

impl Listing {
    /// Returns the id of the inserted row.
    pub async fn insert(
        listing: Listing,
        max_unapproved_listings: u32,
        db: &mut Connection<Db>,
    ) -> Result<i32, String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        let price_sat: i64 = listing.price_sat.try_into().unwrap();
        let created_time_ms: i64 = listing.created_time_ms.try_into().unwrap();

        let insert_result = sqlx::query!(
            "INSERT INTO listings (public_id, user_id, title, description, price_sat, fee_rate_basis_points, submitted, reviewed, approved, deactivated_by_seller, deactivated_by_admin, created_time_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            listing.public_id,
            listing.user_id,
            listing.title,
            listing.description,
            price_sat,
            listing.fee_rate_basis_points,
            listing.submitted,
            listing.reviewed,
            listing.approved,
            listing.deactivated_by_seller,
            listing.deactivated_by_admin,
            created_time_ms,
        )
            .execute(&mut *tx)
            .await
            .map_err(|_| "failed to insert new listing.")?;

        let num_unapproved_listings = sqlx::query!(
            "
select
 COUNT(listings.id) as num_unapproved_listings
from
 listings
WHERE
 listings.user_id = ?
AND
 NOT listings.approved
;",
            listing.user_id,
        )
        .fetch_one(&mut *tx)
        .map_ok(|r| r.num_unapproved_listings as u32)
        .await
        .map_err(|_| "failed to get count of unapproved listings.")?;

        if num_unapproved_listings > max_unapproved_listings {
            return Err(format!(
                "more than {:?} unapproved listings not allowed.",
                max_unapproved_listings
            ));
        }

        tx.commit()
            .await
            .map_err(|_| "failed to commit transaction.")?;

        Ok(insert_result.last_insert_rowid() as _)
    }

    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<Listing, sqlx::Error> {
        let listing = sqlx::query!("select * from listings WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| Listing {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                user_id: r.user_id.try_into().unwrap(),
                title: r.title,
                description: r.description,
                price_sat: r.price_sat.try_into().unwrap(),
                fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                submitted: r.submitted,
                reviewed: r.reviewed,
                approved: r.approved,
                deactivated_by_seller: r.deactivated_by_seller,
                deactivated_by_admin: r.deactivated_by_admin,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(listing)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<Listing, sqlx::Error> {
        let listing = sqlx::query!("select * from listings WHERE public_id = ?;", public_id)
            .fetch_one(&mut **db)
            .map_ok(|r| Listing {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                user_id: r.user_id.try_into().unwrap(),
                title: r.title,
                description: r.description,
                price_sat: r.price_sat.try_into().unwrap(),
                fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                submitted: r.submitted,
                reviewed: r.reviewed,
                approved: r.approved,
                deactivated_by_seller: r.deactivated_by_seller,
                deactivated_by_admin: r.deactivated_by_admin,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(listing)
    }

    pub async fn mark_as_submitted(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE listings SET submitted = true WHERE public_id = ?",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn mark_as_approved(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE listings SET reviewed = true, approved = true WHERE public_id = ?",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn mark_as_rejected(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE listings SET reviewed = true, approved = false WHERE public_id = ?",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn mark_as_deactivated_by_seller(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
UPDATE listings
SET deactivated_by_seller = true
WHERE
 public_id = ?
AND
 approved
AND NOT (deactivated_by_seller OR deactivated_by_admin)
;",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn mark_as_deactivated_by_admin(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
UPDATE listings
SET deactivated_by_admin = true
WHERE
 public_id = ?
AND
 approved
AND NOT (deactivated_by_seller OR deactivated_by_admin)
;",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }

    pub async fn num_pending(db: &mut Connection<Db>) -> Result<u32, sqlx::Error> {
        let num_listings = sqlx::query!(
            "
select
 COUNT(listings.id) as num_pending_listings
from
 listings
WHERE
 listings.submitted
AND
 NOT listings.reviewed
;",
        )
        .fetch_one(&mut **db)
        .map_ok(|r| r.num_pending_listings as u32)
        .await?;

        Ok(num_listings)
    }

    pub async fn delete(listing_id: i32, db: &mut Connection<Db>) -> Result<usize, String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        let delete_result = sqlx::query!(
            "
DELETE FROM listings
WHERE
 id = ?
;",
            listing_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete listing.")?;

        sqlx::query!(
            "
DELETE from listingimages
WHERE
 listing_id = ?
;",
            listing_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete images for listing.")?;

        sqlx::query!(
            "
DELETE from shippingoptions
WHERE
 listing_id = ?
;",
            listing_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete shipping options for listing.")?;

        tx.commit()
            .await
            .map_err(|_| "failed to commit transaction.")?;

        Ok(delete_result.rows_affected() as _)
        //Ok(())
    }
}

impl ListingImage {
    /// Returns the number of affected rows: 1.
    pub async fn insert(
        listingimage: ListingImage,
        db: &mut Connection<Db>,
    ) -> Result<usize, sqlx::Error> {
        let insert_result = sqlx::query!(
            "INSERT INTO listingimages (public_id, listing_id, image_data, is_primary) VALUES (?, ?, ?, ?)",
            listingimage.public_id,
            listingimage.listing_id,
            listingimage.image_data,
            listingimage.is_primary,
        )
        .execute(&mut **db)
        .await?;

        Ok(insert_result.rows_affected() as _)
    }

    pub async fn all_for_listing(
        db: &mut Connection<Db>,
        listing_id: i32,
    ) -> Result<Vec<ListingImage>, sqlx::Error> {
        let listing_images = sqlx::query!(
            "select * from listingimages WHERE listing_id = ? ORDER BY listingimages.is_primary DESC;",
            listing_id
        )
        .fetch(&mut **db)
        .map_ok(|r| ListingImage {
            id: r.id.map(|n| n.try_into().unwrap()),
            public_id: r.public_id,
            listing_id: r.listing_id.try_into().unwrap(),
            image_data: r.image_data,
            is_primary: r.is_primary,
        })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(listing_images)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<ListingImage, sqlx::Error> {
        let listing_image = sqlx::query!(
            "select * from listingimages WHERE public_id = ?;",
            public_id
        )
        .fetch_one(&mut **db)
        .map_ok(|r| ListingImage {
            id: Some(r.id.try_into().unwrap()),
            public_id: r.public_id,
            listing_id: r.listing_id.try_into().unwrap(),
            image_data: r.image_data,
            is_primary: r.is_primary,
        })
        .await?;

        Ok(listing_image)
    }

    pub async fn mark_image_as_primary_by_public_id(
        db: &mut Connection<Db>,
        listing_id: i32,
        image_id: &str,
    ) -> Result<usize, sqlx::Error> {
        // Set all images for listing_id to not primary.
        let update_primary_result = sqlx::query!(
            "
UPDATE
 listingimages
SET
 is_primary = (public_id = ?)
WHERE
 listing_id = ?
;",
            image_id,
            listing_id
        )
        .execute(&mut **db)
        .await?;

        Ok(update_primary_result.rows_affected() as _)
    }

    /// Returns the number of affected rows: 1.
    pub async fn delete_with_public_id(
        public_id: &str,
        db: &mut Connection<Db>,
    ) -> Result<usize, sqlx::Error> {
        let delete_result =
            sqlx::query!("DELETE FROM listingimages WHERE public_id = ?", public_id)
                .execute(&mut **db)
                .await?;

        Ok(delete_result.rows_affected() as _)
    }
}

impl RocketAuthUser {
    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<RocketAuthUser, sqlx::Error> {
        let rocket_auth_user = sqlx::query!("select id, email from users WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| RocketAuthUser {
                id: Some(r.id as i32),
                username: r.email.unwrap(),
            })
            .await?;

        Ok(rocket_auth_user)
    }

    pub async fn single_by_username(
        db: &mut Connection<Db>,
        username: String,
    ) -> Result<RocketAuthUser, sqlx::Error> {
        let rocket_auth_user =
            sqlx::query!("select id, email from users WHERE email = ?;", username)
                .fetch_one(&mut **db)
                .map_ok(|r| RocketAuthUser {
                    id: Some(r.id.unwrap() as i32),
                    username: r.email.unwrap(),
                })
                .await?;

        Ok(rocket_auth_user)
    }
}

impl ListingDisplay {
    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<ListingDisplay, sqlx::Error> {
        let listing = Listing::single_by_public_id(&mut *db, public_id).await?;
        let images = ListingImage::all_for_listing(&mut *db, listing.id.unwrap()).await?;
        let image_displays = images
            .iter()
            .map(|img| ListingImageDisplay {
                id: img.id,
                public_id: img.clone().public_id,
                listing_id: img.listing_id,
                image_data_base64: util::to_base64(&img.image_data),
                is_primary: img.is_primary,
            })
            .collect::<Vec<_>>();
        let shipping_options =
            ShippingOption::all_for_listing(&mut *db, listing.id.unwrap()).await?;
        let rocket_auth_user = RocketAuthUser::single(&mut *db, listing.user_id).await.ok();

        let listing_display = ListingDisplay {
            listing,
            images: image_displays,
            shipping_options,
            user: rocket_auth_user,
        };

        Ok(listing_display)
    }
}

impl ListingCard {
    pub async fn all_active(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of shipped orders.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 listings
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
INNER JOIN
 users
ON
 listings.user_id = users.id
WHERE
 listings.approved
AND
 not (listings.deactivated_by_seller OR listings.deactivated_by_admin)
GROUP BY
 listings.id
ORDER BY listings.created_time_ms DESC
LIMIT ?
OFFSET ?
;", limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                ListingCard {
                    listing: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(listing_cards)
    }

    pub async fn all_pending(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of shipped orders.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 listings
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
INNER JOIN
 users
ON
 listings.user_id = users.id
WHERE
 listings.submitted
AND
 NOT listings.reviewed
GROUP BY
 listings.id
ORDER BY listings.created_time_ms DESC
LIMIT ?
OFFSET ?
;", limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                ListingCard {
                    listing: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(listing_cards)
    }

    pub async fn all_unsubmitted_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of shipped orders.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 listings
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
INNER JOIN
 users
ON
 listings.user_id = users.id
WHERE
 not listings.submitted
AND
 users.id = ?
GROUP BY
 listings.id
ORDER BY listings.created_time_ms DESC
LIMIT ?
OFFSET ?
;", user_id, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                ListingCard {
                    listing: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(listing_cards)
    }

    pub async fn all_pending_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of shipped orders.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 listings
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
INNER JOIN
 users
ON
 listings.user_id = users.id
WHERE
 listings.submitted
AND
 not listings.reviewed
AND
 users.id = ?
GROUP BY
 listings.id
ORDER BY listings.created_time_ms DESC
LIMIT ?
OFFSET ?
;", user_id, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                ListingCard {
                    listing: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(listing_cards)
    }

    pub async fn all_rejected_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of shipped orders.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 listings
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
INNER JOIN
 users
ON
 listings.user_id = users.id
WHERE
 not listings.approved
AND
 listings.reviewed
AND
 users.id = ?
GROUP BY
 listings.id
ORDER BY listings.created_time_ms DESC
LIMIT ?
OFFSET ?
;", user_id, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_admin.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                ListingCard {
                    listing: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(listing_cards)
    }

    pub async fn all_deactivated_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 listings
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
INNER JOIN
 users
ON
 listings.user_id = users.id
WHERE
 listings.approved
AND
 (listings.deactivated_by_seller OR listings.deactivated_by_admin)
AND
 users.id = ?
GROUP BY
 listings.id
ORDER BY listings.created_time_ms DESC
LIMIT ?
OFFSET ?
;", user_id, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_admin.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                ListingCard {
                    listing: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(listing_cards)
    }

    pub async fn all_active_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of shipped orders.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 listings
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
INNER JOIN
 users
ON
 listings.user_id = users.id
WHERE
 listings.approved
AND
 listings.reviewed
AND
 NOT (listings.deactivated_by_seller OR listings.deactivated_by_admin)
AND
 users.id = ?
GROUP BY
 listings.id
ORDER BY listings.created_time_ms DESC
LIMIT ?
OFFSET ?
;", user_id, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                ListingCard {
                    listing: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(listing_cards)
    }

    pub async fn all_active_for_search_text(
        db: &mut Connection<Db>,
        search_text: &str,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of shipped orders.
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        // let uppercase_search_term = search_text.to_owned().to_ascii_uppercase();
        let wildcard_search_term = format!("%{}%", search_text.to_ascii_uppercase());
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 listings
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
INNER JOIN
 users
ON
 listings.user_id = users.id
WHERE
 listings.approved
AND
 listings.reviewed
AND
 NOT (listings.deactivated_by_seller OR listings.deactivated_by_admin)
AND
 (UPPER(listings.title) like ? OR UPPER(listings.description) like ?)
GROUP BY
 listings.id
ORDER BY listings.created_time_ms DESC
LIMIT ?
OFFSET ?
;", wildcard_search_term, wildcard_search_term, limit, offset)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.unwrap().try_into().unwrap()),
                    public_id: r.public_id.unwrap(),
                    user_id: r.user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                ListingCard {
                    listing: l,
                    image: i,
                    user: u.unwrap(),
                }
            })
                .try_collect::<Vec<_>>()
                .await?;

        Ok(listing_cards)
    }
}

impl ListingCardDisplay {
    fn listing_card_to_display(card: &ListingCard) -> ListingCardDisplay {
        ListingCardDisplay {
            listing: card.listing.clone(),
            image: card.image.clone().map(|image| ListingImageDisplay {
                id: image.id,
                public_id: image.public_id,
                listing_id: image.listing_id,
                image_data_base64: util::to_base64(&image.image_data),
                is_primary: image.is_primary,
            }),
            user: card.clone().user,
        }
    }

    pub async fn all_active(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards = ListingCard::all_active(db, page_size, page_num).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(ListingCardDisplay::listing_card_to_display)
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_pending(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards = ListingCard::all_pending(db, page_size, page_num).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(ListingCardDisplay::listing_card_to_display)
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_unsubmitted_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards =
            ListingCard::all_unsubmitted_for_user(db, user_id, page_size, page_num).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(ListingCardDisplay::listing_card_to_display)
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_pending_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards =
            ListingCard::all_pending_for_user(db, user_id, page_size, page_num).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(ListingCardDisplay::listing_card_to_display)
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_rejected_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards =
            ListingCard::all_rejected_for_user(db, user_id, page_size, page_num).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(ListingCardDisplay::listing_card_to_display)
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_deactivated_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards =
            ListingCard::all_deactivated_for_user(db, user_id, page_size, page_num).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(ListingCardDisplay::listing_card_to_display)
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_active_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards =
            ListingCard::all_active_for_user(db, user_id, page_size, page_num).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(ListingCardDisplay::listing_card_to_display)
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_active_for_search_text(
        db: &mut Connection<Db>,
        search_text: &str,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards =
            ListingCard::all_active_for_search_text(db, search_text, page_size, page_num).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(ListingCardDisplay::listing_card_to_display)
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }
}

impl ShippingOption {
    /// Returns the number of affected rows: 1.
    pub async fn insert(
        shipping_option: ShippingOption,
        db: &mut Connection<Db>,
    ) -> Result<usize, sqlx::Error> {
        let price_sat: i64 = shipping_option.price_sat.try_into().unwrap();
        let insert_result = sqlx::query!(
            "INSERT INTO shippingoptions (public_id, listing_id, title, description, price_sat) VALUES (?, ?, ?, ?, ?)",
            shipping_option.public_id,
            shipping_option.listing_id,
            shipping_option.title,
            shipping_option.description,
            price_sat,
        )
            .execute(&mut **db)
            .await?;
        Ok(insert_result.rows_affected() as _)
    }

    pub async fn all_for_listing(
        db: &mut Connection<Db>,
        listing_id: i32,
    ) -> Result<Vec<ShippingOption>, sqlx::Error> {
        let shipping_options = sqlx::query!(
            "select * from shippingoptions WHERE listing_id = ? ORDER BY shippingoptions.price_sat ASC;",
            listing_id
        )
        .fetch(&mut **db)
        .map_ok(|r| ShippingOption {
            id: r.id.map(|n| n.try_into().unwrap()),
            public_id: r.public_id,
            listing_id: r.listing_id.try_into().unwrap(),
            title: r.title,
            description: r.description,
            price_sat: r.price_sat.try_into().unwrap(),
        })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(shipping_options)
    }

    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<ShippingOption, sqlx::Error> {
        let shipping_option = sqlx::query!("select * from shippingoptions WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| ShippingOption {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                listing_id: r.listing_id.try_into().unwrap(),
                title: r.title,
                description: r.description,
                price_sat: r.price_sat.try_into().unwrap(),
            })
            .await?;

        Ok(shipping_option)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<ShippingOption, sqlx::Error> {
        let shipping_option = sqlx::query!(
            "select * from shippingoptions WHERE public_id = ?;",
            public_id,
        )
        .fetch_one(&mut **db)
        .map_ok(|r| ShippingOption {
            id: Some(r.id.try_into().unwrap()),
            public_id: r.public_id,
            listing_id: r.listing_id.try_into().unwrap(),
            title: r.title,
            description: r.description,
            price_sat: r.price_sat.try_into().unwrap(),
        })
        .await?;

        Ok(shipping_option)
    }

    /// Returns the number of affected rows: 1.
    pub async fn delete_with_public_id(
        public_id: &str,
        db: &mut Connection<Db>,
    ) -> Result<usize, sqlx::Error> {
        let delete_result =
            sqlx::query!("DELETE FROM shippingoptions WHERE public_id = ?", public_id)
                .execute(&mut **db)
                .await?;

        Ok(delete_result.rows_affected() as _)
    }
}

impl AdminSettings {
    pub async fn single(db: &mut Connection<Db>) -> Result<AdminSettings, sqlx::Error> {
        let maybe_admin_settings = sqlx::query!("select * from adminsettings;")
            .fetch_optional(&mut **db)
            .map_ok(|maybe_r| {
                maybe_r.map(|r| AdminSettings {
                    id: Some(r.id.try_into().unwrap()),
                    market_name: r.market_name,
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    user_bond_price_sat: r.user_bond_price_sat.try_into().unwrap(),
                    pgp_key: r.pgp_key,
                    squeaknode_pubkey: r.squeaknode_pubkey,
                    squeaknode_address: r.squeaknode_address,
                })
            })
            .await?;

        let admin_settings = maybe_admin_settings.unwrap_or_default();

        Ok(admin_settings)
    }

    async fn insert_if_doesnt_exist(db: &mut Connection<Db>) -> Result<(), sqlx::Error> {
        let admin_settings = AdminSettings::default();
        sqlx::query!(
            "
INSERT INTO
 adminsettings (market_name, fee_rate_basis_points, pgp_key, squeaknode_pubkey, squeaknode_address)
SELECT ?, ?, ?, ?, ?
WHERE NOT EXISTS(SELECT 1 FROM adminsettings)
;",
            admin_settings.market_name,
            admin_settings.fee_rate_basis_points,
            admin_settings.squeaknode_pubkey,
            admin_settings.pgp_key,
            admin_settings.squeaknode_address,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_market_name(
        db: &mut Connection<Db>,
        new_market_name: &str,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!("UPDATE adminsettings SET market_name = ?", new_market_name)
            .execute(&mut **db)
            .await?;

        Ok(())
    }

    pub async fn set_fee_rate(
        db: &mut Connection<Db>,
        new_fee_rate_basis_points: i32,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!(
            "UPDATE adminsettings SET fee_rate_basis_points = ?",
            new_fee_rate_basis_points,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_user_bond_price(
        db: &mut Connection<Db>,
        new_user_bond_price_sat: u64,
    ) -> Result<(), sqlx::Error> {
        let user_bond_price_sat_i64: i64 = new_user_bond_price_sat.try_into().unwrap();

        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!(
            "UPDATE adminsettings SET user_bond_price_sat = ?",
            user_bond_price_sat_i64,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_pgp_key(
        db: &mut Connection<Db>,
        new_pgp_key: &str,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!("UPDATE adminsettings SET pgp_key = ?", new_pgp_key,)
            .execute(&mut **db)
            .await?;

        Ok(())
    }

    pub async fn set_squeaknode_pubkey(
        db: &mut Connection<Db>,
        new_squeaknode_pubkey: &str,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!(
            "UPDATE adminsettings SET squeaknode_pubkey = ?",
            new_squeaknode_pubkey,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_squeaknode_address(
        db: &mut Connection<Db>,
        new_squeaknode_address: &str,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db).await?;

        sqlx::query!(
            "UPDATE adminsettings SET squeaknode_address = ?",
            new_squeaknode_address,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }
}

impl UserSettings {
    pub async fn single(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<UserSettings, sqlx::Error> {
        let maybe_user_settings =
            sqlx::query!("select * from usersettings WHERE user_id = ?;", user_id,)
                .fetch_optional(&mut **db)
                .map_ok(|maybe_r| {
                    maybe_r.map(|r| UserSettings {
                        id: Some(r.id.try_into().unwrap()),
                        pgp_key: r.pgp_key,
                        squeaknode_pubkey: r.squeaknode_pubkey,
                        squeaknode_address: r.squeaknode_address,
                    })
                })
                .await?;
        let user_settings = maybe_user_settings.unwrap_or_default();

        Ok(user_settings)
    }

    /// Returns the number of affected rows: 1.
    async fn insert_if_doesnt_exist(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<(), sqlx::Error> {
        let user_settings = UserSettings::default();
        sqlx::query!(
            "
INSERT INTO
 usersettings (user_id, pgp_key, squeaknode_pubkey, squeaknode_address)
SELECT ?, ?, ?, ?
WHERE NOT EXISTS(SELECT 1 FROM usersettings WHERE user_id = ?)
;",
            user_id,
            user_settings.pgp_key,
            user_settings.squeaknode_pubkey,
            user_settings.squeaknode_address,
            user_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_pgp_key(
        db: &mut Connection<Db>,
        user_id: i32,
        new_pgp_key: &str,
    ) -> Result<(), sqlx::Error> {
        UserSettings::insert_if_doesnt_exist(db, user_id).await?;

        sqlx::query!(
            "UPDATE usersettings SET pgp_key = ? WHERE user_id = ?;",
            new_pgp_key,
            user_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_squeaknode_pubkey(
        db: &mut Connection<Db>,
        user_id: i32,
        new_squeaknode_pubkey: &str,
    ) -> Result<(), sqlx::Error> {
        UserSettings::insert_if_doesnt_exist(db, user_id).await?;

        sqlx::query!(
            "UPDATE usersettings SET squeaknode_pubkey = ? WHERE user_id = ?;",
            new_squeaknode_pubkey,
            user_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn set_squeaknode_address(
        db: &mut Connection<Db>,
        user_id: i32,
        new_squeaknode_address: &str,
    ) -> Result<(), sqlx::Error> {
        UserSettings::insert_if_doesnt_exist(db, user_id).await?;

        sqlx::query!(
            "UPDATE usersettings SET squeaknode_address = ? WHERE user_id = ?;",
            new_squeaknode_address,
            user_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }
}

impl Order {
    /// Returns the id of the inserted row.
    pub async fn insert(order: Order, db: &mut Connection<Db>) -> Result<i32, sqlx::Error> {
        let amount_owed_sat: i64 = order.amount_owed_sat.try_into().unwrap();
        let seller_credit_sat: i64 = order.seller_credit_sat.try_into().unwrap();
        let created_time_ms: i64 = order.created_time_ms.try_into().unwrap();
        let payment_time_ms: i64 = order.payment_time_ms.try_into().unwrap();
        let review_time_ms: i64 = order.review_time_ms.try_into().unwrap();

        let insert_result = sqlx::query!(
            "INSERT INTO orders (public_id, buyer_user_id, seller_user_id, quantity, listing_id, shipping_option_id, shipping_instructions, amount_owed_sat, seller_credit_sat, paid, shipped, canceled_by_seller, canceled_by_buyer, reviewed, review_text, review_rating, invoice_hash, invoice_payment_request, created_time_ms, payment_time_ms, review_time_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            order.public_id,
            order.buyer_user_id,
            order.seller_user_id,
            order.quantity,
            order.listing_id,
            order.shipping_option_id,
            order.shipping_instructions,
            amount_owed_sat,
            seller_credit_sat,
            order.paid,
            order.shipped,
            order.canceled_by_seller,
            order.canceled_by_buyer,
            order.reviewed,
            order.review_text,
            order.review_rating,
            order.invoice_hash,
            order.invoice_payment_request,
            created_time_ms,
            payment_time_ms,
            review_time_ms,
        )
            .execute(&mut **db)
            .await?;

        Ok(insert_result.last_insert_rowid() as _)
    }

    /// Sets a new review for a given order.
    ///
    /// Sets the "review_time_ms" field to current time if this is the first review.
    /// Otherwise, keeps the existing value for "review_time_ms"
    pub async fn set_order_review(
        db: &mut Connection<Db>,
        public_id: &str,
        review_rating: u32,
        review_text: &str,
        review_time_ms: u64,
    ) -> Result<(), sqlx::Error> {
        let review_time_ms_i64: i64 = review_time_ms.try_into().unwrap();

        sqlx::query!(
            "
        UPDATE
         orders
        SET
         reviewed = true,
         review_rating = ?,
         review_text = ?,
         review_time_ms = ?
        WHERE
         public_id = ?
        ;",
            review_rating,
            review_text,
            review_time_ms_i64,
            public_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<Order, sqlx::Error> {
        let order = sqlx::query!("select * from orders WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| Order {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                quantity: r.quantity.try_into().unwrap(),
                buyer_user_id: r.buyer_user_id.try_into().unwrap(),
                seller_user_id: r.seller_user_id.try_into().unwrap(),
                listing_id: r.listing_id.try_into().unwrap(),
                shipping_option_id: r.shipping_option_id.try_into().unwrap(),
                shipping_instructions: r.shipping_instructions,
                amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                paid: r.paid,
                shipped: r.shipped,
                canceled_by_seller: r.canceled_by_seller,
                canceled_by_buyer: r.canceled_by_buyer,
                reviewed: r.reviewed,
                invoice_hash: r.invoice_hash,
                invoice_payment_request: r.invoice_payment_request,
                review_rating: r.review_rating.try_into().unwrap(),
                review_text: r.review_text,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
                payment_time_ms: r.payment_time_ms.try_into().unwrap(),
                review_time_ms: r.review_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(order)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query!("select * from orders WHERE public_id = ?;", public_id)
            .fetch_one(&mut **db)
            .map_ok(|r| Order {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                quantity: r.quantity.try_into().unwrap(),
                buyer_user_id: r.buyer_user_id.try_into().unwrap(),
                seller_user_id: r.seller_user_id.try_into().unwrap(),
                listing_id: r.listing_id.try_into().unwrap(),
                shipping_option_id: r.shipping_option_id.try_into().unwrap(),
                shipping_instructions: r.shipping_instructions,
                amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                paid: r.paid,
                shipped: r.shipped,
                canceled_by_seller: r.canceled_by_seller,
                canceled_by_buyer: r.canceled_by_buyer,
                reviewed: r.reviewed,
                invoice_hash: r.invoice_hash,
                invoice_payment_request: r.invoice_payment_request,
                review_rating: r.review_rating.try_into().unwrap(),
                review_text: r.review_text,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
                payment_time_ms: r.payment_time_ms.try_into().unwrap(),
                review_time_ms: r.review_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(order)
    }

    pub async fn single_by_invoice_hash(
        db: &mut PoolConnection<Sqlite>,
        invoice_hash: &str,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query!("select * from orders WHERE invoice_hash = ?;", invoice_hash)
            .fetch_one(&mut **db)
            .map_ok(|r| Order {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                quantity: r.quantity.try_into().unwrap(),
                buyer_user_id: r.buyer_user_id.try_into().unwrap(),
                seller_user_id: r.seller_user_id.try_into().unwrap(),
                listing_id: r.listing_id.try_into().unwrap(),
                shipping_option_id: r.shipping_option_id.try_into().unwrap(),
                shipping_instructions: r.shipping_instructions,
                amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                paid: r.paid,
                shipped: r.shipped,
                canceled_by_seller: r.canceled_by_seller,
                canceled_by_buyer: r.canceled_by_buyer,
                reviewed: r.reviewed,
                invoice_hash: r.invoice_hash,
                invoice_payment_request: r.invoice_payment_request,
                review_rating: r.review_rating.try_into().unwrap(),
                review_text: r.review_text,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
                payment_time_ms: r.payment_time_ms.try_into().unwrap(),
                review_time_ms: r.review_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(order)
    }

    pub async fn all_older_than(
        db: &mut PoolConnection<Sqlite>,
        created_time_ms: u64,
    ) -> Result<Vec<Order>, sqlx::Error> {
        let created_time_ms_i64: i64 = created_time_ms.try_into().unwrap();

        let orders = sqlx::query!(
            "
select *
from
 orders
WHERE
 created_time_ms < ?
AND
 NOT paid
;",
            created_time_ms_i64,
        )
        .fetch(&mut **db)
        .map_ok(|r| Order {
            id: Some(r.id.try_into().unwrap()),
            public_id: r.public_id,
            quantity: r.quantity.try_into().unwrap(),
            buyer_user_id: r.buyer_user_id.try_into().unwrap(),
            seller_user_id: r.seller_user_id.try_into().unwrap(),
            listing_id: r.listing_id.try_into().unwrap(),
            shipping_option_id: r.shipping_option_id.try_into().unwrap(),
            shipping_instructions: r.shipping_instructions,
            amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
            seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
            paid: r.paid,
            shipped: r.shipped,
            canceled_by_seller: r.canceled_by_seller,
            canceled_by_buyer: r.canceled_by_buyer,
            reviewed: r.reviewed,
            invoice_hash: r.invoice_hash,
            invoice_payment_request: r.invoice_payment_request,
            review_rating: r.review_rating.try_into().unwrap(),
            review_text: r.review_text,
            created_time_ms: r.created_time_ms.try_into().unwrap(),
            payment_time_ms: r.payment_time_ms.try_into().unwrap(),
            review_time_ms: r.review_time_ms.try_into().unwrap(),
        })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(orders)
    }

    pub async fn mark_as_paid(
        db: &mut PoolConnection<Sqlite>,
        order_id: i32,
        time_now_ms: u64,
    ) -> Result<(), sqlx::Error> {
        let time_now_ms_i64: i64 = time_now_ms.try_into().unwrap();

        sqlx::query!(
            "UPDATE orders SET paid = true, payment_time_ms = ? WHERE id = ?",
            time_now_ms_i64,
            order_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn mark_as_shipped(
        db: &mut PoolConnection<Sqlite>,
        order_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
UPDATE
 orders
SET
 shipped = true
WHERE
 id = ?
AND
 paid
AND
 not (shipped OR canceled_by_seller OR canceled_by_buyer)
;",
            order_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn mark_as_canceled_by_seller(
        db: &mut PoolConnection<Sqlite>,
        order_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
UPDATE
 orders
SET
 canceled_by_seller = true
WHERE
 id = ?
AND
 not (shipped OR canceled_by_seller OR canceled_by_buyer)
;",
            order_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn mark_as_canceled_by_buyer(
        db: &mut PoolConnection<Sqlite>,
        order_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
UPDATE
 orders
SET
 canceled_by_buyer = true
WHERE
 id = ?
AND
 not (shipped OR canceled_by_seller OR canceled_by_buyer)
;",
            order_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn delete_expired_order(
        db: &mut PoolConnection<Sqlite>,
        order_id: i32,
        cancel_order_invoice_future: impl Future<
            Output = Result<tonic_openssl_lnd::invoicesrpc::CancelInvoiceResp, String>,
        >,
    ) -> Result<(), String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        sqlx::query!(
            "
DELETE FROM orders
WHERE
 id = ?
AND
 NOT paid
;",
            order_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete order from database.")?;

        cancel_order_invoice_future
            .await
            .map_err(|e| format!("failed to cancel order invoice: {:?}", e))?;

        tx.commit()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        Ok(())
    }

    pub async fn seller_info_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<SellerInfo, sqlx::Error> {
        let total_amount_sold_sat = sqlx::query(
            "
            select
             SUM(orders.amount_owed_sat) as total_amount_sold_sat
            FROM
             orders
            WHERE
             orders.shipped
            AND
             orders.seller_user_id = ?
            GROUP BY
             orders.seller_user_id
            ;",
        )
        .bind(user_id)
        .fetch_optional(&mut **db)
        .map_ok(|maybe_r| {
            maybe_r.map(|r| {
                let amount_sat_i64: i64 = r.try_get("total_amount_sold_sat").unwrap();
                let amount_sat_u64: u64 = amount_sat_i64.try_into().unwrap();
                amount_sat_u64
            })
        })
        .await?;

        let weighted_average_rating = sqlx::query(
            "
            select
             SUM(orders.amount_owed_sat * orders.review_rating * 1000) / SUM(orders.amount_owed_sat) as weighted_average
            FROM
             orders
            WHERE
             orders.reviewed
            AND
             orders.seller_user_id = ?
            GROUP BY
             orders.seller_user_id
            ;")
.bind(user_id)
            .fetch_optional(&mut **db)
            .map_ok(|maybe_r| maybe_r.map(|r| {
                let average_rating_numerator: i64 = r.try_get("weighted_average").unwrap();
                let average_rating: f32 = (average_rating_numerator as f32) / 1000.0;
                average_rating
            }))
        .await?;

        let seller_info = SellerInfo {
            username: "".to_string(),
            total_amount_sold_sat: total_amount_sold_sat.unwrap_or(0),
            weighted_average_rating: weighted_average_rating.unwrap_or(0.0),
        };

        // TODO: remove option from return type.
        Ok(seller_info)
    }

    pub async fn seller_info_for_all_users(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<SellerInfo>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let seller_infos = sqlx::query(
                    "
        SELECT weighted_average, total_amount_sold_sat, users.email
        FROM
         users
        LEFT JOIN
            (select
             SUM(orders.amount_owed_sat) as total_amount_sold_sat, orders.seller_user_id as shipped_seller_user_id
            FROM
             orders
            WHERE
             shipped
            GROUP BY
             orders.seller_user_id) as seller_infos
        ON
         users.id = seller_infos.shipped_seller_user_id
        LEFT JOIN
            (select
             SUM(orders.amount_owed_sat * orders.review_rating * 1000) / SUM(orders.amount_owed_sat) as weighted_average, orders.seller_user_id as reviewed_seller_user_id
            FROM
             orders
            WHERE
             orders.reviewed
            AND
             orders.shipped
            GROUP BY
             orders.seller_user_id) as seller_infos
        ON
         users.id = seller_infos.reviewed_seller_user_id
        WHERE
         total_amount_sold_sat > 0
        ORDER BY
         total_amount_sold_sat DESC
        LIMIT ?
        OFFSET ?
            ;")
            .bind(limit)
            .bind(offset)
            .fetch(&mut **db)
            .map_ok(|r| {
                SellerInfo {
                    username: r.try_get("email").unwrap(),
                    total_amount_sold_sat: {
                        let amount_sat_i64: i64 = r.try_get("total_amount_sold_sat").unwrap();
                        let amount_sat_u64: u64 = amount_sat_i64.try_into().unwrap();
                        amount_sat_u64
                    },
                    weighted_average_rating: {
                        let average_rating_numerator: i64 = r.try_get("weighted_average").unwrap();
                        let average_rating: f32 = (average_rating_numerator as f32) / 1000.0;
                        average_rating
                    },
                }})
            .try_collect::<Vec<_>>()
            .await?;

        Ok(seller_infos)
    }

    // TODO: implement this.
    pub async fn most_recent_paid_order(
        db: &mut PoolConnection<Sqlite>,
    ) -> Result<Option<String>, sqlx::Error> {
        let latest_paid_order_invoice_hash = sqlx::query!(
            "
SELECT
 invoice_hash
FROM
(SELECT invoice_hash, payment_time_ms FROM useraccounts
 UNION ALL
SELECT invoice_hash, payment_time_ms FROM orders)
WHERE
 payment_time_ms = (SELECT MAX(payment_time_ms) FROM
(SELECT invoice_hash, payment_time_ms FROM useraccounts
 UNION ALL
SELECT invoice_hash, payment_time_ms FROM orders))
LIMIT 1
;"
        )
        .fetch_optional(&mut **db)
        .map_ok(|maybe_r| maybe_r.map(|r| r.invoice_hash))
        .await?;

        Ok(latest_paid_order_invoice_hash)
    }

    pub async fn num_processing_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<u32, sqlx::Error> {
        let num_orders = sqlx::query!(
            "
select
 COUNT(orders.id) as num_processing_orders
from
 orders
WHERE
 orders.paid
AND
 not (orders.shipped OR orders.canceled_by_seller OR orders.canceled_by_buyer)
AND
 orders.seller_user_id = ?
;",
            user_id,
        )
        .fetch_one(&mut **db)
        .map_ok(|r| r.num_processing_orders as u32)
        .await?;

        Ok(num_orders)
    }
}

impl OrderCard {
    pub async fn all_unpaid_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<OrderCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let orders = sqlx::query!(
            "
select
 orders.id as order_id, orders.public_id as order_public_id, orders.buyer_user_id as order_buyer_user_id, orders.seller_user_id as order_seller_user_id, orders.quantity as order_quantity, orders.listing_id as order_listing_id, orders.shipping_option_id, orders.shipping_instructions, orders.amount_owed_sat, orders.seller_credit_sat, orders.paid, orders.shipped, orders.canceled_by_seller, orders.canceled_by_buyer, orders.reviewed as order_reviewed, orders.invoice_hash, orders.invoice_payment_request, orders.review_rating, orders.review_text, orders.created_time_ms, orders.payment_time_ms, orders.review_time_ms, listings.id, listings.public_id as listing_public_id, listings.user_id as listing_user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms as listing_created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 orders
LEFT JOIN
 listings
ON
 orders.listing_id = listings.id
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
LEFT JOIN
 users
ON
 listings.user_id = users.id
WHERE
 not orders.paid
AND
 order_buyer_user_id = ?
GROUP BY
 orders.id
ORDER BY orders.created_time_ms DESC
LIMIT ?
OFFSET ?
;",
            user_id,
            limit,
            offset,
        )
            .fetch(&mut **db)
            .map_ok(|r| {
                let o = Order {
                    id: Some(r.order_id.unwrap().try_into().unwrap()),
                    public_id: r.order_public_id.unwrap(),
                    quantity: r.order_quantity.unwrap().try_into().unwrap(),
                    buyer_user_id: r.order_buyer_user_id.unwrap().try_into().unwrap(),
                    seller_user_id: r.order_seller_user_id.unwrap().try_into().unwrap(),
                    listing_id: r.order_listing_id.unwrap().try_into().unwrap(),
                    shipping_option_id: r.shipping_option_id.unwrap().try_into().unwrap(),
                    shipping_instructions: r.shipping_instructions.unwrap(),
                    amount_owed_sat: r.amount_owed_sat.unwrap().try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.unwrap().try_into().unwrap(),
                    paid: r.paid.unwrap(),
                    shipped: r.shipped.unwrap(),
                    canceled_by_seller: r.canceled_by_seller.unwrap(),
                    canceled_by_buyer: r.canceled_by_buyer.unwrap(),
                    reviewed: r.order_reviewed.unwrap(),
                    invoice_hash: r.invoice_hash.unwrap(),
                    invoice_payment_request: r.invoice_payment_request.unwrap(),
                    review_rating: r.review_rating.unwrap().try_into().unwrap(),
                    review_text: r.review_text.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                    payment_time_ms: r.payment_time_ms.unwrap().try_into().unwrap(),
                    review_time_ms: r.review_time_ms.unwrap().try_into().unwrap(),
                };
                let l = r.id.map(|listing_id| Listing {
                    id: Some(listing_id.try_into().unwrap()),
                    public_id: r.listing_public_id.unwrap(),
                    user_id: r.listing_user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.listing_created_time_ms.unwrap().try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                OrderCard {
                    order: o,
                    listing: l,
                    image: i,
                    user: u,
                }
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(orders)
    }

    pub async fn all_paid_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<OrderCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let orders = sqlx::query!(
            "
select
 orders.id as order_id, orders.public_id as order_public_id, orders.buyer_user_id as order_buyer_user_id, orders.seller_user_id as order_seller_user_id, orders.quantity as order_quantity, orders.listing_id as order_listing_id, orders.shipping_option_id, orders.shipping_instructions, orders.amount_owed_sat, orders.seller_credit_sat, orders.paid, orders.shipped, orders.canceled_by_seller, orders.canceled_by_buyer, orders.reviewed as order_reviewed, orders.invoice_hash, orders.invoice_payment_request, orders.review_rating, orders.review_text, orders.created_time_ms, orders.payment_time_ms, orders.review_time_ms, listings.id, listings.public_id as listing_public_id, listings.user_id as listing_user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms as listing_created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 orders
LEFT JOIN
 listings
ON
 orders.listing_id = listings.id
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
LEFT JOIN
 users
ON
 listings.user_id = users.id
WHERE
 orders.paid
AND
 order_buyer_user_id = ?
GROUP BY
 orders.id
ORDER BY orders.payment_time_ms DESC
LIMIT ?
OFFSET ?
;",
            user_id,
            limit,
            offset,
        )
            .fetch(&mut **db)
            .map_ok(|r| {
                let o = Order {
                    id: Some(r.order_id.unwrap().try_into().unwrap()),
                    public_id: r.order_public_id.unwrap(),
                    quantity: r.order_quantity.unwrap().try_into().unwrap(),
                    buyer_user_id: r.order_buyer_user_id.unwrap().try_into().unwrap(),
                    seller_user_id: r.order_seller_user_id.unwrap().try_into().unwrap(),
                    listing_id: r.order_listing_id.unwrap().try_into().unwrap(),
                    shipping_option_id: r.shipping_option_id.unwrap().try_into().unwrap(),
                    shipping_instructions: r.shipping_instructions.unwrap(),
                    amount_owed_sat: r.amount_owed_sat.unwrap().try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.unwrap().try_into().unwrap(),
                    paid: r.paid.unwrap(),
                    shipped: r.shipped.unwrap(),
                    canceled_by_seller: r.canceled_by_seller.unwrap(),
                    canceled_by_buyer: r.canceled_by_buyer.unwrap(),
                    reviewed: r.order_reviewed.unwrap(),
                    invoice_hash: r.invoice_hash.unwrap(),
                    invoice_payment_request: r.invoice_payment_request.unwrap(),
                    review_rating: r.review_rating.unwrap().try_into().unwrap(),
                    review_text: r.review_text.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                    payment_time_ms: r.payment_time_ms.unwrap().try_into().unwrap(),
                    review_time_ms: r.review_time_ms.unwrap().try_into().unwrap(),
                };
                let l = r.id.map(|listing_id| Listing {
                    id: Some(listing_id.try_into().unwrap()),
                    public_id: r.listing_public_id.unwrap(),
                    user_id: r.listing_user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.listing_created_time_ms.unwrap().try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                OrderCard {
                    order: o,
                    listing: l,
                    image: i,
                    user: u,
                }
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(orders)
    }

    pub async fn all_received_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<OrderCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let orders = sqlx::query!(
            "
select
 orders.id as order_id, orders.public_id as order_public_id, orders.buyer_user_id as order_buyer_user_id, orders.seller_user_id as order_seller_user_id, orders.quantity as order_quantity, orders.listing_id as order_listing_id, orders.shipping_option_id, orders.shipping_instructions, orders.amount_owed_sat, orders.seller_credit_sat, orders.paid, orders.shipped, orders.canceled_by_seller, orders.canceled_by_buyer, orders.reviewed as order_reviewed, orders.invoice_hash, orders.invoice_payment_request, orders.review_rating, orders.review_text, orders.created_time_ms, orders.payment_time_ms, orders.review_time_ms, listings.id, listings.public_id as listing_public_id, listings.user_id as listing_user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms as listing_created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 orders
LEFT JOIN
 listings
ON
 orders.listing_id = listings.id
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
LEFT JOIN
 users
ON
 listings.user_id = users.id
WHERE
 orders.shipped
AND
 order_seller_user_id = ?
GROUP BY
 orders.id
ORDER BY orders.payment_time_ms DESC
LIMIT ?
OFFSET ?
;",
            user_id,
            limit,
            offset,
        )
            .fetch(&mut **db)
            .map_ok(|r| {
                let o = Order {
                    id: Some(r.order_id.unwrap().try_into().unwrap()),
                    public_id: r.order_public_id.unwrap(),
                    quantity: r.order_quantity.unwrap().try_into().unwrap(),
                    buyer_user_id: r.order_buyer_user_id.unwrap().try_into().unwrap(),
                    seller_user_id: r.order_seller_user_id.unwrap().try_into().unwrap(),
                    listing_id: r.order_listing_id.unwrap().try_into().unwrap(),
                    shipping_option_id: r.shipping_option_id.unwrap().try_into().unwrap(),
                    shipping_instructions: r.shipping_instructions.unwrap(),
                    amount_owed_sat: r.amount_owed_sat.unwrap().try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.unwrap().try_into().unwrap(),
                    paid: r.paid.unwrap(),
                    shipped: r.shipped.unwrap(),
                    canceled_by_seller: r.canceled_by_seller.unwrap(),
                    canceled_by_buyer: r.canceled_by_buyer.unwrap(),
                    reviewed: r.order_reviewed.unwrap(),
                    invoice_hash: r.invoice_hash.unwrap(),
                    invoice_payment_request: r.invoice_payment_request.unwrap(),
                    review_rating: r.review_rating.unwrap().try_into().unwrap(),
                    review_text: r.review_text.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                    payment_time_ms: r.payment_time_ms.unwrap().try_into().unwrap(),
                    review_time_ms: r.review_time_ms.unwrap().try_into().unwrap(),
                };
                let l = r.id.map(|listing_id| Listing {
                    id: Some(listing_id.try_into().unwrap()),
                    public_id: r.listing_public_id.unwrap(),
                    user_id: r.listing_user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.listing_created_time_ms.unwrap().try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                OrderCard {
                    order: o,
                    listing: l,
                    image: i,
                    user: u,
                }
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(orders)
    }

    pub async fn all_processing_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<OrderCard>, sqlx::Error> {
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let orders = sqlx::query!(
            "
select
 orders.id as order_id, orders.public_id as order_public_id, orders.buyer_user_id as order_buyer_user_id, orders.seller_user_id as order_seller_user_id, orders.quantity as order_quantity, orders.listing_id as order_listing_id, orders.shipping_option_id, orders.shipping_instructions, orders.amount_owed_sat, orders.seller_credit_sat, orders.paid, orders.shipped, orders.canceled_by_seller, orders.canceled_by_buyer, orders.reviewed as order_reviewed, orders.invoice_hash, orders.invoice_payment_request, orders.review_rating, orders.review_text, orders.created_time_ms, orders.payment_time_ms, orders.review_time_ms, listings.id, listings.public_id as listing_public_id, listings.user_id as listing_user_id, listings.title, listings.description, listings.price_sat, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.deactivated_by_seller, listings.deactivated_by_admin, listings.created_time_ms as listing_created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
from
 orders
LEFT JOIN
 listings
ON
 orders.listing_id = listings.id
LEFT JOIN
 listingimages
ON
 listings.id = listingimages.listing_id
AND
 listingimages.is_primary = (SELECT MAX(is_primary) FROM listingimages WHERE listing_id = listings.id)
LEFT JOIN
 users
ON
 listings.user_id = users.id
WHERE
 orders.paid
AND
 not (orders.shipped OR orders.canceled_by_seller OR orders.canceled_by_buyer)
AND
 orders.seller_user_id = ?
GROUP BY
 orders.id
ORDER BY orders.payment_time_ms DESC
LIMIT ?
OFFSET ?
;",
            user_id,
            limit,
            offset,
        )
            .fetch(&mut **db)
            .map_ok(|r| {

                let o = Order {
                    id: Some(r.order_id.unwrap().try_into().unwrap()),
                    public_id: r.order_public_id.unwrap(),
                    quantity: r.order_quantity.unwrap().try_into().unwrap(),
                    buyer_user_id: r.order_buyer_user_id.unwrap().try_into().unwrap(),
                    seller_user_id: r.order_seller_user_id.unwrap().try_into().unwrap(),
                    listing_id: r.order_listing_id.unwrap().try_into().unwrap(),
                    shipping_option_id: r.shipping_option_id.unwrap().try_into().unwrap(),
                    shipping_instructions: r.shipping_instructions.unwrap(),
                    amount_owed_sat: r.amount_owed_sat.unwrap().try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.unwrap().try_into().unwrap(),
                    paid: r.paid.unwrap(),
                    shipped: r.shipped.unwrap(),
                    canceled_by_seller: r.canceled_by_seller.unwrap(),
                    canceled_by_buyer: r.canceled_by_buyer.unwrap(),
                    reviewed: r.order_reviewed.unwrap(),
                    invoice_hash: r.invoice_hash.unwrap(),
                    invoice_payment_request: r.invoice_payment_request.unwrap(),
                    review_rating: r.review_rating.unwrap().try_into().unwrap(),
                    review_text: r.review_text.unwrap(),
                    created_time_ms: r.created_time_ms.unwrap().try_into().unwrap(),
                    payment_time_ms: r.payment_time_ms.unwrap().try_into().unwrap(),
                    review_time_ms: r.review_time_ms.unwrap().try_into().unwrap(),
                };
                let l = r.id.map(|listing_id| Listing {
                    id: Some(listing_id.try_into().unwrap()),
                    public_id: r.listing_public_id.unwrap(),
                    user_id: r.listing_user_id.unwrap().try_into().unwrap(),
                    title: r.title.unwrap(),
                    description: r.description.unwrap(),
                    price_sat: r.price_sat.unwrap().try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.unwrap().try_into().unwrap(),
                    submitted: r.submitted.unwrap(),
                    reviewed: r.reviewed.unwrap(),
                    approved: r.approved.unwrap(),
                    deactivated_by_seller: r.deactivated_by_seller.unwrap(),
                    deactivated_by_admin: r.deactivated_by_admin.unwrap(),
                    created_time_ms: r.listing_created_time_ms.unwrap().try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id.unwrap(),
                    listing_id: r.listing_id.unwrap().try_into().unwrap(),
                    image_data: r.image_data.unwrap(),
                    is_primary: r.is_primary.unwrap(),
                });
                let u = r.rocket_auth_user_id.map(|rocket_auth_user_id| RocketAuthUser {
                    id: Some(rocket_auth_user_id.try_into().unwrap()),
                    username: r.rocket_auth_user_username.unwrap(),
                });
                OrderCard {
                    order: o,
                    listing: l,
                    image: i,
                    user: u,
                }

            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(orders)
    }
}

impl AccountInfo {
    pub async fn account_info_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<AccountInfo, sqlx::Error> {
        let account_balance_sat = AccountInfo::total_account_balance_for_user(db, user_id).await?;
        let num_unshipped_orders = Order::num_processing_for_user(db, user_id).await?;
        Ok(AccountInfo {
            account_balance_sat,
            num_unshipped_orders,
        })
    }

    pub async fn all_account_balance_changes_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<AccountBalanceChange>, sqlx::Error> {
        // TODO: Order by event time in SQL query. When this is fixed: https://github.com/launchbadge/sqlx/issues/1350
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let account_balance_changes = sqlx::query("
SELECT * FROM
(select orders.seller_user_id as user_id, orders.seller_credit_sat as amount_change_sat, 'received_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 orders.shipped
AND
 orders.seller_user_id = ?
UNION ALL
select orders.buyer_user_id as user_id, orders.amount_owed_sat as amount_change_sat, 'refunded_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 (orders.canceled_by_seller OR orders.canceled_by_buyer)
AND
 orders.buyer_user_id = ?
UNION ALL
select withdrawals.user_id as user_id, (0 - withdrawals.amount_sat) as amount_change_sat, 'withdrawal' as event_type, withdrawals.public_id as event_id, withdrawals.created_time_ms as event_time_ms
from
 withdrawals
WHERE
 withdrawals.user_id = ?)
ORDER BY event_time_ms DESC
LIMIT ?
OFFSET ?
;")
            .bind(user_id)
            .bind(user_id)
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch(&mut **db)
            .map_ok(|r| AccountBalanceChange {
                amount_change_sat: r.try_get("amount_change_sat").unwrap(),
                event_type: r.try_get("event_type").unwrap(),
                event_id: r.try_get("event_id").unwrap(),
                event_time_ms: {
                    let time_ms_i64: i64 = r.try_get("event_time_ms").unwrap();
                    time_ms_i64 as u64
                },
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(account_balance_changes)
    }

    pub async fn total_account_balance_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<i64, sqlx::Error> {
        let account_balance_sat = sqlx::query("
SELECT SUM(amount_change_sat) as total_account_balance_sat FROM
(select orders.seller_user_id as user_id, orders.seller_credit_sat as amount_change_sat, 'received_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 orders.shipped
AND
 orders.seller_user_id = ?
UNION ALL
select orders.buyer_user_id as user_id, orders.amount_owed_sat as amount_change_sat, 'refunded_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 (orders.canceled_by_seller OR orders.canceled_by_buyer)
AND
 orders.buyer_user_id = ?
UNION ALL
select withdrawals.user_id as user_id, (0 - withdrawals.amount_sat) as amount_change_sat, 'withdrawal' as event_type, withdrawals.public_id as event_id, withdrawals.created_time_ms as event_time_ms
from
 withdrawals
WHERE
 withdrawals.user_id = ?)
;")
            .bind(user_id)
            .bind(user_id)
            .bind(user_id)
            .fetch_one(&mut **db)
            .map_ok(|r|  {
                let balance_sat_i64: i64 = r.try_get("total_account_balance_sat").unwrap();
                balance_sat_i64
            })
            .await?;

        Ok(account_balance_sat)
    }

    pub async fn all_account_balance_changes(
        db: &mut Connection<Db>,
        page_size: u32,
        page_num: u32,
    ) -> Result<Vec<AccountBalanceChange>, sqlx::Error> {
        // TODO: Order by event time in SQL query. When this is fixed: https://github.com/launchbadge/sqlx/issues/1350
        let offset = (page_num - 1) * page_size;
        let limit = page_size;
        let account_balance_changes = sqlx::query("
SELECT * FROM
(select orders.seller_user_id as user_id, orders.seller_credit_sat as amount_change_sat, 'received_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 orders.shipped
UNION ALL
select orders.buyer_user_id as user_id, orders.amount_owed_sat as amount_change_sat, 'refunded_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 (orders.canceled_by_seller OR orders.canceled_by_buyer)
UNION ALL
select orders.buyer_user_id as user_id, orders.amount_owed_sat as amount_change_sat, 'processing_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 NOT (orders.shipped OR orders.canceled_by_seller OR orders.canceled_by_buyer)
UNION ALL
select withdrawals.user_id as user_id, (0 - withdrawals.amount_sat) as amount_change_sat, 'withdrawal' as event_type, withdrawals.public_id as event_id, withdrawals.created_time_ms as event_time_ms
from
 withdrawals
UNION ALL
select useraccounts.user_id as user_id, useraccounts.amount_owed_sat as amount_change_sat, 'user_activation' as event_type, useraccounts.public_id as event_id, useraccounts.created_time_ms as event_time_ms
from
 useraccounts
WHERE
 useraccounts.paid)
ORDER BY event_time_ms DESC
LIMIT ?
OFFSET ?
;")
            .bind(limit)
            .bind(offset)
            .fetch(&mut **db)
            .map_ok(|r| AccountBalanceChange {
                amount_change_sat: r.try_get("amount_change_sat").unwrap(),
                event_type: r.try_get("event_type").unwrap(),
                event_id: r.try_get("event_id").unwrap(),
                event_time_ms: {
                    let time_ms_i64: i64 = r.try_get("event_time_ms").unwrap();
                    time_ms_i64 as u64
                },
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(account_balance_changes)
    }

    pub async fn total_market_liabilities_sat(db: &mut Connection<Db>) -> Result<i64, sqlx::Error> {
        let market_liabilities_sat = sqlx::query("
SELECT SUM(amount_change_sat) as total_market_liabilities_sat FROM
(select orders.seller_user_id as user_id, orders.seller_credit_sat as amount_change_sat, 'received_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 orders.shipped
UNION ALL
select orders.buyer_user_id as user_id, orders.amount_owed_sat as amount_change_sat, 'refunded_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 (orders.canceled_by_seller OR orders.canceled_by_buyer)
UNION ALL
select orders.buyer_user_id as user_id, orders.amount_owed_sat as amount_change_sat, 'processing_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 NOT (orders.shipped OR orders.canceled_by_seller OR orders.canceled_by_buyer)
UNION ALL
select withdrawals.user_id as user_id, (0 - withdrawals.amount_sat) as amount_change_sat, 'withdrawal' as event_type, withdrawals.public_id as event_id, withdrawals.created_time_ms as event_time_ms
from
 withdrawals
UNION ALL
select useraccounts.user_id as user_id, useraccounts.amount_owed_sat as amount_change_sat, 'user_activation' as event_type, useraccounts.public_id as event_id, useraccounts.created_time_ms as event_time_ms
from
 useraccounts
WHERE
 useraccounts.paid)
;")
            .fetch_one(&mut **db)
            .map_ok(|r|  {
                let balance_sat_i64: i64 = r.try_get("total_market_liabilities_sat").unwrap();
                balance_sat_i64
            })
            .await?;

        Ok(market_liabilities_sat)
    }

    // TODO: Use when sqlx is fixed.
    //     pub async fn account_balance(
    //         db: &mut Connection<Db>,
    //         user_id: i32,
    //     ) -> Result<u64, sqlx::Error> {
    //         let account_balance_result = sqlx::query!("
    // SELECT SUM(data.amount_change_sat) as account_balance, data.user_id as user_id
    // FROM
    // (select listings.user_id as user_id, orders.seller_credit_sat as amount_change_sat, 'received_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
    // from
    //  orders
    // LEFT JOIN
    //  listings
    // ON
    //  orders.listing_id = listings.id
    // WHERE
    //  orders.paid
    // AND
    //  orders.shipped
    // AND
    //  listings.user_id = ?
    // UNION ALL
    // select orders.user_id as user_id, orders.amount_owed_sat as amount_change_sat, 'refunded_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
    // from
    //  orders
    // WHERE
    //  orders.paid
    // AND
    //  not orders.shipped
    // AND
    //  orders.user_id = ?
    // ) data
    // GROUP BY user_id
    // ;",
    //         user_id, user_id)
    //         .fetch_optional(&mut **db)
    //         .await?;

    //         let account_balance = match account_balance_result {
    //             Some(r) => r.account_balance,
    //             None => 0,
    //         };

    //         Ok(account_balance)
    //     }
}

impl Withdrawal {
    pub async fn do_withdrawal(
        withdrawal: Withdrawal,
        db: &mut Connection<Db>,
        send_withdrawal_funds_future: impl Future<
            Output = Result<tonic_openssl_lnd::lnrpc::SendResponse, String>,
        >,
        max_withdrawals_per_interval: u32,
        interval_start_time_ms: u64,
    ) -> Result<i32, String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        // Insert the new withdrawal.
        let amount_sat: i64 = withdrawal.amount_sat.try_into().unwrap();
        let created_time_ms: i64 = withdrawal.created_time_ms.try_into().unwrap();
        let insert_result = sqlx::query!(
            "INSERT INTO withdrawals (public_id, user_id, amount_sat, invoice_hash, invoice_payment_request, created_time_ms) VALUES (?, ?, ?, ?, ?, ?)",
            withdrawal.public_id,
            withdrawal.user_id,
            amount_sat,
            withdrawal.invoice_hash,
            withdrawal.invoice_payment_request,
            created_time_ms,
        )
            .execute(&mut *tx)
            .await
            .map_err(|_| "failed to insert new withdrawal.")?;
        let new_withdrawal_id = insert_result.last_insert_rowid();

        // Check if any constraints are violated.
        let user_id = withdrawal.user_id;

        let start_time_ms_i64: i64 = interval_start_time_ms.try_into().unwrap();
        let withdrawal_count = sqlx::query!(
            "
select count(id) as withdrawal_count from withdrawals
WHERE
 user_id = ?
AND
 created_time_ms > ?
ORDER BY withdrawals.created_time_ms ASC;",
            user_id,
            start_time_ms_i64,
        )
        .fetch_one(&mut *tx)
        .map_ok(|r| r.withdrawal_count as u32)
        .await
        .map_err(|_| "failed to get withdrawal count.")?;

        if withdrawal_count > max_withdrawals_per_interval {
            return Err(format!(
                "More than {:?} withdrawals in a single day not allowed.",
                max_withdrawals_per_interval,
            ));
        }

        let account_balance_sat = sqlx::query("
SELECT SUM(amount_change_sat) as total_account_balance_sat FROM
(select orders.seller_user_id as user_id, orders.seller_credit_sat as amount_change_sat, 'received_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 orders.shipped
AND
 orders.seller_user_id = ?
UNION ALL
select orders.buyer_user_id as user_id, orders.amount_owed_sat as amount_change_sat, 'refunded_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 (orders.canceled_by_seller OR orders.canceled_by_buyer)
AND
 orders.buyer_user_id = ?
UNION ALL
select withdrawals.user_id as user_id, (0 - withdrawals.amount_sat) as amount_change_sat, 'withdrawal' as event_type, withdrawals.public_id as event_id, withdrawals.created_time_ms as event_time_ms
from
 withdrawals
WHERE
 withdrawals.user_id = ?)
;")
            .bind(user_id)
            .bind(user_id)
            .bind(user_id)
            .fetch_one(&mut *tx)
            .map_ok(|r|  {
                let balance_sat_i64: i64 = r.try_get("total_account_balance_sat").unwrap();
                balance_sat_i64
            })
            .await
            .map_err(|_| "failed to insert get account balance changes.")?;

        if account_balance_sat < 0 {
            return Err("Insufficient funds for withdrawal.".to_string());
        }

        let send_response = send_withdrawal_funds_future
            .await
            .map_err(|e| format!("failed to send withdrawal payment: {:?}", e))?;

        // Update the withdrawal row with the payment invoice hash.
        let payment_hash_hex = util::to_hex(&send_response.payment_hash);
        sqlx::query!(
            "UPDATE withdrawals SET invoice_hash = ? WHERE id = ?",
            payment_hash_hex,
            new_withdrawal_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to update new withdrawal payment hash.")?;

        tx.commit()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        Ok(new_withdrawal_id as _)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<Withdrawal, sqlx::Error> {
        let withdrawal = sqlx::query!("select * from withdrawals WHERE public_id = ?;", public_id)
            .fetch_one(&mut **db)
            .map_ok(|r| Withdrawal {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                user_id: r.user_id.try_into().unwrap(),
                amount_sat: r.amount_sat.try_into().unwrap(),
                invoice_hash: r.invoice_hash,
                invoice_payment_request: r.invoice_payment_request,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(withdrawal)
    }

    //     pub async fn count_for_user_since_time_ms(
    //         db: &mut Connection<Db>,
    //         user_id: i32,
    //         start_time_ms: u64,
    //     ) -> Result<u32, sqlx::Error> {
    //         let start_time_ms_i64: i64 = start_time_ms.try_into().unwrap();

    //         let withdrawal_count = sqlx::query!(
    //             "
    // select count(id) as withdrawal_count from withdrawals
    // WHERE
    //  user_id = ?
    // AND
    //  created_time_ms > ?
    // ORDER BY withdrawals.created_time_ms ASC;",
    //             user_id,
    //             start_time_ms_i64,
    //         )
    //         .fetch_one(&mut **db)
    //         .map_ok(|r| r.withdrawal_count)
    //         .await?;

    //         Ok(withdrawal_count.try_into().unwrap())
    //     }
}

impl AdminInfo {
    pub async fn admin_info(db: &mut Connection<Db>) -> Result<AdminInfo, sqlx::Error> {
        let num_pending_listings = Listing::num_pending(db).await?;
        Ok(AdminInfo {
            num_pending_listings,
        })
    }
}

impl UserAccount {
    /// Returns the id of the inserted row.
    pub async fn insert(
        user_account: UserAccount,
        db: &mut Connection<Db>,
    ) -> Result<i32, sqlx::Error> {
        let amount_owed_sat: i64 = user_account.amount_owed_sat.try_into().unwrap();
        let created_time_ms: i64 = user_account.created_time_ms.try_into().unwrap();
        let payment_time_ms: i64 = user_account.payment_time_ms.try_into().unwrap();

        let insert_result = sqlx::query!(
            "INSERT INTO useraccounts (public_id, user_id, amount_owed_sat, paid, disabled, invoice_payment_request, invoice_hash, created_time_ms, payment_time_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            user_account.public_id,
            user_account.user_id,
            amount_owed_sat,
            user_account.paid,
            user_account.disabled,
            user_account.invoice_payment_request,
            user_account.invoice_hash,
            created_time_ms,
            payment_time_ms,
        )
            .execute(&mut **db)
            .await?;

        Ok(insert_result.last_insert_rowid() as _)
    }

    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<UserAccount, sqlx::Error> {
        let user_account = sqlx::query!("select * from useraccounts WHERE user_id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| UserAccount {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                user_id: r.user_id.try_into().unwrap(),
                amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                paid: r.paid,
                disabled: r.disabled,
                invoice_payment_request: r.invoice_payment_request,
                invoice_hash: r.invoice_hash,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
                payment_time_ms: r.payment_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(user_account)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<UserAccount, sqlx::Error> {
        let user_account =
            sqlx::query!("select * from useraccounts WHERE public_id = ?;", public_id)
                .fetch_one(&mut **db)
                .map_ok(|r| UserAccount {
                    id: Some(r.id.try_into().unwrap()),
                    public_id: r.public_id,
                    user_id: r.user_id.try_into().unwrap(),
                    amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                    paid: r.paid,
                    disabled: r.disabled,
                    invoice_payment_request: r.invoice_payment_request,
                    invoice_hash: r.invoice_hash,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                    payment_time_ms: r.payment_time_ms.try_into().unwrap(),
                })
                .await?;

        Ok(user_account)
    }

    pub async fn single_by_invoice_hash(
        db: &mut PoolConnection<Sqlite>,
        invoice_hash: &str,
    ) -> Result<UserAccount, sqlx::Error> {
        let user_account = sqlx::query!(
            "select * from useraccounts WHERE invoice_hash = ?;",
            invoice_hash
        )
        .fetch_one(&mut **db)
        .map_ok(|r| UserAccount {
            id: Some(r.id.try_into().unwrap()),
            public_id: r.public_id,
            user_id: r.user_id.try_into().unwrap(),
            amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
            paid: r.paid,
            disabled: r.disabled,
            invoice_payment_request: r.invoice_payment_request,
            invoice_hash: r.invoice_hash,
            created_time_ms: r.created_time_ms.try_into().unwrap(),
            payment_time_ms: r.payment_time_ms.try_into().unwrap(),
        })
        .await?;

        Ok(user_account)
    }

    pub async fn mark_as_paid(
        db: &mut PoolConnection<Sqlite>,
        user_account_id: i32,
        time_now_ms: u64,
    ) -> Result<(), sqlx::Error> {
        let time_now_ms_i64: i64 = time_now_ms.try_into().unwrap();

        sqlx::query!(
            "UPDATE useraccounts SET paid = true, payment_time_ms = ? WHERE id = ?",
            time_now_ms_i64,
            user_account_id,
        )
        .execute(&mut **db)
        .await?;

        Ok(())
    }

    pub async fn all_older_than(
        db: &mut PoolConnection<Sqlite>,
        created_time_ms: u64,
    ) -> Result<Vec<UserAccount>, sqlx::Error> {
        let created_time_ms_i64: i64 = created_time_ms.try_into().unwrap();

        let user_accounts = sqlx::query!(
            "
select *
from
 useraccounts
WHERE
 created_time_ms < ?
AND
 NOT paid
;",
            created_time_ms_i64,
        )
        .fetch(&mut **db)
        .map_ok(|r| UserAccount {
            id: Some(r.id.try_into().unwrap()),
            public_id: r.public_id,
            user_id: r.user_id.try_into().unwrap(),
            amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
            paid: r.paid,
            disabled: r.disabled,
            invoice_payment_request: r.invoice_payment_request,
            invoice_hash: r.invoice_hash,
            created_time_ms: r.created_time_ms.try_into().unwrap(),
            payment_time_ms: r.payment_time_ms.try_into().unwrap(),
        })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(user_accounts)
    }

    pub async fn delete_expired_user_account(
        db: &mut PoolConnection<Sqlite>,
        user_account_id: i32,
        cancel_user_account_invoice_future: impl Future<
            Output = Result<tonic_openssl_lnd::invoicesrpc::CancelInvoiceResp, String>,
        >,
    ) -> Result<(), String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        sqlx::query!(
            "
DELETE FROM useraccounts
WHERE
 user_id = ?
AND
 NOT paid
;",
            user_account_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete user account from database.")?;

        sqlx::query!(
            "
DELETE FROM users
WHERE
 id = ?
;",
            user_account_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete user from database.")?;

        cancel_user_account_invoice_future
            .await
            .map_err(|e| format!("failed to cancel user account invoice: {:?}", e))?;

        tx.commit()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        Ok(())
    }

    pub async fn delete_users_with_no_account(
        db: &mut PoolConnection<Sqlite>,
    ) -> Result<(), String> {
        sqlx::query!(
            "
DELETE FROM users
WHERE
 NOT is_admin
AND
 id IN
(SELECT users.id FROM users
LEFT JOIN
 useraccounts
ON
 users.id=useraccounts.user_id
WHERE
 useraccounts.user_id IS NULL);
;"
        )
        .execute(&mut **db)
        .await
        .map_err(|_| "failed to delete user account from database.")?;

        Ok(())
    }

    pub async fn do_deactivation(
        amount_sat: u64,
        user_account: UserAccount,
        db: &mut Connection<Db>,
        send_deactivation_funds_future: impl Future<
            Output = Result<tonic_openssl_lnd::lnrpc::SendResponse, String>,
        >,
    ) -> Result<(), String> {
        let mut tx = db
            .begin()
            .await
            .map_err(|_| "failed to begin transaction.")?;

        // Insert the new withdrawal.
        let activation_bond_amount_sat: i64 = user_account.amount_owed_sat.try_into().unwrap();
        let amount_sat: i64 = amount_sat.try_into().unwrap();
        let delete_user_account_result = sqlx::query!(
            "
DELETE FROM useraccounts
WHERE
 user_id = ?
AND
 paid = true
;",
            user_account.user_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete user account.")?;

        // Validate that at least one paid user account was deleted
        if delete_user_account_result.rows_affected() < 1 {
            return Err("No user bond found.".to_string());
        }

        sqlx::query!(
            "
DELETE FROM users
WHERE id = ?
;",
            user_account.user_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "failed to delete user.")?;

        if activation_bond_amount_sat - amount_sat < 0 {
            return Err("Insufficient funds for deactivation.".to_string());
        }

        send_deactivation_funds_future
            .await
            .map_err(|e| format!("failed to send deactivation payment: {:?}", e))?;

        tx.commit()
            .await
            .map_err(|_| "failed to commit transaction.")?;

        Ok(())
    }
}
