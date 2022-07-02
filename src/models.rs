use crate::db::Db;
use crate::rocket::futures::TryFutureExt;
use crate::rocket::futures::TryStreamExt;
use rocket::fs::TempFile;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::{sqlx, Connection};
use std::result::Result;
extern crate base64;
use sqlx::pool::PoolConnection;
use sqlx::Acquire;
use sqlx::Sqlite;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Post {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub title: String,
    pub text: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Listing {
    pub id: Option<i32>,
    pub public_id: String,
    pub user_id: i32,
    pub title: String,
    pub description: String,
    pub price_sat: u64,
    pub quantity: u32,
    pub fee_rate_basis_points: u32,
    pub submitted: bool,
    pub reviewed: bool,
    pub approved: bool,
    pub removed: bool,
    pub created_time_ms: u64,
}

#[derive(Debug, FromForm)]
pub struct InitialListingInfo {
    pub title: String,
    pub description: String,
    pub price_sat: u64,
    pub quantity: u32,
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
    pub user: RocketAuthUser,
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
    pub price_sat: u64,
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
}

#[derive(Debug, FromForm)]
pub struct MarketNameInput {
    pub market_name: String,
}

#[derive(Debug, FromForm)]
pub struct FeeRateInput {
    pub fee_rate_basis_points: i32,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Order {
    pub id: Option<i32>,
    pub public_id: String,
    pub quantity: u32,
    pub user_id: i32,
    pub listing_id: i32,
    pub shipping_option_id: i32,
    pub shipping_instructions: String,
    pub amount_owed_sat: u64,
    pub seller_credit_sat: u64,
    pub paid: bool,
    pub completed: bool,
    pub invoice_hash: String,
    pub invoice_payment_request: String,
    pub created_time_ms: u64,
}

#[derive(Debug, FromForm, Clone)]
pub struct OrderInfo {
    pub quantity: u32,
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
    pub amount_earned_sat: u64,
    pub amount_refunded_sat: u64,
    pub amount_withdrawn_sat: u64,
    pub account_balance_sat: u64,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AccountBalanceChange {
    pub user_id: String,
    pub amount_change_sat: u64,
    pub event_type: String,
    pub event_id: String,
    pub event_time_ms: u64,
}

impl Listing {
    // pub async fn all(db: &mut Connection<Db>) -> Result<Vec<Listing>, sqlx::Error> {
    //     let listings = sqlx::query!("select * from listings;")
    //         .fetch(&mut **db)
    //         .map_ok(|r| Listing {
    //             id: Some(r.id.try_into().unwrap()),
    //             public_id: r.public_id,
    //             user_id: r.user_id.try_into().unwrap(),
    //             title: r.title,
    //             description: r.description,
    //             price_sat: r.price_sat.try_into().unwrap(),
    //             quantity: r.quantity.try_into().unwrap(),
    //             fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
    //             submitted: r.submitted,
    //             reviewed: r.reviewed,
    //             approved: r.approved,
    //             removed: r.removed,
    //             created_time_ms: r.created_time_ms.try_into().unwrap(),
    //         })
    //         .try_collect::<Vec<_>>()
    //         .await?;

    //     println!("{}", listings.len());
    //     println!("{:?}", listings);

    //     Ok(listings)
    // }

    /// Returns the id of the inserted row.
    pub async fn insert(listing: Listing, db: &mut Connection<Db>) -> Result<i32, sqlx::Error> {
        let price_sat: i64 = listing.price_sat.try_into().unwrap();
        let created_time_ms: i64 = listing.created_time_ms.try_into().unwrap();
        println!("price_sat: {:?}", price_sat);
        println!("created_time_ms: {:?}", created_time_ms);

        let insert_result = sqlx::query!(
            "INSERT INTO listings (public_id, user_id, title, description, price_sat, quantity, fee_rate_basis_points, submitted, reviewed, approved, removed, created_time_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            listing.public_id,
            listing.user_id,
            listing.title,
            listing.description,
            price_sat,
            listing.quantity,
            listing.fee_rate_basis_points,
            listing.submitted,
            listing.reviewed,
            listing.approved,
            listing.removed,
            created_time_ms,
        )
            .execute(&mut **db)
            .await?;

        println!("{:?}", insert_result);

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
                quantity: r.quantity.try_into().unwrap(),
                fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                submitted: r.submitted,
                reviewed: r.reviewed,
                approved: r.approved,
                removed: r.removed,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
            })
            .await?;

        println!("{:?}", listing);
        println!("price_sat: {:?}", listing.price_sat);
        println!("created_time_ms: {:?}", listing.created_time_ms);

        Ok(listing)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<Listing, sqlx::Error> {
        let listing = sqlx::query!("select * from listings WHERE public_id = ?;", public_id)
            .fetch_one(&mut **db)
            .map_ok(|r| Listing {
                id: r.id.map(|n| n.try_into().unwrap()),
                public_id: r.public_id,
                user_id: r.user_id.try_into().unwrap(),
                title: r.title,
                description: r.description,
                price_sat: r.price_sat.try_into().unwrap(),
                quantity: r.quantity.try_into().unwrap(),
                fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                submitted: r.submitted,
                reviewed: r.reviewed,
                approved: r.approved,
                removed: r.removed,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
            })
            .await?;

        println!("{:?}", listing);
        println!("price_sat: {:?}", listing.price_sat);
        println!("created_time_ms: {:?}", listing.created_time_ms);

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

    pub async fn mark_as_removed(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE listings SET removed = true WHERE public_id = ?",
            public_id,
        )
        .execute(&mut **db)
        .await?;
        Ok(())
    }
}

impl ListingImage {
    /// Returns the number of affected rows: 1.
    pub async fn insert(
        listingimage: ListingImage,
        db: &mut Connection<Db>,
    ) -> Result<usize, sqlx::Error> {
        let insert_result = sqlx::query!(
            "INSERT INTO listingimages (public_id, listing_id, image_data) VALUES (?, ?, ?)",
            listingimage.public_id,
            listingimage.listing_id,
            listingimage.image_data,
        )
        .execute(&mut **db)
        .await?;

        println!("{:?}", insert_result);

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

        println!("{}", listing_images.len());

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
            id: r.id.map(|n| n.try_into().unwrap()),
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
        let mut tx = db.begin().await?;

        // Set all images for listing_id to not primary.
        sqlx::query!(
            "UPDATE listingimages SET is_primary = false WHERE listing_id = ?",
            listing_id
        )
        .execute(&mut tx)
        .await?;

        // Set image for listing_id and image_id to primary.
        let update_result = sqlx::query!(
            "UPDATE listingimages SET is_primary = true WHERE listing_id = ? AND public_id = ?",
            listing_id,
            image_id,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(update_result.rows_affected() as _)
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
    // pub async fn all(db: &mut Connection<Db>) -> Result<Vec<RocketAuthUser>, sqlx::Error> {
    //     let rocket_auth_users = sqlx::query!("select * from users;")
    //         .fetch(&mut **db)
    //         .map_ok(|r| RocketAuthUser {
    //             id: Some(r.id as i32),
    //             username: r.email.unwrap(),
    //         })
    //         .try_collect::<Vec<_>>()
    //         .await?;

    //     println!("{}", rocket_auth_users.len());

    //     Ok(rocket_auth_users)
    // }

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
                image_data_base64: base64::encode(&img.image_data),
                is_primary: img.is_primary,
            })
            .collect::<Vec<_>>();
        let shipping_options =
            ShippingOption::all_for_listing(&mut *db, listing.id.unwrap()).await?;
        let rocket_auth_user = RocketAuthUser::single(&mut *db, listing.user_id).await?;

        let listing_display = ListingDisplay {
            listing: listing,
            images: image_displays,
            shipping_options: shipping_options,
            user: rocket_auth_user,
        };

        Ok(listing_display)
    }
}

impl ListingCard {
    pub async fn all_approved(db: &mut Connection<Db>) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of completed orders.
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.quantity, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.removed, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
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
 not listings.removed
AND
 listings.quantity > 0
GROUP BY
 listings.id
;")
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.try_into().unwrap()),
                    public_id: r.public_id,
                    user_id: r.user_id.try_into().unwrap(),
                    title: r.title,
                    description: r.description,
                    price_sat: r.price_sat.try_into().unwrap(),
                    quantity: r.quantity.try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    submitted: r.submitted,
                    reviewed: r.reviewed,
                    approved: r.approved,
                    removed: r.removed,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id,
                    listing_id: r.listing_id.try_into().unwrap(),
                    image_data: r.image_data,
                    is_primary: r.is_primary,
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

    pub async fn all_pending(db: &mut Connection<Db>) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of completed orders.
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.quantity, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.removed, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
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
AND
 not listings.removed
AND
 listings.quantity > 0
GROUP BY
 listings.id
;")
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.try_into().unwrap()),
                    public_id: r.public_id,
                    user_id: r.user_id.try_into().unwrap(),
                    title: r.title,
                    description: r.description,
                    price_sat: r.price_sat.try_into().unwrap(),
                    quantity: r.quantity.try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    submitted: r.submitted,
                    reviewed: r.reviewed,
                    approved: r.approved,
                    removed: r.removed,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id,
                    listing_id: r.listing_id.try_into().unwrap(),
                    image_data: r.image_data,
                    is_primary: r.is_primary,
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
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of completed orders.
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.quantity, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.removed, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
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
 not listings.removed
