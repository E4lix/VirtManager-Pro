variable "hypervisor_uri" {
  type        = string
  default     = "qemu:///system"
  description = "URI de connexion libvirt de l'hyperviseur cible"
}

variable "vm_name" {
  type        = string
  default     = "debian-tf-instance"
  description = "Nom unique de la machine virtuelle"
}

variable "vcpus" {
  type        = number
  default     = 2
  description = "Nombre de processeurs virtuels (vCPUs)"
}

variable "memory_mb" {
  type        = number
  default     = 2048
  description = "Mémoire vive RAM allouée en Mo"
}

variable "storage_pool" {
  type        = string
  default     = "default"
  description = "Pool de stockage libvirt pour héberger le disque"
}

variable "base_image_name" {
  type        = string
  default     = "debian13.qcow2"
  description = "Nom de l'image de base (Golden Image) pour le clonage CoW"
}

variable "disk_size_bytes" {
  type        = number
  default     = 21474836480 # 20 Go
  description = "Taille maximale allouée pour le disque en octets"
}
