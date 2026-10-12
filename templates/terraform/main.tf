terraform {
  required_version = ">= 1.5.0"
  required_plugins {
    libvirt = {
      source  = "dmacvicar/libvirt"
      version = ">= 0.7.6"
    }
  }
}

provider "libvirt" {
  uri = var.hypervisor_uri
}

# Image disque principale (Clonage CoW à partir du Golden Image)
resource "libvirt_volume" "vm_os_disk" {
  name             = "${var.vm_name}-disk.qcow2"
  pool             = var.storage_pool
  base_volume_name = var.base_image_name
  size             = var.disk_size_bytes
  format           = "qcow2"
}

# Disque Cloud-Init pour configuration réseau et utilisateurs sans mot de passe
resource "libvirt_cloudinit_disk" "commoninit" {
  name      = "${var.vm_name}-cloudinit.iso"
  pool      = var.storage_pool
  user_data = <<-EOF
    #cloud-config
    hostname: ${var.vm_name}
    fqdn: ${var.vm_name}.virtu.local
    manage_etc_hosts: true
    users:
      - name: debian
        sudo: ALL=(ALL) NOPASSWD:ALL
        shell: /bin/bash
        lock_passwd: false
        passwd: $6$rounds=4096$virtu$6YxI3j
    package_update: true
    packages:
      - qemu-guest-agent
      - curl
      - htop
    runcmd:
      - systemctl enable --now qemu-guest-agent
  EOF
}

# Définition du domaine KVM / libvirt
resource "libvirt_domain" "vm" {
  name   = var.vm_name
  memory = var.memory_mb
  vcpu   = var.vcpus

  cloudinit = libvirt_cloudinit_disk.commoninit.id

  network_interface {
    network_name   = "default"
    wait_for_lease = true
  }

  disk {
    volume_id = libvirt_volume.vm_os_disk.id
  }

  graphics {
    type        = "vnc"
    listen_type = "address"
    autoport    = true
  }

  video {
    type = "vga"
  }

  console {
    type        = "pty"
    target_port = "0"
    target_type = "serial"
  }
}
