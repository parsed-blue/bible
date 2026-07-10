mod bible;
mod images;
mod templates;
mod views;

#[cfg(feature = "erv")]
mod erv;
#[cfg(feature = "kjv")]
mod kjv;
#[cfg(feature = "web")]
mod web;

#[cfg(not(any(feature = "kjv", feature = "erv", feature = "web")))]
compile_error!("Exactly one Bible version feature must be enabled: 'kjv', 'erv', or 'web'.");
#[cfg(all(feature = "kjv", feature = "erv"))]
compile_error!("Features 'kjv' and 'erv' are mutually exclusive.");
#[cfg(all(feature = "kjv", feature = "web"))]
compile_error!("Features 'kjv' and 'web' are mutually exclusive.");
#[cfg(all(feature = "erv", feature = "web"))]
compile_error!("Features 'erv' and 'web' are mutually exclusive.");

use bible::Bible;

use templates::TEMPLATES;

use std::sync::Arc;
use std::{env, net::SocketAddr};

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

use tera::Context;

use crate::bible::BookSlug;
use crate::views::BookView;

const COMMIT_HASH: Option<&str> = option_env!("COMMIT_HASH");

#[cfg(feature = "kjv")]
const VERSION: &str = "KJV";
#[cfg(feature = "erv")]
const VERSION: &str = "ERV";
#[cfg(feature = "web")]
const VERSION: &str = "WEB";

#[cfg(feature = "kjv")]
fn load_bible() -> Bible {
    kjv::load()
}
#[cfg(feature = "erv")]
fn load_bible() -> Bible {
    erv::load()
}
#[cfg(feature = "web")]
fn load_bible() -> Bible {
    web::load()
}

struct AppState {
    commit_hash: String,
    bible: Bible,
    cache: DashMap<BookSlug, String>,
    base_url: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            commit_hash: match COMMIT_HASH {
                Some(var) => String::from(var),
                None => String::from("<unknown>"),
            },
            cache: DashMap::new(),
            bible: load_bible(),
            base_url: env::var("BASE_URL").ok(),
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

    if let Some(cached) = state.cache.get(&book_slug) {
        return Ok(Html(cached.value().clone()));
    }

    let mut context = Context::new();
    context.insert(
        "book",
        &BookView::new(&state.bible, book, state.base_url.as_deref()),
    );
    context.insert("version", &VERSION);
    context.insert("commit_hash", &state.commit_hash.as_str());
    context.insert("bit_addr", &env::var("BIT_ADDR").ok());
    context.insert("eth_addr", &env::var("ETH_ADDR").ok());
    context.insert("xmr_addr", &env::var("XMR_ADDR").ok());
    let rendered = TEMPLATES.render("book.html", &context).unwrap();
    let entry = state.cache.entry(book_slug.clone()).or_insert(rendered);

    Ok(Html(entry.value().clone()))
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

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    info!("Listening on http://{}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
