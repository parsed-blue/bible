#[macro_use]
extern crate rocket;
use dashmap::DashMap;
use lazy_static::lazy_static;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tera::Context;

mod bible;
mod erv;
mod kjv;
mod templates;
mod web;

use bible::Bible;

use templates::TEMPLATES;

const LOGO_SVG: &str = include_str!("./logo.svg");
const LOGO_PNG: &[u8] = include_bytes!("./logo.png");
const LOGO_ICO: &[u8] = include_bytes!("./logo.ico");

#[derive(Serialize, Deserialize)]
enum Version {
    KJV,
    ERV,
    WEB,
}

const VERSION: Version = Version::WEB;

lazy_static! {
    static ref BIBLE: Bible = match VERSION {
        Version::KJV => kjv::load(),
        Version::ERV => erv::load(),
        Version::WEB => web::load(),
    };
    static ref CACHE: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
}

#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(books(BIBLE.order[0].clone())))
}

#[get("/favicon.svg")]
fn favicon_svg() -> &'static str {
    LOGO_SVG
}

#[get("/favicon.png")]
fn favicon_png() -> &'static [u8] {
    LOGO_PNG
}

#[get("/favicon.ico")]
fn favicon_ico() -> &'static [u8] {
    LOGO_ICO
}

#[get("/book/<book_name>")]
fn books(book_name: &str) -> Result<RawHtml<String>, Redirect> {
    let Some(book) = BIBLE.get(book_name) else {
        return Err(Redirect::to(uri!("/")));
    };
    let key = String::from(book_name);
    return Ok(RawHtml(
        CACHE
            .entry(key)
            .or_insert_with(|| {
                let mut context = Context::new();
                context.insert("book", book_name);
                context.insert("prev_book", &BIBLE.previous(book_name));
                context.insert("next_book", &BIBLE.next(book_name));
                context.insert("paragraphs", &book.paragraphs());
                context.insert("books", &BIBLE.order);
                context.insert("version", &VERSION);
                TEMPLATES.render("book.html", &context).unwrap()
            })
            .value()
            .clone(),
    ));
}

#[get("/.info")]
fn cache() -> RawHtml<String> {
    let mut context = Context::new();
    context.insert("entries", &CACHE.len());
    RawHtml(TEMPLATES.render("info.html", &context).unwrap())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![index, books, cache, favicon_svg, favicon_png, favicon_ico],
    )
}
