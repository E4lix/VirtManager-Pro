use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

use crate::api::auth::extract_claims;
use crate::models::{
    ApiResponse, AssignVmRequest, AttachDiskRequest, CloneVmRequest, CreateVmRequest, UserRole,
    VirtualMachine, VmState,
};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/vms",
    responses(
        (status = 200, description = "List of virtual machines", body = ApiResponse<Vec<VirtualMachine>>)
    )
)]
pub async fn list_vms(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);

    // If any registered hypervisor has no VMs in cache yet, trigger an immediate sync
    let needs_sync = {
        let store = state.store.read().await;
        store.hypervisors.keys().any(|hv_id| {
            !store.vms.values().any(|vm| &vm.hypervisor_id == hv_id)
        })
    };
    if needs_sync {
        state.sync_all_hypervisors().await;
    }

    let store = state.store.read().await;

    let vms: Vec<VirtualMachine> = store
        .vms
        .values()
        .filter(|vm| {
            if let Some(ref c) = claims {
                if c.role == UserRole::User {
                    // Standard user only sees their own assigned VMs
                    return vm.assigned_to.as_deref() == Some(&c.username);
                }
            }
            true // Admin or unauthenticated demo sees all
        })
        .cloned()
        .collect();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Machines virtuelles récupérées".to_string(),
            data: Some(vms),
        }),
    )
}

