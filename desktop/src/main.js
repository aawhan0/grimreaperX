import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const timerEl = document.querySelector('#timer');
const activeAppEl = document.querySelector('#active-app');
const thresholdEl = document.querySelector('#threshold');
const timeOverEl = document.querySelector('#time-over');
const trackerStatusEl = document.querySelector('#tracker-status');
const trackerMessageEl = document.querySelector('#tracker-message');
const todayTotalEl = document.querySelector('#today-total');
const topAppsEl = document.querySelector('#top-apps');
const threatCountEl = document.querySelector('#threat-count');
const currentTierEl = document.querySelector('#current-tier');
const settingsButton = document.querySelector('#settings-button');
const settingsDialog = document.querySelector('#settings-dialog');
const settingsForm = document.querySelector('#settings-form');
const thresholdInput = document.querySelector('#threshold-input');
const settingsError = document.querySelector('#settings-error');

function formatTime(totalSeconds) {
  const minutes = Math.floor(totalSeconds / 60).toString().padStart(2, '0');
  const seconds = (totalSeconds % 60).toString().padStart(2, '0');
  return `${minutes}:${seconds}`;
}

function formatDuration(totalSeconds) {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  return hours ? `${hours}h ${minutes}m` : `${minutes}m`;
}

function openSettings() {
  settingsError.textContent = '';
  if (!settingsDialog.open) settingsDialog.showModal();
}

async function refreshStatus() {
  try {
    const snapshot = await invoke('get_tracker_status');
    const elapsedSeconds = Math.max(0, snapshot.elapsed_seconds);
    const thresholdSeconds = snapshot.threshold_minutes * 60;

    timerEl.textContent = formatTime(elapsedSeconds);
    activeAppEl.textContent = snapshot.current_app ?? 'No tracked app';
    thresholdEl.textContent = `${snapshot.threshold_minutes} min`;
    timeOverEl.textContent = formatTime(Math.max(0, elapsedSeconds - thresholdSeconds));
    trackerStatusEl.textContent = snapshot.is_tracking ? 'Tracking' : 'Paused';
    trackerStatusEl.dataset.state = snapshot.is_tracking ? 'active' : 'paused';
    trackerMessageEl.textContent = !snapshot.is_tracking
      ? 'Tracking is paused from the system tray.'
      : snapshot.current_app
        ? 'Monitoring the foreground app.'
        : 'No tracked app is currently in the foreground.';

    if (!settingsDialog.open) thresholdInput.value = snapshot.threshold_minutes;
  } catch {
    trackerStatusEl.textContent = 'Unavailable';
    trackerStatusEl.dataset.state = 'paused';
    trackerMessageEl.textContent = 'The desktop tracking service is not responding.';
  }
}

async function refreshDashboard() {
  try {
    const summary = await invoke('get_dashboard_summary');
    todayTotalEl.textContent = formatDuration(summary.today_total_seconds);
    threatCountEl.textContent = summary.threat_count.toString();
    currentTierEl.textContent = summary.current_tier[0].toUpperCase() + summary.current_tier.slice(1);
    currentTierEl.dataset.tier = summary.current_tier;
    topAppsEl.replaceChildren();

    if (summary.top_apps.length === 0) {
      const emptyItem = document.createElement('li');
      emptyItem.textContent = 'No sessions yet';
      topAppsEl.append(emptyItem);
      return;
    }

    for (const app of summary.top_apps) {
      const item = document.createElement('li');
      const appName = document.createElement('span');
      const duration = document.createElement('span');
      appName.textContent = app.app_name;
      duration.textContent = formatDuration(app.seconds);
      item.append(appName, duration);
      topAppsEl.append(item);
    }
  } catch {
    todayTotalEl.textContent = 'Unavailable';
    threatCountEl.textContent = '–';
    currentTierEl.textContent = 'Unavailable';
  }
}

settingsButton.addEventListener('click', openSettings);
document.querySelector('#close-settings').addEventListener('click', () => settingsDialog.close());

settingsForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  settingsError.textContent = '';
  const minutes = Number(thresholdInput.value);

  try {
    await invoke('set_threshold_minutes', { minutes });
    settingsDialog.close();
    await refreshStatus();
  } catch (error) {
    settingsError.textContent = String(error);
  }
});

listen('open-settings', openSettings).catch(() => {});
refreshStatus();
refreshDashboard();
setInterval(refreshStatus, 1000);
setInterval(refreshDashboard, 10000);