use crate::db::Db;
use crate::rocket::futures::TryFutureExt;
use crate::rocket::futures::TryStreamExt;
use rocket::fs::TempFile;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::{sqlx, Connection};
use std::result::Result;
extern crate base64;
use sqlx::Acquire;

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
    pub price_msat: u64,
    pub submitted: bool,
    pub approved: bool,
    pub created_time_ms: u64,
}

#[derive(Debug, FromForm)]
pub struct InitialListingInfo {
    pub title: String,
    pub description: String,
    pub price_sat: u64,
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
    pub price_msat: u64,
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

impl Listing {
    pub async fn all(db: &mut Connection<Db>) -> Result<Vec<Listing>, sqlx::Error> {
        let listings = sqlx::query!("select * from listings;")
            .fetch(&mut **db)
            .map_ok(|r| Listing {
                id: Some(r.id.try_into().unwrap()),
                public_id: r.public_id as _,
                user_id: r.user_id as _,
                title: r.title,
                description: r.description,
                price_msat: r.price_msat as _,
                submitted: r.submitted,
                approved: r.approved,
                created_time_ms: r.created_time_ms as _,
            })
            .try_collect::<Vec<_>>()
            .await?;

        println!("{}", listings.len());
        println!("{:?}", listings);

        Ok(listings)
    }

    /// Returns the id of the inserted row.
    pub async fn insert(listing: Listing, db: &mut Connection<Db>) -> Result<i32, sqlx::Error> {
        let price_msat: i64 = listing.price_msat as _;
        let created_time_ms: i64 = listing.created_time_ms as _;

        let insert_result = sqlx::query!(
            "INSERT INTO listings (public_id, user_id, title, description, price_msat, submitted, approved, created_time_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            listing.public_id,
            listing.user_id,
            listing.title,
            listing.description,
            price_msat,
            listing.submitted,
            listing.approved,
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
                public_id: r.public_id as _,
                user_id: r.user_id as _,
                title: r.title,
                description: r.description,
                price_msat: r.price_msat as _,
                submitted: r.submitted,
                approved: r.approved,
                created_time_ms: r.created_time_ms as _,
            })
            .await?;

        println!("{:?}", listing);

        Ok(listing)
    }

    pub async fn single_by_public_id(
        db: &mut Connection<Db>,
        public_id: &str,
    ) -> Result<Listing, sqlx::Error> {
        let listing = sqlx::query!("select * from listings WHERE public_id = ?;", public_id)
            .fetch_one(&mut **db)
            .map_ok(|r| Listing {
                id: r.id.map(|n| n as _),
                public_id: r.public_id as _,
                user_id: r.user_id as _,
                title: r.title,
                description: r.description,
                price_msat: r.price_msat as _,
                submitted: r.submitted,
                approved: r.approved,
                created_time_ms: r.created_time_ms as _,
            })
            .await?;

        println!("{:?}", listing);

        Ok(listing)
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
            id: r.id.map(|n| n as _),
            public_id: r.public_id,
            listing_id: r.listing_id as _,
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
            id: r.id.map(|n| n as _),
            public_id: r.public_id,
            listing_id: r.listing_id as _,
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
    pub async fn all(db: &mut Connection<Db>) -> Result<Vec<RocketAuthUser>, sqlx::Error> {
        let rocket_auth_users = sqlx::query!("select * from users;")
            .fetch(&mut **db)
            .map_ok(|r| RocketAuthUser {
                id: Some(r.id as i32),
                username: r.email.unwrap(),
            })
            .try_collect::<Vec<_>>()
            .await?;

        println!("{}", rocket_auth_users.len());

        Ok(rocket_auth_users)
    }

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
    pub async fn all(db: &mut Connection<Db>) -> Result<Vec<ListingCard>, sqlx::Error> {
        // Example query for this kind of join/group by: https://stackoverflow.com/a/63037790/1639564
        // Other example query: https://stackoverflow.com/a/13698334/1639564
        let listing_cards =
            sqlx::query!("
select
 listings.id, listings.public_id, listings.user_id, listings.title, listings.description, listings.price_msat, listings.submitted, listings.approved, listings.created_time_ms, listingimages.id as image_id, listingimages.public_id as image_public_id, listingimages.listing_id, listingimages.image_data, listingimages.is_primary, users.id as rocket_auth_user_id, users.email as rocket_auth_user_username
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
GROUP BY
 listings.id
;")
                .fetch(&mut **db)
            .map_ok(|r| {
                let l = Listing {
                    id: Some(r.id as _),
                    public_id: r.public_id as _,
                    user_id: r.user_id as _,
                    title: r.title,
                    description: r.description,
                    price_msat: r.price_msat as _,
                    submitted: r.submitted,
                    approved: r.approved,
                    created_time_ms: r.created_time_ms as _,
                };
                let i = r.image_id.map(|_| ListingImage {
                    id: Some(r.image_id.unwrap().try_into().unwrap()),
                    public_id: r.image_public_id,
                    listing_id: r.listing_id as _,
                    image_data: r.image_data,
                    is_primary: r.is_primary,
                });
                let u = r.rocket_auth_user_id.map(|_| RocketAuthUser {
                    id: Some(r.rocket_auth_user_id.unwrap().try_into().unwrap()),
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
    pub async fn all(db: &mut Connection<Db>) -> Result<Vec<ListingCardDisplay>, sqlx::Error> {
        let listing_cards = ListingCard::all(db).await?;
        let listing_card_displays = listing_cards
            .iter()
            .map(|card| ListingCardDisplay {
                listing: card.listing.clone(),
                image: card.image.clone().map(|image| ListingImageDisplay {
                    id: image.id,
                    public_id: image.public_id,
                    listing_id: image.listing_id,
                    image_data_base64: base64::encode(&image.image_data),
                    is_primary: image.is_primary,
                }),
                user: card.clone().user,
            })
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
        let price_msat: i64 = shipping_option.price_msat as _;

        println!("inserting shipping option: {:?}", shipping_option);

        let insert_result = sqlx::query!(
            "INSERT INTO shippingoptions (public_id, listing_id, title, description, price_msat) VALUES (?, ?, ?, ?, ?)",
            shipping_option.public_id,
            shipping_option.listing_id,
            shipping_option.title,
            shipping_option.description,
            price_msat,
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
            "select * from shippingoptions WHERE listing_id = ? ORDER BY shippingoptions.price_msat ASC;",
            listing_id
        )
        .fetch(&mut **db)
        .map_ok(|r| ShippingOption {
            id: r.id.map(|n| n as _),
            public_id: r.public_id,
            listing_id: r.listing_id as _,
            title: r.title,
            description: r.description,
            price_msat: r.price_msat as _,
        })
        .try_collect::<Vec<_>>()
        .await?;

        println!("{}", shipping_options.len());

        Ok(shipping_options)
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
            id: r.id.map(|n| n as _),
            public_id: r.public_id,
            listing_id: r.listing_id as _,
            title: r.title,
            description: r.description,
            price_msat: r.price_msat as _,
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
