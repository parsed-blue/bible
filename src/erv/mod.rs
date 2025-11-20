use crate::bible::{Bible, Book, BookName, Verse};
use regex::Regex;
use std::collections::HashMap;

const TEXT: &str = include_str!("./erv.txt");
const VERSE_PATTERN: &str =
    r"(?<book>(\d )?[a-zA-Z]+) (?<chapter>\d+):(?<section>\d+)\s*(?<text>.+)";

pub fn load() -> Bible {
    let re = Regex::new(VERSE_PATTERN).unwrap();
    let mut lines = TEXT.lines();
    lines.next().unwrap();
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
}
