const API = '';

async function fetchStatus() {
  const res = await fetch(`${API}/api/status`);
  if (!res.ok) throw new Error('Failed to fetch status');
  return res.json();
}

async function toggleShare(enabled) {
  const res = await fetch(`${API}/api/share`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ enabled }),
  });
  if (!res.ok) throw new Error('Failed to toggle sharing');
  return res.json();
}

function formatUptime(secs) {
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const parts = [];
  if (d > 0) parts.push(`${d}d`);
  if (h > 0) parts.push(`${h}h`);
  parts.push(`${m}m`);
  return parts.join(' ');
}

function renderStatus(data) {
  document.getElementById('peer-id').textContent = data.peer_id;
  document.getElementById('roles').textContent = data.roles.join(', ') || 'none';
  document.getElementById('sharing-status').textContent = data.sharing ? 'Active' : 'Disabled';
  document.getElementById('sharing-status').style.color = data.sharing ? 'var(--green)' : 'var(--text-muted)';

  const r = data.resources;
  document.getElementById('cpu').textContent = r.cpu_cores;
  document.getElementById('ram').textContent = `${r.ram_available_gb.toFixed(1)} GB`;
  document.getElementById('disk').textContent = `${r.disk_available_gb.toFixed(1)} GB`;

  if (r.gpu_vram_gb != null) {
    const model = r.gpu_model || 'unknown';
    document.getElementById('gpu').textContent = `${model} (${r.gpu_vram_gb.toFixed(0)} GiB)`;
  } else {
    document.getElementById('gpu').textContent = 'Not detected';
  }

  document.getElementById('uptime').textContent = formatUptime(r.uptime_secs);
  document.getElementById('platform').textContent = r.platform;

  const btn = document.getElementById('share-btn');
  if (data.sharing) {
    btn.textContent = 'Disable Sharing';
    btn.classList.add('active');
  } else {
    btn.textContent = 'Enable Sharing';
    btn.classList.remove('active');
  }

  document.getElementById('error').classList.add('hidden');
}

function showError() {
  document.getElementById('error').classList.remove('hidden');
}

async function refresh() {
  try {
    const data = await fetchStatus();
    renderStatus(data);
  } catch {
    showError();
  }
}

document.getElementById('share-btn').addEventListener('click', async () => {
  const btn = document.getElementById('share-btn');
  const currentlyActive = btn.classList.contains('active');
  try {
    const data = await toggleShare(!currentlyActive);
    renderStatus(data);
  } catch {
    showError();
  }
});

refresh();
setInterval(refresh, 5000);
