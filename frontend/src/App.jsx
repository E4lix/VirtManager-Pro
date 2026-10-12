import React, { useState, useEffect } from 'react';
import {
  Server,
  Cpu,
  HardDrive,
  Activity,
  Layers,
  Users,
  Terminal,
  Play,
  Square,
  RotateCcw,
  Copy,
  UserCheck,
  Trash2,
  Plus,
  Monitor,
  ExternalLink,
  Shield,
  Box,
  CheckCircle,
  Clock,
  AlertTriangle,
  Folder,
  ChevronDown,
  ChevronRight,
} from 'lucide-react';
import { api } from './api';
import VncViewer from './components/VncViewer';

export default function App() {
  const [currentTab, setCurrentTab] = useState('dashboard');
  const [currentUser, setCurrentUser] = useState({
    username: 'admin',
    role: 'admin',
    token: 'mock-admin-token',
  });

  const [hypervisors, setHypervisors] = useState([]);
  const [vms, setVms] = useState([]);
  const [disks, setDisks] = useState([]);
  const [pools, setPools] = useState([]);
  const [templates, setTemplates] = useState([]);
  const [tasks, setTasks] = useState([]);
  const [usersList, setUsersList] = useState([]);
  const [selectedVmId, setSelectedVmId] = useState(null);
  const [expandedNodes, setExpandedNodes] = useState({ 'local-kvm': true, 'remote-pve-cluster': true });
  const [activeVmTab, setActiveVmTab] = useState('summary');
  const [loading, setLoading] = useState(true);

  // Derived selectedVm - always stays on the user's selected VM
  const selectedVm = vms.find((v) => v.id === selectedVmId) || (vms.length > 0 ? vms[0] : null);

  // Modals state
  const [modalCreateVm, setModalCreateVm] = useState(false);
  const [modalAddHv, setModalAddHv] = useState(false);
  const [modalCloneVm, setModalCloneVm] = useState(false);
  const [modalAssignVm, setModalAssignVm] = useState(false);
  const [modalCreateDisk, setModalCreateDisk] = useState(false);
  const [modalResizeDisk, setModalResizeDisk] = useState(false);
  const [modalDeployTemplate, setModalDeployTemplate] = useState(false);
  const [modalAttachDisk, setModalAttachDisk] = useState(false);
  const [formAttachDisk, setFormAttachDisk] = useState({ disk_path: '', target_dev: 'vdb' });
  const [activeTaskLog, setActiveTaskLog] = useState(null);

  // Form states
  const [formVm, setFormVm] = useState({
    name: '',
    hypervisor_id: 'local-kvm',
    vcpus: 2,
    memory_mb: 2048,
    disk_size_gb: 20,
    os_type: 'Debian 12',
    template_id: 'tpl-debian',
    assigned_to: '',
  });

  const [formHv, setFormHv] = useState({
    name: '',
    hypervisor_type: 'kvm',
    endpoint: 'qemu:///system',
    is_local: true,
  });

  const [formClone, setFormClone] = useState({ new_name: '', assigned_to: '' });
  const [formAssign, setFormAssign] = useState({ username: '' });
  const [formDisk, setFormDisk] = useState({
    name: '',
    pool_name: 'images',
    size_gb: 20,
    format: 'qcow2',
  });
  const [formResize, setFormResize] = useState({ disk_path: '', new_size_gb: 30 });
  const [formDeployTpl, setFormDeployTpl] = useState({
    template_id: '',
    vm_name: '',
    hypervisor_id: 'local-kvm',
    assigned_to: '',
  });

  // Fetch data
  const refreshAll = async () => {
    try {
      const [hRes, vRes, dRes, pRes, tRes, taskRes, uRes] = await Promise.all([
        api.listHypervisors(),
        api.listVms(currentUser.token),
        api.listDisks(),
        api.listStoragePools(),
        api.listTemplates(),
        api.listTasks(),
        api.listUsers(),
      ]);

      if (hRes.data) setHypervisors(hRes.data);
      if (vRes.data) {
        setVms(vRes.data);
        setSelectedVmId((prevId) => {
          if (prevId && vRes.data.some((v) => v.id === prevId)) {
            return prevId;
          }
          return vRes.data.length > 0 ? vRes.data[0].id : null;
        });
      }
      if (dRes.data) setDisks(dRes.data);
      if (pRes.data) setPools(pRes.data);
      if (tRes.data) setTemplates(tRes.data);
      if (taskRes.data) setTasks(taskRes.data);
      if (uRes.data) setUsersList(uRes.data);
    } catch (e) {
      console.error('Error refreshing state:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refreshAll();
    const interval = setInterval(refreshAll, 5000);
    return () => clearInterval(interval);
  }, [currentUser.role]);

  // Aggregate stats
  const totalVms = vms.length;
  const runningVms = vms.filter((v) => v.state === 'running').length;
  const totalCores = hypervisors.reduce((acc, h) => acc + (h.stats?.cpu_cores || 0), 0);
  const totalMemoryGb = Math.round(
    hypervisors.reduce((acc, h) => acc + (h.stats?.memory_total_mb || 0), 0) / 1024
  );
  const usedMemoryGb = Math.round(
    hypervisors.reduce((acc, h) => acc + (h.stats?.memory_used_mb || 0), 0) / 1024
  );
  const totalDiskGb = hypervisors.reduce((acc, h) => acc + (h.stats?.disk_total_gb || 0), 0);
  const usedDiskGb = hypervisors.reduce((acc, h) => acc + (h.stats?.disk_used_gb || 0), 0);

  // Actions
  const handleStartVm = async (id) => {
    await api.startVm(id);
    refreshAll();
  };

  const handleStopVm = async (id) => {
    await api.stopVm(id);
    refreshAll();
  };

  const handleRebootVm = async (id) => {
    await api.rebootVm(id);
    refreshAll();
  };

  const handleDeleteVm = async (id) => {
    if (confirm(`Confirmez-vous la suppression de la VM '${id}' ?`)) {
      await api.deleteVm(id);
      setSelectedVmId(null);
      refreshAll();
    }
  };

  const handleCreateVmSubmit = async (e) => {
    e.preventDefault();
    await api.createVm({
      name: formVm.name,
      hypervisor_id: formVm.hypervisor_id,
      vcpus: Number(formVm.vcpus),
      memory_mb: Number(formVm.memory_mb),
      disk_size_gb: Number(formVm.disk_size_gb),
      disk_format: 'qcow2',
      template_id: formVm.template_id || null,
      os_type: formVm.os_type,
      assigned_to: formVm.assigned_to || null,
    });
    setModalCreateVm(false);
    refreshAll();
  };

  const handleAttachDiskSubmit = async (e) => {
    e.preventDefault();
    if (!selectedVm || !formAttachDisk.disk_path) return;
    await api.attachDiskToVm(selectedVm.id, formAttachDisk.disk_path, formAttachDisk.target_dev);
    setModalAttachDisk(false);
    refreshAll();
  };

  const handleDetachDisk = async (targetDev) => {
    if (!selectedVm) return;
    if (confirm(`Confirmez-vous le détachement du disque '${targetDev}' de la VM '${selectedVm.name}' ?`)) {
      await api.detachDiskFromVm(selectedVm.id, targetDev);
      refreshAll();
    }
  };

  const handleAddHvSubmit = async (e) => {
    e.preventDefault();
    await api.createHypervisor(formHv, currentUser.token);
    setModalAddHv(false);
    refreshAll();
  };

  const handleDeleteHv = async (id) => {
    if (confirm(`Retirer l'hyperviseur ${id} du pool de ressources ?`)) {
      const res = await api.deleteHypervisor(id, currentUser.token);
      if (!res.success) {
        alert(res.message);
      } else {
        refreshAll();
      }
    }
  };

  const handleCloneSubmit = async (e) => {
    e.preventDefault();
    if (!selectedVm) return;
    await api.cloneVm(selectedVm.id, formClone.new_name, formClone.assigned_to || null);
    setModalCloneVm(false);
    refreshAll();
  };

  const handleAssignSubmit = async (e) => {
    e.preventDefault();
    if (!selectedVm) return;
    await api.assignVm(selectedVm.id, formAssign.username);
    setModalAssignVm(false);
    refreshAll();
  };

  const handleCreateDiskSubmit = async (e) => {
    e.preventDefault();
    await api.createDisk({
      name: formDisk.name,
      pool_name: formDisk.pool_name,
      size_gb: Number(formDisk.size_gb),
      format: formDisk.format,
    });
    setModalCreateDisk(false);
    refreshAll();
  };

  const handleResizeDiskSubmit = async (e) => {
    e.preventDefault();
    await api.resizeDisk(formResize.disk_path, Number(formResize.new_size_gb));
    setModalResizeDisk(false);
    refreshAll();
  };

  const handleTriggerBuild = async (templateId) => {
    const res = await api.triggerTemplateBuild(templateId);
    if (res.data) {
      setActiveTaskLog(res.data);
      refreshAll();
    }
  };

  const handleDeployTemplateSubmit = async (e) => {
    e.preventDefault();
    await api.deployTemplate(formDeployTpl.template_id, {
      vm_name: formDeployTpl.vm_name,
      hypervisor_id: formDeployTpl.hypervisor_id,
      assigned_to: formDeployTpl.assigned_to || null,
    });
    setModalDeployTemplate(false);
    refreshAll();
  };

  return (
    <div className="app-container">
      {/* Top Navbar */}
      <header className="top-navbar">
        <div className="brand">
          <Server size={22} color="#38bdf8" />
          <span>VirtManager Pro</span>
          <span className="brand-badge">Proxmox Cloud Suite</span>
        </div>

        <div className="nav-actions">
          <button
            className="btn btn-primary btn-sm"
            onClick={() => setModalCreateVm(true)}
          >
            <Plus size={15} />
            Nouvelle VM
          </button>

          <button
            className="btn btn-secondary btn-sm"
            onClick={() => setModalAddHv(true)}
          >
            <Server size={15} />
            + Hyperviseur
          </button>

          <a
            href="/swagger-ui/"
            target="_blank"
            rel="noreferrer"
            className="btn btn-secondary btn-sm"
            style={{ textDecoration: 'none' }}
          >
            <Terminal size={15} />
            API Swagger
            <ExternalLink size={12} />
          </a>

          {/* User & Role Switcher for demonstration */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              background: 'var(--bg-card)',
              padding: '0.25rem 0.75rem',
              borderRadius: '6px',
              border: '1px solid var(--border-color)',
              fontSize: '0.8rem',
            }}
          >
            <Shield size={14} color="#38bdf8" />
            <select
              value={currentUser.role}
              onChange={(e) => {
                const role = e.target.value;
                setCurrentUser({
                  username: role === 'admin' ? 'admin' : 'user1',
                  role,
                  token: role === 'admin' ? 'mock-admin-token' : 'mock-user-token',
                });
              }}
              style={{
                background: 'transparent',
                color: 'white',
                border: 'none',
                outline: 'none',
                cursor: 'pointer',
                fontWeight: 600,
              }}
            >
              <option value="admin" style={{ background: '#111827' }}>
                Mode: Administrateur (admin)
              </option>
              <option value="user" style={{ background: '#111827' }}>
                Mode: Étudiant 5A (user1)
              </option>
            </select>
          </div>
        </div>
      </header>

      {/* Main Layout */}
      <div className="main-layout">
        {/* Sidebar */}
        <aside className="sidebar">
          <div className="sidebar-section-title">Datacenter & Vues</div>

          <div
            className={`nav-item ${currentTab === 'dashboard' ? 'active' : ''}`}
            onClick={() => setCurrentTab('dashboard')}
          >
            <Activity size={18} />
            <span>Vue d'ensemble</span>
          </div>

          <div
            className={`nav-item ${currentTab === 'vms' ? 'active' : ''}`}
            onClick={() => setCurrentTab('vms')}
          >
            <Box size={18} />
            <span>Toutes les VM</span>
            <span className="nav-item-badge">{vms.length}</span>
          </div>

          <div
            className={`nav-item ${currentTab === 'hypervisors' ? 'active' : ''}`}
            onClick={() => setCurrentTab('hypervisors')}
          >
            <Server size={18} />
            <span>Hyperviseurs</span>
            <span className="nav-item-badge">{hypervisors.length}</span>
          </div>

          <div
            className={`nav-item ${currentTab === 'storage' ? 'active' : ''}`}
            onClick={() => setCurrentTab('storage')}
          >
            <HardDrive size={18} />
            <span>Stockage & Disques</span>
            <span className="nav-item-badge">{disks.length}</span>
          </div>

          {/* Proxmox-Style Resource Tree: Nodes & VMs in the Nav */}
          <div className="sidebar-section-title">Arborescence Datacenter</div>
          <div style={{ padding: '0 0.5rem', marginBottom: '0.5rem' }}>
            {hypervisors.map((hv) => {
              const hvVms = vms.filter((v) => v.hypervisor_id === hv.id);
              const isExpanded = expandedNodes[hv.id] !== false;
              return (
                <div key={hv.id} style={{ marginBottom: '0.4rem' }}>
                  {/* Node Header */}
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '0.35rem',
                      padding: '0.4rem 0.5rem',
                      borderRadius: '5px',
                      cursor: 'pointer',
                      fontSize: '0.785rem',
                      fontWeight: 600,
                      color: 'var(--text-main)',
                      background: 'rgba(255, 255, 255, 0.03)',
                    }}
                    onClick={() => {
                      setExpandedNodes((prev) => ({ ...prev, [hv.id]: !isExpanded }));
                    }}
                  >
                    {isExpanded ? <ChevronDown size={13} color="#94a3b8" /> : <ChevronRight size={13} color="#94a3b8" />}
                    <Server size={14} color="#38bdf8" />
                    <span
                      style={{
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                        flex: 1,
                      }}
                      title={hv.name}
                    >
                      {hv.name}
                    </span>
                    <span className="nav-item-badge" style={{ fontSize: '0.65rem', padding: '0.05rem 0.35rem' }}>
                      {hvVms.length}
                    </span>
                  </div>

                  {/* Nested VM Items */}
                  {isExpanded && (
                    <div style={{ paddingLeft: '0.75rem', marginTop: '0.2rem' }}>
                      {hvVms.map((vm) => {
                        const isVmActive = selectedVmId === vm.id && currentTab === 'vms';
                        return (
                          <div
                            key={vm.id}
                            className={`nav-item ${isVmActive ? 'active' : ''}`}
                            style={{
                              padding: '0.35rem 0.5rem',
                              fontSize: '0.765rem',
                              gap: '0.45rem',
                              borderRadius: '4px',
                              marginBottom: '0.15rem',
                              borderLeft: isVmActive ? '3px solid var(--primary)' : '3px solid transparent',
                            }}
                            onClick={() => {
                              setSelectedVmId(vm.id);
                              setCurrentTab('vms');
                            }}
                          >
                            <span
                              className={`status-dot ${
                                vm.state === 'running'
                                  ? 'badge-running'
                                  : vm.state === 'stopped'
                                  ? 'badge-stopped'
                                  : 'badge-paused'
                              }`}
                              style={{ width: '6px', height: '6px', flexShrink: 0 }}
                            />
                            <Box size={13} style={{ flexShrink: 0, color: isVmActive ? '#38bdf8' : '#94a3b8' }} />
                            <span
                              style={{
                                overflow: 'hidden',
                                textOverflow: 'ellipsis',
                                whiteSpace: 'nowrap',
                              }}
                              title={vm.name}
                            >
                              {vm.name}
                            </span>
                          </div>
                        );
                      })}
                      {hvVms.length === 0 && (
                        <div
                          style={{
                            padding: '0.25rem 0.5rem',
                            fontSize: '0.7rem',
                            color: 'var(--text-dark)',
                          }}
                        >
                          Aucune VM sur cet hôte
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>

          <div className="sidebar-section-title">Automatisation</div>

          <div
            className={`nav-item ${currentTab === 'templates' ? 'active' : ''}`}
            onClick={() => setCurrentTab('templates')}
          >
            <Layers size={18} />
            <span>Packer & Ansible</span>
            <span className="nav-item-badge">{templates.length}</span>
          </div>

          <div
            className={`nav-item ${currentTab === 'tasks' ? 'active' : ''}`}
            onClick={() => setCurrentTab('tasks')}
          >
            <Clock size={18} />
            <span>Tâches & Builds</span>
            <span className="nav-item-badge">{tasks.length}</span>
          </div>

          <div className="sidebar-section-title">Administration</div>

          <div
            className={`nav-item ${currentTab === 'users' ? 'active' : ''}`}
            onClick={() => setCurrentTab('users')}
          >
            <Users size={18} />
            <span>Utilisateurs & Accès</span>
            <span className="nav-item-badge">{usersList.length}</span>
          </div>

          <div
            className="nav-item"
            onClick={() => window.open('/swagger-ui/', '_blank')}
          >
            <Terminal size={18} />
            <span>Documentation REST</span>
            <ExternalLink size={14} style={{ marginLeft: 'auto' }} />
          </div>
        </aside>

        {/* Content Area */}
        <main className="content-area">
          {/* TAB: DASHBOARD */}
          {currentTab === 'dashboard' && (
            <div>
              <div className="page-header">
                <div>
                  <h1 className="page-title">Datacenter Overview</h1>
                  <p className="page-subtitle">
                    Gestion centralisée de l'infrastructure virtualisée locale et distante
                  </p>
                </div>
              </div>

              {/* Metrics Grid */}
              <div className="metrics-grid">
                <div className="metric-card">
                  <div className="metric-header">
                    <span>Machines Virtuelles</span>
                    <Box size={16} />
                  </div>
                  <div className="metric-value">
                    {runningVms} <span style={{ fontSize: '1rem', color: '#64748b' }}>/ {totalVms}</span>
                  </div>
                  <div className="metric-footer">
                    <span className="badge badge-running">
                      <span className="status-dot"></span> {runningVms} Actives
                    </span>
                    <span>{totalVms - runningVms} Éteintes</span>
                  </div>
                </div>

                <div className="metric-card">
                  <div className="metric-header">
                    <span>Cœurs vCPU Alloués</span>
                    <Cpu size={16} />
                  </div>
                  <div className="metric-value">{totalCores} Cœurs</div>
                  <div className="metric-footer">
                    Sur {hypervisors.length} hyperviseurs actifs
                  </div>
                </div>

                <div className="metric-card">
                  <div className="metric-header">
                    <span>Mémoire RAM Totale</span>
                    <Activity size={16} />
                  </div>
                  <div className="metric-value">
                    {usedMemoryGb} <span style={{ fontSize: '1rem', color: '#64748b' }}>/ {totalMemoryGb} GB</span>
                  </div>
                  <div className="progress-bar-bg">
                    <div
                      className="progress-bar-fill"
                      style={{
                        width: `${Math.min(100, (usedMemoryGb / (totalMemoryGb || 1)) * 100)}%`,
                        backgroundColor: '#38bdf8',
                      }}
                    ></div>
                  </div>
                </div>

                <div className="metric-card">
                  <div className="metric-header">
                    <span>Stockage Virtuel</span>
                    <HardDrive size={16} />
                  </div>
                  <div className="metric-value">
                    {usedDiskGb} <span style={{ fontSize: '1rem', color: '#64748b' }}>/ {totalDiskGb} GB</span>
                  </div>
                  <div className="progress-bar-bg">
                    <div
                      className="progress-bar-fill"
                      style={{
                        width: `${Math.min(100, (usedDiskGb / (totalDiskGb || 1)) * 100)}%`,
                        backgroundColor: '#10b981',
                      }}
                    ></div>
                  </div>
                </div>
              </div>

              {/* Hypervisors Summary */}
              <div className="panel-card">
                <div className="panel-card-header">
                  <div className="panel-card-title">
                    <Server size={18} />
                    Nœuds Hyperviseurs du Datacenter
                  </div>
                  <button
                    className="btn btn-secondary btn-sm"
                    onClick={() => setModalAddHv(true)}
                  >
                    <Plus size={14} />
                    Ajouter un Nœud
                  </button>
                </div>
                <div className="table-responsive">
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Nom du Nœud</th>
                        <th>Type</th>
                        <th>URI Endpoint</th>
                        <th>État</th>
                        <th>CPU Cores</th>
                        <th>RAM Utilisée</th>
                        <th>Stockage</th>
                        <th>VM Actives</th>
                      </tr>
                    </thead>
                    <tbody>
                      {hypervisors.map((hv) => (
                        <tr key={hv.id}>
                          <td style={{ fontWeight: 600 }}>{hv.name}</td>
                          <td>
                            <span
                              style={{
                                background: '#1e293b',
                                padding: '0.2rem 0.5rem',
                                borderRadius: '4px',
                                fontSize: '0.75rem',
                                textTransform: 'uppercase',
                              }}
                            >
                              {hv.hypervisor_type}
                            </span>
                          </td>
                          <td style={{ fontFamily: 'var(--font-mono)', fontSize: '0.8rem' }}>
                            {hv.endpoint}
                          </td>
                          <td>
                            <span
                              className={`badge ${
                                hv.status === 'online' ? 'badge-running' : 'badge-stopped'
                              }`}
                            >
                              <span className="status-dot"></span>
                              {hv.status}
                            </span>
                          </td>
                          <td>{hv.stats?.cpu_cores || '-'}</td>
                          <td>
                            {Math.round((hv.stats?.memory_used_mb || 0) / 1024)} /{' '}
                            {Math.round((hv.stats?.memory_total_mb || 0) / 1024)} GB
                          </td>
                          <td>
                            {hv.stats?.disk_used_gb || 0} / {hv.stats?.disk_total_gb || 0} GB
                          </td>
                          <td style={{ fontWeight: 600 }}>
                            {hv.stats?.active_vms || 0} / {hv.stats?.total_vms || 0}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>

              {/* Recent Virtual Machines */}
              <div className="panel-card">
                <div className="panel-card-header">
                  <div className="panel-card-title">
                    <Box size={18} />
                    Machines Virtuelles Récemment Déployées
                  </div>
                  <button
                    className="btn btn-primary btn-sm"
                    onClick={() => setCurrentTab('vms')}
                  >
                    Voir Toutes les VM
                  </button>
                </div>
                <div className="table-responsive">
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Nom</th>
                        <th>Hyperviseur</th>
                        <th>État</th>
                        <th>vCPUs</th>
                        <th>RAM</th>
                        <th>Disques</th>
                        <th>Attribué à</th>
                        <th>Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {vms.slice(0, 5).map((vm) => (
                        <tr key={vm.id}>
                          <td style={{ fontWeight: 600 }}>{vm.name}</td>
                          <td>{vm.hypervisor_name}</td>
                          <td>
                            <span
                              className={`badge ${
                                vm.state === 'running'
                                  ? 'badge-running'
                                  : vm.state === 'stopped'
                                  ? 'badge-stopped'
                                  : 'badge-paused'
                              }`}
                            >
                              <span className="status-dot"></span>
                              {vm.state}
                            </span>
                          </td>
                          <td>{vm.vcpus} vCPU</td>
                          <td>{vm.memory_mb} MB</td>
                          <td>{vm.disks.length} disque(s)</td>
                          <td>
                            {vm.assigned_to ? (
                              <span className="badge badge-ready">
                                <UserCheck size={12} /> {vm.assigned_to}
                              </span>
                            ) : (
                              <span style={{ color: 'var(--text-dark)' }}>Non assigné</span>
                            )}
                          </td>
                          <td>
                            <button
                              className="btn btn-secondary btn-sm"
                              onClick={() => {
                                setSelectedVmId(vm.id);
                                setCurrentTab('vms');
                                setActiveVmTab('console');
                              }}
                            >
                              <Monitor size={14} /> Console VNC
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          )}

          {/* TAB: VMS (GESTIONNAIRE DE VM) */}
          {currentTab === 'vms' && (
            <div>
              <div className="page-header">
                <div>
                  <h1 className="page-title">Gestionnaire de Machines Virtuelles</h1>
                  <p className="page-subtitle">
                    Cycle de vie, console VNC en direct, disques virtuels et attribution utilisateur
                  </p>
                </div>
                <button
                  className="btn btn-primary"
                  onClick={() => setModalCreateVm(true)}
                >
                  <Plus size={16} />
                  Créer une Machine Virtuelle
                </button>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '320px 1fr', gap: '1.25rem' }}>
                {/* VM Selector List */}
                <div className="panel-card" style={{ height: 'calc(100vh - 180px)', overflowY: 'auto' }}>
                  <div className="panel-card-header">
                    <span style={{ fontWeight: 600, fontSize: '0.9rem' }}>Liste des VM ({vms.length})</span>
                  </div>
                  <div>
                    {vms.map((vm) => (
                      <div
                        key={vm.id}
                        onClick={() => setSelectedVmId(vm.id)}
                        style={{
                          padding: '0.85rem 1rem',
                          borderBottom: '1px solid var(--border-color)',
                          cursor: 'pointer',
                          backgroundColor:
                            selectedVm?.id === vm.id ? 'var(--bg-card-hover)' : 'transparent',
                          borderLeft:
                            selectedVm?.id === vm.id ? '3px solid var(--primary)' : '3px solid transparent',
                        }}
                      >
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                          <span style={{ fontWeight: 600, fontSize: '0.9rem' }}>{vm.name}</span>
                          <span
                            className={`badge ${
                              vm.state === 'running'
                                ? 'badge-running'
                                : vm.state === 'stopped'
                                ? 'badge-stopped'
                                : 'badge-paused'
                            }`}
                          >
                            <span className="status-dot"></span>
                            {vm.state}
                          </span>
                        </div>
                        <div
                          style={{
                            display: 'flex',
                            gap: '0.75rem',
                            fontSize: '0.75rem',
                            color: 'var(--text-muted)',
                            marginTop: '0.35rem',
                          }}
                        >
                          <span>{vm.vcpus} vCPU</span>
                          <span>•</span>
                          <span>{vm.memory_mb} MB RAM</span>
                          {vm.assigned_to && (
                            <>
                              <span>•</span>
                              <span style={{ color: '#38bdf8' }}>@{vm.assigned_to}</span>
                            </>
                          )}
                        </div>
                      </div>
                    ))}
                    {vms.length === 0 && (
                      <div style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-muted)' }}>
                        Aucune VM trouvée. Créez-en une nouvelle !
                      </div>
                    )}
                  </div>
                </div>

                {/* VM Detailed Control View */}
                {selectedVm ? (
                  <div className="panel-card" style={{ height: 'calc(100vh - 180px)', display: 'flex', flexDirection: 'column' }}>
                    {/* VM Top Controls Bar */}
                    <div
                      style={{
                        padding: '1rem 1.25rem',
                        borderBottom: '1px solid var(--border-color)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        background: 'rgba(0,0,0,0.2)',
                      }}
                    >
                      <div>
                        <h2 style={{ fontSize: '1.25rem', fontWeight: 700 }}>{selectedVm.name}</h2>
                        <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                          Hôte : {selectedVm.hypervisor_name} • Port VNC : {selectedVm.vnc_port || 5900}
                        </span>
                      </div>

                      {/* Power & Lifecycle Buttons */}
                      <div style={{ display: 'flex', gap: '0.5rem' }}>
                        {selectedVm.state === 'stopped' ? (
                          <button
                            className="btn btn-success btn-sm"
                            onClick={() => handleStartVm(selectedVm.id)}
                          >
                            <Play size={14} /> Démarrer
                          </button>
                        ) : (
                          <button
                            className="btn btn-danger btn-sm"
                            onClick={() => handleStopVm(selectedVm.id)}
                          >
                            <Square size={14} /> Arrêter
                          </button>
                        )}

                        <button
                          className="btn btn-secondary btn-sm"
                          onClick={() => handleRebootVm(selectedVm.id)}
                        >
                          <RotateCcw size={14} /> Redémarrer
                        </button>

                        <button
                          className="btn btn-secondary btn-sm"
                          onClick={() => {
                            setFormClone({ new_name: `${selectedVm.name}-clone`, assigned_to: '' });
                            setModalCloneVm(true);
                          }}
                        >
                          <Copy size={14} /> Cloner
                        </button>

                        <button
                          className="btn btn-secondary btn-sm"
                          onClick={() => {
                            setFormAssign({ username: selectedVm.assigned_to || 'user1' });
                            setModalAssignVm(true);
                          }}
                        >
                          <UserCheck size={14} /> Attribuer
                        </button>

                        <button
                          className="btn btn-danger btn-sm"
                          onClick={() => handleDeleteVm(selectedVm.id)}
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </div>

                    {/* VM Tabs header */}
                    <div style={{ display: 'flex', borderBottom: '1px solid var(--border-color)', padding: '0 1rem' }}>
                      <button
                        className={`nav-item ${activeVmTab === 'summary' ? 'active' : ''}`}
                        onClick={() => setActiveVmTab('summary')}
                        style={{ borderBottom: 'none' }}
                      >
                        Résumé
                      </button>
                      <button
                        className={`nav-item ${activeVmTab === 'console' ? 'active' : ''}`}
                        onClick={() => setActiveVmTab('console')}
                        style={{ borderBottom: 'none' }}
                      >
                        Console VNC
                      </button>
                      <button
                        className={`nav-item ${activeVmTab === 'hardware' ? 'active' : ''}`}
                        onClick={() => setActiveVmTab('hardware')}
                        style={{ borderBottom: 'none' }}
                      >
                        Matériel
                      </button>
                    </div>

                    {/* Tab contents */}
                    <div style={{ flex: 1, overflowY: 'auto', padding: '1.25rem' }}>
                      {activeVmTab === 'summary' && (
                        <div>
                          <div className="metrics-grid">
                            <div className="metric-card">
                              <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>État de la VM</span>
                              <span style={{ fontSize: '1.25rem', fontWeight: 700, marginTop: '0.25rem' }}>
                                <span
                                  className={`badge ${
                                    selectedVm.state === 'running' ? 'badge-running' : 'badge-stopped'
                                  }`}
                                >
                                  <span className="status-dot"></span> {selectedVm.state.toUpperCase()}
                                </span>
                              </span>
                            </div>

                            <div className="metric-card">
                              <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Processeurs</span>
                              <span style={{ fontSize: '1.25rem', fontWeight: 700, marginTop: '0.25rem' }}>
                                {selectedVm.vcpus} vCPU
                              </span>
                            </div>

                            <div className="metric-card">
                              <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Mémoire Vive</span>
                              <span style={{ fontSize: '1.25rem', fontWeight: 700, marginTop: '0.25rem' }}>
                                {selectedVm.memory_mb} MB
                              </span>
                            </div>

                            <div className="metric-card">
                              <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Attribution</span>
                              <span style={{ fontSize: '1.25rem', fontWeight: 700, marginTop: '0.25rem' }}>
                                {selectedVm.assigned_to ? `@${selectedVm.assigned_to}` : 'Libre'}
                              </span>
                            </div>
                          </div>

                          <div className="panel-card">
                            <div className="panel-card-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                              <span style={{ fontWeight: 600 }}>Disques Virtuels Associés</span>
                              <button
                                className="btn btn-primary btn-sm"
                                onClick={() => {
                                  setFormAttachDisk({
                                    disk_path: disks[0]?.path || '/var/lib/libvirt/images/',
                                    target_dev: `vd${String.fromCharCode(97 + selectedVm.disks.length)}`,
                                  });
                                  setModalAttachDisk(true);
                                }}
                              >
                                <Plus size={14} /> Attacher un Disque
                              </button>
                            </div>
                            <table className="data-table">
                              <thead>
                                <tr>
                                  <th>Cible</th>
                                  <th>Chemin Image</th>
                                  <th>Taille</th>
                                  <th>Format</th>
                                  <th>Pool</th>
                                  <th>Actions</th>
                                </tr>
                              </thead>
                              <tbody>
                                {selectedVm.disks.map((d) => (
                                  <tr key={d.id}>
                                    <td style={{ fontWeight: 600 }}>{d.name}</td>
                                    <td style={{ fontFamily: 'var(--font-mono)', fontSize: '0.8rem' }}>{d.path}</td>
                                    <td>{d.size_gb} GB</td>
                                    <td>{d.format}</td>
                                    <td>{d.pool_name}</td>
                                    <td>
                                      {d.name !== 'vda' ? (
                                        <button
                                          className="btn btn-danger btn-sm"
                                          style={{ padding: '2px 8px', fontSize: '0.75rem' }}
                                          onClick={() => handleDetachDisk(d.name)}
                                        >
                                          Détacher
                                        </button>
                                      ) : (
                                        <span style={{ color: 'var(--text-muted)', fontSize: '0.75rem' }}>Disque Système</span>
                                      )}
                                    </td>
                                  </tr>
                                ))}
                                {selectedVm.disks.length === 0 && (
                                  <tr>
                                    <td colSpan="6" style={{ textAlign: 'center', color: 'var(--text-muted)' }}>
                                      Disque standard qcow2 assigné par défaut
                                    </td>
                                  </tr>
                                )}
                              </tbody>
                            </table>
                          </div>
                        </div>
                      )}

                      {activeVmTab === 'console' && (
                        selectedVm.state === 'running' ? (
                          <VncViewer vmId={selectedVm.id} vmName={selectedVm.name} />
                        ) : (
                          <div
                            className="panel-card"
                            style={{
                              display: 'flex',
                              flexDirection: 'column',
                              alignItems: 'center',
                              justifyContent: 'center',
                              minHeight: '380px',
                              gap: '1rem',
                              background: '#0b1120',
                              border: '1px solid #1e293b',
                              borderRadius: '8px',
                              padding: '2rem',
                            }}
                          >
                            <div
                              style={{
                                width: '64px',
                                height: '64px',
                                borderRadius: '50%',
                                background: 'rgba(239, 68, 68, 0.12)',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                color: '#ef4444',
                              }}
                            >
                              <Square size={32} />
                            </div>
                            <h3 style={{ fontSize: '1.25rem', fontWeight: 700, color: '#f8fafc' }}>
                              Machine Virtuelle Éteinte
                            </h3>
                            <p style={{ color: '#94a3b8', maxWidth: '420px', textAlign: 'center', fontSize: '0.875rem' }}>
                              La console VNC est inaccessible car le domaine QEMU/KVM est arrêté. Démarrez la machine pour ouvrir la session graphique en direct.
                            </p>
                            <button
                              className="btn btn-success"
                              onClick={() => handleStartVm(selectedVm.id)}
                              style={{ display: 'flex', alignItems: 'center', gap: '8px' }}
                            >
                              <Play size={16} /> Démarrer {selectedVm.name}
                            </button>
                          </div>
                        )
                      )}

                      {activeVmTab === 'hardware' && (
                        <div className="panel-card">
                          <table className="data-table">
                            <thead>
                              <tr>
                                <th>Composant</th>
                                <th>Spécification</th>
                              </tr>
                            </thead>
                            <tbody>
                              <tr>
                                <td style={{ fontWeight: 600 }}>Processeur (CPU)</td>
                                <td>{selectedVm.vcpus} cœurs logiques virtuels (KVM Accelerator)</td>
                              </tr>
                              <tr>
                                <td style={{ fontWeight: 600 }}>Mémoire vive (RAM)</td>
                                <td>{selectedVm.memory_mb} MB alloués</td>
                              </tr>
                              <tr>
                                <td style={{ fontWeight: 600 }}>Contrôleur Graphique</td>
                                <td>VNC / SPICE avec écouteur réseau 0.0.0.0 (Port : {selectedVm.vnc_port || 5900})</td>
                              </tr>
                              <tr>
                                <td style={{ fontWeight: 600 }}>Interface Réseau</td>
                                <td>VirtIO Network Adapter (Réseau NAT 'default')</td>
                              </tr>
                            </tbody>
                          </table>
                        </div>
                      )}
                    </div>
                  </div>
                ) : (
                  <div className="panel-card" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                    <p style={{ color: 'var(--text-muted)' }}>Sélectionnez une machine virtuelle pour afficher les détails</p>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB: HYPERVISORS */}
          {currentTab === 'hypervisors' && (
            <div>
              <div className="page-header">
                <div>
                  <h1 className="page-title">Gestionnaire d'Hyperviseurs</h1>
                  <p className="page-subtitle">
                    Gestion des nœuds locaux et distants (KVM, libvirt SSH, Proxmox VE)
                  </p>
                </div>
                <button
                  className="btn btn-primary"
                  onClick={() => setModalAddHv(true)}
                >
                  <Plus size={16} />
                  Enregistrer un Hyperviseur
                </button>
              </div>

              <div className="panel-card">
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Nom du Nœud</th>
                      <th>Type</th>
                      <th>Endpoint de Connexion</th>
                      <th>Statut</th>
                      <th>CPU Cores</th>
                      <th>RAM Utilisée</th>
                      <th>Stockage Alloué</th>
                      <th>VM sur l'Hôte</th>
                      <th>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {hypervisors.map((hv) => (
                      <tr key={hv.id}>
                        <td style={{ fontWeight: 600 }}>{hv.name}</td>
                        <td>
                          <span
                            style={{
                              background: '#1e293b',
                              padding: '0.2rem 0.5rem',
                              borderRadius: '4px',
                              fontSize: '0.75rem',
                              textTransform: 'uppercase',
                            }}
                          >
                            {hv.hypervisor_type}
                          </span>
                        </td>
                        <td style={{ fontFamily: 'var(--font-mono)', fontSize: '0.8rem' }}>{hv.endpoint}</td>
                        <td>
                          <span
                            className={`badge ${
                              hv.status === 'online' ? 'badge-running' : 'badge-stopped'
                            }`}
                          >
                            <span className="status-dot"></span>
                            {hv.status}
                          </span>
                        </td>
                        <td>{hv.stats?.cpu_cores || 0}</td>
                        <td>
                          {Math.round((hv.stats?.memory_used_mb || 0) / 1024)} /{' '}
                          {Math.round((hv.stats?.memory_total_mb || 0) / 1024)} GB
                        </td>
                        <td>
                          {hv.stats?.disk_used_gb || 0} / {hv.stats?.disk_total_gb || 0} GB
                        </td>
                        <td>{hv.stats?.active_vms || 0} actives</td>
                        <td>
                          {!hv.is_local && (
                            <button
                              className="btn btn-danger btn-sm"
                              onClick={() => handleDeleteHv(hv.id)}
                            >
                              <Trash2 size={14} /> Retirer
                            </button>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* TAB: STORAGE & DISKS */}
          {currentTab === 'storage' && (
            <div>
              <div className="page-header">
                <div>
                  <h1 className="page-title">Stockage & Disques Virtuels</h1>
                  <p className="page-subtitle">
                    Gestion des pools libvirt, création et extension dynamique des images qcow2/raw
                  </p>
                </div>
                <button
                  className="btn btn-primary"
                  onClick={() => setModalCreateDisk(true)}
                >
                  <Plus size={16} />
                  Nouveau Disque Virtuel
                </button>
              </div>

              {/* Storage Pools Breakdown */}
              <div className="panel-card">
                <div className="panel-card-header">
                  <span style={{ fontWeight: 600 }}>Pools de Stockage Libvirt</span>
                </div>
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Nom du Pool</th>
                      <th>Chemin Répertoire</th>
                      <th>Type</th>
                      <th>Capacité Totale</th>
                      <th>Alloué</th>
                      <th>Disponible</th>
                    </tr>
                  </thead>
                  <tbody>
                    {pools.map((p) => (
                      <tr key={p.name}>
                        <td style={{ fontWeight: 600 }}>{p.name}</td>
                        <td style={{ fontFamily: 'var(--font-mono)', fontSize: '0.8rem' }}>{p.path}</td>
                        <td>{p.pool_type}</td>
                        <td>{p.capacity_gb} GB</td>
                        <td>{p.allocated_gb} GB</td>
                        <td style={{ color: '#10b981', fontWeight: 600 }}>{p.available_gb} GB</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              {/* Disks List */}
              <div className="panel-card">
                <div className="panel-card-header">
                  <span style={{ fontWeight: 600 }}>Images Disques Virtuelles ({disks.length})</span>
                </div>
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Nom du Fichier</th>
                      <th>Chemin Absolu</th>
                      <th>Taille</th>
                      <th>Format</th>
                      <th>Pool</th>
                      <th>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {disks.map((d) => (
                      <tr key={d.id}>
                        <td style={{ fontWeight: 600 }}>{d.name}</td>
                        <td style={{ fontFamily: 'var(--font-mono)', fontSize: '0.8rem' }}>{d.path}</td>
                        <td style={{ fontWeight: 600 }}>{d.size_gb} GB</td>
                        <td>
                          <span
                            style={{
                              background: '#1e293b',
                              padding: '0.2rem 0.5rem',
                              borderRadius: '4px',
                              fontSize: '0.75rem',
                              textTransform: 'uppercase',
                            }}
                          >
                            {d.format}
                          </span>
                        </td>
                        <td>{d.pool_name}</td>
                        <td>
                          <button
                            className="btn btn-secondary btn-sm"
                            onClick={() => {
                              setFormResize({ disk_path: d.path, new_size_gb: d.size_gb + 10 });
                              setModalResizeDisk(true);
                            }}
                          >
                            <HardDrive size={14} /> Étendre la taille
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* TAB: TEMPLATES & AUTOMATION */}
          {currentTab === 'templates' && (
            <div>
              <div className="page-header">
                <div>
                  <h1 className="page-title">Gestionnaire de Templates & Automatisation</h1>
                  <p className="page-subtitle">
                    Construction automatisée d'images via HashiCorp Packer et transformation logicielle via Ansible
                  </p>
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(360px, 1fr))', gap: '1.25rem' }}>
                {templates.map((tpl) => (
                  <div key={tpl.id} className="panel-card">
                    <div className="panel-card-header">
                      <span style={{ fontWeight: 700, fontSize: '1.05rem' }}>{tpl.name}</span>
                      <span className={`badge ${tpl.status === 'ready' ? 'badge-ready' : 'badge-building'}`}>
                        <span className="status-dot"></span>
                        {tpl.status.toUpperCase()}
                      </span>
                    </div>
                    <div style={{ padding: '1.25rem' }}>
                      <p style={{ color: 'var(--text-muted)', fontSize: '0.875rem', marginBottom: '1rem' }}>
                        {tpl.description}
                      </p>

                      <div style={{ background: '#090d17', padding: '0.75rem', borderRadius: '6px', fontSize: '0.8rem', marginBottom: '1rem', border: '1px solid var(--border-color)' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.35rem' }}>
                          <span style={{ color: 'var(--text-muted)' }}>Fichier Packer :</span>
                          <span style={{ fontFamily: 'var(--font-mono)', color: '#38bdf8' }}>{tpl.packer_file}</span>
                        </div>
                        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.35rem' }}>
                          <span style={{ color: 'var(--text-muted)' }}>Playbook Ansible :</span>
                          <span style={{ fontFamily: 'var(--font-mono)', color: '#10b981' }}>{tpl.ansible_playbook}</span>
                        </div>
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                          <span style={{ color: 'var(--text-muted)' }}>Ressources par défaut :</span>
                          <span>{tpl.default_vcpus} vCPU • {tpl.default_memory_mb} MB RAM • {tpl.default_disk_gb} GB</span>
                        </div>
                      </div>

                      <div style={{ display: 'flex', gap: '0.75rem' }}>
                        <button
                          className="btn btn-secondary btn-sm"
                          style={{ flex: 1 }}
                          onClick={() => handleTriggerBuild(tpl.id)}
                        >
                          <Terminal size={14} />
                          Lancer Build Packer/Ansible
                        </button>
                        <button
                          className="btn btn-primary btn-sm"
                          style={{ flex: 1 }}
                          onClick={() => {
                            setFormDeployTpl({
                              template_id: tpl.id,
                              vm_name: `vm-${tpl.os.toLowerCase()}-${Math.floor(Math.random() * 900 + 100)}`,
                              hypervisor_id: 'local-kvm',
                              assigned_to: '',
                            });
                            setModalDeployTemplate(true);
                          }}
                        >
                          <Play size={14} />
                          Déployer en 1 Clic
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>

              {/* SECTION TERRAFORM (IaC) */}
              <div className="panel-card" style={{ marginTop: '2rem' }}>
                <div className="panel-card-header" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                    <Layers size={18} style={{ color: '#7c3aed' }} />
                    <span style={{ fontWeight: 700, fontSize: '1.05rem' }}>
                      Infrastructure as Code avec HashiCorp Terraform (Provider Libvirt)
                    </span>
                  </div>
                  <span className="badge badge-ready">
                    <span className="status-dot"></span> TEMPLATES TERRAFORM PRÊTS
                  </span>
                </div>
                <div style={{ padding: '1.25rem' }}>
                  <p style={{ color: 'var(--text-muted)', fontSize: '0.875rem', marginBottom: '1rem' }}>
                    Provisionnement déclaratif d'instances KVM/QEMU avec configuration Cloud-Init automatique, attribution d'adresses IP et génération de clés SSH sans intervention manuelle.
                  </p>

                  <div style={{ background: '#090d17', borderRadius: '8px', padding: '1rem', border: '1px solid var(--border-color)', marginBottom: '1rem' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem', color: '#94a3b8', fontSize: '0.8rem' }}>
                      <span>templates/terraform/main.tf</span>
                      <span>HCL (HashiCorp Language)</span>
                    </div>
                    <pre style={{ margin: 0, fontFamily: 'var(--font-mono)', fontSize: '0.78rem', color: '#38bdf8', overflowX: 'auto', maxHeight: '180px' }}>
{`resource "libvirt_volume" "vm_os_disk" {
  name             = "\${var.vm_name}-disk.qcow2"
  pool             = "default"
  base_volume_name = "debian13.qcow2" # Copy-on-Write Golden Image
  size             = 21474836480       # 20 GB
}

resource "libvirt_domain" "vm" {
  name   = var.vm_name
  memory = var.memory_mb
  vcpu   = var.vcpus
  cloudinit = libvirt_cloudinit_disk.commoninit.id
  network_interface { network_name = "default" }
  disk { volume_id = libvirt_volume.vm_os_disk.id }
  graphics { type = "vnc" autoport = true }
}`}
                    </pre>
                  </div>

                  <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                    <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>
                      Fichiers disponibles : <code style={{ color: '#10b981' }}>templates/terraform/main.tf</code> • <code style={{ color: '#10b981' }}>variables.tf</code> • <code style={{ color: '#10b981' }}>outputs.tf</code>
                    </span>
                    <button
                      className="btn btn-secondary btn-sm"
                      style={{ marginLeft: 'auto' }}
                      onClick={() => alert("Manifest Terraform prêt dans templates/terraform/. Vous pouvez exécuter 'terraform init && terraform apply'.")}
                    >
                      Copier Manifest Terraform
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB: TASKS & LOGS */}
          {currentTab === 'tasks' && (
            <div>
              <div className="page-header">
                <div>
                  <h1 className="page-title">Journal des Tâches Asynchrones</h1>
                  <p className="page-subtitle">
                    Suivi de l'exécution en temps réel des builds Packer, playbooks Ansible et clonages
                  </p>
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '350px 1fr', gap: '1.25rem' }}>
                <div className="panel-card">
                  <div className="panel-card-header">
                    <span style={{ fontWeight: 600 }}>Historique des Tâches</span>
                  </div>
                  <div>
                    {tasks.map((task) => (
                      <div
                        key={task.id}
                        onClick={() => setActiveTaskLog(task)}
                        style={{
                          padding: '0.85rem 1rem',
                          borderBottom: '1px solid var(--border-color)',
                          cursor: 'pointer',
                          backgroundColor:
                            activeTaskLog?.id === task.id ? 'var(--bg-card-hover)' : 'transparent',
                        }}
                      >
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                          <span style={{ fontWeight: 600, fontSize: '0.85rem' }}>{task.title}</span>
                          <span
                            className={`badge ${
                              task.status === 'success' ? 'badge-ready' : 'badge-building'
                            }`}
                          >
                            {task.status}
                          </span>
                        </div>
                        <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                          {new Date(task.started_at).toLocaleTimeString()}
                        </span>
                      </div>
                    ))}
                    {tasks.length === 0 && (
                      <div style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-muted)' }}>
                        Aucune tâche en cours.
                      </div>
                    )}
                  </div>
                </div>

                <div className="panel-card">
                  <div className="panel-card-header">
                    <span style={{ fontWeight: 600 }}>
                      Sortie Terminal (Logs) : {activeTaskLog ? activeTaskLog.title : 'Sélectionnez une tâche'}
                    </span>
                  </div>
                  <div style={{ padding: '1rem' }}>
                    <div className="terminal-window">
                      {activeTaskLog ? (
                        activeTaskLog.logs.map((line, idx) => (
                          <div key={idx} className="terminal-line">
                            {line}
                          </div>
                        ))
                      ) : (
                        <p style={{ color: 'var(--text-muted)' }}>
                          Cliquez sur une tâche pour inspecter ses logs d'exécution.
                        </p>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB: USERS */}
          {currentTab === 'users' && (
            <div>
              <div className="page-header">
                <div>
                  <h1 className="page-title">Gestion des Utilisateurs & Attribution</h1>
                  <p className="page-subtitle">
                    Gestion des rôles (Admin vs Utilisateur standard) et des machines attribuées
                  </p>
                </div>
              </div>

              <div className="panel-card">
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Nom d'Utilisateur</th>
                      <th>Nom Complet</th>
                      <th>Email</th>
                      <th>Rôle</th>
                      <th>VMs Attribuées</th>
                    </tr>
                  </thead>
                  <tbody>
                    {usersList.map((u) => {
                      const userVms = vms.filter((v) => v.assigned_to === u.username);
                      return (
                        <tr key={u.id}>
                          <td style={{ fontWeight: 600 }}>@{u.username}</td>
                          <td>{u.full_name}</td>
                          <td>{u.email}</td>
                          <td>
                            <span
                              style={{
                                background: u.role === 'admin' ? '#0369a1' : '#1e293b',
                                color: 'white',
                                padding: '0.2rem 0.6rem',
                                borderRadius: '4px',
                                fontSize: '0.75rem',
                                fontWeight: 700,
                                textTransform: 'uppercase',
                              }}
                            >
                              {u.role}
                            </span>
                          </td>
                          <td>
                            {userVms.length > 0 ? (
                              <div style={{ display: 'flex', gap: '0.35rem', flexWrap: 'wrap' }}>
                                {userVms.map((v) => (
                                  <span key={v.id} className="badge badge-ready">
                                    {v.name}
                                  </span>
                                ))}
                              </div>
                            ) : (
                              <span style={{ color: 'var(--text-dark)' }}>Aucune VM attribuée</span>
                            )}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </main>
      </div>

      {/* MODAL: CRÉER VM */}
      {modalCreateVm && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3 style={{ fontSize: '1.1rem', fontWeight: 600 }}>Créer une Nouvelle Machine Virtuelle</h3>
              <button
                onClick={() => setModalCreateVm(false)}
                style={{ background: 'none', border: 'none', color: 'white', cursor: 'pointer' }}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleCreateVmSubmit}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Nom de la Machine</label>
                  <input
                    type="text"
                    required
                    className="form-control"
                    value={formVm.name}
                    onChange={(e) => setFormVm({ ...formVm, name: e.target.value })}
                    placeholder="ex: web-server-01"
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Hyperviseur Cible</label>
                  <select
                    className="form-control"
                    value={formVm.hypervisor_id}
                    onChange={(e) => setFormVm({ ...formVm, hypervisor_id: e.target.value })}
                  >
                    {hypervisors.map((hv) => (
                      <option key={hv.id} value={hv.id}>
                        {hv.name} ({hv.hypervisor_type})
                      </option>
                    ))}
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Modèle d'image / Système d'exploitation</label>
                  <select
                    className="form-control"
                    value={formVm.template_id}
                    onChange={(e) => setFormVm({ ...formVm, template_id: e.target.value })}
                  >
                    <option value="tpl-debian">⚡ Debian 12/13 Golden Image (Instanciation instantanée Copy-on-Write)</option>
                    <option value="tpl-nginx">🌐 Serveur Web Nginx (Provisionné via Ansible)</option>
                    <option value="tpl-docker">🐳 Hôte Docker Engine & Compose</option>
                    <option value="">💾 Disque vierge (Sans OS - Installation manuelle)</option>
                  </select>
                  <small style={{ color: '#38bdf8', fontSize: '0.78rem', marginTop: '4px', display: 'block' }}>
                    {formVm.template_id
                      ? '✨ La VM démarrera immédiatement avec un Debian opérationnel en 2 secondes grâce au CoW backing file.'
                      : '⚠️ Un disque vierge sera créé. Vous devrez attacher un ISO pour installer un OS.'}
                  </small>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
                  <div className="form-group">
                    <label className="form-label">vCPUs</label>
                    <input
                      type="number"
                      min="1"
                      max="16"
                      className="form-control"
                      value={formVm.vcpus}
                      onChange={(e) => setFormVm({ ...formVm, vcpus: e.target.value })}
                    />
                  </div>
                  <div className="form-group">
                    <label className="form-label">RAM (MB)</label>
                    <input
                      type="number"
                      step="512"
                      min="512"
                      className="form-control"
                      value={formVm.memory_mb}
                      onChange={(e) => setFormVm({ ...formVm, memory_mb: e.target.value })}
                    />
                  </div>
                </div>

                <div className="form-group">
                  <label className="form-label">Taille du Disque (GB, format qcow2)</label>
                  <input
                    type="number"
                    min="5"
                    className="form-control"
                    value={formVm.disk_size_gb}
                    onChange={(e) => setFormVm({ ...formVm, disk_size_gb: e.target.value })}
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Attribuer à un Utilisateur (Optionnel)</label>
                  <select
                    className="form-control"
                    value={formVm.assigned_to}
                    onChange={(e) => setFormVm({ ...formVm, assigned_to: e.target.value })}
                  >
                    <option value="">Aucun (Libre / Non assigné)</option>
                    {usersList.map((u) => (
                      <option key={u.id} value={u.username}>
                        {u.full_name} (@{u.username})
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setModalCreateVm(false)}
                >
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Créer et Démarrer
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL: ATTACHER UN DISQUE */}
      {modalAttachDisk && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3 style={{ fontSize: '1.1rem', fontWeight: 600 }}>
                Attacher un Disque Virtuel à {selectedVm?.name}
              </h3>
              <button
                onClick={() => setModalAttachDisk(false)}
                style={{ background: 'none', border: 'none', color: 'white', cursor: 'pointer' }}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleAttachDiskSubmit}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Sélectionner un Disque Existant du Pool</label>
                  <select
                    className="form-control"
                    value={formAttachDisk.disk_path}
                    onChange={(e) => setFormAttachDisk({ ...formAttachDisk, disk_path: e.target.value })}
                  >
                    <option value="">-- Choisir un disque existant --</option>
                    {disks.map((d) => (
                      <option key={d.id} value={d.path}>
                        {d.name} ({d.size_gb} GB - {d.format})
                      </option>
                    ))}
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Ou saisir un chemin d'image personnalisé</label>
                  <input
                    type="text"
                    className="form-control"
                    value={formAttachDisk.disk_path}
                    onChange={(e) => setFormAttachDisk({ ...formAttachDisk, disk_path: e.target.value })}
                    placeholder="/var/lib/libvirt/images/mon-disque.qcow2"
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Périphérique Cible (Bus VirtIO)</label>
                  <input
                    type="text"
                    required
                    className="form-control"
                    value={formAttachDisk.target_dev}
                    onChange={(e) => setFormAttachDisk({ ...formAttachDisk, target_dev: e.target.value })}
                    placeholder="vdb, vdc, vdd..."
                  />
                </div>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setModalAttachDisk(false)}
                >
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Attacher le Disque
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL: AJOUTER HYPERVISEUR */}
      {modalAddHv && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3 style={{ fontSize: '1.1rem', fontWeight: 600 }}>Enregistrer un Nouvel Hyperviseur</h3>
              <button
                onClick={() => setModalAddHv(false)}
                style={{ background: 'none', border: 'none', color: 'white', cursor: 'pointer' }}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleAddHvSubmit}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Nom de l'Hyperviseur</label>
                  <input
                    type="text"
                    required
                    className="form-control"
                    value={formHv.name}
                    onChange={(e) => setFormHv({ ...formHv, name: e.target.value })}
                    placeholder="ex: KVM-Node-02 ou Proxmox-Cluster"
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Type d'Hyperviseur</label>
                  <select
                    className="form-control"
                    value={formHv.hypervisor_type}
                    onChange={(e) => setFormHv({ ...formHv, hypervisor_type: e.target.value })}
                  >
                    <option value="kvm">KVM / Libvirt (Local ou SSH)</option>
                    <option value="proxmox">Proxmox VE (REST API)</option>
                    <option value="remote_kvm">KVM Distant Dédié</option>
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">URI de Connexion (Endpoint)</label>
                  <input
                    type="text"
                    required
                    className="form-control"
                    value={formHv.endpoint}
                    onChange={(e) => setFormHv({ ...formHv, endpoint: e.target.value })}
                    placeholder="qemu:///system ou qemu+ssh://root@192.168.1.100/system"
                  />
                </div>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setModalAddHv(false)}
                >
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Connecter et Enregistrer
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL: CLONER VM */}
      {modalCloneVm && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3 style={{ fontSize: '1.1rem', fontWeight: 600 }}>Cloner la Machine : {selectedVm?.name}</h3>
              <button
                onClick={() => setModalCloneVm(false)}
                style={{ background: 'none', border: 'none', color: 'white', cursor: 'pointer' }}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleCloneSubmit}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Nouveau Nom de la Machine Clônée</label>
                  <input
                    type="text"
                    required
                    className="form-control"
                    value={formClone.new_name}
                    onChange={(e) => setFormClone({ ...formClone, new_name: e.target.value })}
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Attribuer le Clone à un Utilisateur</label>
                  <select
                    className="form-control"
                    value={formClone.assigned_to}
                    onChange={(e) => setFormClone({ ...formClone, assigned_to: e.target.value })}
                  >
                    <option value="">Aucun</option>
                    {usersList.map((u) => (
                      <option key={u.id} value={u.username}>
                        {u.full_name} (@{u.username})
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setModalCloneVm(false)}
                >
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Lancer le Clonage
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL: ATTRIBUER VM */}
      {modalAssignVm && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3 style={{ fontSize: '1.1rem', fontWeight: 600 }}>Attribuer la VM '{selectedVm?.name}'</h3>
              <button
                onClick={() => setModalAssignVm(false)}
                style={{ background: 'none', border: 'none', color: 'white', cursor: 'pointer' }}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleAssignSubmit}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Sélectionner l'Utilisateur Bénéficiaire</label>
                  <select
                    className="form-control"
                    value={formAssign.username}
                    onChange={(e) => setFormAssign({ username: e.target.value })}
                  >
                    {usersList.map((u) => (
                      <option key={u.id} value={u.username}>
                        {u.full_name} (@{u.username}) - Rôle: {u.role}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setModalAssignVm(false)}
                >
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Confirmer l'Attribution
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL: CRÉER DISQUE */}
      {modalCreateDisk && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3 style={{ fontSize: '1.1rem', fontWeight: 600 }}>Nouveau Disque Virtuel</h3>
              <button
                onClick={() => setModalCreateDisk(false)}
                style={{ background: 'none', border: 'none', color: 'white', cursor: 'pointer' }}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleCreateDiskSubmit}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Nom du Disque</label>
                  <input
                    type="text"
                    required
                    className="form-control"
                    value={formDisk.name}
                    onChange={(e) => setFormDisk({ ...formDisk, name: e.target.value })}
                    placeholder="ex: data-vol01"
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Pool de Stockage Cible</label>
                  <select
                    className="form-control"
                    value={formDisk.pool_name}
                    onChange={(e) => setFormDisk({ ...formDisk, pool_name: e.target.value })}
                  >
                    {pools.map((p) => (
                      <option key={p.name} value={p.name}>
                        {p.name} ({p.path})
                      </option>
                    ))}
                  </select>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
                  <div className="form-group">
                    <label className="form-label">Taille (GB)</label>
                    <input
                      type="number"
                      min="1"
                      className="form-control"
                      value={formDisk.size_gb}
                      onChange={(e) => setFormDisk({ ...formDisk, size_gb: e.target.value })}
                    />
                  </div>
                  <div className="form-group">
                    <label className="form-label">Format de Disque</label>
                    <select
                      className="form-control"
                      value={formDisk.format}
                      onChange={(e) => setFormDisk({ ...formDisk, format: e.target.value })}
                    >
                      <option value="qcow2">qcow2 (Optimisé avec snapshots)</option>
                      <option value="raw">raw (Performance brute)</option>
                      <option value="vmdk">vmdk (Compatibilité VMware)</option>
                    </select>
                  </div>
                </div>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setModalCreateDisk(false)}
                >
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Créer le Disque
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL: REDIMENSIONNER DISQUE */}
      {modalResizeDisk && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3 style={{ fontSize: '1.1rem', fontWeight: 600 }}>Étendre la Taille du Disque Virtuel</h3>
              <button
                onClick={() => setModalResizeDisk(false)}
                style={{ background: 'none', border: 'none', color: 'white', cursor: 'pointer' }}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleResizeDiskSubmit}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Chemin du Disque</label>
                  <input
                    type="text"
                    readOnly
                    className="form-control"
                    value={formResize.disk_path}
                    style={{ background: '#0d1322' }}
                  />
                </div>
                <div className="form-group">
                  <label className="form-label">Nouvelle Taille Totale (GB)</label>
                  <input
                    type="number"
                    min="1"
                    required
                    className="form-control"
                    value={formResize.new_size_gb}
                    onChange={(e) => setFormResize({ ...formResize, new_size_gb: e.target.value })}
                  />
                </div>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setModalResizeDisk(false)}
                >
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Appliquer l'Extension
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL: DÉPLOYER TEMPLATE */}
      {modalDeployTemplate && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3 style={{ fontSize: '1.1rem', fontWeight: 600 }}>Déployer une VM depuis le Template</h3>
              <button
                onClick={() => setModalDeployTemplate(false)}
                style={{ background: 'none', border: 'none', color: 'white', cursor: 'pointer' }}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleDeployTemplateSubmit}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Nom de la Machine Virtuelle</label>
                  <input
                    type="text"
                    required
                    className="form-control"
                    value={formDeployTpl.vm_name}
                    onChange={(e) => setFormDeployTpl({ ...formDeployTpl, vm_name: e.target.value })}
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Hyperviseur Cible</label>
                  <select
                    className="form-control"
                    value={formDeployTpl.hypervisor_id}
                    onChange={(e) => setFormDeployTpl({ ...formDeployTpl, hypervisor_id: e.target.value })}
                  >
                    {hypervisors.map((hv) => (
                      <option key={hv.id} value={hv.id}>
                        {hv.name} ({hv.hypervisor_type})
                      </option>
                    ))}
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Attribuer à un Utilisateur (Optionnel)</label>
                  <select
                    className="form-control"
                    value={formDeployTpl.assigned_to}
                    onChange={(e) => setFormDeployTpl({ ...formDeployTpl, assigned_to: e.target.value })}
                  >
                    <option value="">Aucun</option>
                    {usersList.map((u) => (
                      <option key={u.id} value={u.username}>
                        {u.full_name} (@{u.username})
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setModalDeployTemplate(false)}
                >
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Déployer Immédiatement
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
