# Outils et Technologies Utilisés - VirtManager Pro

Projet de virtualisation - Master 2 / 5ème Année d'Ingénieur

Ce document récapitule les technologies, langages, bibliothèques et logiciels employés pour le développement du projet.

---

## 1. Backend

- **Rust (Édition 2021)** : Langage principal du backend. Choisi pour ses performances d'exécution, sa gestion stricte de la mémoire sans ramasse-miettes et son adéquation avec les appels système de virtualisation.
- **Axum (0.7)** : Framework web HTTP bâti sur Tokio. Assure le routage des endpoints REST et la gestion des connexions WebSockets.
- **Tokio (1.x)** : Runtime asynchrone utilisé pour le traitement non-bloquant des requêtes, le proxy TCP/WebSocket VNC et les boucles de synchronisation en tâche de fond.
- **Tower / Tower-HTTP (0.5)** : Gestion des en-têtes CORS et service des fichiers statiques frontend et noVNC.
- **Utoipa (4.2) & Utoipa-Swagger-UI (7.1)** : Génération automatique de la spécification OpenAPI 3.0 et interface Swagger intégrée.
- **JsonWebToken (9.3)** : Génération et vérification des jetons d'authentification signés (HMAC SHA-256).
- **Serde / Serde JSON (1.0)** : Sérialisation et désérialisation des structures de données.
- **Reqwest (0.12)** : Client HTTP asynchrone pour les requêtes vers l'API REST de Proxmox VE.
- **Chrono (0.4) & UUID (1.8)** : Horodatage UTC et génération des identifiants uniques.

---

## 2. Frontend

- **Node.js (v20 / v22 LTS)** : Environnement d'exécution JavaScript et gestionnaire de paquets npm.
- **React (18.3)** : Bibliothèque UI pour la construction de l'interface utilisateur monopage (SPA).
- **Vite (5.4)** : Outil de build et serveur de développement (remplace webpack pour des temps de compilation réduits).
- **@novnc/novnc (1.5)** : Client web JavaScript pour le protocole RFB (Remote FrameBuffer). Assure l'affichage et l'interaction avec la console graphique des machines virtuelles via un élément HTML5 Canvas.
- **Lucide React (0.451)** : Jeu d'icônes pour l'interface.
- **CSS Vanilla** : Feuilles de styles basées sur des variables CSS, sans framework externe lourd, pour une interface typée gestionnaire d'infrastructure.

---

## 3. Virtualisation et Stockage

- **KVM (Kernel-based Virtual Machine)** : Module noyau Linux pour la virtualisation matérielle assistée par CPU.
- **QEMU** : Émulation des composants matériels des machines virtuelles (périphériques, contrôleurs disque, cartes réseau virtio).
- **Libvirt / virsh** : API et outil en ligne de commande pour le pilotage du cycle de vie des VMs KVM et l'inspection de leur configuration XML.
- **qemu-img** : Création, conversion et redimensionnement dynamique des images disques au format qcow2.
- **virt-clone** : Outil de clonage des machines virtuelles KVM.
- **Proxmox VE (PVE)** : Hyperviseur distant connecté via l'API REST officielle (`/api2/json`).

---

## 4. Automatisation et Déploiement

- **HashiCorp Packer** : Construction automatisée d'images de VM standardisées (`.pkr.hcl`).
- **Ansible** : Playbooks YAML pour automatiser l'installation et le durcissement des applications dans les templates.
- **Terraform** : Fichiers d'infrastructure-as-code pour le provisionnement déclaratif.
- **Docker & Docker Compose** : Conteneurisation de l'application via un Dockerfile multi-stage (étape 1 : compilation Rust et Node, étape 2 : runtime Debian allégé).
- **Git & GitHub** : Gestion de versions et travail d'équipe sur le dépôt [`E4lix/VirtManager-Pro`](https://github.com/E4lix/VirtManager-Pro).
