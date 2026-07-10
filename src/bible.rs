use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

pub type VerseRecord = (&'static str, usize, usize, &'static str);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text(pub String);

impl Text {
    pub fn new(source: &str) -> Text {
        Text(source.trim().to_string())
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct BookSlug(pub String);

impl fmt::Display for BookSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct BookName(pub String);

impl BookName {
    pub fn new(source: &str) -> BookName {
        BookName(source.trim().to_string())
    }

    pub fn slug(&self) -> BookSlug {
        BookSlug(self.0.to_lowercase().replace(" ", "-"))
    }
}

impl fmt::Display for BookName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize, PartialOrd, Ord, Copy)]
pub struct Section(pub usize);

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize, PartialOrd, Ord, Copy)]
pub struct Chapter(pub usize);

#[derive(Debug)]
pub struct Bible {
    pub order: Vec<BookSlug>,
    pub books: HashMap<BookSlug, Book>,
}

impl Bible {
    pub fn from_records(records: &[VerseRecord]) -> Bible {
        let mut order: Vec<BookSlug> = vec![];

        let mut books: HashMap<BookSlug, Book> = HashMap::new();

        for &(book, chapter, section, text) in records {
            let name = BookName::new(book);
            let slug = name.slug();
            if !books.contains_key(&slug) {
                order.push(slug.clone());
            }

            let book = books.entry(slug).or_insert_with(|| Book::new(name));
            let chapter = book.chapters.entry(Chapter(chapter)).or_default();
            chapter.insert(Section(section), Text::new(text));
        }

        Bible { order, books }
    }

    pub fn get(&self, book: &BookSlug) -> Option<&Book> {
        self.books.get(book)
    }

    pub fn previous(&self, book: &BookSlug) -> Option<(&BookName, &BookSlug)> {
        self.order.windows(2).find(|w| &w[1] == book).map(|w| {
            let prev = &w[0];
            (&self.books[prev].name, prev)
        })
    }

    pub fn next(&self, book: &BookSlug) -> Option<(&BookName, &BookSlug)> {
        for c in (1..self.order.len()).rev() {
            if self.order.get(c - 1) == Some(book) {
                let next = self.order.get(c).unwrap();
                let book = self.books.get(next).unwrap();
                return Some((&book.name, next));
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct Book {
    pub name: BookName,
    pub chapters: BTreeMap<Chapter, BTreeMap<Section, Text>>,
}

impl Book {
    pub fn new(name: BookName) -> Book {
        Book {
            name,
            chapters: BTreeMap::new(),
        }
    }
}