#[utoipa::path(
    get,
    path = "/api/vms/{id}",
    responses(
        (status = 200, description = "VM details", body = ApiResponse<VirtualMachine>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "VM not found")
    )
)]
pub async fn get_vm(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    let store = state.store.read().await;

    if let Some(vm) = store.vms.get(&id) {
        if let Some(ref c) = claims {
            if c.role == UserRole::User && vm.assigned_to.as_deref() != Some(&c.username) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse {
                        success: false,
                        message: "Accès refusé: cette machine virtuelle ne vous est pas attribuée".to_string(),
                        data: None,
                    }),
                );
            }
        }
        return (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: "Détails de la machine virtuelle".to_string(),
                data: Some(vm.clone()),
            }),
        );
    }

    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse {
            success: false,
            message: "Machine virtuelle introuvable".to_string(),
            data: None,
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/vms",
    request_body = CreateVmRequest,
    responses(
        (status = 201, description = "VM created", body = ApiResponse<VirtualMachine>),
        (status = 400, description = "Failed to create VM")
    )
)]
pub async fn create_vm(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateVmRequest>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    let driver = state.get_driver(&payload.hypervisor_id).await;

    if let Some(drv) = driver {
        match drv.create_vm(&payload).await {
            Ok(mut new_vm) => {
                if let Some(ref c) = claims {
                    if c.role == UserRole::User {
                        new_vm.assigned_to = Some(c.username.clone());
                    } else {
                        new_vm.assigned_to = payload.assigned_to.clone();
                    }
                } else {
                    new_vm.assigned_to = payload.assigned_to.clone();
                }

                {
                    let mut store = state.store.write().await;
                    store.vms.insert(new_vm.id.clone(), new_vm.clone());
                }
                let _ = state.save_to_disk().await;

                return (
                    StatusCode::CREATED,
                    Json(ApiResponse {
                        success: true,
                        message: "Machine virtuelle créée et démarrée avec succès".to_string(),
                        data: Some(new_vm),
                    }),
                );
            }
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Erreur lors de la création de la VM: {}", e),
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
            message: "Hyperviseur spécifié introuvable".to_string(),
            data: None,
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/vms/{id}/start",
    responses(
        (status = 200, description = "VM started", body = ApiResponse<bool>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "VM not found")
    )
)]
pub async fn start_vm(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    let (hypervisor_id, vm_name) = {
        let store = state.store.read().await;
        if let Some(vm) = store.vms.get(&id) {
            if let Some(ref c) = claims {
                if c.role == UserRole::User && vm.assigned_to.as_deref() != Some(&c.username) {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse {
                            success: false,
                            message: "Accès refusé: cette machine virtuelle ne vous est pas attribuée".to_string(),
                            data: None,
                        }),
                    );
                }
            }
            (vm.hypervisor_id.clone(), vm.name.clone())
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "VM introuvable".to_string(),
                    data: None,
                }),
            );
        }
    };

    if let Some(drv) = state.get_driver(&hypervisor_id).await {
        let _ = drv.start_vm(&vm_name).await;
    }

    {
        let mut store = state.store.write().await;
        if let Some(vm) = store.vms.get_mut(&id) {
            vm.state = VmState::Running;
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Machine virtuelle démarrée".to_string(),
            data: Some(true),
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/vms/{id}/stop",
    responses(
        (status = 200, description = "VM stopped", body = ApiResponse<bool>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "VM not found")
    )
)]
pub async fn stop_vm(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    let (hypervisor_id, vm_name) = {
        let store = state.store.read().await;
        if let Some(vm) = store.vms.get(&id) {
            if let Some(ref c) = claims {
                if c.role == UserRole::User && vm.assigned_to.as_deref() != Some(&c.username) {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse {
                            success: false,
                            message: "Accès refusé: cette machine virtuelle ne vous est pas attribuée".to_string(),
                            data: None,
                        }),
                    );
                }
            }
            (vm.hypervisor_id.clone(), vm.name.clone())
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "VM introuvable".to_string(),
                    data: None,
                }),
            );
        }
    };

    if let Some(drv) = state.get_driver(&hypervisor_id).await {
        let _ = drv.stop_vm(&vm_name, false).await;
    }

    {
        let mut store = state.store.write().await;
        if let Some(vm) = store.vms.get_mut(&id) {
            vm.state = VmState::Stopped;
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Signal d'arrêt envoyé à la machine virtuelle".to_string(),
            data: Some(true),
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/vms/{id}/reboot",
    responses(
        (status = 200, description = "VM rebooted", body = ApiResponse<bool>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "VM not found")
    )
)]
pub async fn reboot_vm(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    let (hypervisor_id, vm_name) = {
        let store = state.store.read().await;
        if let Some(vm) = store.vms.get(&id) {
            if let Some(ref c) = claims {
                if c.role == UserRole::User && vm.assigned_to.as_deref() != Some(&c.username) {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse {
                            success: false,
                            message: "Accès refusé: cette machine virtuelle ne vous est pas attribuée".to_string(),
                            data: None,
                        }),
                    );
                }
            }
            (vm.hypervisor_id.clone(), vm.name.clone())
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "VM introuvable".to_string(),
                    data: None,
                }),
            );
        }
    };

    if let Some(drv) = state.get_driver(&hypervisor_id).await {
        let _ = drv.reboot_vm(&vm_name).await;
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Redémarrage de la machine virtuelle initié".to_string(),
            data: Some(true),
        }),
    )
}

