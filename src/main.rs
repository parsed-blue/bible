mod bible;

mod images;
mod templates;

mod erv;
mod kjv;
mod views;
mod web;

use bible::Bible;

use templates::TEMPLATES;

use std::env;
use std::sync::Arc;

use tracing::{Level, info};
use tracing_subscriber::{EnvFilter, fmt};

use dashmap::DashMap;

use axum::{
    Router,
    extract::{Path, State},
    response::{Html, Redirect},
    routing::get,
};

use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use serde::{Deserialize, Serialize};

use tera::Context;

use crate::bible::BookSlug;
use crate::views::BookView;

#[derive(Serialize, Deserialize)]
enum Version {
    Kjv,
    Erv,
    Web,
}

const VERSION: Version = Version::Web;

struct AppState {
    commit_hash: String,
    bible: Bible,
    cache: Arc<DashMap<BookSlug, String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            commit_hash: match std::env::var("COMMIT_HASH") {
                Ok(var) => var,
                Err(_) => String::from("[UNKNOWN]"),
            },
            cache: Arc::new(DashMap::new()),
            bible: match VERSION {
                Version::Kjv => kjv::load(),
                Version::Erv => erv::load(),
                Version::Web => web::load(),
            },
        }
    }
}

async fn index(State(state): State<Arc<AppState>>) -> Redirect {
    let first_book = state.bible.order[0].clone();
    Redirect::to(&format!("/book/{}", first_book))
}

async fn books(
    Path(book_slug): Path<BookSlug>,
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, Redirect> {
    let Some(book) = state.bible.get(&book_slug) else {
        return Err(Redirect::to("/"));
    };

    Ok(Html(
        state
            .cache
            .entry(book_slug.clone())
            .or_insert_with(|| {
                let mut context = Context::new();
                context.insert("book", &BookView::new(&state.bible, &book));
                context.insert("version", &VERSION);
                context.insert("commit_hash", &state.commit_hash.as_str());
                context.insert("bit_addr", &env::var("BIT_ADDR").ok());
                context.insert("eth_addr", &env::var("ETH_ADDR").ok());
                context.insert("xmr_addr", &env::var("XMR_ADDR").ok());
                TEMPLATES.render("book.html", &context).unwrap()
            })
            .value()
            .clone(),
    ))
}

async fn info(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("entries", &state.cache.len());
    Html(TEMPLATES.render("info.html", &context).unwrap())
}

async fn fallback() -> Redirect {
    Redirect::to("/")
}

#[tokio::main]
async fn main() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let app_state = Arc::new(AppState::default());

    let app = Router::new()
        .route("/", get(index))
        .route("/book/:book_name", get(books))
        .route("/.info", get(info))
        .route("/favicon.svg", get(images::favicon_svg))
        .route("/favicon.png", get(images::favicon_png))
        .route("/favicon.ico", get(images::favicon_ico))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .fallback(fallback)
        .with_state(app_state);

    let port = env::var("ROCKET_PORT")
        .or_else(|_| env::var("PORT"))
        .unwrap_or_else(|_| "8000".to_string());

    let addr = format!("127.0.0.1:{}", port)
        .parse::<std::net::SocketAddr>()
        .unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    info!("Listening on http://{}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
