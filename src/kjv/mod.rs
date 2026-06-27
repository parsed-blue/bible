use crate::bible::{Bible, BookName, Chapter, Section, Text, Verse};
use regex::Regex;

const TEXT: &str = include_str!("./kjv.txt");
const VERSE_PATTERN: &str = r"(?<book>\d?[a-zA-Z]+)(?<chapter>\d+):(?<section>\d+)\s*(?<text>.+)";

pub fn load() -> Bible {
    let re = Regex::new(VERSE_PATTERN).unwrap();
    let mut lines = TEXT.lines();
    lines.next().unwrap();

    let verses: Vec<Verse> = lines
        .map(|line| {
            let caps = re.captures(line).unwrap();
            let book = &caps["book"];
            let chapter = &caps["chapter"].parse::<usize>().unwrap();
            let section = &caps["section"].parse::<usize>().unwrap();
            let text = &caps["text"];
            Verse {
                book: BookName::new(book),
                chapter: Chapter(*chapter),
                section: Section(*section),
                text: Text::new(text),
            }
        })
        .collect();

    Bible::from_verses(verses)
}
