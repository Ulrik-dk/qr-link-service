use std::sync::{Arc, Mutex};

use axum::extract::Query;
use axum::http::header;
use axum::response::IntoResponse;
use axum::{
    Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use error::{Error, QrLinkResult};
use image::Luma;
use qrcode::QrCode;
use serde::Deserialize;
use std::io::Cursor;
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
    Path(external_id): Path<u64>,
    State(app_state): State<AppState>,
) -> QrLinkResult<Redirect> {
    let conn = get_connection(&app_state)?;

    let mut stmt = conn
        .prepare("SELECT external_id FROM urls WHERE id = ? AND deleted_at IS NULL")
        .map_err(Error::DatabaseError)?;
    let url: String = stmt
        .query_row([external_id], |row| row.get(0))
        .map_err(Error::DatabaseError)?;

    Ok(Redirect::to(&url))
}

/// GET /<id>/qr?size=300 draws a QR-kode for /<id>, size is optional
#[derive(Deserialize)]
struct QrQuery {
    size: Option<u32>,
    format: Option<String>, // "ascii" or "png"
}

async fn get_qr(
    Path(external_id): Path<u64>,
    State(_app_state): State<AppState>,
    Query(params): Query<QrQuery>,
) -> QrLinkResult<impl IntoResponse> {
    // Replace this with the actual URL lookup from DB
    let url = format!("https://example.com/{}", external_id);

    match params.format.as_deref() {
        Some("ascii") => {
            let code = QrCode::new(url)
                .map_err(|_| Error::DatabaseErrorTwo("QR code generation failed".into()))?; // Replaced error handling
            let rendered = code
                .render::<char>()
                .quiet_zone(false)
                .module_dimensions(2, 1)
                .build();
            Ok(([(header::CONTENT_TYPE, "text/plain")], rendered).into_response())
        }
        _ => {
            // Default to PNG output
            let code = QrCode::new(url)
                .map_err(|_| Error::DatabaseErrorTwo("QR code generation failed".into()))?;
            let image = code
                .render::<Luma<u8>>()
                .min_dimensions(params.size.unwrap_or(300), params.size.unwrap_or(300))
                .build();

            let mut buffer = Cursor::new(Vec::new());
            image
                .write_to(&mut buffer, image::ImageFormat::Png)
                .unwrap(); // Updated to use the correct method

            let body = buffer.into_inner();
            Ok(([(header::CONTENT_TYPE, "image/png")], body).into_response())
        }
    }
}

/// GET /<id>/meta returns a JSON object with meta data
async fn get_meta(
    Path(external_id): Path<u64>,
    State(app_state): State<AppState>,
) -> QrLinkResult<axum::Json<serde_json::Value>> {
    let conn = get_connection(&app_state)?;

    let mut stmt = conn
        .prepare("SELECT id, external_id FROM urls WHERE id = ? AND deleted_at IS NULL")
        .map_err(Error::DatabaseError)?;

    let (id, url): (u64, String) = stmt
        .query_row([external_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(Error::DatabaseError)?;

    Ok(axum::Json(serde_json::json!({
        "stored_id": id.to_string(),
        "stored_url": url
    })))
}

/// GET /info returns an OpenAPI schema
async fn get_info(
    State(_app_state): State<AppState>,
) -> QrLinkResult<axum::Json<serde_json::Value>> {
    Ok(axum::Json(serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": "QR Link Shortener API",
            "version": "1.0.0"
        },
        "paths": {
            "/{id}": { "get": { "summary": "Redirect to URL" }},
            "/{id}/qr": { "get": { "summary": "Return QR code" }},
            "/{id}/meta": { "get": { "summary": "Return metadata" }},
            "/": { "post": { "summary": "Create short URL" }}
        }
    })))
}

#[derive(Deserialize)]
struct CreateUrlParams {
    url: String,
}

/// POST /?url=... creates a databased URL and forwards to /<id>/meta
async fn create_url(
    Query(params): Query<CreateUrlParams>,
    State(app_state): State<AppState>,
) -> QrLinkResult<axum::Json<serde_json::Value>> {
    let conn = get_connection(&app_state)?;

    conn.execute("INSERT INTO urls (external_id) VALUES (?)", [&params.url])
        .map_err(Error::DatabaseError)?;

    let id = conn.last_insert_rowid();
    let external_id = id.to_string();

    Ok(axum::Json(serde_json::json!({
        "stored_id": external_id,
        "stored_url": params.url
    })))
}

fn get_connection(
    app_state: &AppState,
) -> QrLinkResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    app_state
        .database
        .lock()
        .map_err(|poison_err| Error::LockError(format!("{:?}", poison_err)))
}
