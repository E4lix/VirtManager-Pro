use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::models::{ApiResponse, User, UserRole};
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: UserRole,
    pub full_name: String,
    pub email: String,
}

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of users", body = ApiResponse<Vec<User>>)
    )
)]
pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let store = state.store.read().await;
    let users: Vec<User> = store.users.values().cloned().collect();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Utilisateurs récupérés".to_string(),
            data: Some(users),
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = ApiResponse<User>),
        (status = 400, description = "User already exists")
    )
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    let mut store = state.store.write().await;

    if store.users.contains_key(&payload.username) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Ce nom d'utilisateur existe déjà".to_string(),
                data: None,
            }),
        );
    }

    let user = User {
        id: format!("u-{}", uuid::Uuid::new_v4().simple()),
        username: payload.username.clone(),
        password_hash: payload.password,
        role: payload.role,
        full_name: payload.full_name,
        email: payload.email,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    store.users.insert(payload.username, user.clone());

    (
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            message: "Utilisateur créé avec succès".to_string(),
            data: Some(user),
        }),
    )
}
