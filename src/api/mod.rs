use std::sync::Arc;

use axum::{middleware::from_fn, Extension, Router};
use tower_http::trace::TraceLayer;

use crate::{
    AppState,
    routes::{auth::auth_routes, users::user_routes, resumes::resume_routes},
    services::middleware::auth,
};

pub fn create_api(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .nest("/auth", auth_routes())
        .nest("/users", user_routes().layer(from_fn(auth)))
        .nest("/resumes", resume_routes().layer(from_fn(auth)))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(app_state));

    Router::new().nest("/api", api_route)
}
