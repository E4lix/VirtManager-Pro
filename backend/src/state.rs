use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::drivers::kvm::KvmDriver;
use crate::drivers::proxmox::ProxmoxDriver;
use crate::drivers::HypervisorDriver;
use crate::models::{
    BackgroundTask, Claims, Hypervisor, HypervisorStatus, HypervisorType, Template,
    TemplateStatus, User, UserRole, VirtualMachine,
};

const JWT_SECRET: &[u8] = b"virmox-super-secret-5a-virtu-key-2026";

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct DataStore {
    pub users: HashMap<String, User>,
    pub hypervisors: HashMap<String, Hypervisor>,
    pub vms: HashMap<String, VirtualMachine>,
    pub templates: HashMap<String, Template>,
    pub tasks: HashMap<String, BackgroundTask>,
}

pub struct AppState {
    pub store: Arc<RwLock<DataStore>>,
    pub drivers: Arc<RwLock<HashMap<String, Arc<dyn HypervisorDriver>>>>,
    pub db_path: PathBuf,
}

impl AppState {
    pub async fn new(db_path: PathBuf) -> Self {
        let store = Arc::new(RwLock::new(DataStore::default()));
        let drivers = Arc::new(RwLock::new(HashMap::new()));

        let state = Self {
            store,
            drivers,
            db_path,
        };

        let loaded = state.load_from_disk().await.unwrap_or(false);
        if !loaded {
            state.init_defaults().await;
        }
        state.sync_with_kvm().await;
        let _ = state.save_to_disk().await;

        state
    }

    pub async fn save_to_disk(&self) -> Result<()> {
        if let Some(parent) = self.db_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        let store = self.store.read().await;
        let json = serde_json::to_string_pretty(&*store)?;
        tokio::fs::write(&self.db_path, json).await?;
        Ok(())
    }

