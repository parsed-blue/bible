use lazy_static::lazy_static;
use tera::Tera;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template("book.html", include_str!("./templates/book.html"))
            .unwrap();
        tera.add_raw_template("info.html", include_str!("./templates/info.html"))
            .unwrap();
        tera.add_raw_template("pager.html", include_str!("./templates/pager.html"))
            .unwrap();
        tera.add_raw_template("book-list.html", include_str!("./templates/book-list.html"))
            .unwrap();
        tera.add_raw_template("title.html", include_str!("./templates/title.html"))
            .unwrap();
        tera
    };
}
