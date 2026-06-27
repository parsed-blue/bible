use serde::Serialize;

use crate::bible::{Bible, Book, BookName, Chapter, Section, Text};

#[derive(Serialize)]
pub struct BookView {
    pub title: BookName,
    pub chapters: Vec<ChapterView>,
    pub link: BookNameView,
    pub index: Vec<BookNameView>,
    pub prev: Option<BookNameView>,
    pub next: Option<BookNameView>,
}

#[derive(Serialize)]
pub struct BookNameView {
    pub display: String,
    pub href: String,
}

#[derive(Serialize)]
pub struct ChapterView {
    pub number: Chapter,
    pub verses: Vec<VerseView>,
    pub id: String,
}

#[derive(Serialize)]
pub struct VerseView {
    pub number: Section,
    pub text: Text,
    pub id: String,
}

impl BookView {
    pub fn new(bible: &Bible, book: &Book) -> BookView {
        let mut view: BookView = BookView {
            title: book.name.clone(),
            chapters: Vec::new(),
            link: BookNameView {
                display: book.name.0.clone(),
                href: format!("/book/{}", book.name.slug().0),
            },
            index: bible
                .order
                .iter()
                .map(|slug| {
                    let book = bible.books.get(slug).expect("");
                    BookNameView {
                        display: book.name.0.clone(),
                        href: format!("/book/{}", slug.0),
                    }
                })
                .collect(),
            prev: bible
                .previous(&book.name.slug())
                .map(|(a, b)| BookNameView {
                    display: a.0.clone(),
                    href: format!("/book/{}", b.0),
                }),
            next: bible.next(&book.name.slug()).map(|(a, b)| BookNameView {
                display: a.0.clone(),
                href: format!("/book/{}", b.0),
            }),
        };

        for (chapter, sections) in book.chapters.iter() {
            let mut chapter_view = ChapterView {
                id: format!("c{}", chapter.0),
                number: *chapter,
                verses: Vec::new(),
            };
            for (section, text) in sections.iter() {
                chapter_view.verses.push(VerseView {
                    number: *section,
                    text: text.clone(),
                    id: format!("c{}v{}", chapter.0, section.0),
                })
            }
            view.chapters.push(chapter_view);
        }

        view
    }
}
