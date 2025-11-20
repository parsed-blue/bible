use serde::{Deserialize, Serialize};
use std::collections::{
    HashMap, BTreeMap
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Verse {
    pub book: String,
    pub chapter: usize,
    pub section: usize,
    pub text: String,
}

pub type BookName = String;
type Section = usize;
type Chapter = usize;

#[derive(Debug)]
pub struct Bible {
    pub order: Vec<BookName>,
    pub books: HashMap<String, Book>,
}

impl Bible {
    pub fn get(&self, book: &str) -> Option<&Book> {
        self.books.get(book)
    }

    pub fn previous(&self, book: &str) -> Option<&BookName> {
        for c in 0..self.order.len() {
            if self.order.get(c + 1).map(|s| s.as_str()) == Some(book) {
                return self.order.get(c);
            }
        }

        None
    }

    pub fn next(&self, book: &str) -> Option<&BookName> {
        for c in (1..self.order.len()).rev() {
            if self.order.get(c - 1).map(|s| s.as_str()) == Some(book) {
                return self.order.get(c);
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct Book {
    pub name: BookName,
    pub chapters: BTreeMap<Chapter, BTreeMap<Section, String>>,
}

impl Book {
    pub fn new(name: BookName) -> Book {
        Book {
            name,
            chapters: BTreeMap::new(),
        }
    }

    pub fn paragraphs(&self) -> Vec<Vec<Verse>> {
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
