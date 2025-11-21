#[macro_use]
extern crate rocket;
use dashmap::DashMap;
use rocket::State;
use rocket::http::Status;
use rocket::response::content::RawHtml;
use rocket::{Request, response::Redirect};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tera::Context;

mod bible;
mod erv;
mod images;
mod kjv;
mod templates;
mod web;

use bible::Bible;

use templates::TEMPLATES;

#[derive(Serialize, Deserialize)]
enum Version {
    Kjv,
    Erv,
    Web,
}

const VERSION: Version = Version::Web;

struct AppState {
    commit_hash: String,
    bible: Bible,
    cache: Arc<DashMap<String, String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            commit_hash: match std::env::var("COMMIT_HASH") {
                Ok(var) => var,
                Err(_) => String::from("[UNKNOWN]"),
            },
            cache: Arc::new(DashMap::new()),
            bible: match VERSION {
                Version::Kjv => kjv::load(),
                Version::Erv => erv::load(),
                Version::Web => web::load(),
            },
        }
    }
}

#[get("/")]
fn index(state: &State<AppState>) -> Redirect {
    Redirect::to(uri!(books(state.bible.order[0].clone())))
}

#[get("/book/<book_name>")]
fn books(book_name: &str, state: &State<AppState>) -> Result<RawHtml<String>, Redirect> {
    let Some(book) = state.bible.get(book_name) else {
        return Err(Redirect::to(uri!("/")));
    };
    let key = String::from(book_name);
    Ok(RawHtml(
        state
            .cache
            .entry(key)
            .or_insert_with(|| {
                let mut context = Context::new();
                context.insert("book", book_name);
                context.insert("prev_book", &state.bible.previous(book_name));
                context.insert("next_book", &state.bible.next(book_name));
                context.insert("paragraphs", &book.paragraphs());
                context.insert("books", &state.bible.order);
                context.insert("version", &VERSION);
                context.insert("commit_hash", &state.commit_hash.as_str());
                TEMPLATES.render("book.html", &context).unwrap()
            })
            .value()
            .clone(),
    ))
}

#[get("/.info")]
fn info(state: &State<AppState>) -> RawHtml<String> {
    let mut context = Context::new();
    context.insert("entries", &state.cache.len());
    RawHtml(TEMPLATES.render("info.html", &context).unwrap())
}

#[catch(default)]
fn default_catcher(_: Status, _: &Request) -> Redirect {
    Redirect::to(uri!("/"))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .register("/", catchers![default_catcher])
        .manage(AppState::default())
        .mount(
            "/",
            routes![
                index,
                books,
                info,
                images::favicon_svg,
                images::favicon_png,
                images::favicon_ico
            ],
        )
}
