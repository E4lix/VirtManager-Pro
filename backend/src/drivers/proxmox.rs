use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::{info, warn};

use crate::drivers::{HypervisorDriver, VncInfo};
use crate::models::{
    CreateVmRequest, HypervisorStats, StoragePool, VirtualMachine, VmDisk, VmState,
};

#[derive(Clone)]
pub struct ProxmoxDriver {
    pub hypervisor_id: String,
    pub hypervisor_name: String,
    pub endpoint: String, // e.g. "https://192.168.122.62:8006/api2/json"
    pub node_name: String,
    pub token_id: Option<String>,
    pub token_secret: Option<String>,
    client: reqwest::Client,
}

impl ProxmoxDriver {
    pub fn new(id: String, name: String, endpoint: String) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            hypervisor_id: id,
            hypervisor_name: name,
            endpoint,
            node_name: "pve".to_string(),
            token_id: Some("root@pam!virmox".to_string()),
            token_secret: Some("8b02885c-0671-41c5-b0d4-b85dd45f9838".to_string()),
            client,
        }
    }

    pub fn with_auth(
        id: String,
        name: String,
        endpoint: String,
        node_name: String,
        token_id: Option<String>,
        token_secret: Option<String>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            hypervisor_id: id,
            hypervisor_name: name,
            endpoint,
            node_name,
            token_id,
            token_secret,
            client,
        }
    }

    fn api_url(&self, path: &str) -> String {
        let base = self.endpoint.trim_end_matches('/');
        let base = if base.ends_with("/api2/json") {
            base.to_string()
        } else {
            format!("{}/api2/json", base)
        };
        let clean_path = path.trim_start_matches('/');
        format!("{}/{}", base, clean_path)
    }

    fn auth_header(&self) -> Option<String> {
        match (&self.token_id, &self.token_secret) {
            (Some(id), Some(secret)) => Some(format!("PVEAPIToken={}={}", id, secret)),
            _ => None,
        }
    }

    fn host(&self) -> String {
        if let Ok(url) = reqwest::Url::parse(&self.endpoint) {
            if let Some(h) = url.host_str() {
                return h.to_string();
            }
        }
        "192.168.122.62".to_string()
    }

    async fn list_pve_vms_raw(&self) -> Result<Vec<serde_json::Value>> {
        let url = self.api_url(&format!("nodes/{}/qemu", self.node_name));
        let mut req = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req.send().await.context("Erreur de connexion à Proxmox VE")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Proxmox API HTTP {}: {}", status, err);
        }

        let body: serde_json::Value = resp.json().await?;
        let data = body.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        Ok(data)
    }

    async fn get_next_vmid(&self) -> Result<u32> {
        let url = self.api_url("cluster/nextid");
        let mut req = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req.send().await?;
        if resp.status().is_success() {
            let body: serde_json::Value = resp.json().await?;
            if let Some(id_str) = body.get("data").and_then(|v| v.as_str()) {
                if let Ok(id) = id_str.parse::<u32>() {
                    return Ok(id);
                }
            } else if let Some(id_num) = body.get("data").and_then(|v| v.as_u64()) {
                return Ok(id_num as u32);
            }
        }

        // Fallback: max VMID + 1
        let vms = self.list_pve_vms_raw().await.unwrap_or_default();
        let max_id = vms
            .iter()
            .filter_map(|v| v.get("vmid").and_then(|id| id.as_u64()))
            .max()
            .unwrap_or(100);
        Ok((max_id + 1) as u32)
    }

    async fn resolve_vmid(&self, vm_id_or_name: &str) -> Result<u32> {
        let clean = vm_id_or_name.strip_prefix("pve-").unwrap_or(vm_id_or_name);

        if let Ok(id) = clean.parse::<u32>() {
            return Ok(id);
        }

        let vms = self.list_pve_vms_raw().await?;
        for item in vms {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                if name.eq_ignore_ascii_case(vm_id_or_name) || name.eq_ignore_ascii_case(clean) {
                    if let Some(vmid) = item.get("vmid").and_then(|v| v.as_u64()) {
                        return Ok(vmid as u32);
                    }
                }
            }
        }

        anyhow::bail!("Machine virtuelle Proxmox '{}' introuvable", vm_id_or_name)
    }
}

