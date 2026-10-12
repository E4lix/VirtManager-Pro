output "vm_id" {
  value       = libvirt_domain.vm.id
  description = "Identifiant UUID du domaine libvirt créé"
}

output "vm_name" {
  value       = libvirt_domain.vm.name
  description = "Nom de la machine virtuelle créée"
}

output "vm_vnc_port" {
  value       = libvirt_domain.vm.graphics[0].port
  description = "Port VNC alloué pour la console à distance"
}

output "vm_ip_address" {
  value       = libvirt_domain.vm.network_interface[0].addresses
  description = "Adresses IP obtenues via DHCP"
}
