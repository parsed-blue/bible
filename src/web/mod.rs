use crate::bible::{Bible, BookName, Chapter, Section, Verse};
use regex::Regex;

const TEXT: &str = include_str!("./web.txt");
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
                book: BookName(book.clone()),
                chapter: Chapter(*chapter),
                section: Section(*section),
                text,
            }
        })
        .collect();

    Bible::from_verses(verses)
}
