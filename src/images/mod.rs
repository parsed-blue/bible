#[get("/favicon.svg")]
pub fn favicon_svg() -> &'static str {
    include_str!("./logo.svg")
}

#[get("/favicon.png")]
pub fn favicon_png() -> &'static [u8] {
    include_bytes!("./logo.png")
}

#[get("/favicon.ico")]
pub fn favicon_ico() -> &'static [u8] {
    include_bytes!("./logo.ico")
}
