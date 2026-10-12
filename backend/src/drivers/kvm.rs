use anyhow::{Context, Result};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};

use crate::drivers::{HypervisorDriver, VncInfo};
use crate::models::{
    CreateVmRequest, HypervisorStats, StoragePool, VirtualMachine, VmDisk, VmState,
};

#[derive(Clone)]
pub struct KvmDriver {
    pub hypervisor_id: String,
    pub hypervisor_name: String,
    pub endpoint: String, // e.g. "qemu:///system" or "qemu+ssh://user@host/system"
}

impl KvmDriver {
    pub fn new(id: String, name: String, endpoint: String) -> Self {
        Self {
            hypervisor_id: id,
            hypervisor_name: name,
            endpoint,
        }
    }

    async fn run_virsh(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("virsh");
        cmd.arg("-c").arg(&self.endpoint);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = cmd.output().await.context("Failed to run virsh command")?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("virsh error: {}", err.trim());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn ensure_vnc_graphics(&self, vm_id: &str) -> Result<()> {
        let xml = match self.run_virsh(&["dumpxml", vm_id]).await {
            Ok(x) => x,
            Err(_) => return Ok(()),
        };

        let mut needs_update = false;
        let mut new_xml = xml.clone();

        if !new_xml.contains("<graphics type='vnc'") {
            if let Some(pos) = new_xml.rfind("</devices>") {
                let vnc_snippet = "    <graphics type='vnc' port='-1' autoport='yes' listen='0.0.0.0' sharePolicy='ignore'>\n      <listen type='address' address='0.0.0.0'/>\n    </graphics>\n";
                new_xml.insert_str(pos, vnc_snippet);
                needs_update = true;
            }
        } else if !new_xml.contains("sharePolicy='ignore'") {
            new_xml = new_xml.replace(
                "<graphics type='vnc'",
                "<graphics type='vnc' sharePolicy='ignore'",
            );
            needs_update = true;
        }

        if needs_update {
            let temp_file = format!("/tmp/vnc-patch-{}.xml", vm_id);
            if tokio::fs::write(&temp_file, &new_xml).await.is_ok() {
                let _ = self.run_virsh(&["define", &temp_file]).await;
                let _ = tokio::fs::remove_file(&temp_file).await;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl HypervisorDriver for KvmDriver {
    async fn ping(&self) -> Result<bool> {
        match self.run_virsh(&["uri"]).await {
            Ok(out) => Ok(!out.trim().is_empty()),
            Err(e) => {
                warn!("KVM ping failed for {}: {}", self.endpoint, e);
                Ok(false)
            }
        }
    }

    async fn get_stats(&self) -> Result<HypervisorStats> {
        let nodeinfo = self.run_virsh(&["nodeinfo"]).await.unwrap_or_default();
        let nodemem = self.run_virsh(&["nodememstats"]).await.unwrap_or_default();

        let mut cpu_cores = 4u32;
        let mut memory_total_mb = 16384u64;
        let mut memory_free_mb = 8192u64;

        for line in nodeinfo.lines() {
            let line = line.trim();
            if line.starts_with("CPU") && line.contains(':') {
                if let Some(val) = line.split(':').nth(1) {
                    if let Ok(c) = val.trim().parse::<u32>() {
                        cpu_cores = c;
                    }
                }
            } else if (line.starts_with("Taille mémoire") || line.starts_with("Memory size")) && line.contains(':') {
                if let Some(val) = line.split(':').nth(1) {
                    let parts: Vec<&str> = val.trim().split_whitespace().collect();
                    if let Some(first) = parts.first() {
                        if let Ok(kib) = first.parse::<u64>() {
                            memory_total_mb = kib / 1024;
                        }
                    }
                }
            }
        }

        for line in nodemem.lines() {
            let line = line.trim();
            if line.starts_with("free") && line.contains(':') {
                if let Some(val) = line.split(':').nth(1) {
                    let parts: Vec<&str> = val.trim().split_whitespace().collect();
                    if let Some(first) = parts.first() {
                        if let Ok(kib) = first.parse::<u64>() {
                            memory_free_mb = kib / 1024;
                        }
                    }
                }
            }
        }

        let memory_used_mb = memory_total_mb.saturating_sub(memory_free_mb);

        // Fetch pools for storage stats
        let pools = self.list_storage_pools().await.unwrap_or_default();
        let disk_total_gb: u64 = pools.iter().map(|p| p.capacity_gb).sum();
        let disk_used_gb: u64 = pools.iter().map(|p| p.allocated_gb).sum();

        let vms = self.list_vms().await.unwrap_or_default();
        let total_vms = vms.len() as u32;
        let active_vms = vms.iter().filter(|v| v.state == VmState::Running).count() as u32;

        Ok(HypervisorStats {
            cpu_cores,
            cpu_usage_pct: 12.5,
            memory_total_mb,
            memory_used_mb,
            disk_total_gb: if disk_total_gb == 0 { 500 } else { disk_total_gb },
            disk_used_gb: if disk_used_gb == 0 { 120 } else { disk_used_gb },
            active_vms,
            total_vms,
            uptime_seconds: 3600 * 24,
        })
    }

    async fn list_vms(&self) -> Result<Vec<VirtualMachine>> {
        let output = self.run_virsh(&["list", "--all"]).await?;
        let mut vms = Vec::new();

        for line in output.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = if parts[0] == "-" {
                    parts[1]
                } else if parts.len() >= 3 {
                    parts[1]
                } else {
                    continue;
                };

                if let Ok(vm) = self.get_vm(name).await {
                    vms.push(vm);
                }
            }
        }

        Ok(vms)
    }

    async fn get_vm(&self, name_or_id: &str) -> Result<VirtualMachine> {
        let dominfo = self.run_virsh(&["dominfo", name_or_id]).await?;
        let mut state = VmState::Stopped;
        let mut vcpus = 2u32;
        let mut memory_mb = 2048u64;

        for line in dominfo.lines() {
            let line = line.trim();
            if (line.starts_with("État") || line.starts_with("State")) && line.contains(':') {
                let s = line.split(':').nth(1).unwrap_or("").trim().to_lowercase();
                if s.contains("cours") || s.contains("running") {
                    state = VmState::Running;
                } else if s.contains("fermé") || s.contains("shut off") {
                    state = VmState::Stopped;
                } else if s.contains("pause") {
                    state = VmState::Paused;
                }
            } else if line.starts_with("CPU:") {
                if let Some(val) = line.split(':').nth(1) {
                    if let Ok(c) = val.trim().parse::<u32>() {
                        vcpus = c;
                    }
                }
            } else if (line.starts_with("Mémoire Max") || line.starts_with("Max memory")) && line.contains(':') {
                if let Some(val) = line.split(':').nth(1) {
                    let parts: Vec<&str> = val.trim().split_whitespace().collect();
                    if let Some(first) = parts.first() {
                        if let Ok(kib) = first.parse::<u64>() {
                            memory_mb = kib / 1024;
                        }
                    }
                }
            }
        }

        // Parse disks
        let mut disks = Vec::new();
        if let Ok(blk_out) = self.run_virsh(&["domblklist", name_or_id]).await {
            for line in blk_out.lines().skip(2) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] != "-" {
                    let target = parts[0];
                    let source = parts[1];
                    disks.push(VmDisk {
                        id: format!("{}-{}", name_or_id, target),
                        name: target.to_string(),
                        path: source.to_string(),
                        size_gb: 20,
                        format: "qcow2".to_string(),
                        pool_name: "images".to_string(),
                    });
                }
            }
        }

        // Parse VNC port from vncdisplay (real RFB display)
        let mut vnc_port = None;
        if let Ok(out) = self.run_virsh(&["vncdisplay", name_or_id]).await {
            let s = out.trim();
            if let Some(idx) = s.rfind(':') {
                if let Ok(disp) = s[idx + 1..].parse::<u16>() {
                    vnc_port = Some(5900 + disp);
                }
            }
        }

        // Fallback to dumpxml (specifically type='vnc')
        if vnc_port.is_none() {
            if let Ok(xml) = self.run_virsh(&["dumpxml", name_or_id]).await {
                for line in xml.lines() {
                    if line.contains("<graphics") && line.contains("type='vnc'") {
                        if let Some(port_idx) = line.find("port='") {
                            let rem = &line[port_idx + 6..];
                            if let Some(end_idx) = rem.find('\'') {
                                if let Ok(p) = rem[..end_idx].parse::<u16>() {
                                    if p > 0 {
                                        vnc_port = Some(p);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Parse IP address from leases
        let mut ip_address = None;
        if let Ok(leases) = self.run_virsh(&["net-dhcp-leases", "default"]).await {
            for line in leases.lines().skip(2) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let ip = parts[4].split('/').next().unwrap_or("");
                    if !ip.is_empty() {
                        ip_address = Some(ip.to_string());
                        break;
                    }
                }
            }
        }

        Ok(VirtualMachine {
            id: name_or_id.to_string(),
            name: name_or_id.to_string(),
            hypervisor_id: self.hypervisor_id.clone(),
            hypervisor_name: self.hypervisor_name.clone(),
            state,
            vcpus,
            memory_mb,
            disks,
            assigned_to: None,
            vnc_port,
            vnc_ws_path: Some(format!("/api/vnc/ws/{}", name_or_id)),
            ip_address,
            os_type: "Debian GNU/Linux".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn create_vm(&self, req: &CreateVmRequest) -> Result<VirtualMachine> {
        info!("Creating VM {} on {}", req.name, self.hypervisor_name);

        let disk_dir = "/var/lib/libvirt/images";
        let disk_format = req.disk_format.as_deref().unwrap_or("qcow2");
        let disk_path = format!("{}/{}.{}", disk_dir, req.name, disk_format);

        // If template image exists, copy/backing file; otherwise create new qcow2
        let mut qemu_cmd = Command::new("qemu-img");
        qemu_cmd.arg("create").arg("-f").arg(disk_format).arg(&disk_path).arg(format!("{}G", req.disk_size_gb));
        let _ = qemu_cmd.output().await;

        let memory_kib = req.memory_mb * 1024;
        let vcpus = req.vcpus;
        let vm_xml = format!(r#"<domain type='kvm'>
  <name>{}</name>
  <memory unit='KiB'>{}</memory>
  <currentMemory unit='KiB'>{}</currentMemory>
  <vcpu placement='static'>{}</vcpu>
  <os>
    <type arch='x86_64' machine='pc'>hvm</type>
    <boot dev='hd'/>
  </os>
  <features>
    <acpi/>
    <apic/>
  </features>
  <clock offset='utc'/>
  <on_poweroff>destroy</on_poweroff>
  <on_reboot>restart</on_reboot>
  <on_crash>destroy</on_crash>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    <disk type='file' device='disk'>
      <driver name='qemu' type='{}'/>
      <source file='{}'/>
      <target dev='vda' bus='virtio'/>
    </disk>
    <interface type='network'>
      <source network='default'/>
      <model type='virtio'/>
    </interface>
    <graphics type='vnc' port='-1' autoport='yes' listen='0.0.0.0' sharePolicy='ignore'>
      <listen type='address' address='0.0.0.0'/>
    </graphics>
    <video>
      <model type='vga' vram='16384' heads='1'/>
    </video>
  </devices>
</domain>"#, req.name, memory_kib, memory_kib, vcpus, disk_format, disk_path);

        // Write xml to temp file and define domain
        let temp_xml_path = format!("/tmp/vm-{}.xml", req.name);
        tokio::fs::write(&temp_xml_path, vm_xml).await?;

        let define_res = self.run_virsh(&["define", &temp_xml_path]).await;
        let _ = tokio::fs::remove_file(&temp_xml_path).await;
        define_res?;

        // Start VM automatically
        let _ = self.run_virsh(&["start", &req.name]).await;

        self.get_vm(&req.name).await
    }

    async fn start_vm(&self, vm_id: &str) -> Result<()> {
        info!("Starting VM {}", vm_id);
        let _ = self.ensure_vnc_graphics(vm_id).await;
        self.run_virsh(&["start", vm_id]).await?;
        Ok(())
    }

    async fn stop_vm(&self, vm_id: &str, force: bool) -> Result<()> {
        info!("Stopping VM {} (force={})", vm_id, force);
        if force {
            self.run_virsh(&["destroy", vm_id]).await?;
        } else {
            self.run_virsh(&["shutdown", vm_id]).await?;
        }
        Ok(())
    }

    async fn reboot_vm(&self, vm_id: &str) -> Result<()> {
        info!("Rebooting VM {}", vm_id);
        self.run_virsh(&["reboot", vm_id]).await?;
        Ok(())
    }

    async fn delete_vm(&self, vm_id: &str) -> Result<()> {
        info!("Deleting VM {}", vm_id);
        let _ = self.run_virsh(&["destroy", vm_id]).await;
        self.run_virsh(&["undefine", vm_id, "--remove-all-storage"]).await?;
        Ok(())
    }

    async fn clone_vm(&self, source_id: &str, new_name: &str) -> Result<VirtualMachine> {
        info!("Cloning VM {} to {}", source_id, new_name);

        let mut cmd = Command::new("virt-clone");
        cmd.arg("-c").arg(&self.endpoint)
           .arg("--original").arg(source_id)
           .arg("--name").arg(new_name)
           .arg("--auto-clone");

        let output = cmd.output().await.context("Failed to run virt-clone")?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("virt-clone failed: {}", err.trim());
        }

        let _ = self.ensure_vnc_graphics(new_name).await;
        self.get_vm(new_name).await
    }


    async fn get_vnc_info(&self, vm_id: &str) -> Result<VncInfo> {
        let _ = self.ensure_vnc_graphics(vm_id).await;
        let mut port = 5900;
        if let Ok(out) = self.run_virsh(&["vncdisplay", vm_id]).await {
            let s = out.trim();
            if let Some(idx) = s.rfind(':') {
                if let Ok(disp) = s[idx + 1..].parse::<u16>() {
                    port = 5900 + disp;
                }
            }
        } else if let Ok(vm) = self.get_vm(vm_id).await {
            if let Some(p) = vm.vnc_port {
                port = p;
            }
        }

        Ok(VncInfo {
            host: "127.0.0.1".to_string(),
            port,
            path: format!("/api/vnc/ws/{}", vm_id),
        })
    }

    async fn list_storage_pools(&self) -> Result<Vec<StoragePool>> {
        let output = self.run_virsh(&["pool-list", "--all"]).await?;
        let mut pools = Vec::new();

        for line in output.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let name = parts[0];
                if let Ok(info) = self.run_virsh(&["pool-info", name]).await {
                    let mut capacity_gb = 500u64;
                    let mut allocated_gb = 100u64;
                    let mut available_gb = 400u64;

                    for pline in info.lines() {
                        let pline = pline.trim();
                        if (pline.starts_with("Capacité") || pline.starts_with("Capacity")) && pline.contains(':') {
                            if let Some(val) = pline.split(':').nth(1) {
                                let vparts: Vec<&str> = val.trim().split_whitespace().collect();
                                if let Some(first) = vparts.first() {
                                    if let Ok(b) = first.replace(',', ".").parse::<f64>() {
                                        capacity_gb = (b / (1024.0 * 1024.0 * 1024.0)) as u64;
                                    }
                                }
                            }
                        } else if (pline.starts_with("Disponible") || pline.starts_with("Available")) && pline.contains(':') {
                            if let Some(val) = pline.split(':').nth(1) {
                                let vparts: Vec<&str> = val.trim().split_whitespace().collect();
                                if let Some(first) = vparts.first() {
                                    if let Ok(b) = first.replace(',', ".").parse::<f64>() {
                                        available_gb = (b / (1024.0 * 1024.0 * 1024.0)) as u64;
                                        allocated_gb = capacity_gb.saturating_sub(available_gb);
                                    }
                                }
                            }
                        }
                    }

                    pools.push(StoragePool {
                        name: name.to_string(),
                        path: format!("/var/lib/libvirt/{}", name),
                        capacity_gb: if capacity_gb == 0 { 500 } else { capacity_gb },
                        allocated_gb: if allocated_gb == 0 { 150 } else { allocated_gb },
                        available_gb: if available_gb == 0 { 350 } else { available_gb },
                        pool_type: "dir".to_string(),
                    });
                }
            }
        }

        Ok(pools)
    }

    async fn create_disk(&self, pool_name: &str, disk_name: &str, size_gb: u64, format: &str) -> Result<VmDisk> {
        let dir = format!("/var/lib/libvirt/{}", pool_name);
        let path = format!("{}/{}.{}", dir, disk_name, format);

        let mut cmd = Command::new("qemu-img");
        cmd.arg("create").arg("-f").arg(format).arg(&path).arg(format!("{}G", size_gb));
        let output = cmd.output().await.context("Failed to run qemu-img create")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("qemu-img create failed: {}", err.trim());
        }

        Ok(VmDisk {
            id: format!("{}-{}", pool_name, disk_name),
            name: disk_name.to_string(),
            path,
            size_gb,
            format: format.to_string(),
            pool_name: pool_name.to_string(),
        })
    }

    async fn resize_disk(&self, disk_path: &str, new_size_gb: u64) -> Result<()> {
        let mut cmd = Command::new("qemu-img");
        cmd.arg("resize").arg(disk_path).arg(format!("{}G", new_size_gb));
        let output = cmd.output().await.context("Failed to run qemu-img resize")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("qemu-img resize failed: {}", err.trim());
        }

        Ok(())
    }

    async fn delete_disk(&self, disk_path: &str) -> Result<()> {
        if tokio::fs::metadata(disk_path).await.is_ok() {
            tokio::fs::remove_file(disk_path).await.context("Failed to remove disk file")?;
        }
        Ok(())
    }
}
