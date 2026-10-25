# Guide de Déploiement - VirtManager Pro

Ce document détaille les étapes d'installation et de mise en route du projet sur un système Linux (Debian 12/13 ou Ubuntu 22.04/24.04).

---

## 1. Prérequis système

- Système Linux avec processeur supportant la virtualisation matérielle (`/dev/kvm`).
- Accès `sudo` sur la machine hôte.

### Paquets requis
```bash
sudo apt update
sudo apt install -y \
  qemu-system-x86 qemu-utils \
  libvirt-daemon-system libvirt-clients \
  novnc websockify \
  build-essential pkg-config libssl-dev \
  curl git jq
```

### Configuration des droits libvirt
Ajouter l'utilisateur aux groupes nécessaires :
```bash
sudo usermod -aG libvirt,kvm "$USER"
```

Pour permettre l'accès au socket libvirt sans invite Polkit, vérifier `/etc/libvirt/libvirtd.conf` :
```bash
sudo sed -i 's/#unix_sock_group = "libvirt"/unix_sock_group = "libvirt"/' /etc/libvirt/libvirtd.conf
sudo sed -i 's/#unix_sock_rw_perms = "0770"/unix_sock_rw_perms = "0770"/' /etc/libvirt/libvirtd.conf
sudo sed -i 's/#auth_unix_rw = "polkit"/auth_unix_rw = "none"/' /etc/libvirt/libvirtd.conf
sudo systemctl restart libvirtd
```

Tester l'accès :
```bash
virsh -c qemu:///system list --all
```

---

## 2. Installation des runtimes

### Node.js (version 20 ou 22 LTS)
```bash
curl -fsSL https://deb.nodesource.com/setup_22.x -o /tmp/nodesource_setup.sh
sudo -E bash /tmp/nodesource_setup.sh
sudo apt install -y nodejs
```

### Rust et Cargo
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

---

## 3. Lancement du projet

### Méthode 1 : Démarrage direct via script (Recommandé)
Un script se charge de compiler le frontend, le backend et de démarrer le serveur :

```bash
cd projet-virtu
./scripts/start.sh
```

Le serveur démarre sur le port `8080`.

---

### Méthode 2 : Démarrage manuel

#### Compilation du frontend
```bash
cd frontend
npm install
npm run build
cd ..
```

#### Démarrage du backend
```bash
cd backend
cargo run
```

---

### Méthode 3 : Déploiement via Docker Compose
Le projet dispose d'une configuration conteneurisée :

```bash
docker compose up --build -d
```

---

## 4. Accès à la plateforme

Une fois le serveur lancé :

- **Interface web principale** : http://127.0.0.1:8080
- **Documentation API (Swagger UI)** : http://127.0.0.1:8080/swagger-ui/
- **Console noVNC directe** : http://127.0.0.1:8080/novnc/vnc.html

### Comptes pré-configurés
- **Administrateur** : `admin` / `admin`
- **Étudiant** : `user1` / `user1`

---

## 5. Vérification rapide

Tester le bon fonctionnement de l'API en ligne de commande :

```bash
# Vérifier la liste des machines virtuelles
curl -s http://127.0.0.1:8080/api/vms | jq .

# Tester l'authentification
curl -s -X POST http://127.0.0.1:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}' | jq .
```
