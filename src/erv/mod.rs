use crate::bible::{Bible, VerseRecord};

static VERSES: &[VerseRecord] = include!(concat!(env!("OUT_DIR"), "/erv_verses.rs"));

pub fn load() -> Bible {
    Bible::from_records(VERSES)
}
