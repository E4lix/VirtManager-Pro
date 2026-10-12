const API_BASE = '/api';

export const api = {
  // Auth
  async login(username, password) {
    const res = await fetch(`${API_BASE}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    });
    return res.json();
  },

  async getMe(token) {
    const res = await fetch(`${API_BASE}/auth/me`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    return res.json();
  },

  // Hypervisors
  async listHypervisors() {
    const res = await fetch(`${API_BASE}/hypervisors`);
    return res.json();
  },

  async createHypervisor(data, token) {
    const res = await fetch(`${API_BASE}/hypervisors`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(data),
    });
    return res.json();
  },

  async deleteHypervisor(id, token) {
    const res = await fetch(`${API_BASE}/hypervisors/${id}`, {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${token}` },
    });
    return res.json();
  },

  // Virtual Machines
  async listVms(token) {
    const res = await fetch(`${API_BASE}/vms`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });
    return res.json();
  },

  async getVm(id) {
    const res = await fetch(`${API_BASE}/vms/${id}`);
    return res.json();
  },

  async createVm(data) {
    const res = await fetch(`${API_BASE}/vms`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    return res.json();
  },

  async startVm(id) {
    const res = await fetch(`${API_BASE}/vms/${id}/start`, { method: 'POST' });
    return res.json();
  },

  async stopVm(id) {
    const res = await fetch(`${API_BASE}/vms/${id}/stop`, { method: 'POST' });
    return res.json();
  },

  async rebootVm(id) {
    const res = await fetch(`${API_BASE}/vms/${id}/reboot`, { method: 'POST' });
    return res.json();
  },

  async deleteVm(id) {
    const res = await fetch(`${API_BASE}/vms/${id}`, { method: 'DELETE' });
    return res.json();
  },

  async cloneVm(id, newName, assignedTo) {
    const res = await fetch(`${API_BASE}/vms/${id}/clone`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ new_name: newName, assigned_to: assignedTo }),
    });
    return res.json();
  },

  async assignVm(id, username) {
    const res = await fetch(`${API_BASE}/vms/${id}/assign`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username }),
    });
    return res.json();
  },

  async getVncInfo(id) {
    const res = await fetch(`${API_BASE}/vms/${id}/vnc`);
    return res.json();
  },

  // Disks & Storage
  async listDisks() {
    const res = await fetch(`${API_BASE}/disks`);
    return res.json();
  },

  async createDisk(data) {
    const res = await fetch(`${API_BASE}/disks`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    return res.json();
  },

  async resizeDisk(diskPath, newSizeGb) {
    const res = await fetch(`${API_BASE}/disks/resize`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ disk_path: diskPath, new_size_gb: newSizeGb }),
    });
    return res.json();
  },

  async listStoragePools() {
    const res = await fetch(`${API_BASE}/storage/pools`);
    return res.json();
  },

  // Templates & Automation
  async listTemplates() {
    const res = await fetch(`${API_BASE}/templates`);
    return res.json();
  },

  async triggerTemplateBuild(id) {
    const res = await fetch(`${API_BASE}/templates/${id}/build`, {
      method: 'POST',
    });
    return res.json();
  },

  async deployTemplate(id, data) {
    const res = await fetch(`${API_BASE}/templates/${id}/deploy`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    return res.json();
  },

  async listTasks() {
    const res = await fetch(`${API_BASE}/tasks`);
    return res.json();
  },

  // Users
  async listUsers() {
    const res = await fetch(`${API_BASE}/users`);
    return res.json();
  },

  async createUser(data) {
    const res = await fetch(`${API_BASE}/users`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    return res.json();
  },
};
