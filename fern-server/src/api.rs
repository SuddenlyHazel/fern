use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};

use crate::{
    Server,
    server::{CreateResponse, GuestInfo, UpdateResponse},
};

pub mod client;
pub use client::*;

pub async fn api_server(server: Server) {
    let app = Router::new()
        .route(
            "/api/guest",
            get(list_guests).post(create_module).put(update_module),
        )
        .with_state(server);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateModule {
    guest_name: String,
    module: Vec<u8>,
}

async fn create_module(
    State(server): State<Server>,
    Json(CreateModule { guest_name, module }): Json<CreateModule>,
) -> Result<Json<CreateResponse>, AppError> {
    Ok(Json(server.create_module(guest_name, module).await?))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateModule {
    guest_name: String,
    module: Vec<u8>,
}

async fn update_module(
    State(server): State<Server>,
    Json(UpdateModule { guest_name, module }): Json<UpdateModule>,
) -> Result<Json<UpdateResponse>, AppError> {
    Ok(Json(server.update_module(guest_name, module).await?))
}

async fn list_guests(State(server): State<Server>) -> Result<Json<Vec<GuestInfo>>, AppError> {
    Ok(Json(server.guest_info().await?))
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
