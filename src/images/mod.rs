use axum::{http::header, response::IntoResponse};

pub async fn favicon_svg() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/svg+xml")],
        include_str!("./logo.svg"),
    )
}

pub async fn favicon_png() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/png")],
        include_bytes!("./logo.png"),
    )
}

pub async fn favicon_ico() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/x-icon")],
        include_bytes!("./logo.ico"),
    )
}
