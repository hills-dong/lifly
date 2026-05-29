//! Operations admin panel.
//!
//! Config-based authentication (independent of the `users` table) plus a
//! registry-driven generic CRUD layer over every manageable table. See
//! `docs/tdd.md` §5.3 for the design.

pub mod auth;
pub mod handlers;
pub mod registry;
pub mod repo;

use axum::Router;
use axum::routing::{get, post};

use crate::common::AppState;

/// Build the admin sub-router (all mounted under `/api/admin`).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/admin/login", post(auth::login_handler))
        .route("/api/admin/me", get(auth::me_handler))
        .route("/api/admin/meta", get(handlers::meta_handler))
        .route(
            "/api/admin/data/{resource}",
            get(handlers::list_handler).post(handlers::create_handler),
        )
        .route(
            "/api/admin/data/{resource}/{id}",
            get(handlers::get_one_handler)
                .put(handlers::update_handler)
                .delete(handlers::delete_one_handler),
        )
}
