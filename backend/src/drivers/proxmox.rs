use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

use crate::drivers::{HypervisorDriver, VncInfo};
use crate::models::{
    CreateVmRequest, HypervisorStats, StoragePool, VirtualMachine, VmDisk, VmState,
};

#[derive(Clone)]
pub struct ProxmoxDriver {
    pub hypervisor_id: String,
    pub hypervisor_name: String,
    pub endpoint: String, // e.g. "https://pve1.local:8006/api2/json"
    pub node_name: String,
    pub token_id: Option<String>,
    pub token_secret: Option<String>,
}

impl ProxmoxDriver {
    pub fn new(id: String, name: String, endpoint: String) -> Self {
        Self {
            hypervisor_id: id,
            hypervisor_name: name,
            endpoint,
            node_name: "pve-node1".to_string(),
            token_id: None,
            token_secret: None,
        }
    }
}

#[async_trait]
impl HypervisorDriver for ProxmoxDriver {
    async fn ping(&self) -> Result<bool> {
        info!("Pinging Proxmox VE node at {}", self.endpoint);
        Ok(true)
    }

    async fn get_stats(&self) -> Result<HypervisorStats> {
        Ok(HypervisorStats {
            cpu_cores: 16,
            cpu_usage_pct: 18.4,
            memory_total_mb: 32768,
            memory_used_mb: 12288,
            disk_total_gb: 1000,
            disk_used_gb: 340,
            active_vms: 3,
            total_vms: 5,
            uptime_seconds: 3600 * 24 * 7,
        })
    }

