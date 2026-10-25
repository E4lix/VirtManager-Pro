# VirtManager Pro - Gestionnaire d'Infrastructure Virtualisée

Projet de virtualisation - Master 2 / 5ème Année d'Ingénieur

## Membres du groupe
- GRANIER Emmy
- DIACK Amadou
- MINVIELLE Melvin
- DJEMAI Yacine

Dépôt GitHub : https://github.com/E4lix/VirtManager-Pro

---

## Description du projet

VirtManager Pro est une application web permettant d'administrer et d'orchestrer un parc d'hyperviseurs et de machines virtuelles. Le système gère les hyperviseurs KVM/libvirt (locaux et distants via SSH) ainsi que les serveurs Proxmox VE via leur API REST.

L'application intègre une console graphique web noVNC (protocole RFB sur WebSockets) directement dans le navigateur, sans plugin additionnel.

### Architecture technique
- **Backend** : Rust avec le framework web Axum et le runtime asynchrone Tokio.
- **Frontend** : Node.js avec React 18 et l'outil de build Vite.
- **Automatisation & Modèles** : HashiCorp Packer et Ansible pour la création et la configuration automatisée des images de machines virtuelles.

---

## Fonctionnalités implémentées

### 1. Gestion des Machines Virtuelles (Module A)
- **Cycle de vie** : Création de VM, démarrage, arrêt propre ACPI (`virsh shutdown`), arrêt forcé (`destroy`), redémarrage et suppression.
- **Clonage** : Duplication rapide de machines virtuelles existantes via `virt-clone`.
- **Attribution de VM** : Attribution d'une machine virtuelle à un étudiant spécifique (fonctionnalité "donner une VM").
- **Console graphique VNC** : Accès à la console écran de la machine virtuelle directement dans l'interface web (gestion clavier, souris, signaux et plein écran).
- **Gestion des disques virtuels** : Création de disques au format qcow2, redimensionnement dynamique de taille (`qemu-img resize`) et attachement/détachement de disques.

### 2. Gestion des Hyperviseurs (Module B)
- **Support multi-hyperviseurs** : Ajout et retrait d'hyperviseurs locaux (KVM/libvirt), distants par SSH et de nœuds Proxmox VE (port 8006).
- **Architecture modulaire** : Utilisation d'un trait Rust (`HypervisorDriver`) pour faciliter l'ajout de nouveaux types d'hyperviseurs.
- **Surveillance des ressources** : Suivi en direct du statut (en ligne/hors ligne), de la charge CPU, de l'utilisation de la mémoire vive et de l'espace disque.
- **Sécurité au retrait** : Interdiction de supprimer un hyperviseur hébergeant des machines virtuelles actives.

### 3. Gestion des Modèles et Automatisation (Module C)
- **Templates de base** : Debian 12 Bookworm, Serveur Web Nginx et Hôte Docker.
- **Automatisation Packer** : Spécifications HCL (`.pkr.hcl`) pour la construction automatique d'images de VM standardisées.
- **Provisionnement Ansible** : Playbooks pour automatiser l'installation et la configuration logicielle à l'intérieur des images.
- **Exécution asynchrone** : Traitement des builds en tâche de fond avec affichage des journaux d'exécution.
- **Déploiement rapide** : Instanciation directe d'une VM à partir d'un template.

### 4. Authentification et Contrôle d'Accès (RBAC)
- **Authentification JWT** : Jetons signés avec expiration.
- **Rôle Administrateur (`admin`)** : Gestion globale de l'infrastructure, ajout/suppression d'hyperviseurs, attribution des VMs aux étudiants, déclenchement des builds Packer.
- **Rôle Étudiant (`user1`)** : Accès restreint uniquement aux machines virtuelles qui lui sont attribuées.

---

## Identifiants de test

| Utilisateur | Mot de passe | Rôle | Droits |
| :--- | :--- | :--- | :--- |
| `admin` | `admin` | Administrateur | Accès complet : gestion des nœuds, attribution de VM, modèles |
| `user1` | `user1` | Étudiant | Accès limité à ses propres machines virtuelles |

---

## Structure du dépôt

```text
projet-virtu/
├── backend/          # Serveur Rust (API Axum, proxy VNC, pilotes KVM/Proxmox)
├── frontend/         # Application React (interface web, client noVNC)
├── templates/        # Configurations Packer (.pkr.hcl) et Ansible
├── scripts/          # Scripts de démarrage et de configuration de nœud
├── docs/             # Documentation technique (architecture, API, manuel)
├── Dockerfile        # Image multi-stage pour déploiement conteneurisé
└── docker-compose.yml
```
