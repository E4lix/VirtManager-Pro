# 🚀 VirtManager Pro (Proxmox-like Cloud Management Platform)

> **Projet de Virtualisation - 5ème Année d'Ingénieur / Master 2**  
> Plateforme unifiée de gestion d'infrastructure virtualisée, orchestrateur de machines virtuelles et gestionnaire de templates automatisés.

---

## 🌟 Présentation du Projet

**VirtManager Pro** est une application web moderne (inspirée de l'ergonomie et de la robustesse de **Proxmox VE**) permettant de centraliser et d'automatiser la gestion d'un parc de virtualisation hétérogène (KVM/libvirt local, hyperviseurs distants par SSH, clusters Proxmox VE).

### 🛠️ Stack Technique Imposée & Choix Technologiques
- **Backend** : **Rust** (Framework web **Axum**, runtime asynchrone **Tokio**, abstractions de pilotes par `Trait`, sérialisation `serde`, documentation `utoipa`/Swagger).
  - *Justification* : Haute performance, sécurité mémoire garantie, exécution non-bloquante des opérations lourdes (création de disques, clonage, build Packer) et gestion efficace des sockets VNC.
- **Frontend** : **Node.js** avec **React 18** et **Vite**.
  - *Justification* : Interface réactive, tableau de bord Datacenter avec graphiques et jauges, intégration de **noVNC** (RFB natif sur WebSockets) sans plugin.
- **Automatisation & Modèles** : **HashiCorp Packer** (génération d'images dorées) et **Ansible** (transformation et durcissement des systèmes d'exploitation).

---

## 📋 Périmètre Fonctionnel (Modules Réalisés)

### 🖥️ Module A : Gestionnaire de Machines Virtuelles (VM)
- **Cycle de vie complet** : Création, Démarrage, Arrêt propre ACPI (`virsh shutdown`), Arrêt forcé, Redémarrage, Suppression.
- **Clonage instantané** : Clonage de VM existantes via `virt-clone`.
- **Attribution & Sécurité RBAC** : Affectation de machines à un utilisateur (`user1`), filtrage des vues selon les droits.
- **Console graphique VNC intégrée** : Client **noVNC** natif embarqué dans l'application web avec contrôles clavier/souris, Ctrl+Alt+Suppr et mode plein écran.
- **Gestion du stockage** : Découverte et gestion des disques (`.qcow2`, `.raw`), création de volumes virtuels et extension dynamique de taille (`qemu-img resize`).

### 🖧 Module B : Gestionnaire d'Hyperviseurs
- **Pool de ressources extensible** : Ajout et enregistrement d'hyperviseurs locaux (`qemu:///system`) et distants (via SSH `qemu+ssh://...` ou API REST Proxmox VE).
- **Architecture par Trait Rust** (`HypervisorDriver`) : Permet d'ajouter de futurs hyperviseurs (ex: VMware ESXi, AWS EC2) sans modifier le cœur du backend.
- **Télémétrie en temps réel** : Surveillance de l'état de santé, pourcentage CPU, mémoire vive totale et utilisée, stockage des pools.
- **Retrait sécurisé** : Blocage du retrait d'un hyperviseur si des machines virtuelles y sont en cours d'exécution.

### 📦 Module C : Gestionnaire de Templates (Modèles)
- **Modèles standardisés** : Debian 12 Bookworm Cloud-Init, Serveur Web Nginx, Hôte Docker Engine.
- **Automatisation Packer** : Spécifications HCL `.pkr.hcl` pour la construction automatisée d'images.
- **Configuration Ansible** : Playbooks de provisionnement automatique pour transformer les modèles.
- **Exécution asynchrone** : Les opérations longues sont déléguées à Tokio avec streaming des logs d'exécution dans le panneau des tâches.
- **Déploiement en 1 clic** : Instanciation immédiate d'une VM prête à l'emploi à partir d'un modèle.

---

## 🚀 Démarrage Rapide (En 1 Commande)

Lancez la plateforme directement à la racine du dépôt :
```bash
./scripts/start.sh
```

L'application est immédiatement accessible :
- 🌐 **Interface Web Principale** : [http://127.0.0.1:8080](http://127.0.0.1:8080)
- 📖 **Documentation Interactive Swagger UI** : [http://127.0.0.1:8080/swagger-ui/](http://127.0.0.1:8080/swagger-ui/)
- 🖥️ **Console noVNC Directe** : [http://127.0.0.1:8080/novnc/vnc.html](http://127.0.0.1:8080/novnc/vnc.html)

### 🔑 Comptes de Test Prédéfinis
- **Administrateur** : `admin` / `admin`
- **Utilisateur Étudiant** : `user1` / `user1`

---

## 📁 Structure du Projet

```text
projet-virtu/
├── backend/                  # Serveur Rust (Axum, Tokio, Libvirt/KVM, OpenAPI)
│   ├── Cargo.toml            # Dépendances du projet Rust
│   └── src/
│       ├── main.rs           # Point d'entrée, configuration serveur et CORS
│       ├── models.rs         # Modèles de données (VM, Hyperviseur, Disques, User)
│       ├── state.rs          # État persistant, découverte KVM locale, JWT
│       ├── vnc.rs            # Proxy WebSocket RFB/VNC haute performance
│       ├── drivers/          # Abstraction par Trait Rust
│       │   ├── mod.rs        # Définition du Trait HypervisorDriver
│       │   ├── kvm.rs        # Driver KVM/libvirt (local & SSH distant)
│       │   └── proxmox.rs    # Driver Proxmox VE (REST API)
│       └── api/              # Contrôleurs REST et documentation Swagger
│           ├── auth.rs       # Authentification JWT et RBAC
│           ├── vms.rs        # Cycle de vie, clonage, attribution, VNC
│           ├── hypervisors.rs# Ajout, suppression et télémétrie des nœuds
│           ├── disks.rs      # Création et extension des disques virtuels
│           ├── templates.rs  # Orchestration Packer / Ansible asynchrone
│           └── users.rs      # Gestion des comptes et rôles
├── frontend/                 # Client Web Node.js / React 18 + Vite
│   ├── package.json
│   ├── vite.config.js
│   ├── index.html
│   └── src/
│       ├── main.jsx
│       ├── App.jsx           # Tableau de bord Datacenter Proxmox-like
│       ├── api.js            # Client API REST et WebSocket
│       ├── index.css         # Design system sombre (Dark theme Proxmox)
│       └── components/
│           └── VncViewer.jsx # Client VNC noVNC interactif intégré
├── templates/                # Spécifications d'automatisation
│   ├── packer/               # Configurations Packer (.pkr.hcl)
│   └── ansible/              # Playbooks Ansible de transformation
├── docs/                     # Documentation complète du projet
│   ├── ARCHITECTURE.md       # Schémas d'architecture et conception
│   ├── API_DOCUMENTATION.md  # Spécification détaillée de l'API REST
│   ├── USER_MANUAL.md        # Manuel utilisateur et scénario de démonstration
│   └── DEPLOYMENT.md         # Guide de déploiement (scripts, Docker, systemd)
├── scripts/
│   └── start.sh              # Script de lancement automatique 1-clic
├── Dockerfile                # Image conteneur multi-stage
├── docker-compose.yml        # Orchestration conteneurisée
└── enonce                    # Cahier des charges initial
```

---

## 📚 Documentation Complète
Pour plus d'informations, consultez les dossiers de documentation :
- [Schémas et Architecture Détaillée](file:///home/adiack/Documents/5A/projet-virtu/docs/ARCHITECTURE.md)
- [Documentation de l'API REST](file:///home/adiack/Documents/5A/projet-virtu/docs/API_DOCUMENTATION.md)
- [Manuel Utilisateur & Démonstration](file:///home/adiack/Documents/5A/projet-virtu/docs/USER_MANUAL.md)
- [Guide de Déploiement](file:///home/adiack/Documents/5A/projet-virtu/docs/DEPLOYMENT.md)

---

## 👥 Auteurs du Projet
- **GRANIER Emmy**
- **DIACK Amadou**
- **MINVIELLE Melvin**
- **DJEMAI Yacine**

