#[macro_use]
extern crate rocket;
use dashmap::DashMap;
use lazy_static::lazy_static;
use regex::Regex;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};

const TEXT: &str = include_str!("./kjv.txt");
const BOOK_HTML: &str = include_str!("./book.html");
const CACHE_HTML: &str = include_str!("./cache.html");
const LOGO_SVG: &str = include_str!("./logo.svg");
const LOGO_PNG: &[u8] = include_bytes!("./logo.png");
const LOGO_ICO: &[u8] = include_bytes!("./logo.ico");

const VERSE_PATTERN: &str = r"(?<book>\d?[a-zA-Z]+)(?<chapter>\d+):(?<section>\d+)\s*(?<text>.+)";

#[derive(Debug, Serialize, Deserialize)]
struct Verse {
    book: String,
    chapter: usize,
    section: usize,
    text: String,
}

type BookName = String;
type Section = usize;
type Chapter = usize;

#[derive(Debug)]
struct Bible {
    order: Vec<BookName>,
    books: HashMap<String, Book>,
}

impl Bible {
    fn get(&self, book: &str) -> Option<&Book> {
        self.books.get(book)
    }

    fn previous(&self, book: &str) -> Option<&BookName> {
        for c in 0..self.order.len() {
            if self.order.get(c + 1).map(|s| s.as_str()) == Some(book) {
                return self.order.get(c);
            }
        }

        None
    }

    fn next(&self, book: &str) -> Option<&BookName> {
        for c in (1..self.order.len()).rev() {
            if self.order.get(c - 1).map(|s| s.as_str()) == Some(book) {
                return self.order.get(c);
            }
        }

        None
    }
}

#[derive(Debug)]
struct Book {
    name: BookName,
    chapters: BTreeMap<Chapter, BTreeMap<Section, String>>,
}

impl Book {
    fn new(name: BookName) -> Book {
        Book {
            name,
            chapters: BTreeMap::new(),
        }
    }

    fn paragraphs(&self) -> Vec<Vec<Verse>> {
        let mut book: Vec<Vec<Verse>> = vec![];

        for (chapter, sections) in self.chapters.iter() {
            let mut verses: Vec<Verse> = vec![];
            for (section, text) in sections.iter() {
                verses.push(Verse {
                    book: self.name.clone(),
                    chapter: *chapter,
                    section: *section,
                    text: text.clone(),
                })
            }
            book.push(verses);
        }

        book
    }
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template("book.html", BOOK_HTML)
            .expect("Could not create book template");
        tera.add_raw_template("cache.html", CACHE_HTML)
            .expect("Could not create cache template");
        tera
    };
    static ref BIBLE: Bible = {
        let re = Regex::new(VERSE_PATTERN).unwrap();
        let mut lines = TEXT.lines();
        lines.next().unwrap();
        let verses: Vec<Verse> = lines
            .map(|line| {
                let caps = re.captures(line).unwrap();
                let book = &caps["book"].to_lowercase();
                let chapter = &caps["chapter"].parse::<usize>().unwrap();
                let section = &caps["section"].parse::<usize>().unwrap();
                let text = String::from(&caps["text"]);
                Verse {
                    book: book.clone(),
                    chapter: *chapter,
                    section: *section,
                    text,
                }
            })
            .collect();

        let mut order: Vec<BookName> = vec![];
        let mut books: HashMap<BookName, Book> = HashMap::new();

        for verse in verses.iter() {
            if order.last() != Some(&verse.book) {
                order.push(verse.book.clone());
            }

            let book = books
                .entry(verse.book.clone())
                .or_insert_with(|| Book::new(verse.book.clone()));
            let chapter = book.chapters.entry(verse.chapter).or_default();
            chapter.insert(verse.section, verse.text.clone());
        }

        Bible { order, books }
    };
    static ref CACHE: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
}

#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(books(String::from("ge"))))
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
fn books(book_name: &str) -> RawHtml<String> {
    let key = String::from(book_name);
    return RawHtml(
        CACHE
            .entry(key)
            .or_insert_with(|| {
                let mut context = Context::new();
                let book = BIBLE.get(book_name).unwrap();
                context.insert("book", book_name);
                context.insert("prev_book", &BIBLE.previous(book_name));
                context.insert("next_book", &BIBLE.next(book_name));
                context.insert("paragraphs", &book.paragraphs());
                context.insert("books", &BIBLE.order);
                TEMPLATES.render("book.html", &context).unwrap()
            })
            .value()
            .clone(),
    );
}

#[get("/cache")]
fn cache() -> RawHtml<String> {
    let mut context = Context::new();
    context.insert("entries", &CACHE.len());
    RawHtml(TEMPLATES.render("cache.html", &context).unwrap())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![index, books, cache, favicon_svg, favicon_png, favicon_ico],
    )
}
