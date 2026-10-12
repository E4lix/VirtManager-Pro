# Guide de Déploiement & Installation - VirtManager Pro

Ce document détaille les méthodes d'installation et de déploiement de **VirtManager Pro** sur un serveur hôte Linux Debian / Ubuntu.

---

## 1. Prérequis Système

- **Système d'exploitation** : Debian 12/13 ou Ubuntu 22.04/24.04 LTS
- **Virtualisation matérielle** : Intel VT-x ou AMD-V activé (`kvm-ok` ou `/dev/kvm` accessible)
- **Outils installés** :
  - `qemu-system-x86_64`, `qemu-utils`
  - `libvirt-daemon-system`, `libvirt-clients`
  - `rustc` / `cargo` (ou utilisation de Docker)
  - `nodejs` / `npm` (v20+)
  - `novnc`, `websockify`

---

## 2. Déploiement Local Rapide (Recommandé pour Développement & Démo)

Exécutez simplement le script d'initialisation et de lancement :
```bash
cd /home/adiack/Documents/5A/projet-virtu
./scripts/start.sh
```

Le script se charge de :
1. Vérifier et compiler les dépendances Node.js et React (`npm run build`).
2. Compiler le serveur Rust haute performance (`cargo build`).
3. Démarrer le serveur HTTP Axum, le proxy WebSocket VNC et Swagger UI sur le port `8080`.

L'application est disponible immédiatement à l'adresse :
👉 `http://127.0.0.1:8080`

---

## 3. Déploiement via Docker & Docker Compose

Pour un déploiement conteneurisé complet et isolé :

```bash
docker compose up --build -d
```

Le conteneur expose le port `8080` et monte les sockets libvirt hôte pour piloter directement les machines virtuelles physiques.

---

## 4. Déploiement sous forme de Service Systemd (Production)

Pour que la plateforme démarre automatiquement au boot de la machine :

Créez le fichier de service `/etc/systemd/system/virtmanager.service` :
```ini
[Unit]
Description=VirtManager Pro - Cloud Virtualization Platform
After=network.target libvirtd.service

[Service]
Type=simple
User=adiack
WorkingDirectory=/home/adiack/Documents/5A/projet-virtu/backend
ExecStart=/home/adiack/Documents/5A/projet-virtu/backend/target/debug/virtmanager-backend
Restart=always
RestartSec=5
Environment=PORT=8080
Environment=RUST_LOG=virtmanager_backend=info

[Install]
WantedBy=multi-user.target
```

Activez et démarrez le service :
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now virtmanager.service
sudo systemctl status virtmanager.service
```
