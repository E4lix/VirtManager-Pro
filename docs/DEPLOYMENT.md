# Guide de Déploiement & Installation - VirtManager Pro

Ce document détaille les méthodes d'installation et de déploiement de **VirtManager Pro** sur un serveur hôte Linux Debian / Ubuntu.

---

## 1. Prérequis Système

- **Système d'exploitation** : Debian 12/13 ou Ubuntu 22.04/24.04 LTS
- **Virtualisation matérielle** : Intel VT-x ou AMD-V activé (`kvm-ok` ou `/dev/kvm` accessible)
- **Installation des outils (Debian / Ubuntu)** :

  Mettez à jour les paquets et installez QEMU, libvirt, noVNC ainsi que les outils de compilation requis par le backend Rust :

  ```bash
  sudo apt update
  sudo apt install -y \
    qemu-system-x86 qemu-utils \
    libvirt-daemon-system libvirt-clients \
    novnc websockify \
    cpu-checker \
    build-essential pkg-config libssl-dev \
    curl ca-certificates
  ```

  Activez libvirt et ajoutez votre compte aux groupes permettant d'utiliser libvirt et KVM :

  ```bash
  sudo systemctl enable --now libvirtd
  sudo usermod -aG libvirt,kvm "$USER"
  ```

  Déconnectez-vous puis reconnectez-vous pour que les nouveaux groupes soient pris en compte.

  Installez Node.js 22 (version LTS, compatible avec la version minimale requise de Node.js 20) :

  ```bash
  curl -fsSL https://deb.nodesource.com/setup_22.x -o /tmp/nodesource_setup.sh
  sudo -E bash /tmp/nodesource_setup.sh
  sudo apt install -y nodejs
  ```

  Installez Rust stable avec rustup afin d'avoir une version récente de `rustc` et `cargo` :

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  . "$HOME/.cargo/env"
  ```

  Vérifiez les installations et l'accès à la virtualisation :

  ```bash
  node --version
  npm --version
  rustc --version
  cargo --version
  virsh --version
  qemu-system-x86_64 --version
  kvm-ok
  ```

  Si vous déployez avec Docker, l'installation de Rust et Node.js sur l'hôte n'est pas nécessaire : ils sont utilisés lors de la construction de l'image. QEMU et libvirt restent requis sur l'hôte pour gérer les VM locales.

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
