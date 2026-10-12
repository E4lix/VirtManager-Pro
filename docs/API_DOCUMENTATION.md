# Documentation de l'API REST - VirtManager Pro

L'API de VirtManager Pro est implémentée en **Rust (Axum)** de manière entièrement asynchrone avec **Tokio**.
Elle dispose d'une documentation interactive Swagger UI accessible directement via l'URL :
👉 **`http://127.0.0.1:8080/swagger-ui/`**

---

## 1. Authentification & Sécurité

### `POST /api/auth/login`
Authentifie un utilisateur et génère un token JWT (durée 24h).
- **Body JSON** :
  ```json
  {
    "username": "admin",
    "password": "admin"
  }
  ```
- **Réponse (200 OK)** :
  ```json
  {
    "success": true,
    "message": "Connexion réussie",
    "data": {
      "token": "eyJhbGciOiJIUzI1NiIsIn...",
      "user": {
        "id": "u-admin",
        "username": "admin",
        "role": "admin",
        "full_name": "Administrateur Système",
        "email": "admin@virmox.local"
      }
    }
  }
  ```

### `GET /api/auth/me`
Renvoie les informations de l'utilisateur actuellement connecté via son token Bearer.
- **Header requis** : `Authorization: Bearer <token>`

---

## 2. Module A : Machines Virtuelles (VM)

### `GET /api/vms`
Liste toutes les machines virtuelles.
- **Contrôle d'accès (RBAC)** :
  - Si l'utilisateur est `admin` : renvoie toutes les VM du cluster.
  - Si l'utilisateur est `user` : renvoie uniquement les VM qui lui sont attribuées (`assigned_to`).

### `POST /api/vms`
Crée une nouvelle machine virtuelle et initialise son disque virtuel qcow2.
- **Body JSON** :
  ```json
  {
    "name": "vm-production-01",
    "hypervisor_id": "local-kvm",
    "vcpus": 2,
    "memory_mb": 4096,
    "disk_size_gb": 30,
    "disk_format": "qcow2",
    "os_type": "Debian 12",
    "assigned_to": "user1"
  }
  ```

### `GET /api/vms/{id}`
Récupère les détails d'une VM spécifique (état, mémoire, processeurs, disques, port VNC).

### `POST /api/vms/{id}/start`
Démarre la machine virtuelle via `virsh start`.

### `POST /api/vms/{id}/stop`
Envoie un signal d'arrêt ACPI propre via `virsh shutdown`.

### `POST /api/vms/{id}/reboot`
Redémarre la machine virtuelle via `virsh reboot`.

### `DELETE /api/vms/{id}`
Arrête et détruit la machine virtuelle, avec libération de son stockage associé via `virsh undefine --remove-all-storage`.

### `POST /api/vms/{id}/clone`
Clone une machine virtuelle existante de manière non-bloquante via `virt-clone`.
- **Body JSON** :
  ```json
  {
    "new_name": "vm-production-01-clone",
    "assigned_to": "user1"
  }
  ```

### `POST /api/vms/{id}/assign`
Attribue une machine virtuelle à un utilisateur spécifique.
- **Body JSON** :
  ```json
  {
    "username": "user1"
  }
  ```

### `GET /api/vms/{id}/vnc`
Renvoie les informations de connexion VNC :
```json
{
  "success": true,
  "data": {
    "vm_id": "debian13",
    "vnc_port": 5900,
    "ws_path": "/api/vnc/ws/debian13",
    "novnc_url": "/novnc/vnc_lite.html?path=api/vnc/ws/debian13"
  }
}
```

### `GET /api/vnc/ws/{vm_id}`
Endpoint WebSocket du proxy RFB/VNC. Pont bidirectionnel haute performance entre WebSocket (navigateur web / noVNC) et socket TCP (serveur VNC de la VM).

---

## 3. Module B : Hyperviseurs

### `GET /api/hypervisors`
Liste tous les hyperviseurs enregistrés avec télémétrie en temps réel (CPU %, RAM totale/utilisée, stockage, nombre de VM actives).

### `POST /api/hypervisors`
Enregistre un nouvel hyperviseur local ou distant (Réservé aux administrateurs).
- **Body JSON** :
  ```json
  {
    "name": "KVM-Host-Distant",
    "hypervisor_type": "kvm",
    "endpoint": "qemu+ssh://root@192.168.1.120/system",
    "is_local": false
  }
  ```

### `DELETE /api/hypervisors/{id}`
Retire un hyperviseur du système.
> **Règle de sécurité imposée** : Le retrait est strictement rejeté avec un code `400 Bad Request` si des VM sont en cours d'exécution sur cet hyperviseur.

---

## 4. Module C : Disques & Stockage Virtuel

### `GET /api/disks`
Liste toutes les images de disques virtuels (`.qcow2`, `.raw`, `.vmdk`) avec leur taille et leur pool d'appartenance.

### `POST /api/disks`
Crée un nouveau disque virtuel via `qemu-img create`.
- **Body JSON** :
  ```json
  {
    "name": "volume-data-01",
    "pool_name": "images",
    "size_gb": 50,
    "format": "qcow2"
  }
  ```

### `POST /api/disks/resize`
Étend la capacité d'un disque virtuel existant via `qemu-img resize`.
- **Body JSON** :
  ```json
  {
    "disk_path": "/var/lib/libvirt/images/debian13.qcow2",
    "new_size_gb": 40
  }
  ```

### `GET /api/storage/pools`
Liste les pools de stockage libvirt et leurs métriques d'espace.

---

## 5. Module D : Templates & Automatisation (Packer & Ansible)

### `GET /api/templates`
Liste les templates disponibles (Debian Cloud-Init, Nginx Web Server, Docker Host).

### `POST /api/templates/{id}/build`
Déclenche la construction automatisée d'un template via **Packer** et son provisionnement via **Ansible** en tâche asynchrone non-bloquante.
Renvoie un identifiant de tâche `task-id`.

### `POST /api/templates/{id}/deploy`
Instancie immédiatement une nouvelle machine virtuelle à partir du modèle spécifié.
- **Body JSON** :
  ```json
  {
    "vm_name": "web-instance-01",
    "hypervisor_id": "local-kvm",
    "assigned_to": "user1"
  }
  ```

### `GET /api/tasks`
Récupère l'historique et les journaux de logs détaillés (stdout/stderr) de chaque build et déploiement en arrière-plan.
