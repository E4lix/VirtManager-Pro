#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR"

echo "========================================================================"
echo "   🚀 Démarrage de VirtManager Pro (Proxmox-like Cloud Platform)"
echo "   Projet Virtualisation 5A - Backend Rust & Frontend Node.js"
echo "========================================================================"

# Source cargo environment if present
if [ -f "$HOME/.cargo/env" ]; then
  source "$HOME/.cargo/env"
fi

# 1. Vérifier si le frontend est compilé
if [ ! -d "frontend/dist" ]; then
  echo "📦 Compilation initiale du frontend..."
  cd frontend
  npm install
  npm run build
  cd "$DIR"
fi

# 2. Libérer le port 8080 si une ancienne instance est en cours d'exécution
fuser -k 8080/tcp 2>/dev/null || true
pkill -f virtmanager-backend 2>/dev/null || true
sleep 1

# 3. Compiler et démarrer le backend Rust
echo "⚙️  Démarrage du backend Rust Axum sur le port 8080..."
cd backend
cargo build

echo ""
echo "========================================================================"
echo "   ✅ VirtManager Pro est opérationnel !"
echo "   🌐 Interface Web (Proxmox-like) : http://127.0.0.1:8080"
echo "   📖 Swagger UI (API Docs)        : http://127.0.0.1:8080/swagger-ui/"
echo "   🔑 Identifiants : "
echo "        • Admin : admin / admin"
echo "        • User  : user1 / user1"
echo "========================================================================"
echo ""

exec ./target/debug/virtmanager-backend