#[utoipa::path(
    delete,
    path = "/api/vms/{id}",
    responses(
        (status = 200, description = "VM deleted", body = ApiResponse<bool>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "VM not found")
    )
)]
pub async fn delete_vm(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    let (hypervisor_id, vm_name) = {
        let store = state.store.read().await;
        if let Some(vm) = store.vms.get(&id) {
            if let Some(ref c) = claims {
                if c.role == UserRole::User && vm.assigned_to.as_deref() != Some(&c.username) {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse {
                            success: false,
                            message: "Accès refusé: vous ne pouvez pas supprimer une machine virtuelle qui ne vous appartient pas".to_string(),
                            data: None,
                        }),
                    );
                }
            }
            (vm.hypervisor_id.clone(), vm.name.clone())
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "VM introuvable".to_string(),
                    data: None,
                }),
            );
        }
    };

    if let Some(drv) = state.get_driver(&hypervisor_id).await {
        let _ = drv.delete_vm(&vm_name).await;
    }

    {
        let mut store = state.store.write().await;
        store.vms.remove(&id);
    }
    let _ = state.save_to_disk().await;

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Machine virtuelle supprimée".to_string(),
            data: Some(true),
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/vms/{id}/clone",
    request_body = CloneVmRequest,
    responses(
        (status = 201, description = "VM cloned", body = ApiResponse<VirtualMachine>),
        (status = 403, description = "Forbidden"),
        (status = 400, description = "Clone failed")
    )
)]
pub async fn clone_vm(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CloneVmRequest>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    let (hypervisor_id, vm_name) = {
        let store = state.store.read().await;
        if let Some(vm) = store.vms.get(&id) {
            if let Some(ref c) = claims {
                if c.role == UserRole::User && vm.assigned_to.as_deref() != Some(&c.username) {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse {
                            success: false,
                            message: "Accès refusé: vous ne pouvez cloner qu'une machine virtuelle qui vous est attribuée".to_string(),
                            data: None,
                        }),
                    );
                }
            }
            (vm.hypervisor_id.clone(), vm.name.clone())
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "VM source introuvable".to_string(),
                    data: None,
                }),
            );
        }
    };

    if let Some(drv) = state.get_driver(&hypervisor_id).await {
        match drv.clone_vm(&vm_name, &payload.new_name).await {
            Ok(mut cloned_vm) => {
                if let Some(ref c) = claims {
                    if c.role == UserRole::User {
                        cloned_vm.assigned_to = Some(c.username.clone());
                    } else {
                        cloned_vm.assigned_to = payload.assigned_to.clone();
                    }
                } else {
                    cloned_vm.assigned_to = payload.assigned_to.clone();
                }

                {
                    let mut store = state.store.write().await;
                    store.vms.insert(cloned_vm.id.clone(), cloned_vm.clone());
                }
                let _ = state.save_to_disk().await;

                return (
                    StatusCode::CREATED,
                    Json(ApiResponse {
                        success: true,
                        message: "Machine virtuelle clonée avec succès".to_string(),
                        data: Some(cloned_vm),
                    }),
                );
            }
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Échec du clonage: {}", e),
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
    post,
    path = "/api/vms/{id}/assign",
    request_body = AssignVmRequest,
    responses(
        (status = 200, description = "VM assigned to user", body = ApiResponse<VirtualMachine>),
        (status = 403, description = "Forbidden (Admin only)"),
        (status = 404, description = "VM not found")
    )
)]
pub async fn assign_vm(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AssignVmRequest>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    if let Some(ref c) = claims {
        if c.role != UserRole::Admin {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse {
                    success: false,
                    message: "Action réservée aux administrateurs. Seul l'administrateur peut attribuer ou transférer une VM.".to_string(),
                    data: None,
                }),
            );
        }
    }

    let mut store = state.store.write().await;

    if let Some(vm) = store.vms.get_mut(&id) {
        vm.assigned_to = Some(payload.username.clone());
        let updated = vm.clone();
        drop(store);
        let _ = state.save_to_disk().await;

        return (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: format!("VM attribuée à l'utilisateur {}", payload.username),
                data: Some(updated),
            }),
        );
    }

    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse {
            success: false,
            message: "Machine virtuelle introuvable".to_string(),
            data: None,
        }),
    )
}

