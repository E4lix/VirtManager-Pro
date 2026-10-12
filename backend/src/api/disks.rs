use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

use crate::models::{
    ApiResponse, CreateDiskRequest, ResizeDiskRequest, VmDisk,
};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/disks",
    responses(
        (status = 200, description = "List of all virtual disks", body = ApiResponse<Vec<VmDisk>>)
    )
)]
pub async fn list_disks(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut all_disks = Vec::new();

    // Query disks from local /var/lib/libvirt/images
    if let Ok(mut entries) = tokio::fs::read_dir("/var/lib/libvirt/images").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "qcow2" || ext == "raw" || ext == "img" || ext == "vmdk" {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let metadata = entry.metadata().await.ok();
                    let size_gb = metadata.map(|m| m.len() / (1024 * 1024 * 1024)).unwrap_or(10);

                    all_disks.push(VmDisk {
                        id: format!("disk-{}", name),
                        name: name.clone(),
                        path: path.to_string_lossy().to_string(),
                        size_gb: if size_gb == 0 { 20 } else { size_gb },
                        format: ext.to_string(),
                        pool_name: "images".to_string(),
                    });
                }
            }
        }
    }

    // Also include disks registered in active VMs
    {
        let store = state.store.read().await;
        for vm in store.vms.values() {
            for d in &vm.disks {
                if !all_disks.iter().any(|existing| existing.path == d.path) {
                    all_disks.push(d.clone());
                }
            }
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Disques virtuels récupérés".to_string(),
            data: Some(all_disks),
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/disks",
    request_body = CreateDiskRequest,
    responses(
        (status = 201, description = "Disk created", body = ApiResponse<VmDisk>),
        (status = 400, description = "Failed to create disk")
    )
)]
pub async fn create_disk(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateDiskRequest>,
) -> impl IntoResponse {
    let driver = state.get_driver("local-kvm").await;

    if let Some(drv) = driver {
        match drv
            .create_disk(
                &payload.pool_name,
                &payload.name,
                payload.size_gb,
                &payload.format,
            )
            .await
        {
            Ok(disk) => {
                return (
                    StatusCode::CREATED,
                    Json(ApiResponse {
                        success: true,
                        message: "Disque virtuel créé avec succès".to_string(),
                        data: Some(disk),
                    }),
                );
            }
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Échec de création du disque: {}", e),
                        data: None,
                    }),
                );
            }
        }
    }

    (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse {
            success: false,
            message: "Hyperviseur local indisponible".to_string(),
            data: None,
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/disks/resize",
    request_body = ResizeDiskRequest,
    responses(
        (status = 200, description = "Disk resized", body = ApiResponse<bool>),
        (status = 400, description = "Resize failed")
    )
)]
pub async fn resize_disk(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResizeDiskRequest>,
) -> impl IntoResponse {
    let driver = state.get_driver("local-kvm").await;

    if let Some(drv) = driver {
        match drv.resize_disk(&payload.disk_path, payload.new_size_gb).await {
            Ok(_) => {
                return (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        message: format!(
                            "Disque étendu avec succès à {} GB",
                            payload.new_size_gb
                        ),
                        data: Some(true),
                    }),
                );
            }
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Échec du redimensionnement: {}", e),
                        data: None,
                    }),
                );
            }
        }
    }

    (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse {
            success: false,
            message: "Hyperviseur indisponible".to_string(),
            data: None,
        }),
    )
}

#[utoipa::path(
    get,
    path = "/api/storage/pools",
    responses(
        (status = 200, description = "List storage pools", body = ApiResponse<Vec<StoragePool>>)
    )
)]
pub async fn list_storage_pools(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let driver = state.get_driver("local-kvm").await;

    let pools = if let Some(drv) = driver {
        drv.list_storage_pools().await.unwrap_or_default()
    } else {
        vec![]
    };

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Pools de stockage récupérés".to_string(),
            data: Some(pools),
        }),
    )
}
