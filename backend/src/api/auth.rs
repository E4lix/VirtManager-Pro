use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

use crate::models::{ApiResponse, Claims, LoginRequest, LoginResponse};
use crate::state::AppState;

pub fn extract_claims(headers: &HeaderMap, state: &AppState) -> Option<Claims> {
    let auth_header = headers.get("authorization")?.to_str().ok()?;
    if !auth_header.starts_with("Bearer ") {
        return None;
    }
    let token = &auth_header[7..];
    state.verify_token(token).ok()
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<LoginResponse>),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let store = state.store.read().await;

    if let Some(user) = store.users.get(&payload.username) {
        if user.password_hash == payload.password {
            if let Ok(token) = state.generate_token(user) {
                return (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        message: "Connexion réussie".to_string(),
                        data: Some(LoginResponse {
                            token,
                            user: user.clone(),
                        }),
                    }),
                );
            }
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(ApiResponse {
            success: false,
            message: "Nom d'utilisateur ou mot de passe incorrect".to_string(),
            data: None,
        }),
    )
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Current authenticated user", body = ApiResponse<User>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn me_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(claims) = extract_claims(&headers, &state) {
        let store = state.store.read().await;
        if let Some(user) = store.users.get(&claims.username) {
            return (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    message: "Utilisateur authentifié".to_string(),
                    data: Some(user.clone()),
                }),
            );
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(ApiResponse {
            success: false,
            message: "Session invalide ou expirée".to_string(),
            data: None,
        }),
    )
}