    pub async fn load_from_disk(&self) -> Result<bool> {
        if self.db_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&self.db_path).await {
                if let Ok(loaded_store) = serde_json::from_str::<DataStore>(&content) {
                    let mut store = self.store.write().await;
                    *store = loaded_store;
                    info!("Loaded data store from {:?}", self.db_path);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub async fn get_driver(&self, hypervisor_id: &str) -> Option<Arc<dyn HypervisorDriver>> {
        let drivers = self.drivers.read().await;
        drivers.get(hypervisor_id).cloned()
    }

    pub async fn register_driver(&self, hypervisor: &Hypervisor) {
        let driver: Arc<dyn HypervisorDriver> = match hypervisor.hypervisor_type {
            HypervisorType::Kvm | HypervisorType::RemoteKvm => Arc::new(KvmDriver::new(
                hypervisor.id.clone(),
                hypervisor.name.clone(),
                hypervisor.endpoint.clone(),
            )),
            HypervisorType::Proxmox => Arc::new(ProxmoxDriver::new(
                hypervisor.id.clone(),
                hypervisor.name.clone(),
                hypervisor.endpoint.clone(),
            )),
        };

        let mut drivers = self.drivers.write().await;
        drivers.insert(hypervisor.id.clone(), driver);
    }

    pub async fn init_defaults(&self) {
        let mut store = self.store.write().await;

        // 1. Initialize Users (admin / user)
        if store.users.is_empty() {
            store.users.insert(
                "admin".to_string(),
                User {
                    id: "u-admin".to_string(),
                    username: "admin".to_string(),
                    password_hash: "admin".to_string(),
                    role: UserRole::Admin,
                    full_name: "Administrateur Système".to_string(),
                    email: "admin@virmox.local".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
            );

            store.users.insert(
                "user1".to_string(),
                User {
                    id: "u-user1".to_string(),
                    username: "user1".to_string(),
                    password_hash: "user1".to_string(),
                    role: UserRole::User,
                    full_name: "Utilisateur Étudiant 5A".to_string(),
                    email: "etudiant@virmox.local".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
            );
        }

        // 2. Initialize Local KVM Hypervisor
        if store.hypervisors.is_empty() {
            let local_hv = Hypervisor {
                id: "local-kvm".to_string(),
                name: "KVM-Host-Local (Debian 13)".to_string(),
                hypervisor_type: HypervisorType::Kvm,
                endpoint: "qemu:///system".to_string(),
                status: HypervisorStatus::Online,
                is_local: true,
                stats: crate::models::HypervisorStats {
                    cpu_cores: 20,
                    cpu_usage_pct: 14.2,
                    memory_total_mb: 31524,
                    memory_used_mb: 14200,
                    disk_total_gb: 475,
                    disk_used_gb: 120,
                    active_vms: 2,
                    total_vms: 3,
                    uptime_seconds: 86400 * 3,
                },
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            store.hypervisors.insert(local_hv.id.clone(), local_hv);

            // Register Proxmox VE node for local cluster
            let remote_pve = Hypervisor {
                id: "remote-pve-cluster".to_string(),
                name: "Proxmox-VE-Local (pve)".to_string(),
                hypervisor_type: HypervisorType::Proxmox,
                endpoint: "https://192.168.122.62:8006/api2/json".to_string(),
                status: HypervisorStatus::Online,
                is_local: false,
                stats: crate::models::HypervisorStats {
                    cpu_cores: 16,
                    cpu_usage_pct: 5.0,
                    memory_total_mb: 24576,
                    memory_used_mb: 2048,
                    disk_total_gb: 150,
                    disk_used_gb: 16,
                    active_vms: 0,
                    total_vms: 6,
                    uptime_seconds: 3600,
                },
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            store.hypervisors.insert(remote_pve.id.clone(), remote_pve);
        }

        // 3. Initialize Templates (Packer / Ansible)
        if store.templates.is_empty() {
            store.templates.insert(
                "tpl-debian".to_string(),
                Template {
                    id: "tpl-debian".to_string(),
                    name: "Debian 12 Bookworm (Cloud-Init)".to_string(),
                    description: "Image Debian standardisée avec agent QEMU, Cloud-Init et outils de base".to_string(),
                    os: "Debian".to_string(),
                    version: "12.7".to_string(),
                    packer_file: "templates/packer/debian.pkr.hcl".to_string(),
                    ansible_playbook: "templates/ansible/playbooks/base.yml".to_string(),
                    default_vcpus: 2,
                    default_memory_mb: 2048,
                    default_disk_gb: 20,
                    status: TemplateStatus::Ready,
                    disk_image_path: Some("/var/lib/libvirt/images/debian13.qcow2".to_string()),
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
            );

            store.templates.insert(
                "tpl-nginx".to_string(),
                Template {
                    id: "tpl-nginx".to_string(),
                    name: "Serveur Web Nginx Haute Performance".to_string(),
                    description: "Template provisionné automatiquement par Ansible avec Nginx, pare-feu UFW et page d'accueil".to_string(),
                    os: "Debian / Nginx".to_string(),
                    version: "1.24".to_string(),
                    packer_file: "templates/packer/debian.pkr.hcl".to_string(),
                    ansible_playbook: "templates/ansible/playbooks/webserver.yml".to_string(),
                    default_vcpus: 2,
                    default_memory_mb: 4096,
                    default_disk_gb: 25,
                    status: TemplateStatus::Ready,
                    disk_image_path: Some("/var/lib/libvirt/images/debian13-clone.qcow2".to_string()),
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
            );

            store.templates.insert(
                "tpl-docker".to_string(),
                Template {
                    id: "tpl-docker".to_string(),
                    name: "Docker Engine & Compose Host".to_string(),
                    description: "Hôte de conteneurs prêt à l'emploi avec Docker CE, containerd et Docker Compose".to_string(),
                    os: "Ubuntu".to_string(),
                    version: "24.04 LTS".to_string(),
                    packer_file: "templates/packer/debian.pkr.hcl".to_string(),
                    ansible_playbook: "templates/ansible/playbooks/docker.yml".to_string(),
                    default_vcpus: 4,
                    default_memory_mb: 8192,
                    default_disk_gb: 40,
                    status: TemplateStatus::Ready,
                    disk_image_path: None,
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
            );
        }
    }

    pub async fn sync_with_kvm(&self) {
        info!("Synchronizing state with local KVM and Proxmox VE hypervisors...");
        let local_driver = Arc::new(KvmDriver::new(
            "local-kvm".to_string(),
            "KVM-Host-Local (Debian 13)".to_string(),
            "qemu:///system".to_string(),
        ));

        let pve_driver = Arc::new(ProxmoxDriver::new(
            "remote-pve-cluster".to_string(),
            "Proxmox-VE-Local (pve)".to_string(),
            "https://192.168.122.62:8006/api2/json".to_string(),
        ));

        // Register drivers
        {
            let mut drivers = self.drivers.write().await;
            drivers.insert("local-kvm".to_string(), local_driver.clone());
            drivers.insert("remote-pve-cluster".to_string(), pve_driver.clone());
        }

        // Ensure Hypervisor metadata is updated in store
        {
            let mut store = self.store.write().await;
            if let Some(hv) = store.hypervisors.get_mut("remote-pve-cluster") {
                hv.name = "Proxmox-VE-Local (pve)".to_string();
                hv.endpoint = "https://192.168.122.62:8006/api2/json".to_string();
                hv.status = HypervisorStatus::Online;
            } else {
                let remote_pve = Hypervisor {
                    id: "remote-pve-cluster".to_string(),
                    name: "Proxmox-VE-Local (pve)".to_string(),
                    hypervisor_type: HypervisorType::Proxmox,
                    endpoint: "https://192.168.122.62:8006/api2/json".to_string(),
                    status: HypervisorStatus::Online,
                    is_local: false,
                    stats: crate::models::HypervisorStats {
                        cpu_cores: 16,
                        cpu_usage_pct: 5.0,
                        memory_total_mb: 24576,
                        memory_used_mb: 2048,
                        disk_total_gb: 150,
                        disk_used_gb: 16,
                        active_vms: 0,
                        total_vms: 6,
                        uptime_seconds: 3600,
                    },
                    created_at: chrono::Utc::now().to_rfc3339(),
                };
                store.hypervisors.insert(remote_pve.id.clone(), remote_pve);
            }
        }

        // 1. Query real VMs from libvirt
        if let Ok(vms) = local_driver.list_vms().await {
            let active_kvm_ids: HashSet<String> = vms.iter().map(|v| v.id.clone()).collect();
            let mut store = self.store.write().await;
            if let Some(hv) = store.hypervisors.get_mut("local-kvm") {
                hv.status = HypervisorStatus::Online;
            }
            store.vms.retain(|id, vm| vm.hypervisor_id != "local-kvm" || active_kvm_ids.contains(id));
            for mut vm in vms {
                if let Some(existing) = store.vms.get(&vm.id) {
                    if existing.assigned_to.is_some() {
                        vm.assigned_to = existing.assigned_to.clone();
                    }
                } else if vm.name == "debian13-clone" {
                    vm.assigned_to = Some("user1".to_string());
                }
                store.vms.insert(vm.id.clone(), vm);
            }
            info!("Imported {} VMs from local KVM", store.vms.len());
        }

        // 2. Query real VMs from Proxmox VE
        match pve_driver.list_vms().await {
            Ok(pve_vms) => {
                let active_pve_ids: HashSet<String> = pve_vms.iter().map(|v| v.id.clone()).collect();
                let mut store = self.store.write().await;
                if let Some(hv) = store.hypervisors.get_mut("remote-pve-cluster") {
                    hv.status = HypervisorStatus::Online;
                }
                store.vms.retain(|id, vm| vm.hypervisor_id != "remote-pve-cluster" || active_pve_ids.contains(id));
                let count = pve_vms.len();
                for mut vm in pve_vms {
                    if let Some(existing) = store.vms.get(&vm.id) {
                        if existing.assigned_to.is_some() {
                            vm.assigned_to = existing.assigned_to.clone();
                        }
                    }
                    store.vms.insert(vm.id.clone(), vm);
                }
                info!("Imported {} VMs from local Proxmox VE node", count);
            }
            Err(e) => {
                let mut store = self.store.write().await;
                if let Some(hv) = store.hypervisors.get_mut("remote-pve-cluster") {
                    hv.status = HypervisorStatus::Offline;
                }
                warn!("Could not list VMs from Proxmox VE node: {}", e);
            }
        }

        // 3. Update stats directly from drivers
        if let Ok(stats) = local_driver.get_stats().await {
            let mut store = self.store.write().await;
            if let Some(hv) = store.hypervisors.get_mut("local-kvm") {
                hv.stats = stats;
            }
        }
        if let Ok(stats) = pve_driver.get_stats().await {
            let mut store = self.store.write().await;
            if let Some(hv) = store.hypervisors.get_mut("remote-pve-cluster") {
                hv.stats = stats;
            }
        }
    }

    pub fn start_periodic_sync(self: &Arc<Self>) {
        let state = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            interval.tick().await;
            loop {
                interval.tick().await;
                state.sync_with_kvm().await;
            }
        });
    }

    pub fn generate_token(&self, user: &User) -> Result<String> {
        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        )
        .map_err(|e| anyhow::anyhow!("JWT error: {}", e))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::default(),
        )?;
        Ok(decoded.claims)
    }
}