#[utoipa::path(
    get,
    path = "/api/vms/{id}/vnc",
    responses(
        (status = 200, description = "VNC connection info", body = ApiResponse<serde_json::Value>),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "VM not found")
    )
)]
pub async fn get_vm_vnc(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    let (hypervisor_id, vm_name, vnc_port) = {
        let store = state.store.read().await;
        if let Some(vm) = store.vms.get(&id) {
            if let Some(ref c) = claims {
                if c.role == UserRole::User && vm.assigned_to.as_deref() != Some(&c.username) {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse {
                            success: false,
                            message: "Accès à la console VNC réservé au propriétaire de la machine virtuelle".to_string(),
                            data: None,
                        }),
                    );
                }
            }
            (vm.hypervisor_id.clone(), vm.name.clone(), vm.vnc_port)
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "VM introuvable".to_string(),
                    data: None,
                }),
            );
        }
    };

    let resolved_port = if let Some(drv) = state.get_driver(&hypervisor_id).await {
        drv.get_vnc_info(&vm_name).await.map(|i| i.port).unwrap_or(5900)
    } else {
        vnc_port.unwrap_or(5900)
    };

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Informations VNC disponibles".to_string(),
            data: Some(serde_json::json!({
                "vm_id": id,
                "vnc_port": resolved_port,
                "ws_path": format!("/api/vnc/ws/{}", id),
                "novnc_url": format!("/novnc/vnc_lite.html?path=api/vnc/ws/{}", id)
            })),
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/vms/{id}/disks",
    request_body = AttachDiskRequest,
    responses(
        (status = 200, description = "Disk attached to VM", body = ApiResponse<bool>),
        (status = 403, description = "Forbidden"),
        (status = 400, description = "Failed to attach disk")
    )
)]
pub async fn attach_disk_to_vm(
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AttachDiskRequest>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    let (hypervisor_id, vm_name) = {
        let store = state.store.read().await;
        if let Some(vm) = store.vms.get(&id) {
            if let Some(ref c) = claims {
                if c.role == UserRole::User && vm.assigned_to.as_deref() != Some(&c.username) {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse {
                            success: false,
                            message: "Accès refusé: cette machine virtuelle ne vous est pas attribuée".to_string(),
                            data: None,
                        }),
                    );
                }
            }
            (vm.hypervisor_id.clone(), vm.name.clone())
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Machine virtuelle introuvable".to_string(),
                    data: None,
                }),
            );
        }
    };

    let target_dev = payload.target_dev.unwrap_or_else(|| "vdb".to_string());

    if let Some(drv) = state.get_driver(&hypervisor_id).await {
        match drv.attach_disk(&vm_name, &payload.disk_path, &target_dev).await {
            Ok(_) => {
                // Refresh VM details
                if let Ok(updated_vm) = drv.get_vm(&vm_name).await {
                    let mut store = state.store.write().await;
                    store.vms.insert(id, updated_vm);
                }
                let _ = state.save_to_disk().await;

                return (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        message: format!("Disque '{}' attaché avec succès sur {}", payload.disk_path, target_dev),
                        data: Some(true),
                    }),
                );
            }
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Échec de l'attachement du disque: {}", e),
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
    delete,
    path = "/api/vms/{id}/disks/{target}",
    responses(
        (status = 200, description = "Disk detached from VM", body = ApiResponse<bool>),
        (status = 403, description = "Forbidden"),
        (status = 400, description = "Failed to detach disk")
    )
)]
pub async fn detach_disk_from_vm(
    headers: HeaderMap,
    Path((id, target)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let claims = extract_claims(&headers, &state);
    let (hypervisor_id, vm_name) = {
        let store = state.store.read().await;
        if let Some(vm) = store.vms.get(&id) {
            if let Some(ref c) = claims {
                if c.role == UserRole::User && vm.assigned_to.as_deref() != Some(&c.username) {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse {
                            success: false,
                            message: "Accès refusé: cette machine virtuelle ne vous est pas attribuée".to_string(),
                            data: None,
                        }),
                    );
                }
            }
            (vm.hypervisor_id.clone(), vm.name.clone())
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Machine virtuelle introuvable".to_string(),
                    data: None,
                }),
            );
        }
    };

    if let Some(drv) = state.get_driver(&hypervisor_id).await {
        match drv.detach_disk(&vm_name, &target).await {
            Ok(_) => {
                if let Ok(updated_vm) = drv.get_vm(&vm_name).await {
                    let mut store = state.store.write().await;
                    store.vms.insert(id, updated_vm);
                }
                let _ = state.save_to_disk().await;

                return (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        message: format!("Disque {} détaché avec succès de la machine", target),
                        data: Some(true),
                    }),
                );
            }
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Échec du détachement du disque: {}", e),
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

