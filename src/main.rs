#[macro_use]
extern crate rocket;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::BTreeMap;
use std::collections::HashMap;
use tera::{Context, Tera};

const TEXT: &str = include_str!("./kjv.txt");
const BOOK_HTML: &str = include_str!("./book.html");

const VERSE_PATTERN: &str = r"(?<book>\d?[a-zA-Z]+)(?<chapter>\d+):(?<section>\d+)\s*(?<text>.+)";

#[derive(Debug, Serialize, Deserialize)]
struct Verse {
    book: String,
    chapter: usize,
    section: usize,
    text: String,
}

impl Verse {
    fn id(&self) -> String {
        format!("{}-{}-{}", self.book, self.chapter, self.section)
    }
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
    fn get(&self, book: &BookName) -> Option<&Book> {
        self.books.get(book)
    }

    fn previous(&self, book: &BookName) -> Option<&BookName> {
        None
    }

    fn next(&self, book: &BookName) -> Option<&BookName> {
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

        for chapter_num in 1..=self.chapters.len() {
            let mut verses: Vec<Verse> = vec![];
            let chapter = self.chapters.get(&chapter_num).unwrap();
            for section_num in 1..=chapter.len() {
                let text = chapter.get(&section_num).unwrap();
                verses.push(Verse {
                    book: self.name.clone(),
                    chapter: chapter_num,
                    section: section_num,
                    text: text.clone(),
                })
            }
            book.push(verses);
        }

        return book;
    }
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template("book.html", BOOK_HTML)
            .expect("Could not create book template");
        tera
    };
    static ref TITLE: String = {
        let mut lines = TEXT.lines();
        return String::from(lines.next().unwrap());
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
                return Verse {
                    book: book.clone(),
                    chapter: chapter.clone(),
                    section: section.clone(),
                    text,
                };
            })
            .collect();

        let mut order: Vec<BookName> = vec![];
        let mut books: HashMap<BookName, Book> = HashMap::new();

        for verse in verses.iter() {
            if order.last() != Some(&verse.book) {
                order.push(verse.book.clone());
            }

            if !books.contains_key(&verse.book) {
                books.insert(verse.book.clone(), Book::new(verse.book.clone()));
            }
            let book = books.get_mut(&verse.book).expect("could not get book");
            if !book.chapters.contains_key(&verse.chapter) {
                book.chapters.insert(verse.chapter, BTreeMap::new());
            }
            let chapter = book
                .chapters
                .get_mut(&verse.chapter)
                .expect("could not get chapter");
            chapter.insert(verse.section, verse.text.clone());
        }

        Bible { order, books }
    };
    static ref VERSES: Vec<Verse> = {
        let re = Regex::new(VERSE_PATTERN).unwrap();
        let mut lines = TEXT.lines();
        lines.next().unwrap();

        return lines
            .map(|line| {
                let caps = re.captures(line).unwrap();
                let book = &caps["book"].to_lowercase();
                let chapter = &caps["chapter"].parse::<usize>().unwrap();
                let section = &caps["section"].parse::<usize>().unwrap();
                let text = String::from(&caps["text"]);
                return Verse {
                    book: book.clone(),
                    chapter: chapter.clone(),
                    section: section.clone(),
                    text,
                };
            })
            .collect();
    };
}

#[get("/")]
fn index() -> &'static str {
    println!("{:?}", *BIBLE);
    "Hello, world!"
}

#[get("/book/<book_name>")]
fn books(book_name: String) -> rocket::response::content::RawHtml<String> {
    let mut context = Context::new();
    let book = BIBLE.get(&book_name).unwrap();
    context.insert("book", &book_name);
    context.insert("prev_book", &BIBLE.previous(&book_name));
    context.insert("next_book", &BIBLE.next(&book_name));
    context.insert("paragraphs", &book.paragraphs());
    return rocket::response::content::RawHtml(TEMPLATES.render("book.html", &context).unwrap());
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, books])
}