AND
 listings.quantity > 0
AND
 users.id = ?
GROUP BY
 listings.id
;", user_id)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.try_into().unwrap()),
                    public_id: r.public_id,
                    user_id: r.user_id.try_into().unwrap(),
                    title: r.title,
                    description: r.description,
                    price_sat: r.price_sat.try_into().unwrap(),
                    quantity: r.quantity.try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    submitted: r.submitted,
                    reviewed: r.reviewed,
                    approved: r.approved,
                    removed: r.removed,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id,
                    listing_id: r.listing_id.try_into().unwrap(),
                    image_data: r.image_data,
                    is_primary: r.is_primary,
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
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of completed orders.
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.quantity, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.removed, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
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
 not listings.removed
AND
 listings.quantity > 0
AND
 users.id = ?
GROUP BY
 listings.id
;", user_id)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.try_into().unwrap()),
                    public_id: r.public_id,
                    user_id: r.user_id.try_into().unwrap(),
                    title: r.title,
                    description: r.description,
                    price_sat: r.price_sat.try_into().unwrap(),
                    quantity: r.quantity.try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    submitted: r.submitted,
                    reviewed: r.reviewed,
                    approved: r.approved,
                    removed: r.removed,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id,
                    listing_id: r.listing_id.try_into().unwrap(),
                    image_data: r.image_data,
                    is_primary: r.is_primary,
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
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of completed orders.
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.quantity, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.removed, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
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
 not listings.removed