    async fn list_vms(&self) -> Result<Vec<VirtualMachine>> {
        Ok(vec![
            VirtualMachine {
                id: "pve-100".to_string(),
                name: "pve-web-cluster-01".to_string(),
                hypervisor_id: self.hypervisor_id.clone(),
                hypervisor_name: self.hypervisor_name.clone(),
                state: VmState::Running,
                vcpus: 4,
                memory_mb: 4096,
                disks: vec![VmDisk {
                    id: "pve-100-disk0".to_string(),
                    name: "scsi0".to_string(),
                    path: "local-lvm:vm-100-disk-0".to_string(),
                    size_gb: 32,
                    format: "raw".to_string(),
                    pool_name: "local-lvm".to_string(),
                }],
                assigned_to: None,
                vnc_port: Some(5902),
                vnc_ws_path: Some("/api/vnc/ws/pve-100".to_string()),
                ip_address: Some("192.168.1.150".to_string()),
                os_type: "Ubuntu 24.04 LTS".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            VirtualMachine {
                id: "pve-101".to_string(),
                name: "pve-database-pg".to_string(),
                hypervisor_id: self.hypervisor_id.clone(),
                hypervisor_name: self.hypervisor_name.clone(),
                state: VmState::Running,
                vcpus: 4,
                memory_mb: 8192,
                disks: vec![VmDisk {
                    id: "pve-101-disk0".to_string(),
                    name: "scsi0".to_string(),
                    path: "local-lvm:vm-101-disk-0".to_string(),
                    size_gb: 64,
                    format: "raw".to_string(),
                    pool_name: "local-lvm".to_string(),
                }],
                assigned_to: None,
                vnc_port: Some(5903),
                vnc_ws_path: Some("/api/vnc/ws/pve-101".to_string()),
                ip_address: Some("192.168.1.151".to_string()),
                os_type: "Debian 12 Bookworm".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        ])
    }

    async fn get_vm(&self, vm_id: &str) -> Result<VirtualMachine> {
        let vms = self.list_vms().await?;
        if let Some(vm) = vms.into_iter().find(|v| v.id == vm_id) {
            Ok(vm)
        } else {
            anyhow::bail!("VM {} not found on Proxmox node", vm_id)
        }
    }

    async fn create_vm(&self, req: &CreateVmRequest) -> Result<VirtualMachine> {
        info!("Creating VM {} on Proxmox VE", req.name);
        Ok(VirtualMachine {
            id: format!("pve-{}", uuid::Uuid::new_v4().simple()),
            name: req.name.clone(),
            hypervisor_id: self.hypervisor_id.clone(),
            hypervisor_name: self.hypervisor_name.clone(),
            state: VmState::Running,
            vcpus: req.vcpus,
            memory_mb: req.memory_mb,
            disks: vec![VmDisk {
                id: format!("{}-disk0", req.name),
                name: "scsi0".to_string(),
                path: format!("local-lvm:vm-{}-disk-0", req.name),
                size_gb: req.disk_size_gb,
                format: "raw".to_string(),
                pool_name: "local-lvm".to_string(),
            }],
            assigned_to: req.assigned_to.clone(),
            vnc_port: Some(5905),
            vnc_ws_path: Some(format!("/api/vnc/ws/{}", req.name)),
            ip_address: Some("192.168.1.155".to_string()),
            os_type: "Debian 12".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn start_vm(&self, vm_id: &str) -> Result<()> {
        info!("Starting Proxmox VM {}", vm_id);
        Ok(())
    }

    async fn stop_vm(&self, vm_id: &str, _force: bool) -> Result<()> {
        info!("Stopping Proxmox VM {}", vm_id);
        Ok(())
    }

    async fn reboot_vm(&self, vm_id: &str) -> Result<()> {
        info!("Rebooting Proxmox VM {}", vm_id);
        Ok(())
    }

    async fn delete_vm(&self, vm_id: &str) -> Result<()> {
        info!("Deleting Proxmox VM {}", vm_id);
        Ok(())
    }

    async fn clone_vm(&self, source_id: &str, new_name: &str) -> Result<VirtualMachine> {
        info!("Cloning Proxmox VM {} to {}", source_id, new_name);
        Ok(VirtualMachine {
            id: format!("pve-{}", uuid::Uuid::new_v4().simple()),
            name: new_name.to_string(),
            hypervisor_id: self.hypervisor_id.clone(),
            hypervisor_name: self.hypervisor_name.clone(),
            state: VmState::Stopped,
            vcpus: 2,
            memory_mb: 4096,
            disks: vec![],
            assigned_to: None,
            vnc_port: Some(5906),
            vnc_ws_path: Some(format!("/api/vnc/ws/{}", new_name)),
            ip_address: None,
            os_type: "Debian 12".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn get_vnc_info(&self, vm_id: &str) -> Result<VncInfo> {
        Ok(VncInfo {
            host: "127.0.0.1".to_string(),
            port: 5902,
            path: format!("/api/vnc/ws/{}", vm_id),
        })
    }

    async fn list_storage_pools(&self) -> Result<Vec<StoragePool>> {
        Ok(vec![
            StoragePool {
                name: "local".to_string(),
                path: "/var/lib/vz".to_string(),
                capacity_gb: 100,
                allocated_gb: 25,
                available_gb: 75,
                pool_type: "dir".to_string(),
            },
            StoragePool {
                name: "local-lvm".to_string(),
                path: "/dev/pve/data".to_string(),
                capacity_gb: 900,
                allocated_gb: 315,
                available_gb: 585,
                pool_type: "lvmthin".to_string(),
            },
        ])
    }

    async fn create_disk(&self, pool_name: &str, disk_name: &str, size_gb: u64, format: &str) -> Result<VmDisk> {
        Ok(VmDisk {
            id: format!("{}-{}", pool_name, disk_name),
            name: disk_name.to_string(),
            path: format!("{}:{}.{}", pool_name, disk_name, format),
            size_gb,
            format: format.to_string(),
            pool_name: pool_name.to_string(),
        })
    }

    async fn resize_disk(&self, _disk_path: &str, _new_size_gb: u64) -> Result<()> {
        Ok(())
    }

    async fn delete_disk(&self, _disk_path: &str) -> Result<()> {
        Ok(())
    }

    async fn attach_disk(&self, _vm_id: &str, _disk_path: &str, _target_dev: &str) -> Result<()> {
        Ok(())
    }

    async fn detach_disk(&self, _vm_id: &str, _target_dev: &str) -> Result<()> {
        Ok(())
    }
}

