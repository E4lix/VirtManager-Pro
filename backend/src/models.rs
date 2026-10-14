use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum UserRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "user")]
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub full_name: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: UserRole,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum HypervisorType {
    #[serde(rename = "kvm")]
    Kvm,
    #[serde(rename = "proxmox")]
    Proxmox,
    #[serde(rename = "remote_kvm")]
    RemoteKvm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum HypervisorStatus {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HypervisorStats {
    pub cpu_cores: u32,
    pub cpu_usage_pct: f32,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub disk_total_gb: u64,
    pub disk_used_gb: u64,
    pub active_vms: u32,
    pub total_vms: u32,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Hypervisor {
    pub id: String,
    pub name: String,
    pub hypervisor_type: HypervisorType,
    pub endpoint: String, // e.g. "qemu:///system", "qemu+ssh://root@192.168.1.50/system"
    pub status: HypervisorStatus,
    pub is_local: bool,
    pub stats: HypervisorStats,
    pub created_at: String,
    #[serde(default)]
    pub node_name: Option<String>,
    #[serde(default)]
    pub token_id: Option<String>,
    #[serde(default)]
    pub token_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateHypervisorRequest {
    pub name: String,
    pub hypervisor_type: HypervisorType,
    pub endpoint: String,
    pub is_local: bool,
    #[serde(default)]
    pub node_name: Option<String>,
    #[serde(default)]
    pub token_id: Option<String>,
    #[serde(default)]
    pub token_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum VmState {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VmDisk {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_gb: u64,
    pub format: String, // qcow2, raw, vmdk
    pub pool_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VirtualMachine {
    pub id: String,
    pub name: String,
    pub hypervisor_id: String,
    pub hypervisor_name: String,
    pub state: VmState,
    pub vcpus: u32,
    pub memory_mb: u64,
    pub disks: Vec<VmDisk>,
    pub assigned_to: Option<String>, // username
    pub vnc_port: Option<u16>,
    pub vnc_ws_path: Option<String>,
    pub ip_address: Option<String>,
    pub os_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVmRequest {
    pub name: String,
    pub hypervisor_id: String,
    pub vcpus: u32,
    pub memory_mb: u64,
    pub disk_size_gb: u64,
    pub disk_format: Option<String>, // qcow2 by default
    pub template_id: Option<String>,
    pub os_type: Option<String>,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CloneVmRequest {
    pub new_name: String,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AssignVmRequest {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StoragePool {
    pub name: String,
    pub path: String,
    pub capacity_gb: u64,
    pub allocated_gb: u64,
    pub available_gb: u64,
    pub pool_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDiskRequest {
    pub name: String,
    pub pool_name: String,
    pub size_gb: u64,
    pub format: String, // qcow2, raw, vmdk
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResizeDiskRequest {
    pub disk_path: String,
    pub new_size_gb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum TemplateStatus {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "building")]
    Building,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub os: String,
    pub version: String,
    pub packer_file: String,
    pub ansible_playbook: String,
    pub default_vcpus: u32,
    pub default_memory_mb: u64,
    pub default_disk_gb: u64,
    pub status: TemplateStatus,
    pub disk_image_path: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeployTemplateRequest {
    pub vm_name: String,
    pub hypervisor_id: String,
    pub vcpus: Option<u32>,
    pub memory_mb: Option<u64>,
    pub disk_size_gb: Option<u64>,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum TaskStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BackgroundTask {
    pub id: String,
    pub title: String,
    pub task_type: String, // "packer_build", "ansible_run", "vm_clone"
    pub status: TaskStatus,
    pub logs: Vec<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttachDiskRequest {
    pub disk_path: String,
    pub target_dev: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