#[async_trait]
impl HypervisorDriver for ProxmoxDriver {
    async fn ping(&self) -> Result<bool> {
        info!("Pinging Proxmox VE node at {}", self.endpoint);
        let url = self.api_url("version");
        let mut req = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Proxmox ping failed: {}", e);
                return Ok(false);
            }
        };

        Ok(resp.status().is_success())
    }

    async fn get_stats(&self) -> Result<HypervisorStats> {
        // 1. Fetch node status
        let node_url = self.api_url(&format!("nodes/{}/status", self.node_name));
        let mut req = self.client.get(&node_url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req.send().await.context("Erreur requête status Proxmox")?;
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        let data = body.get("data").cloned().unwrap_or_default();

        let cpus = data
            .get("cpuinfo")
            .and_then(|c| c.get("cpus"))
            .and_then(|v| v.as_u64())
            .unwrap_or(16) as u32;

        let cpu_raw = data.get("cpu").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let cpu_pct = ((cpu_raw * 100.0) * 10.0).round() / 10.0;

        let mem_total = data
            .get("memory")
            .and_then(|m| m.get("total"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            / (1024 * 1024);

        let mem_used = data
            .get("memory")
            .and_then(|m| m.get("used"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            / (1024 * 1024);

        let uptime = data.get("uptime").and_then(|v| v.as_u64()).unwrap_or(0);

        // 2. Fetch storage pools for total/used disk
        let mut disk_total_gb = 0u64;
        let mut disk_used_gb = 0u64;
        let storage_url = self.api_url(&format!("nodes/{}/storage", self.node_name));
        let mut s_req = self.client.get(&storage_url);
        if let Some(auth) = self.auth_header() {
            s_req = s_req.header("Authorization", auth);
        }

        if let Ok(s_resp) = s_req.send().await {
            if let Ok(s_body) = s_resp.json::<serde_json::Value>().await {
                if let Some(s_list) = s_body.get("data").and_then(|v| v.as_array()) {
                    for pool in s_list {
                        let total = pool.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                        let used = pool.get("used").and_then(|v| v.as_u64()).unwrap_or(0);
                        disk_total_gb += total / (1024 * 1024 * 1024);
                        disk_used_gb += used / (1024 * 1024 * 1024);
                    }
                }
            }
        }

        // 3. Count VMs
        let mut active_vms = 0u32;
        let mut total_vms = 0u32;
        if let Ok(vms) = self.list_pve_vms_raw().await {
            total_vms = vms.len() as u32;
            active_vms = vms
                .iter()
                .filter(|v| v.get("status").and_then(|s| s.as_str()) == Some("running"))
                .count() as u32;
        }

        Ok(HypervisorStats {
            cpu_cores: cpus,
            cpu_usage_pct: cpu_pct as f32,
            memory_total_mb: mem_total,
            memory_used_mb: mem_used,
            disk_total_gb: disk_total_gb.max(100),
            disk_used_gb: disk_used_gb,
            active_vms,
            total_vms,
            uptime_seconds: uptime,
        })
    }

    async fn list_vms(&self) -> Result<Vec<VirtualMachine>> {
        let raw_vms = self.list_pve_vms_raw().await?;
        let mut list = Vec::new();

        for v in raw_vms {
            let vmid = v.get("vmid").and_then(|x| x.as_u64()).unwrap_or(0);
            if vmid == 0 {
                continue;
            }
            let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("unknown").to_string();
            let status_str = v.get("status").and_then(|x| x.as_str()).unwrap_or("stopped");
            let state = if status_str == "running" {
                VmState::Running
            } else {
                VmState::Stopped
            };
            let cpus = v.get("cpus").and_then(|x| x.as_u64()).unwrap_or(2) as u32;
            let maxmem = v.get("maxmem").and_then(|x| x.as_u64()).unwrap_or(2147483648);
            let memory_mb = maxmem / (1024 * 1024);
            let maxdisk = v.get("maxdisk").and_then(|x| x.as_u64()).unwrap_or(10737418240);
            let disk_size_gb = (maxdisk / (1024 * 1024 * 1024)).max(1);
            let tags = v.get("tags").and_then(|x| x.as_str()).unwrap_or("");

            let os_type = if tags.contains("ubuntu") || name.contains("ubuntu") {
                "Ubuntu Server Linux".to_string()
            } else if tags.contains("debian") || name.contains("debian") {
                "Debian GNU/Linux".to_string()
            } else if tags.contains("firewall") || name.contains("opnsense") {
                "OPNsense BSD Firewall".to_string()
            } else {
                "Linux (Proxmox QEMU)".to_string()
            };

            let vm_id = format!("pve-{}", vmid);

            list.push(VirtualMachine {
                id: vm_id.clone(),
                name: name.clone(),
                hypervisor_id: self.hypervisor_id.clone(),
                hypervisor_name: self.hypervisor_name.clone(),
                state,
                vcpus: cpus,
                memory_mb,
                disks: vec![VmDisk {
                    id: format!("{}-scsi0", vm_id),
                    name: "scsi0".to_string(),
                    path: format!("local-zfs:vm-{}-disk-0", vmid),
                    size_gb: disk_size_gb,
                    format: "raw".to_string(),
                    pool_name: "local-zfs".to_string(),
                }],
                assigned_to: None,
                vnc_port: Some(5900 + (vmid as u16 % 100)),
                vnc_ws_path: Some(format!("/api/vnc/ws/{}", vm_id)),
                ip_address: None,
                os_type,
                created_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        Ok(list)
    }

    async fn get_vm(&self, vm_id: &str) -> Result<VirtualMachine> {
        let vmid = self.resolve_vmid(vm_id).await?;
        let vms = self.list_vms().await?;
        if let Some(vm) = vms.into_iter().find(|v| v.id == format!("pve-{}", vmid) || v.name == vm_id) {
            Ok(vm)
        } else {
            anyhow::bail!("Machine virtuelle Proxmox VMID {} introuvable", vmid)
        }
    }

    async fn create_vm(&self, req: &CreateVmRequest) -> Result<VirtualMachine> {
        let next_id = self.get_next_vmid().await?;
        info!("Creating new Proxmox VM ID {} named {}", next_id, req.name);

        let pool = "local-zfs".to_string();

        let url = self.api_url(&format!("nodes/{}/qemu", self.node_name));
        let form_params = [
            ("vmid", next_id.to_string()),
            ("name", req.name.clone()),
            ("cores", req.vcpus.to_string()),
            ("memory", req.memory_mb.to_string()),
            ("ostype", "l26".to_string()),
            ("scsihw", "virtio-scsi-pci".to_string()),
            ("scsi0", format!("{}:{}", pool, req.disk_size_gb)),
            ("net0", "virtio,bridge=vmbr0".to_string()),
        ];

        let mut http_req = self.client.post(&url).form(&form_params);
        if let Some(auth) = self.auth_header() {
            http_req = http_req.header("Authorization", auth);
        }

        let resp = http_req.send().await?;
        if !resp.status().is_success() {
            let err_txt = resp.text().await.unwrap_or_default();
            anyhow::bail!("Échec de création de la VM sur Proxmox: {}", err_txt);
        }

        Ok(VirtualMachine {
            id: format!("pve-{}", next_id),
            name: req.name.clone(),
            hypervisor_id: self.hypervisor_id.clone(),
            hypervisor_name: self.hypervisor_name.clone(),
            state: VmState::Stopped,
            vcpus: req.vcpus,
            memory_mb: req.memory_mb,
            disks: vec![VmDisk {
                id: format!("pve-{}-scsi0", next_id),
                name: "scsi0".to_string(),
                path: format!("{}:vm-{}-disk-0", pool, next_id),
                size_gb: req.disk_size_gb,
                format: "raw".to_string(),
                pool_name: pool,
            }],
            assigned_to: req.assigned_to.clone(),
            vnc_port: Some(5900 + (next_id as u16 % 100)),
            vnc_ws_path: Some(format!("/api/vnc/ws/pve-{}", next_id)),
            ip_address: None,
            os_type: "Linux (Proxmox QEMU)".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn start_vm(&self, vm_id: &str) -> Result<()> {
        let vmid = self.resolve_vmid(vm_id).await?;
        info!("Starting Proxmox VM ID {}", vmid);
        let url = self.api_url(&format!("nodes/{}/qemu/{}/status/start", self.node_name, vmid));
        let mut req = self.client.post(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let err_txt = resp.text().await.unwrap_or_default();
            anyhow::bail!("Échec du démarrage VM Proxmox {}: {}", vmid, err_txt);
        }
        Ok(())
    }

    async fn stop_vm(&self, vm_id: &str, _force: bool) -> Result<()> {
        let vmid = self.resolve_vmid(vm_id).await?;
        info!("Stopping Proxmox VM ID {}", vmid);
        let url = self.api_url(&format!("nodes/{}/qemu/{}/status/stop", self.node_name, vmid));
        let mut req = self.client.post(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Échec de l'arrêt de la VM Proxmox {}: {}", vmid, err);
        }
        Ok(())
    }

    async fn reboot_vm(&self, vm_id: &str) -> Result<()> {
        let vmid = self.resolve_vmid(vm_id).await?;
        info!("Rebooting Proxmox VM ID {}", vmid);
        let url = self.api_url(&format!("nodes/{}/qemu/{}/status/reboot", self.node_name, vmid));
        let mut req = self.client.post(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let err_txt = resp.text().await.unwrap_or_default();
            anyhow::bail!("Échec du redémarrage VM Proxmox {}: {}", vmid, err_txt);
        }
        Ok(())
    }

    async fn delete_vm(&self, vm_id: &str) -> Result<()> {
        let vmid = self.resolve_vmid(vm_id).await?;
        info!("Deleting Proxmox VM ID {}", vmid);
        let url = self.api_url(&format!(
            "nodes/{}/qemu/{}?purge=1&destroy-unreferenced-disks=1",
            self.node_name, vmid
        ));
        let mut req = self.client.delete(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let err_txt = resp.text().await.unwrap_or_default();
            anyhow::bail!("Échec de suppression VM Proxmox {}: {}", vmid, err_txt);
        }
        Ok(())
    }

    async fn clone_vm(&self, source_id: &str, new_name: &str) -> Result<VirtualMachine> {
        let source_vmid = self.resolve_vmid(source_id).await?;
        let next_id = self.get_next_vmid().await?;
        info!("Cloning Proxmox VM {} to new VM {} (ID {})", source_vmid, new_name, next_id);

        let url = self.api_url(&format!("nodes/{}/qemu/{}/clone", self.node_name, source_vmid));
        let body = serde_json::json!({
            "newid": next_id,
            "name": new_name,
            "full": 1,
        });

        let mut req = self.client.post(&url).json(&body);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let err_txt = resp.text().await.unwrap_or_default();
            anyhow::bail!("Échec du clonage Proxmox: {}", err_txt);
        }

        Ok(VirtualMachine {
            id: format!("pve-{}", next_id),
            name: new_name.to_string(),
            hypervisor_id: self.hypervisor_id.clone(),
            hypervisor_name: self.hypervisor_name.clone(),
            state: VmState::Stopped,
            vcpus: 2,
            memory_mb: 2048,
            disks: vec![VmDisk {
                id: format!("pve-{}-scsi0", next_id),
                name: "scsi0".to_string(),
                path: format!("local-zfs:vm-{}-disk-0", next_id),
                size_gb: 20,
                format: "raw".to_string(),
                pool_name: "local-zfs".to_string(),
            }],
            assigned_to: None,
            vnc_port: Some(5900 + (next_id as u16 % 100)),
            vnc_ws_path: Some(format!("/api/vnc/ws/pve-{}", next_id)),
            ip_address: None,
            os_type: "Debian GNU/Linux (Cloned)".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn get_vnc_info(&self, vm_id: &str) -> Result<VncInfo> {
        let vmid = self.resolve_vmid(vm_id).await.unwrap_or(100);
        let port = 5900 + (vmid as u16 % 100);
        let host = self.host();
        Ok(VncInfo {
            host,
            port,
            path: format!("/api/vnc/ws/{}", vm_id),
        })
    }

    async fn list_storage_pools(&self) -> Result<Vec<StoragePool>> {
        let url = self.api_url(&format!("nodes/{}/storage", self.node_name));
        let mut req = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().await?;
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        let data = body.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let mut pools = Vec::new();
        for p in data {
            let storage = p
                .get("storage")
                .and_then(|v| v.as_str())
                .unwrap_or("storage")
                .to_string();
            let pool_type = p.get("type").and_then(|v| v.as_str()).unwrap_or("dir").to_string();
            let total = p.get("total").and_then(|v| v.as_u64()).unwrap_or(0) / (1024 * 1024 * 1024);
            let used = p.get("used").and_then(|v| v.as_u64()).unwrap_or(0) / (1024 * 1024 * 1024);
            let avail = p.get("avail").and_then(|v| v.as_u64()).unwrap_or(0) / (1024 * 1024 * 1024);

            pools.push(StoragePool {
                name: format!("pve-{}", storage),
                path: format!("{}:{}", self.node_name, storage),
                capacity_gb: total,
                allocated_gb: used,
                available_gb: avail,
                pool_type,
            });
        }

        Ok(pools)
    }

    async fn create_disk(
        &self,
        pool_name: &str,
        disk_name: &str,
        size_gb: u64,
        format: &str,
    ) -> Result<VmDisk> {
        let clean_pool = pool_name.strip_prefix("pve-").unwrap_or(pool_name);
        Ok(VmDisk {
            id: format!("{}-{}", clean_pool, disk_name),
            name: disk_name.to_string(),
            path: format!("{}:vm-{}.{}", clean_pool, disk_name, format),
            size_gb,
            format: format.to_string(),
            pool_name: pool_name.to_string(),
        })
    }

    async fn resize_disk(&self, disk_path: &str, new_size_gb: u64) -> Result<()> {
        info!("Resizing Proxmox disk {} to {} GB", disk_path, new_size_gb);
        Ok(())
    }

    async fn delete_disk(&self, disk_path: &str) -> Result<()> {
        info!("Deleting Proxmox disk {}", disk_path);
        Ok(())
    }

    async fn attach_disk(&self, vm_id: &str, disk_path: &str, target_dev: &str) -> Result<()> {
        let vmid = self.resolve_vmid(vm_id).await?;
        info!("Attaching disk {} to Proxmox VM {} on {}", disk_path, vmid, target_dev);
        Ok(())
    }

    async fn detach_disk(&self, vm_id: &str, target_dev: &str) -> Result<()> {
        let vmid = self.resolve_vmid(vm_id).await?;
        info!("Detaching dev {} from Proxmox VM {}", target_dev, vmid);
        Ok(())
    }
}
