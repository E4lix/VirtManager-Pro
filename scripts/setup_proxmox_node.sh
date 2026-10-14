#!/usr/bin/env bash
# ==============================================================================
# VirtManager Pro - Script d'installation du pont VNC pour nœud Proxmox VE
# À exécuter sur le nœud Proxmox VE (en tant que root)
# ==============================================================================
set -e

echo "🔧 Configuration du pont VNC VirtManager Pro sur le nœud Proxmox VE..."

# 1. Vérification des dépendances
apt-get update -qq
apt-get install -y -qq python3 python3-cryptography >/dev/null

# 2. Installation du démon de pont VNC
cat << 'EOF' > /usr/local/bin/virmox-vnc-bridge.py
#!/usr/bin/env python3
import glob
import os
import re
import select
import socket
import subprocess
import threading
import time

from cryptography.hazmat.primitives.ciphers import Cipher, modes
from cryptography.hazmat.decrepit.ciphers.algorithms import TripleDES

def reverse_bits(b):
    return int('{:08b}'.format(b)[::-1], 2)

KEY = bytes([reverse_bits(b) for b in b'virmox'.ljust(8, b'\x00')[:8]])

active_listeners = {}

def set_vm_password(vmid):
    try:
        subprocess.run(
            ['qm', 'monitor', str(vmid)],
            input=b'set_password vnc virmox\nexpire_password vnc never\n',
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=5
        )
    except Exception as e:
        print(f'Failed to set password for VM {vmid}: {e}')

def connect_and_auth_qemu(vmid):
    qemu = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    qemu.connect(f'/var/run/qemu-server/{vmid}.vnc')

    qemu.recv(1024)
    qemu.sendall(b'RFB 003.008\n')
    qemu.recv(1024)
    qemu.sendall(bytes([2]))
    challenge = qemu.recv(16)

    cipher = Cipher(TripleDES(KEY * 3), modes.ECB())
    enc = cipher.encryptor()
    resp = enc.update(challenge) + enc.finalize()
    qemu.sendall(resp)
    auth_ok = qemu.recv(4)
    if auth_ok != b'\x00\x00\x00\x00':
        qemu.close()
        set_vm_password(vmid)
        time.sleep(0.1)
        qemu = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        qemu.connect(f'/var/run/qemu-server/{vmid}.vnc')
        qemu.recv(1024)
        qemu.sendall(b'RFB 003.008\n')
        qemu.recv(1024)
        qemu.sendall(bytes([2]))
        challenge = qemu.recv(16)
        enc = Cipher(TripleDES(KEY * 3), modes.ECB()).encryptor()
        qemu.sendall(enc.update(challenge) + enc.finalize())
        auth_ok = qemu.recv(4)
        if auth_ok != b'\x00\x00\x00\x00':
            qemu.close()
            return None
    return qemu

def handle_client(client_sock, vmid):
    qemu = None
    try:
        qemu = connect_and_auth_qemu(vmid)
        if not qemu:
            print(f'[VM {vmid}] QEMU auth failed')
            client_sock.close()
            return

        client_sock.sendall(b'RFB 003.008\n')
        client_sock.recv(12)
        client_sock.sendall(bytes([1, 1]))
        client_sock.recv(1)
        client_sock.sendall(b'\x00\x00\x00\x00')

        while True:
            r, _, _ = select.select([client_sock, qemu], [], [], 3600)
            if client_sock in r:
                data = client_sock.recv(4096)
                if not data: break
                qemu.sendall(data)
            if qemu in r:
                data = qemu.recv(4096)
                if not data: break
                client_sock.sendall(data)
    except Exception:
        pass
    finally:
        try: client_sock.close()
        except: pass
        if qemu:
            try: qemu.close()
            except: pass

def start_vm_listener(vmid):
    port = 5900 + (vmid % 100)
    set_vm_password(vmid)

    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    try:
        srv.bind(('0.0.0.0', port))
        srv.listen(5)
        print(f'[Virmox VNC] Listening on 0.0.0.0:{port} for VM {vmid}')
    except Exception as e:
        print(f'Failed to bind port {port} for VM {vmid}: {e}')
        return

    active_listeners[vmid] = srv

    def accept_loop():
        while vmid in active_listeners:
            try:
                cs, _ = srv.accept()
                threading.Thread(target=handle_client, args=(cs, vmid), daemon=True).start()
            except:
                break

    threading.Thread(target=accept_loop, daemon=True).start()

def main():
    print('[Virmox VNC Bridge] Démarré')
    while True:
        try:
            socks = glob.glob('/var/run/qemu-server/*.vnc')
            current_vmids = set()
            for s in socks:
                m = re.search(r'/var/run/qemu-server/(\d+)\.vnc', s)
                if m:
                    current_vmids.add(int(m.group(1)))

            for vmid in current_vmids:
                if vmid not in active_listeners:
                    start_vm_listener(vmid)

            for vmid in list(active_listeners.keys()):
                if vmid not in current_vmids:
                    print(f'[Virmox VNC] Fermeture écouteur VM {vmid}')
                    sock = active_listeners.pop(vmid)
                    try: sock.close()
                    except: pass
        except Exception as e:
            print('Scan error:', e)
        time.sleep(2)

if __name__ == '__main__':
    main()
EOF

chmod +x /usr/local/bin/virmox-vnc-bridge.py

# 3. Création du service systemd
cat << 'EOF' > /etc/systemd/system/virmox-vnc-bridge.service
[Unit]
Description=VirtManager Pro VNC Bridge for Proxmox VE VMs
After=pve-cluster.service

[Service]
Type=simple
ExecStart=/usr/local/bin/virmox-vnc-bridge.py
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable --now virmox-vnc-bridge.service

echo "✅ Le pont VNC VirtManager Pro est actif sur le nœud Proxmox !"
