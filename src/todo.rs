use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket_auth::User;

use rocket_dyn_templates::Template;

use crate::task::{Task, Todo};

use rocket_db_pools::Connection;

use crate::db::Db;

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Context {
    flash: Option<(String, String)>,
    tasks: Vec<Task>,
    user: Option<User>,
}

impl Context {
    pub async fn err<M: std::fmt::Display>(
        db: Connection<Db>,
        msg: M,
        user: Option<User>,
    ) -> Context {
        Context {
            flash: Some(("error".into(), msg.to_string())),
            tasks: Task::all(db).await.unwrap_or_default(),
            user: user,
        }
    }

    pub async fn raw(
        db: Connection<Db>,
        flash: Option<(String, String)>,
        user: Option<User>,
    ) -> Context {
        match Task::all(db).await {
            Ok(tasks) => Context { flash, tasks, user },
            Err(e) => {
                error_!("DB Task::all() error: {}", e);
                Context {
                    flash: Some(("error".into(), "Fail to access database.".into())),
                    tasks: vec![],
                    user: user,
                }
            }
        }
    }
}

#[post("/", data = "<todo_form>")]
async fn new(todo_form: Form<Todo>, db: Connection<Db>, _user: User) -> Flash<Redirect> {
    let todo = todo_form.into_inner();
    if todo.description.is_empty() {
        Flash::error(Redirect::to("/"), "Description cannot be empty.")
    } else if let Err(e) = Task::insert(todo, db).await {
        error_!("DB insertion error: {}", e);
        Flash::error(
            Redirect::to("/"),
            "Todo could not be inserted due an internal error.",
        )
    } else {
        Flash::success(Redirect::to("/"), "Todo successfully added.")
    }
}

#[put("/<id>")]
async fn toggle(id: i32, mut db: Connection<Db>, user: User) -> Result<Redirect, Template> {
    match Task::toggle_with_id(id, &mut db).await {
        Ok(_) => Ok(Redirect::to("/")),
        Err(e) => {
            error_!("DB toggle({}) error: {}", id, e);
            Err(Template::render(
                "index",
                Context::err(db, "Failed to toggle task.", Some(user)).await,
            ))
        }
    }
}

#[delete("/<id>")]
async fn delete(id: i32, mut db: Connection<Db>, user: User) -> Result<Flash<Redirect>, Template> {
    match Task::delete_with_id(id, &mut db).await {
        Ok(_) => Ok(Flash::success(Redirect::to("/"), "Todo was deleted.")),
        Err(e) => {
            error_!("DB deletion({}) error: {}", id, e);
            Err(Template::render(
                "index",
                Context::err(db, "Failed to delete task.", Some(user)).await,
            ))
        }
    }
}

#[get("/")]
async fn index(
    flash: Option<FlashMessage<'_>>,
    db: Connection<Db>,
    user: Option<User>,
) -> Template {
    let flash = flash.map(FlashMessage::into_inner);
    Template::render("todoindex", Context::raw(db, flash, user).await)
}

pub fn todo_stage() -> AdHoc {
    AdHoc::on_ignite("Todo Stage", |rocket| async {
        rocket
            .mount("/", FileServer::from(relative!("static")))
            .mount("/", routes![index])
            .mount("/todo", routes![new, toggle, delete])
    })
}