AND
 listings.quantity > 0
AND
 users.id = ?
GROUP BY
 listings.id
;", user_id)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.try_into().unwrap()),
                    public_id: r.public_id,
                    user_id: r.user_id.try_into().unwrap(),
                    title: r.title,
                    description: r.description,
                    price_sat: r.price_sat.try_into().unwrap(),
                    quantity: r.quantity.try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    submitted: r.submitted,
                    reviewed: r.reviewed,
                    approved: r.approved,
                    removed: r.removed,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id,
                    listing_id: r.listing_id.try_into().unwrap(),
                    image_data: r.image_data,
                    is_primary: r.is_primary,
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

    pub async fn all_approved_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        // TODO: change WHERE condition to use dynamically calculated remaining quantity
        // based on number of completed orders.
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_sat, listings.quantity, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.removed, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
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
 not listings.removed
AND
 listings.quantity > 0
AND
 users.id = ?
GROUP BY
 listings.id
;", user_id)
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id.try_into().unwrap()),
                    public_id: r.public_id,
                    user_id: r.user_id.try_into().unwrap(),
                    title: r.title,
                    description: r.description,
                    price_sat: r.price_sat.try_into().unwrap(),
                    quantity: r.quantity.try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    submitted: r.submitted,
                    reviewed: r.reviewed,
                    approved: r.approved,
                    removed: r.removed,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                };
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id,
                    listing_id: r.listing_id.try_into().unwrap(),
                    image_data: r.image_data,
                    is_primary: r.is_primary,
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
                image_data_base64: base64::encode(&image.image_data),
                is_primary: image.is_primary,
            }),
            user: card.clone().user,
        }
    }

    pub async fn all_approved(
        db: &mut Connection<Db>,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards = ListingCard::all_approved(db).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(|card| ListingCardDisplay::listing_card_to_display(card))
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_pending(
        db: &mut Connection<Db>,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards = ListingCard::all_pending(db).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(|card| ListingCardDisplay::listing_card_to_display(card))
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_unsubmitted_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards = ListingCard::all_unsubmitted_for_user(db, user_id).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(|card| ListingCardDisplay::listing_card_to_display(card))
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_pending_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards = ListingCard::all_pending_for_user(db, user_id).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(|card| ListingCardDisplay::listing_card_to_display(card))
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_rejected_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards = ListingCard::all_rejected_for_user(db, user_id).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(|card| ListingCardDisplay::listing_card_to_display(card))
            .collect::<Vec<_>>();

        Ok(listing_card_displays)
    }

    pub async fn all_approved_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards = ListingCard::all_approved_for_user(db, user_id).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(|card| ListingCardDisplay::listing_card_to_display(card))
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
        // let my_uuid_str = Uuid::new_v4().to_string();
        let price_sat: i64 = shipping_option.price_sat.try_into().unwrap();

        println!("inserting shipping option: {:?}", shipping_option);

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

        println!("insert_result: {:?}", insert_result);

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

        println!("{}", shipping_options.len());

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
            id: r.id.map(|n| n.try_into().unwrap()),
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
    pub async fn single(
        db: &mut Connection<Db>,
        default_admin_settings: AdminSettings,
    ) -> Result<AdminSettings, sqlx::Error> {
        let maybe_admin_settings = sqlx::query!("select * from adminsettings;")
            .fetch_optional(&mut **db)
            .map_ok(|maybe_r| {
                maybe_r.map(|r| AdminSettings {
                    id: Some(r.id.try_into().unwrap()),
                    market_name: r.market_name,
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                })
            })
            .await?;

        let admin_settings = maybe_admin_settings.unwrap_or(default_admin_settings);

        println!("{:?}", admin_settings);

        Ok(admin_settings)
    }

    /// Returns the number of affected rows: 1.
    async fn insert_if_doesnt_exist(
        db: &mut Connection<Db>,
        admin_settings: AdminSettings,
    ) -> Result<(), sqlx::Error> {
        let mut tx = db.begin().await?;

        let maybe_admin_settings = sqlx::query!("select * from adminsettings;")
            .fetch_optional(&mut tx)
            .await?;

        if let None = maybe_admin_settings {
            sqlx::query!(
                "INSERT INTO adminsettings (market_name, fee_rate_basis_points) VALUES (?, ?)",
                admin_settings.market_name,
                admin_settings.fee_rate_basis_points,
            )
            .execute(&mut tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn set_market_name(
        db: &mut Connection<Db>,
        new_market_name: &str,
        default_admin_settings: AdminSettings,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db, default_admin_settings).await?;

        sqlx::query!("UPDATE adminsettings SET market_name = ?", new_market_name)
            .execute(&mut **db)
            .await?;

        Ok(())
    }

    pub async fn set_fee_rate(
        db: &mut Connection<Db>,
        new_fee_rate_basis_points: i32,
        default_admin_settings: AdminSettings,
    ) -> Result<(), sqlx::Error> {
        AdminSettings::insert_if_doesnt_exist(db, default_admin_settings).await?;

        sqlx::query!(
            "UPDATE adminsettings SET fee_rate_basis_points = ?",
            new_fee_rate_basis_points,
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

        println!("inserting order: {:?}", order);

        let insert_result = sqlx::query!(
            "INSERT INTO orders (public_id, user_id, quantity, listing_id, shipping_option_id, shipping_instructions, amount_owed_sat, seller_credit_sat, paid, completed, invoice_hash, invoice_payment_request, created_time_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            order.public_id,
            order.user_id,
            order.quantity,
            order.listing_id,
            order.shipping_option_id,
            order.shipping_instructions,
            amount_owed_sat,
            seller_credit_sat,
            order.paid,
            order.completed,
            order.invoice_hash,
            order.invoice_payment_request,
            created_time_ms,
        )
            .execute(&mut **db)
            .await?;

        println!("insert_result: {:?}", insert_result);

        Ok(insert_result.last_insert_rowid() as _)
    }

    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<Order, sqlx::Error> {
        let order = sqlx::query!("select * from orders WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| Order {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id,
                quantity: r.quantity.try_into().unwrap(),
                user_id: r.user_id.try_into().unwrap(),
                listing_id: r.listing_id.try_into().unwrap(),
                shipping_option_id: r.shipping_option_id.try_into().unwrap(),
                shipping_instructions: r.shipping_instructions,
                amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                paid: r.paid,
                completed: r.completed,
                invoice_hash: r.invoice_hash,
                invoice_payment_request: r.invoice_payment_request,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
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
                id: r.id.map(|n| n.try_into().unwrap()),
                public_id: r.public_id,
                quantity: r.quantity.try_into().unwrap(),
                user_id: r.user_id.try_into().unwrap(),
                listing_id: r.listing_id.try_into().unwrap(),
                shipping_option_id: r.shipping_option_id.try_into().unwrap(),
                shipping_instructions: r.shipping_instructions,
                amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                paid: r.paid,
                completed: r.completed,
                invoice_hash: r.invoice_hash,
                invoice_payment_request: r.invoice_payment_request,
                created_time_ms: r.created_time_ms.try_into().unwrap(),
            })
            .await?;

        Ok(order)
    }

    pub async fn quantity_of_listing_sold(
        db: &mut Connection<Db>,
        listing_id: i32,
    ) -> Result<u32, sqlx::Error> {
        let sold_items = sqlx::query!(
            "
select
 SUM(orders.quantity) as quantity_sold
FROM
 orders
WHERE
 listing_id = ?
AND
 completed
GROUP BY
 listing_id
;",
            listing_id,
        )
        .fetch_optional(&mut **db)
        .await?;
        let quantity_sold = match sold_items {
            Some(r) => r.quantity_sold,
            None => 0,
        };
        Ok(quantity_sold.try_into().unwrap())
    }

    // pub async fn single_by_invoice_hash(
    //     db: &mut Connection<Db>,
    //     invoice_hash: &str,
    // ) -> Result<Option<Order>, sqlx::Error> {
    //     let order = sqlx::query!("select * from orders WHERE invoice_hash = ?;", invoice_hash)
    //         .fetch_optional(&mut **db)
    //         // .map_ok(|maybe_r| {
    //         //     maybe_r.map(|r| AdminSettings {
    //         //         id: Some(r.id.try_into().unwrap()),
    //         //         market_name: r.market_name,
    //         //         fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
    //         //     })
    //         // })
    //         .map_ok(|maybe_r| {
    //             maybe_r.map(|r| Order {
    //                 id: Some(r.id.try_into().unwrap()),
    //                 public_id: r.public_id,
    //                 quantity: r.quantity.try_into().unwrap(),
    //                 user_id: r.user_id.try_into().unwrap(),
    //                 listing_id: r.listing_id.try_into().unwrap(),
    //                 shipping_option_id: r.shipping_option_id.try_into().unwrap(),
    //                 shipping_instructions: r.shipping_instructions,
    //                 amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
    //                 seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
    //                 paid: r.paid,
    //                 completed: r.completed,
    //                 invoice_hash: r.invoice_hash,
    //                 invoice_payment_request: r.invoice_payment_request,
    //                 created_time_ms: r.created_time_ms.try_into().unwrap(),
    //             })
    //         })
    //         .await?;

    //     Ok(order)
    // }

    pub async fn update_order_on_paid(
        db: &mut PoolConnection<Sqlite>,
        invoice_hash: &str,
    ) -> Result<(), sqlx::Error> {
        let mut tx = db.begin().await?;

        let maybe_order =
            sqlx::query!("select * from orders WHERE invoice_hash = ?;", invoice_hash)
                .fetch_optional(&mut tx)
                .await?;
        println!("maybe_order: {:?}", maybe_order);

        let maybe_order_2 =
            sqlx::query!("select * from orders WHERE invoice_hash = ?;", invoice_hash)
                .fetch_optional(&mut tx)
                .await?;
        println!("maybe_order_2: {:?}", maybe_order_2);

        if let Some(order) = maybe_order {
            println!("set order as paid here...");
            sqlx::query!("UPDATE orders SET paid = true WHERE id = ?", order.id,)
                .execute(&mut tx)
                .await?;

            let listing = sqlx::query!("select * from listings WHERE id = ?;", order.listing_id)
                .fetch_one(&mut tx)
                .await?;
            println!("listing: {:?}", listing);

            let sold_items = sqlx::query!(
                "
select
 SUM(orders.quantity) as quantity_sold
FROM
 orders
WHERE
 listing_id = ?
AND
 completed
GROUP BY
 listing_id
;",
                order.listing_id,
            )
            .fetch_optional(&mut tx)
            .await?;
            println!("sold items: {:?}", sold_items);

            let quantity_sold = match sold_items {
                Some(r) => r.quantity_sold,
                None => 0,
            };
            println!("quantity sold: {:?}", quantity_sold);
            let quantity_in_stock = listing.quantity - quantity_sold;
            if quantity_in_stock >= order.quantity {
                println!("set order as completed here...");
                sqlx::query!("UPDATE orders SET completed = true WHERE id = ?", order.id,)
                    .execute(&mut tx)
                    .await?;
            }
        }

        // // Set all images for listing_id to not primary.
        // sqlx::query!(
        //     "UPDATE listingimages SET is_primary = false WHERE listing_id = ?",
        //     listing_id
        // )
        // .execute(&mut tx)
        // .await?;

        // // Set image for listing_id and image_id to primary.
        // let update_result = sqlx::query!(
        //     "UPDATE listingimages SET is_primary = true WHERE listing_id = ? AND public_id = ?",
        //     listing_id,
        //     image_id,
        // )
        // .execute(&mut tx)
        // .await?;

        tx.commit().await?;

        // Ok(update_result.rows_affected() as _)
        Ok(())
    }

    pub async fn amount_earned_sat(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<u64, sqlx::Error> {
        let received_order_total_result = sqlx::query!(
            "
select
 SUM(orders.seller_credit_sat) as total_amount_paid, listings.user_id as listing_user_id
FROM
 orders
LEFT JOIN
 listings
ON
 orders.listing_id = listings.id
WHERE
 listing_user_id = ?
AND
 completed
GROUP BY
 listing_user_id
;",
            user_id,
        )
        .fetch_optional(&mut **db)
        .await?;
        println!(
            "received_order_total_result: {:?}",
            received_order_total_result
        );

        let received_order_total = match received_order_total_result {
            Some(r) => r.total_amount_paid,
            None => 0,
        };
        println!("received_order_total: {:?}", received_order_total);

        Ok(received_order_total.try_into().unwrap())
    }

    pub async fn amount_refunded_sat(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<u64, sqlx::Error> {
        let refunded_order_total_result = sqlx::query!(
            "
select
 SUM(orders.amount_owed_sat) as total_amount_refunded, orders.user_id as order_user_id
FROM
 orders
WHERE
 order_user_id = ?
AND
 paid
AND
 not completed
GROUP BY
 order_user_id
;",
            user_id,
        )
        .fetch_optional(&mut **db)
        .await?;
        println!(
            "refunded_order_total_result: {:?}",
            refunded_order_total_result
        );

        let refunded_order_total = match refunded_order_total_result {
            Some(r) => r.total_amount_refunded,
            None => 0,
        };
        println!("refunded_order_total: {:?}", refunded_order_total);

        Ok(refunded_order_total.try_into().unwrap())
    }
}

impl OrderCard {
    pub async fn all_unpaid_for_user(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<Vec<OrderCard>, sqlx::Error> {
        let orders = sqlx::query!(
            "
select
 orders.id as order_id, orders.public_id as order_public_id, orders.user_id as order_user_id, orders.listing_id as order_listing_id, orders.shipping_option_id, orders.shipping_instructions, orders.amount_owed_sat, orders.seller_credit_sat, orders.paid, orders.completed, orders.invoice_hash, orders.invoice_payment_request, orders.created_time_ms, listings.id, listings.public_id as listing_public_id, listings.user_id as listing_user_id, listings.title, listings.description, listings.price_sat, listings.quantity, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.removed, listings.created_time_ms as listing_created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
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
 order_user_id = ?
GROUP BY
 orders.id
ORDER BY orders.created_time_ms DESC
;",
            user_id,
        )
            .fetch(&mut **db)
            .map_ok(|r| {
                let o = Order {
                    id: Some(r.order_id.unwrap().try_into().unwrap()),
                    public_id: r.order_public_id,
                    quantity: r.quantity.try_into().unwrap(),
                    user_id: r.order_user_id.try_into().unwrap(),
                    listing_id: r.order_listing_id.try_into().unwrap(),
                    shipping_option_id: r.shipping_option_id.try_into().unwrap(),
                    shipping_instructions: r.shipping_instructions,
                    amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                    paid: r.paid,
                    completed: r.completed,
                    invoice_hash: r.invoice_hash,
                    invoice_payment_request: r.invoice_payment_request,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                };
                let l = r.id.map(|listing_id| Listing {
                    id: Some(listing_id.try_into().unwrap()),
                    public_id: r.listing_public_id,
                    user_id: r.listing_user_id.try_into().unwrap(),
                    title: r.title,
                    description: r.description,
                    price_sat: r.price_sat.try_into().unwrap(),
                    quantity: r.quantity.try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    submitted: r.submitted,
                    reviewed: r.reviewed,
                    approved: r.approved,
                    removed: r.removed,
                    created_time_ms: r.listing_created_time_ms.try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id,
                    listing_id: r.listing_id.try_into().unwrap(),
                    image_data: r.image_data,
                    is_primary: r.is_primary,
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
    ) -> Result<Vec<OrderCard>, sqlx::Error> {
        let orders = sqlx::query!(
            "
select
 orders.id as order_id, orders.public_id as order_public_id, orders.user_id as order_user_id, orders.listing_id as order_listing_id, orders.shipping_option_id, orders.shipping_instructions, orders.amount_owed_sat, orders.seller_credit_sat, orders.paid, orders.completed, orders.invoice_hash, orders.invoice_payment_request, orders.created_time_ms, listings.id, listings.public_id as listing_public_id, listings.user_id as listing_user_id, listings.title, listings.description, listings.price_sat, listings.quantity, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.removed, listings.created_time_ms as listing_created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
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
 order_user_id = ?
GROUP BY
 orders.id
ORDER BY orders.created_time_ms DESC
;",
            user_id,
        )
            .fetch(&mut **db)
            .map_ok(|r| {
                let o = Order {
                    id: Some(r.order_id.unwrap().try_into().unwrap()),
                    public_id: r.order_public_id,
                    quantity: r.quantity.try_into().unwrap(),
                    user_id: r.order_user_id.try_into().unwrap(),
                    listing_id: r.order_listing_id.try_into().unwrap(),
                    shipping_option_id: r.shipping_option_id.try_into().unwrap(),
                    shipping_instructions: r.shipping_instructions,
                    amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                    paid: r.paid,
                    completed: r.completed,
                    invoice_hash: r.invoice_hash,
                    invoice_payment_request: r.invoice_payment_request,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                };
                let l = r.id.map(|listing_id| Listing {
                    id: Some(listing_id.try_into().unwrap()),
                    public_id: r.listing_public_id,
                    user_id: r.listing_user_id.try_into().unwrap(),
                    title: r.title,
                    description: r.description,
                    price_sat: r.price_sat.try_into().unwrap(),
                    quantity: r.quantity.try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    submitted: r.submitted,
                    reviewed: r.reviewed,
                    approved: r.approved,
                    removed: r.removed,
                    created_time_ms: r.listing_created_time_ms.try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id,
                    listing_id: r.listing_id.try_into().unwrap(),
                    image_data: r.image_data,
                    is_primary: r.is_primary,
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
    ) -> Result<Vec<OrderCard>, sqlx::Error> {
        // TODO: change to only show paid and completed orders.
        let orders = sqlx::query!(
            "
select
 orders.id as order_id, orders.public_id as order_public_id, orders.user_id as order_user_id, orders.listing_id as order_listing_id, orders.shipping_option_id, orders.shipping_instructions, orders.amount_owed_sat, orders.seller_credit_sat, orders.paid, orders.completed, orders.invoice_hash, orders.invoice_payment_request, orders.created_time_ms, listings.id, listings.public_id as listing_public_id, listings.user_id as listing_user_id, listings.title, listings.description, listings.price_sat, listings.quantity, listings.fee_rate_basis_points, listings.submitted, listings.reviewed, listings.approved, listings.removed, listings.created_time_ms as listing_created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
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
 orders.completed
AND
 listing_user_id = ?
GROUP BY
 orders.id
ORDER BY orders.created_time_ms DESC
;",
            user_id,
        )
            .fetch(&mut **db)
            .map_ok(|r| {
                let o = Order {
                    id: Some(r.order_id.unwrap().try_into().unwrap()),
                    public_id: r.order_public_id,
                    quantity: r.quantity.try_into().unwrap(),
                    user_id: r.order_user_id.try_into().unwrap(),
                    listing_id: r.order_listing_id.try_into().unwrap(),
                    shipping_option_id: r.shipping_option_id.try_into().unwrap(),
                    shipping_instructions: r.shipping_instructions,
                    amount_owed_sat: r.amount_owed_sat.try_into().unwrap(),
                    seller_credit_sat: r.seller_credit_sat.try_into().unwrap(),
                    paid: r.paid,
                    completed: r.completed,
                    invoice_hash: r.invoice_hash,
                    invoice_payment_request: r.invoice_payment_request,
                    created_time_ms: r.created_time_ms.try_into().unwrap(),
                };
                let l = r.id.map(|listing_id| Listing {
                    id: Some(listing_id.try_into().unwrap()),
                    public_id: r.listing_public_id,
                    user_id: r.listing_user_id.try_into().unwrap(),
                    title: r.title,
                    description: r.description,
                    price_sat: r.price_sat.try_into().unwrap(),
                    quantity: r.quantity.try_into().unwrap(),
                    fee_rate_basis_points: r.fee_rate_basis_points.try_into().unwrap(),
                    submitted: r.submitted,
                    reviewed: r.reviewed,
                    approved: r.approved,
                    removed: r.removed,
                    created_time_ms: r.listing_created_time_ms.try_into().unwrap(),
                });
                let i = r.image_id.map(|image_id| ListingImage {
                    id: Some(image_id.try_into().unwrap()),
                    public_id: r.image_public_id,
                    listing_id: r.listing_id.try_into().unwrap(),
                    image_data: r.image_data,
                    is_primary: r.is_primary,
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
        let amount_earned = Order::amount_earned_sat(db, user_id).await?;
        let amount_refunded = Order::amount_refunded_sat(db, user_id).await?;
        let account_balance_sat = amount_earned + amount_refunded;
        let account_info = AccountInfo {
            amount_earned_sat: amount_earned,
            amount_refunded_sat: amount_refunded,
            amount_withdrawn_sat: 0, // TODO.
            account_balance_sat: account_balance_sat,
        };

        Ok(account_info)
    }

    pub async fn all_account_balance_changes(
        db: &mut Connection<Db>,
        user_id: i32,
    ) -> Result<Vec<AccountBalanceChange>, sqlx::Error> {
        // TODO: Order by event time in SQL query. When this is fixed: https://github.com/launchbadge/sqlx/issues/1350
        let mut account_balance_changes = sqlx::query!("
select listings.user_id as user_id, orders.seller_credit_sat as amount_change_sat, 'received_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
LEFT JOIN
 listings
ON
 orders.listing_id = listings.id
WHERE
 orders.paid
AND
 orders.completed
AND
 listings.user_id = ?
UNION ALL
select orders.user_id as user_id, orders.amount_owed_sat as amount_change_sat, 'refunded_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
from
 orders
WHERE
 orders.paid
AND
 not orders.completed
AND
 orders.user_id = ?
;",
        user_id, user_id)
            .fetch(&mut **db)
            .map_ok(|r| AccountBalanceChange {
                user_id: r.user_id.to_string(),
                amount_change_sat: r.amount_change_sat.try_into().unwrap(),
                event_type: r.event_type,
                event_id: r.event_id,
                event_time_ms: r.event_time_ms.try_into().unwrap(),

            })
            .try_collect::<Vec<_>>()
            .await?;

        println!("{:?}", account_balance_changes);

        // Sort by event time
        account_balance_changes.sort_by(|a, b| b.event_time_ms.cmp(&a.event_time_ms));

        Ok(account_balance_changes)
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
    //  orders.completed
    // AND
    //  listings.user_id = ?
    // UNION ALL
    // select orders.user_id as user_id, orders.amount_owed_sat as amount_change_sat, 'refunded_order' as event_type, orders.public_id as event_id, orders.created_time_ms as event_time_ms
    // from
    //  orders
    // WHERE
    //  orders.paid
    // AND
    //  not orders.completed
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
    //         println!("account_balance: {:?}", account_balance);

    //         Ok(account_balance)
    //     }
}
