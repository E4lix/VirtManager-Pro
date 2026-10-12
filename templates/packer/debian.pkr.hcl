packer {
  required_plugins {
    qemu = {
      version = ">= 1.0.10"
      source  = "github.com/hashicorp/qemu"
    }
  }
}

variable "vm_name" {
  type    = string
  default = "debian-template"
}

variable "memory" {
  type    = number
  default = 2048
}

variable "cpus" {
  type    = number
  default = 2
}

variable "disk_size" {
  type    = string
  default = "10G"
}

source "qemu" "debian" {
  vm_name          = "${var.vm_name}.qcow2"
  iso_url          = "https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/debian-12.7.0-amd64-netinst.iso"
  iso_checksum     = "file:https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/SHA512SUMS"
  output_directory = "output-debian"
  shutdown_command = "echo 'packer' | sudo -S shutdown -P now"
  disk_size        = var.disk_size
  format           = "qcow2"
  accelerator      = "kvm"
  http_directory   = "http"
  ssh_username     = "debian"
  ssh_password     = "debian"
  ssh_timeout      = "20m"
  qemuargs = [
    ["-m", "${var.memory}M"],
    ["-smp", "${var.cpus}"]
  ]
}

build {
  sources = ["source.qemu.debian"]

  provisioner "shell" {
    inline = [
      "echo '==> Configuration initiale du template'",
      "sudo apt-get update",
      "sudo apt-get install -y qemu-guest-agent curl cloud-init",
      "sudo systemctl enable qemu-guest-agent"
    ]
  }

  provisioner "ansible" {
    playbook_file = "../ansible/playbooks/base.yml"
  }
}
