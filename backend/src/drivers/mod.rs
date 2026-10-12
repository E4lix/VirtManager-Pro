pub mod kvm;
pub mod proxmox;

use anyhow::Result;
use async_trait::async_trait;
use crate::models::{
    CreateVmRequest, HypervisorStats, StoragePool, VirtualMachine, VmDisk,
};

#[derive(Debug, Clone)]
pub struct VncInfo {
    pub host: String,
    pub port: u16,
    pub path: String,
}

/// Extensible Driver Trait for Hypervisors (KVM/libvirt, Proxmox VE, VMware ESXi, etc.)
#[async_trait]
pub trait HypervisorDriver: Send + Sync {
    /// Test connection to the hypervisor
    async fn ping(&self) -> Result<bool>;

    /// Fetch system metrics and telemetry
    async fn get_stats(&self) -> Result<HypervisorStats>;

    /// List all virtual machines
    async fn list_vms(&self) -> Result<Vec<VirtualMachine>>;

    /// Get details of a specific VM
    async fn get_vm(&self, vm_id: &str) -> Result<VirtualMachine>;

    /// Create and provision a new VM
    async fn create_vm(&self, req: &CreateVmRequest) -> Result<VirtualMachine>;

    /// Start a stopped VM
    async fn start_vm(&self, vm_id: &str) -> Result<()>;

    /// Stop a running VM (graceful or forced)
    async fn stop_vm(&self, vm_id: &str, force: bool) -> Result<()>;

    /// Reboot a VM
    async fn reboot_vm(&self, vm_id: &str) -> Result<()>;

    /// Delete a VM and optionally its disks
    async fn delete_vm(&self, vm_id: &str) -> Result<()>;

    /// Clone an existing VM
    async fn clone_vm(&self, source_id: &str, new_name: &str) -> Result<VirtualMachine>;

    /// Get VNC display connection info
    async fn get_vnc_info(&self, vm_id: &str) -> Result<VncInfo>;

    /// List storage pools
    async fn list_storage_pools(&self) -> Result<Vec<StoragePool>>;

    /// Create virtual disk image
    async fn create_disk(&self, pool_name: &str, disk_name: &str, size_gb: u64, format: &str) -> Result<VmDisk>;

    /// Resize an existing disk image
    async fn resize_disk(&self, disk_path: &str, new_size_gb: u64) -> Result<()>;

    /// Delete a disk image
    async fn delete_disk(&self, disk_path: &str) -> Result<()>;

    /// Attach a disk to a VM
    async fn attach_disk(&self, vm_id: &str, disk_path: &str, target_dev: &str) -> Result<()>;

    /// Detach a disk from a VM
    async fn detach_disk(&self, vm_id: &str, target_dev: &str) -> Result<()>;
}

