const API_BASE = '/api';

const authHeaders = (token, extra = {}) => ({
  ...(token ? { Authorization: `Bearer ${token}` } : {}),
  ...extra,
});

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
      headers: authHeaders(token),
    });
    return res.json();
  },

  // Hypervisors
  async listHypervisors(token) {
    const res = await fetch(`${API_BASE}/hypervisors`, {
      headers: authHeaders(token),
    });
    return res.json();
  },

  async createHypervisor(data, token) {
    const res = await fetch(`${API_BASE}/hypervisors`, {
      method: 'POST',
      headers: authHeaders(token, { 'Content-Type': 'application/json' }),
      body: JSON.stringify(data),
    });
    return res.json();
  },

  async deleteHypervisor(id, token) {
    const res = await fetch(`${API_BASE}/hypervisors/${id}`, {
      method: 'DELETE',
      headers: authHeaders(token),
    });
    return res.json();
  },

  // Virtual Machines
  async listVms(token) {
    const res = await fetch(`${API_BASE}/vms`, {
      headers: authHeaders(token),
    });
    return res.json();
  },

  async getVm(id, token) {
    const res = await fetch(`${API_BASE}/vms/${id}`, {
      headers: authHeaders(token),
    });
    return res.json();
  },

  async createVm(data, token) {
    const res = await fetch(`${API_BASE}/vms`, {
      method: 'POST',
      headers: authHeaders(token, { 'Content-Type': 'application/json' }),
      body: JSON.stringify(data),
    });
    return res.json();
  },

  async startVm(id, token) {
    const res = await fetch(`${API_BASE}/vms/${id}/start`, {
      method: 'POST',
      headers: authHeaders(token),
    });
    return res.json();
  },

  async stopVm(id, token) {
    const res = await fetch(`${API_BASE}/vms/${id}/stop`, {
      method: 'POST',
      headers: authHeaders(token),
    });
    return res.json();
  },

  async rebootVm(id, token) {
    const res = await fetch(`${API_BASE}/vms/${id}/reboot`, {
      method: 'POST',
      headers: authHeaders(token),
    });
    return res.json();
  },

  async deleteVm(id, token) {
    const res = await fetch(`${API_BASE}/vms/${id}`, {
      method: 'DELETE',
      headers: authHeaders(token),
    });
    return res.json();
  },

  async cloneVm(id, newName, assignedTo, token) {
    const res = await fetch(`${API_BASE}/vms/${id}/clone`, {
      method: 'POST',
      headers: authHeaders(token, { 'Content-Type': 'application/json' }),
      body: JSON.stringify({ new_name: newName, assigned_to: assignedTo }),
    });
    return res.json();
  },

  async assignVm(id, username, token) {
    const res = await fetch(`${API_BASE}/vms/${id}/assign`, {
      method: 'POST',
      headers: authHeaders(token, { 'Content-Type': 'application/json' }),
      body: JSON.stringify({ username }),
    });
    return res.json();
  },

  async getVncInfo(id, token) {
    const res = await fetch(`${API_BASE}/vms/${id}/vnc`, {
      headers: authHeaders(token),
    });
    return res.json();
  },

  async attachDiskToVm(id, diskPath, targetDev, token) {
    const res = await fetch(`${API_BASE}/vms/${id}/disks`, {
      method: 'POST',
      headers: authHeaders(token, { 'Content-Type': 'application/json' }),
      body: JSON.stringify({ disk_path: diskPath, target_dev: targetDev }),
    });
    return res.json();
  },

  async detachDiskFromVm(id, targetDev, token) {
    const res = await fetch(`${API_BASE}/vms/${id}/disks/${targetDev}`, {
      method: 'DELETE',
      headers: authHeaders(token),
    });
    return res.json();
  },

  // Disks & Storage
  async listDisks(token) {
    const res = await fetch(`${API_BASE}/disks`, {
      headers: authHeaders(token),
    });
    return res.json();
  },

  async createDisk(data, token) {
    const res = await fetch(`${API_BASE}/disks`, {
      method: 'POST',
      headers: authHeaders(token, { 'Content-Type': 'application/json' }),
      body: JSON.stringify(data),
    });
    return res.json();
  },

  async resizeDisk(diskPath, newSizeGb, token) {
    const res = await fetch(`${API_BASE}/disks/resize`, {
      method: 'POST',
      headers: authHeaders(token, { 'Content-Type': 'application/json' }),
      body: JSON.stringify({ disk_path: diskPath, new_size_gb: newSizeGb }),
    });
    return res.json();
  },

  async listStoragePools(token) {
    const res = await fetch(`${API_BASE}/storage/pools`, {
      headers: authHeaders(token),
    });
    return res.json();
  },

  // Templates & Automation
  async listTemplates(token) {
    const res = await fetch(`${API_BASE}/templates`, {
      headers: authHeaders(token),
    });
    return res.json();
  },

  async triggerTemplateBuild(id, token) {
    const res = await fetch(`${API_BASE}/templates/${id}/build`, {
      method: 'POST',
      headers: authHeaders(token),
    });
    return res.json();
  },

  async deployTemplate(id, data, token) {
    const res = await fetch(`${API_BASE}/templates/${id}/deploy`, {
      method: 'POST',
      headers: authHeaders(token, { 'Content-Type': 'application/json' }),
      body: JSON.stringify(data),
    });
    return res.json();
  },

  async listTasks(token) {
    const res = await fetch(`${API_BASE}/tasks`, {
      headers: authHeaders(token),
    });
    return res.json();
  },

  // Users
  async listUsers(token) {
    const res = await fetch(`${API_BASE}/users`, {
      headers: authHeaders(token),
    });
    return res.json();
  },

  async createUser(data, token) {
    const res = await fetch(`${API_BASE}/users`, {
      method: 'POST',
      headers: authHeaders(token, { 'Content-Type': 'application/json' }),
      body: JSON.stringify(data),
    });
    return res.json();
  },
};
