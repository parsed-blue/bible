use std::sync::LazyLock;
use tera::Tera;

pub static TEMPLATES: LazyLock<Tera> = LazyLock::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_template("book.html", include_str!("./book.html"))
        .unwrap();
    tera.add_raw_template("info.html", include_str!("./info.html"))
        .unwrap();
    tera.add_raw_template("pager.html", include_str!("./pager.html"))
        .unwrap();
    tera.add_raw_template("book-list.html", include_str!("./book-list.html"))
        .unwrap();
    tera.add_raw_template("title.html", include_str!("./title.html"))
        .unwrap();
    tera.add_raw_template("wallets.html", include_str!("./wallets.html"))
        .unwrap();
    tera.add_raw_template("content.html", include_str!("./content.html"))
        .unwrap();
    tera.add_raw_template("styles.html", include_str!("./styles.html"))
        .unwrap();
    tera
});
