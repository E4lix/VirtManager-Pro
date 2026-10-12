use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

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

        state.init_defaults().await;
        state.sync_with_kvm().await;

        state
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

            // Register Proxmox VE remote node for cluster demo
            let remote_pve = Hypervisor {
                id: "remote-pve-cluster".to_string(),
                name: "Proxmox-VE-Cluster (Datacenter)".to_string(),
                hypervisor_type: HypervisorType::Proxmox,
                endpoint: "https://pve.virtu.lab:8006/api2/json".to_string(),
                status: HypervisorStatus::Online,
                is_local: false,
                stats: crate::models::HypervisorStats {
                    cpu_cores: 16,
                    cpu_usage_pct: 22.0,
                    memory_total_mb: 32768,
                    memory_used_mb: 16384,
                    disk_total_gb: 1000,
                    disk_used_gb: 420,
                    active_vms: 3,
                    total_vms: 4,
                    uptime_seconds: 86400 * 12,
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
        info!("Synchronizing state with local KVM hypervisor...");
        let local_driver = Arc::new(KvmDriver::new(
            "local-kvm".to_string(),
            "KVM-Host-Local (Debian 13)".to_string(),
            "qemu:///system".to_string(),
        ));

        // Register driver
        {
            let mut drivers = self.drivers.write().await;
            drivers.insert("local-kvm".to_string(), local_driver.clone());

            let pve_driver = Arc::new(ProxmoxDriver::new(
                "remote-pve-cluster".to_string(),
                "Proxmox-VE-Cluster (Datacenter)".to_string(),
                "https://pve.virtu.lab:8006/api2/json".to_string(),
            ));
            drivers.insert("remote-pve-cluster".to_string(), pve_driver);
        }

        // Query real VMs from libvirt
        if let Ok(vms) = local_driver.list_vms().await {
            let mut store = self.store.write().await;
            for mut vm in vms {
                // If it's debian13-clone, assign to user1 as example
                if vm.name == "debian13-clone" {
                    vm.assigned_to = Some("user1".to_string());
                }
                store.vms.insert(vm.id.clone(), vm);
            }
            info!("Imported {} VMs from local KVM", store.vms.len());
        }
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
