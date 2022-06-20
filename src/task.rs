use rocket::serde::Serialize;

use std::result::Result;

use crate::db::Db;

use rocket_db_pools::{sqlx, Connection};

use crate::rocket::futures::TryStreamExt;

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Task {
    pub id: Option<i32>,
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, FromForm)]
pub struct Todo {
    pub description: String,
}

impl Task {
    pub async fn all(mut db: Connection<Db>) -> Result<Vec<Task>, sqlx::Error> {
        let tasks = sqlx::query!("select * from tasks;")
            .fetch(&mut *db)
            .map_ok(|r| Task {
                id: Some(r.id.try_into().unwrap()),
                description: r.description,
                completed: r.completed,
            })
            .try_collect::<Vec<_>>()
            .await?;

        println!("{}", tasks.len());
        println!("{:?}", tasks);

        Ok(tasks)

        // conn.run(|c| all_tasks.order(tasks::id.desc()).load::<Task>(c))
        //     .await
    }

    /// Returns the number of affected rows: 1.
    pub async fn insert(todo: Todo, mut db: Connection<Db>) -> Result<usize, sqlx::Error> {
        let insert_result = sqlx::query!(
            "INSERT INTO tasks (description, completed) VALUES (?, ?)",
            todo.description,
            false,
        )
        .execute(&mut *db)
        .await?;

        println!("{:?}", insert_result);

        Ok(insert_result.rows_affected() as _)

        // conn.run(|c| {
        //     let t = Task {
        //         id: None,
        //         description: todo.description,
        //         completed: false,
        //     };
        //     diesel::insert_into(tasks::table).values(&t).execute(c)
        // })
        // .await
    }

    // /// Returns the number of affected rows: 1.
    // pub async fn toggle_with_id(id: i32, conn: &DbConn) -> QueryResult<usize> {
    //     conn.run(move |c| {
    //         let task = all_tasks.find(id).get_result::<Task>(c)?;
    //         let new_status = !task.completed;
    //         let updated_task = diesel::update(all_tasks.find(id));
    //         updated_task.set(task_completed.eq(new_status)).execute(c)
    //     })
    //     .await
    // }

    // /// Returns the number of affected rows: 1.
    // pub async fn delete_with_id(id: i32, conn: &DbConn) -> QueryResult<usize> {
    //     conn.run(move |c| diesel::delete(all_tasks.find(id)).execute(c))
    //         .await
    // }

    // /// Returns the number of affected rows.
    // #[cfg(test)]
    // pub async fn delete_all(conn: &DbConn) -> QueryResult<usize> {
    //     conn.run(|c| diesel::delete(all_tasks).execute(c)).await
    // }
}
