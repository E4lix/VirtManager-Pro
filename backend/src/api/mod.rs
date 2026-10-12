pub mod auth;
pub mod disks;
pub mod hypervisors;
pub mod templates;
pub mod users;
pub mod vms;

use axum::routing::{delete, get, post};
use axum::Router;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::models::*;
use crate::state::AppState;
use crate::vnc::vnc_ws_handler;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::login_handler,
        auth::me_handler,
        hypervisors::list_hypervisors,
        hypervisors::get_hypervisor,
        hypervisors::create_hypervisor,
        hypervisors::delete_hypervisor,
        vms::list_vms,
        vms::get_vm,
        vms::create_vm,
        vms::start_vm,
        vms::stop_vm,
        vms::reboot_vm,
        vms::delete_vm,
        vms::clone_vm,
        vms::assign_vm,
        vms::get_vm_vnc,
        vms::attach_disk_to_vm,
        vms::detach_disk_from_vm,
        disks::list_disks,
        disks::create_disk,
        disks::resize_disk,
        disks::list_storage_pools,
        templates::list_templates,
        templates::get_template,
        templates::trigger_template_build,
        templates::deploy_template,
        templates::list_tasks,
        users::list_users,
        users::create_user,
    ),
    components(
        schemas(
            User,
            UserRole,
            LoginRequest,
            LoginResponse,
            Hypervisor,
            HypervisorType,
            HypervisorStatus,
            HypervisorStats,
            CreateHypervisorRequest,
            VirtualMachine,
            VmState,
            VmDisk,
            CreateVmRequest,
            CloneVmRequest,
            AssignVmRequest,
            AttachDiskRequest,
            StoragePool,
            CreateDiskRequest,
            ResizeDiskRequest,
            Template,
            TemplateStatus,
            DeployTemplateRequest,
            BackgroundTask,
            TaskStatus,
            users::CreateUserRequest,
        )
    ),
    tags(
        (name = "Auth", description = "Authentification et profils utilisateurs"),
        (name = "Hypervisors", description = "Gestion des hyperviseurs locaux et distants"),
        (name = "VirtualMachines", description = "Cycle de vie, clonage, attribution et VNC des machines virtuelles"),
        (name = "Storage", description = "Gestion des disques virtuels et des pools de stockage"),
        (name = "Templates", description = "Automatisation Packer, provisionnement Ansible et déploiement"),
        (name = "Users", description = "Gestion des rôles (Admin/User) et permissions"),
    ),
    info(
        title = "VirtManager Pro REST API",
        version = "1.0.0",
        description = "Documentation interactive de l'API de gestion d'infrastructure virtualisée (Projet 5A)",
        contact(name = "Équipe Projet Virtualisation 5A")
    )
)]
pub struct ApiDoc;

pub fn create_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        // Auth
        .route("/auth/login", post(auth::login_handler))
        .route("/auth/me", get(auth::me_handler))
        // Hypervisors
        .route("/hypervisors", get(hypervisors::list_hypervisors))
        .route("/hypervisors", post(hypervisors::create_hypervisor))
        .route("/hypervisors/:id", get(hypervisors::get_hypervisor))
        .route("/hypervisors/:id", delete(hypervisors::delete_hypervisor))
        // VMs
        .route("/vms", get(vms::list_vms))
        .route("/vms", post(vms::create_vm))
        .route("/vms/:id", get(vms::get_vm))
        .route("/vms/:id", delete(vms::delete_vm))
        .route("/vms/:id/start", post(vms::start_vm))
        .route("/vms/:id/stop", post(vms::stop_vm))
        .route("/vms/:id/reboot", post(vms::reboot_vm))
        .route("/vms/:id/clone", post(vms::clone_vm))
        .route("/vms/:id/assign", post(vms::assign_vm))
        .route("/vms/:id/vnc", get(vms::get_vm_vnc))
        .route("/vms/:id/disks", post(vms::attach_disk_to_vm))
        .route("/vms/:id/disks/:target", delete(vms::detach_disk_from_vm))
        // Storage & Disks
        .route("/disks", get(disks::list_disks))
        .route("/disks", post(disks::create_disk))
        .route("/disks/resize", post(disks::resize_disk))
        .route("/storage/pools", get(disks::list_storage_pools))
        // Templates & Automation (Packer / Ansible)
        .route("/templates", get(templates::list_templates))
        .route("/templates/:id", get(templates::get_template))
        .route("/templates/:id/build", post(templates::trigger_template_build))
        .route("/templates/:id/deploy", post(templates::deploy_template))
        .route("/tasks", get(templates::list_tasks))
        // Users
        .route("/users", get(users::list_users))
        .route("/users", post(users::create_user))
        // VNC WebSocket Proxy
        .route("/vnc/ws/:vm_id", get(vnc_ws_handler));

    Router::new()
        .nest("/api", api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
