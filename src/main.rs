use std::sync::{Arc, Mutex};

use axum::{
    Router,
    extract::{Path, Query, State},
    response::Redirect,
    routing::{get, post},
};
use error::{Error, QrLinkResult};
use serde::Deserialize;
use tokio::net::TcpListener;

mod error;

#[derive(Clone)]
struct AppState {
    pub database: Arc<Mutex<rusqlite::Connection>>,
}

pub static SQL: &str = "
CREATE TABLE IF NOT EXISTS urls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    external_id TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS stats (
    url_id INTEGER NOT NULL,
    ip_addr TEXT NOT NULL,
    clicked_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (url_id) REFERENCES urls(id) ON DELETE CASCADE
);
";

#[tokio::main]
async fn main() {
    let conn = rusqlite::Connection::open("forum.db").unwrap();
    conn.execute_batch(SQL).unwrap();
    let database = Arc::new(Mutex::new(conn));
    let app_state = AppState { database };
    let app = Router::new()
        .route("/{external_id}", get(get_url))
        .route("/{external_id}/qr", get(get_qr))
        .route("/{external_id}/meta", get(get_meta))
        .route("/", get(get_info))
        .route("/", post(create_url))
        .with_state(app_state);
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// GET /<id> forwards to a databased URL, or 404s
async fn get_url(
    Path(_external_id): Path<u64>,
    State(app_state): State<AppState>,
) -> QrLinkResult<Redirect> {
    todo!("Forward to databased URL");

    Ok(Redirect::to("..."))
}

/// GET /<id>/qr?size=300 draws a QR-kode for /<id>, size is optional
async fn get_qr(
    Path(_external_id): Path<u64>,
    State(_app_state): State<AppState>,
) -> QrLinkResult<Redirect> {
    todo!("Send a QR-code back")
}

/// GET /<id>/meta returns a JSON object with meta data
async fn get_meta(
    Path(_external_id): Path<u64>,
    State(_app_state): State<AppState>,
) -> QrLinkResult<Redirect> {
    todo!("Send the databased JSON object")
}

/// GET /info returns an OpenAPI schema
async fn get_info(State(_app_state): State<AppState>) -> QrLinkResult<Redirect> {
    todo!("Send OpenAPI schema")
}

#[derive(Deserialize)]
struct CreateUrlParams {
    url: String,
}

/// POST /?url=... creates a databased URL and forwards to /<id>/meta
async fn create_url(
    Query(_params): Query<CreateUrlParams>,
    State(_app_state): State<AppState>,
) -> QrLinkResult<Redirect> {
    todo!("Create databased URL and forward to /meta");

    Ok(Redirect::to("..."))
}

fn get_connection(
    app_state: &AppState,
) -> QrLinkResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    app_state
        .database
        .lock()
        .map_err(|poison_err| Error::LockError(format!("{:?}", poison_err)))
}
