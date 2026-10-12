# Manuel Utilisateur & Guide de Démonstration - VirtManager Pro

Bienvenue sur le guide d'utilisation de **VirtManager Pro**, la plateforme unifiée de gestion d'infrastructure virtualisée développée pour le projet 5A.

---

## 1. Accès à la Plateforme

1. Lancez la plateforme avec le script de démarrage :
   ```bash
   ./scripts/start.sh
   ```
2. Ouvrez votre navigateur web sur :
   👉 **`http://127.0.0.1:8080`**

### Comptes prédéfinis
| Rôle | Nom d'utilisateur | Mot de passe | Permissions |
| :--- | :--- | :--- | :--- |
| **Administrateur** | `admin` | `admin` | Gestion totale (Hyperviseurs, VM, Disques, Templates, Utilisateurs) |
| **Étudiant / User** | `user1` | `user1` | Accès réservé aux VM attribuées, console VNC, contrôle démarrage/arrêt |

Le sélecteur de rôle présent dans la barre supérieure permet de basculer instantanément de rôle lors de vos démonstrations.

---

## 2. Déroulement du Scénario de Démonstration (Bout en Bout)

Le cahier des charges demande une démonstration complète couvrant :
`Ajout d'hyperviseur → Création de template avec Packer/Ansible → Déploiement d'une VM → Accès VNC`

Voici les étapes exactes pour réaliser cette démonstration :

### Étape 1 : Ajout d'un Hyperviseur
1. Dans le menu de gauche, cliquez sur **"Hyperviseurs"**.
2. Cliquez sur le bouton bleu **"+ Enregistrer un Hyperviseur"** (ou "+ Hyperviseur" dans la barre supérieure).
3. Renseignez :
   - **Nom** : `KVM-Host-Distant-Lab`
   - **Type** : `KVM / Libvirt (Local ou SSH)`
   - **URI Endpoint** : `qemu+ssh://root@192.168.1.100/system`
4. Cliquez sur **"Connecter et Enregistrer"**.
5. Le nouvel hyperviseur apparaît immédiatement dans le pool avec sa télémétrie en temps réel (CPU %, RAM %, disques).

### Étape 2 : Création de Template avec Packer & Ansible
1. Dans le menu de gauche, rendez-vous dans **"Packer & Ansible"** (Templates).
2. Choisissez le template **"Serveur Web Nginx Haute Performance"**.
3. Observez la configuration :
   - Fichier Packer : `templates/packer/debian.pkr.hcl`
   - Playbook Ansible : `templates/ansible/playbooks/webserver.yml`
4. Cliquez sur le bouton **"Lancer Build Packer/Ansible"**.
5. Vous êtes redirigé vers l'onglet **"Tâches & Builds"** où vous pouvez suivre en direct les logs d'exécution du compilateur Packer et du provisionneur Ansible.

### Étape 3 : Déploiement d'une VM depuis le Template
1. Toujours sur la carte du template, cliquez sur **"Déployer en 1 Clic"**.
2. Nommez la nouvelle VM : `demo-web-prod`.
3. Attribuez-la à l'utilisateur `user1`.
4. Cliquez sur **"Déployer Immédiatement"**.
5. La machine est automatiquement instanciée avec son disque qcow2 et démarre instantanément.

### Étape 4 : Accès à la Console Graphique VNC
1. Allez dans l'onglet **"Machines Virtuelles"**.
2. Sélectionnez la machine `demo-web-prod` ou l'une des machines en cours d'exécution (`debian13`, `debian13-clone`).
3. Cliquez sur l'onglet **"Console VNC"**.
4. L'écran de la machine virtuelle s'affiche en temps réel dans votre navigateur :
   - Vous pouvez interagir à la souris et au clavier.
   - Bouton **"Ctrl+Alt+Suppr"** pour déverrouiller ou ouvrir le gestionnaire.
   - Bouton **"Plein écran"** pour un confort maximal.

---

## 3. Fonctionnalités Avancées

### Gestion du Stockage & Disques
- Onglet **"Stockage & Disques"** :
  - Visualisez la capacité et l'espace alloué des pools libvirt (`images`, `datastoreon`, `tmp`).
  - Cliquez sur **"Nouveau Disque Virtuel"** pour créer un volume `qcow2`, `raw` ou `vmdk`.
  - Cliquez sur **"Étendre la taille"** sur n'importe quel disque existant pour augmenter sa capacité à chaud via `qemu-img resize`.

### Attribution & Sécurité Multi-utilisateurs
- Onglet **"Machines Virtuelles"** :
  - Utilisez le bouton **"Attribuer"** pour confier la gestion d'une VM à un étudiant.
  - Basculez le mode de l'interface sur `Mode: Étudiant 5A (user1)` dans la barre supérieure : l'interface filtre automatiquement et n'affiche plus que les VM appartenant à cet utilisateur.

### Documentation Interactive de l'API
- Cliquez sur **"API Swagger"** dans la barre supérieure ou ouvrez `http://127.0.0.1:8080/swagger-ui/` pour tester directement chaque endpoint de l'API REST Rust.
