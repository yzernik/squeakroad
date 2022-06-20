use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::request::FlashMessage;
use rocket::serde::Serialize;

use rocket_dyn_templates::Template;

use crate::task::Task;

use rocket_db_pools::Connection;

use crate::db::Db;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    tasks: Vec<Task>,
}

impl Context {
    // pub async fn err<M: std::fmt::Display>(db: Connection<Db>, msg: M) -> Context {
    //     Context {
    //         flash: Some(("error".into(), msg.to_string())),
    //         tasks: Task::all(db).await.unwrap_or_default(),
    //     }
    // }

    pub async fn raw(db: Connection<Db>, flash: Option<(String, String)>) -> Context {
        match Task::all(db).await {
            Ok(tasks) => Context { flash, tasks },
            Err(e) => {
                error_!("DB Task::all() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Fail to access database.".into())),
                    tasks: vec![],
                }
            }
        }
    }
}

// #[post("/", data = "<todo_form>")]
// async fn new(todo_form: Form<Todo>, mut db: Connection<Db>) -> Flash<Redirect> {
//     let todo = todo_form.into_inner();
//     if todo.description.is_empty() {
//         Flash::error(Redirect::to("/"), "Description cannot be empty.")
//     } else if let Err(e) = Task::insert(todo, &db).await {
//         error_!("DB insertion error: {}", e);
//         Flash::error(
//             Redirect::to("/"),
//             "Todo could not be inserted due an internal error.",
//         )
//     } else {
//         Flash::success(Redirect::to("/"), "Todo successfully added.")
//     }
// }

// #[put("/<id>")]
// async fn toggle(id: i32, mut db: Connection<Db>) -> Result<Redirect, Template> {
//     match Task::toggle_with_id(id, &db).await {
//         Ok(_) => Ok(Redirect::to("/")),
//         Err(e) => {
//             error_!("DB toggle({}) error: {}", id, e);
//             Err(Template::render(
//                 "index",
//                 Context::err(&db, "Failed to toggle task.").await,
//             ))
//         }
//     }
// }

// #[delete("/<id>")]
// async fn delete(id: i32, mut db: Connection<Db>) -> Result<Flash<Redirect>, Template> {
//     match Task::delete_with_id(id, &db).await {
//         Ok(_) => Ok(Flash::success(Redirect::to("/"), "Todo was deleted.")),
//         Err(e) => {
//             error_!("DB deletion({}) error: {}", id, e);
//             Err(Template::render(
//                 "index",
//                 Context::err(&db, "Failed to delete task.").await,
//             ))
//         }
//     }
// }

#[get("/")]
async fn index(flash: Option<FlashMessage<'_>>, db: Connection<Db>) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("todoindex", Context::raw(db, flash).await)
}

pub fn todo_stage() -> AdHoc {
    AdHoc::on_ignite("Todo Stage", |rocket| async {
        rocket
            .mount("/", FileServer::from(relative!("static")))
            .mount("/todo", routes![index])
        //.mount("/todo", routes![new, toggle, delete])
    })
}