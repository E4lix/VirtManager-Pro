use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

use crate::api::auth::extract_claims;
use crate::models::{
    ApiResponse, CreateHypervisorRequest, Hypervisor, HypervisorStats, HypervisorStatus,
    UserRole, VmState,
};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/hypervisors",
    responses(
        (status = 200, description = "List of all hypervisors", body = ApiResponse<Vec<Hypervisor>>)
    )
)]
pub async fn list_hypervisors(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let store = state.store.read().await;
    let mut hvs: Vec<Hypervisor> = store.hypervisors.values().cloned().collect();

    // Refresh telemetry stats asynchronously from active drivers
    for hv in &mut hvs {
        if let Some(driver) = state.get_driver(&hv.id).await {
            if let Ok(stats) = driver.get_stats().await {
                hv.stats = stats;
            }
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Hyperviseurs récupérés avec succès".to_string(),
            data: Some(hvs),
        }),
    )
}

#[utoipa::path(
    get,
    path = "/api/hypervisors/{id}",
    responses(
        (status = 200, description = "Hypervisor details", body = ApiResponse<Hypervisor>),
        (status = 404, description = "Hypervisor not found")
    )
)]
pub async fn get_hypervisor(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let store = state.store.read().await;
    if let Some(mut hv) = store.hypervisors.get(&id).cloned() {
        if let Some(driver) = state.get_driver(&id).await {
            if let Ok(stats) = driver.get_stats().await {
                hv.stats = stats;
            }
        }

        return (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: "Hyperviseur trouvé".to_string(),
                data: Some(hv),
            }),
        );
    }

    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse {
            success: false,
            message: "Hyperviseur introuvable".to_string(),
            data: None,
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/hypervisors",
    request_body = CreateHypervisorRequest,
    responses(
        (status = 201, description = "Hypervisor created", body = ApiResponse<Hypervisor>),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden (Admin only)")
    )
)]
pub async fn create_hypervisor(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateHypervisorRequest>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    if claims.map(|c| c.role != UserRole::Admin).unwrap_or(false) {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Action réservée aux administrateurs".to_string(),
                data: None,
            }),
        );
    }

    let id = format!("hv-{}", uuid::Uuid::new_v4().simple());
    let new_hv = Hypervisor {
        id: id.clone(),
        name: payload.name,
        hypervisor_type: payload.hypervisor_type,
        endpoint: payload.endpoint,
        status: HypervisorStatus::Online,
        is_local: payload.is_local,
        node_name: payload.node_name,
        token_id: payload.token_id,
        token_secret: payload.token_secret,
        stats: HypervisorStats {
            cpu_cores: 8,
            cpu_usage_pct: 0.0,
            memory_total_mb: 8192,
            memory_used_mb: 0,
            disk_total_gb: 100,
            disk_used_gb: 0,
            active_vms: 0,
            total_vms: 0,
            uptime_seconds: 0,
        },
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // Register driver
    state.register_driver(&new_hv).await;

    // Ping test
    let is_online = if let Some(driver) = state.get_driver(&id).await {
        driver.ping().await.unwrap_or(false)
    } else {
        false
    };

    let mut final_hv = new_hv.clone();
    final_hv.status = if is_online {
        HypervisorStatus::Online
    } else {
        HypervisorStatus::Offline
    };

    {
        let mut store = state.store.write().await;
        store.hypervisors.insert(id.clone(), final_hv.clone());
    }
    let _ = state.save_to_disk().await;

    // Trigger sync in background so HTTP response is immediate
    let state_clone = state.clone();
    tokio::spawn(async move {
        state_clone.sync_all_hypervisors().await;
        let _ = state_clone.save_to_disk().await;
    });

    (
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            message: "Hyperviseur ajouté avec succès".to_string(),
            data: Some(final_hv),
        }),
    )
}

#[utoipa::path(
    delete,
    path = "/api/hypervisors/{id}",
    responses(
        (status = 200, description = "Hypervisor deleted", body = ApiResponse<bool>),
        (status = 400, description = "Cannot delete hypervisor with active VMs"),
        (status = 404, description = "Hypervisor not found")
    )
)]
pub async fn delete_hypervisor(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    if claims.map(|c| c.role != UserRole::Admin).unwrap_or(false) {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Action réservée aux administrateurs".to_string(),
                data: None,
            }),
        );
    }

    let mut store = state.store.write().await;

    // Module B Requirement: "vérification qu'aucune VM active n'y est hébergée"
    let has_active_vms = store
        .vms
        .values()
        .any(|vm| vm.hypervisor_id == id && vm.state == VmState::Running);

    if has_active_vms {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Impossible de supprimer cet hyperviseur : des machines virtuelles sont actuellement en cours d'exécution dessus.".to_string(),
                data: None,
            }),
        );
    }

    if store.hypervisors.remove(&id).is_some() {
        let mut drivers = state.drivers.write().await;
        drivers.remove(&id);
        drop(drivers);

        // Remove VMs belonging to this hypervisor
        store.vms.retain(|_, vm| vm.hypervisor_id != id);
        drop(store);

        let _ = state.save_to_disk().await;

        return (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: "Hyperviseur retiré avec succès".to_string(),
                data: Some(true),
            }),
        );
    }

    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse {
            success: false,
            message: "Hyperviseur introuvable".to_string(),
            data: None,
        }),
    )
}
