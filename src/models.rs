use crate::db::Db;
use crate::rocket::futures::TryFutureExt;
use crate::rocket::futures::TryStreamExt;
use rocket::fs::TempFile;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::{sqlx, Connection};
use std::result::Result;
extern crate base64;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Post {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub title: String,
    pub text: String,
}

// #[derive(Serialize, Debug, Clone)]
// #[serde(crate = "rocket::serde")]
// pub struct Task {
//     pub id: Option<i32>,
//     pub description: String,
//     pub completed: bool,
// }

// #[derive(Debug, FromForm)]
// pub struct Todo {
//     pub description: String,
// }

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Listing {
    pub id: Option<i32>,
    pub user_id: i32,
    pub title: String,
    pub description: String,
    pub price_msat: u64,
    pub completed: bool,
    pub approved: bool,
    pub created_time_ms: u64,
}

#[derive(Debug, FromForm)]
pub struct InitialListingInfo {
    pub title: String,
    pub description: String,
    pub price_msat: u64,
}

#[derive(FromForm)]
pub struct FileUploadForm<'f> {
    pub file: TempFile<'f>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ListingImage {
    pub id: Option<i32>,
    pub listing_id: i32,
    pub image_data: Vec<u8>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ListingDisplay {
    pub listing: Listing,
    pub images: Vec<ListingImageDisplay>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ListingImageDisplay {
    pub id: Option<i32>,
    pub listing_id: i32,
    pub image_data_base64: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct RocketAuthUser {
    pub id: Option<i32>,
    pub username: String,
}

// #[derive(Debug, FromForm)]
// pub struct InitialListingInfo {
//     pub title: String,
//     pub description: String,
//     pub price_msat: u64,
//     pub created_time_ms: u64,
// }

// impl Task {
//     pub async fn all(mut db: Connection<Db>) -> Result<Vec<Task>, sqlx::Error> {
//         let tasks = sqlx::query!("select * from tasks;")
//             .fetch(&mut *db)
//             .map_ok(|r| Task {
//                 id: Some(r.id.try_into().unwrap()),
//                 description: r.description,
//                 completed: r.completed,
//             })
//             .try_collect::<Vec<_>>()
//             .await?;

//         println!("{}", tasks.len());
//         println!("{:?}", tasks);

//         Ok(tasks)
//     }

//     /// Returns the number of affected rows: 1.
//     pub async fn insert(todo: Todo, mut db: Connection<Db>) -> Result<usize, sqlx::Error> {
//         let insert_result = sqlx::query!(
//             "INSERT INTO tasks (description, completed) VALUES (?, ?)",
//             todo.description,
//             false,
//         )
//         .execute(&mut *db)
//         .await?;

//         println!("{:?}", insert_result);

//         Ok(insert_result.rows_affected() as _)
//     }

//     /// Returns the number of affected rows: 1.
//     pub async fn toggle_with_id(id: i32, db: &mut Connection<Db>) -> Result<usize, sqlx::Error> {
//         let mut tx = db.begin().await?;

//         let get_task_completed = sqlx::query!("select * from tasks WHERE id = ?;", id)
//             .fetch_one(&mut tx)
//             .map_ok(|r| r.completed)
//             // .try_collect::<bool>()
//             .await?;

//         let new_completed = !get_task_completed;

//         let update_result = sqlx::query!(
//             "UPDATE tasks SET completed = ? WHERE id = ?",
//             // !task.completed,
//             new_completed,
//             id,
//         )
//         .execute(&mut tx)
//         .await?;

//         tx.commit().await?;

//         println!("{:?}", update_result);

//         Ok(update_result.rows_affected() as _)
//     }

//     /// Returns the number of affected rows: 1.
//     pub async fn delete_with_id(id: i32, db: &mut Connection<Db>) -> Result<usize, sqlx::Error> {
//         let delete_result = sqlx::query!("DELETE FROM tasks WHERE id = ?", id)
//             .execute(&mut **db)
//             .await?;

//         Ok(delete_result.rows_affected() as _)
//     }

//     // /// Returns the number of affected rows.
//     // #[cfg(test)]
//     // pub async fn delete_all(conn: &DbConn) -> QueryResult<usize> {
//     //     conn.run(|c| diesel::delete(all_tasks).execute(c)).await
//     // }
// }

impl Listing {
    pub async fn all(mut db: Connection<Db>) -> Result<Vec<Listing>, sqlx::Error> {
        let listings = sqlx::query!("select * from listings;")
            .fetch(&mut *db)
            .map_ok(|r| Listing {
                id: Some(r.id.try_into().unwrap()),
                user_id: r.user_id as _,
                title: r.title,
                description: r.description,
                price_msat: r.price_msat as _,
                completed: r.completed,
                approved: r.approved,
                created_time_ms: r.created_time_ms as _,
            })
            .try_collect::<Vec<_>>()
            .await?;

        println!("{}", listings.len());
        println!("{:?}", listings);

        Ok(listings)
    }

    /// Returns the number of affected rows: 1.
    pub async fn insert(listing: Listing, db: &mut Connection<Db>) -> Result<usize, sqlx::Error> {
        let price_msat: i64 = listing.price_msat as _;
        let created_time_ms: i64 = listing.created_time_ms as _;

        let insert_result = sqlx::query!(
            "INSERT INTO listings (user_id, title, description, price_msat, completed, approved, created_time_ms) VALUES (?, ?, ?, ?, ?, ?, ?)",
            listing.user_id,
            listing.title,
            listing.description,
            price_msat,
            listing.completed,
            listing.approved,
            created_time_ms,
        )
        .execute(&mut **db)
        .await?;

        println!("{:?}", insert_result);

        Ok(insert_result.rows_affected() as _)
    }

    pub async fn single(db: &mut Connection<Db>, id: i32) -> Result<Option<Listing>, sqlx::Error> {
        let listing = sqlx::query!("select * from listings WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| Listing {
                id: Some(r.id.try_into().unwrap()),
                user_id: r.user_id as _,
                title: r.title,
                description: r.description,
                price_msat: r.price_msat as _,
                completed: r.completed,
                approved: r.approved,
                created_time_ms: r.created_time_ms as _,
            })
            .await?;

        println!("{:?}", listing);

        Ok(Some(listing))
    }

    pub async fn single_display(
        db: &mut Connection<Db>,
        id: i32,
    ) -> Result<Option<ListingDisplay>, sqlx::Error> {
        let listing = Listing::single(&mut *db, id).await?;
        let images = ListingImage::all_for_listing(&mut *db, id).await?;
        let image_displays = images
            .iter()
            .map(|img| ListingImageDisplay {
                id: img.id,
                listing_id: img.listing_id,
                image_data_base64: base64::encode(&img.image_data),
            })
            .collect::<Vec<_>>();

        let listing_display = listing.map(|l| ListingDisplay {
            listing: l,
            images: image_displays,
        });

        Ok(listing_display)
    }
}

impl ListingImage {
    /// Returns the number of affected rows: 1.
    pub async fn insert(
        listingimage: ListingImage,
        db: &mut Connection<Db>,
    ) -> Result<usize, sqlx::Error> {
        let insert_result = sqlx::query!(
            "INSERT INTO listingimages (listing_id, image_data) VALUES (?, ?)",
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
            "select * from listingimages WHERE listing_id = ?;",
            listing_id
        )
        .fetch(&mut **db)
        .map_ok(|r| ListingImage {
            id: Some(r.id.try_into().unwrap()),
            listing_id: r.listing_id as _,
            image_data: r.image_data,
        })
        .try_collect::<Vec<_>>()
        .await?;

        println!("{}", listing_images.len());

        Ok(listing_images)
    }

    pub async fn single(
        db: &mut Connection<Db>,
        id: i32,
    ) -> Result<Option<ListingImage>, sqlx::Error> {
        let listing_image = sqlx::query!("select * from listingimages WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| ListingImage {
                id: Some(r.id.try_into().unwrap()),
                listing_id: r.listing_id as _,
                image_data: r.image_data,
            })
            .await?;

        Ok(Some(listing_image))
    }

    /// Returns the number of affected rows: 1.
    pub async fn delete_with_id(id: i32, db: &mut Connection<Db>) -> Result<usize, sqlx::Error> {
        let delete_result = sqlx::query!("DELETE FROM listingimages WHERE id = ?", id)
            .execute(&mut **db)
            .await?;

        Ok(delete_result.rows_affected() as _)
    }
}

impl RocketAuthUser {
    /// Returns the number of affected rows: 1.

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

    pub async fn single(id: i32, db: &mut Connection<Db>) -> Result<RocketAuthUser, sqlx::Error> {
        let rocket_auth_user = sqlx::query!("select * from users WHERE id = ?;", id)
            .fetch_one(&mut **db)
            .map_ok(|r| RocketAuthUser {
                id: Some(r.id as i32),
                username: r.email.unwrap(),
            })
            .await?;

        Ok(rocket_auth_user)
    }
}
