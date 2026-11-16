async function init() {
  const settings = await window.eyeSaver.getSettings();

  const bind = (id, key, type = 'checkbox') => {
    const el = document.getElementById(id);
    if (!el) return;
    if (type === 'checkbox') {
      el.checked = !!settings[key];
    } else if (type === 'range') {
      el.value = settings[key];
    } else {
      el.value = settings[key];
    }
    el.addEventListener('change', async () => {
      let val;
      if (type === 'checkbox') val = el.checked;
      else if (type === 'number') val = Number(el.value);
      else if (type === 'range') val = Number(el.value);
      else val = el.value;
      const patch = { [key]: val };
      await window.eyeSaver.updateSettings(patch);
      updateStatus('Saved');
      refreshRangeLabels();
    });
  };

  bind('breaksEnabled', 'breaksEnabled');
  bind('breakIntervalMinutes', 'breakIntervalMinutes', 'number');
  bind('breakDurationMinutes', 'breakDurationMinutes', 'number');
  bind('enableSkipping', 'enableSkipping');
  bind('microPausesEnabled', 'microPausesEnabled');
  bind('microPauseIntervalMinutes', 'microPauseIntervalMinutes', 'number');
  bind('microPauseDurationSeconds', 'microPauseDurationSeconds', 'number');
  bind('microPausesSound', 'microPausesSound');

  bind('blueFilterEnabled', 'blueFilterEnabled');
  bind('blueFilterIntensity', 'blueFilterIntensity', 'range');
  bind('overlayEnabled', 'overlayEnabled');
  bind('overlayOpacity', 'overlayOpacity', 'range');
  bind('grayscaleEnabled', 'grayscaleEnabled');
  bind('invertEnabled', 'invertEnabled');
  bind('autoSetMaxBrightness', 'autoSetMaxBrightness');

  bind('resetTimersAfterIdle', 'resetTimersAfterIdle');
  bind('idleResetMinutes', 'idleResetMinutes', 'number');

  bind('preventSleeping', 'preventSleeping');
  bind('autostart', 'autostart');

  const stretch = document.getElementById('reminderStretch');
  const water = document.getElementById('reminderWater');
  stretch.checked = !!settings.reminders?.stretch;
  water.checked = !!settings.reminders?.water;
  const saveReminders = async () => {
    await window.eyeSaver.updateSettings({
      reminders: { stretch: stretch.checked, water: water.checked }
    });
    updateStatus('Saved');
  };
  stretch.addEventListener('change', saveReminders);
  water.addEventListener('change', saveReminders);

  document.getElementById('btnPreviewBreak').addEventListener('click', () => {
    window.eyeSaver.previewBreak();
  });
  document.getElementById('btnPreviewMicro').addEventListener('click', () => {
    window.eyeSaver.previewMicro();
  });
  document.getElementById('btnReset').addEventListener('click', async () => {
    if (confirm('Reset all Eye Saver settings to defaults?')) {
      await window.eyeSaver.resetSettings();
      location.reload();
    }
  });
  document.getElementById('btnSave').addEventListener('click', async () => {
    await window.eyeSaver.updateSettings({});
    updateStatus('All settings saved');
  });

  // Tabs
  document.querySelectorAll('.tab').forEach((btn) => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.tab').forEach((b) => b.classList.remove('active'));
      document.querySelectorAll('.panel').forEach((p) => p.classList.remove('active'));
      btn.classList.add('active');
      document.getElementById('tab-' + btn.dataset.tab).classList.add('active');
    });
  });

  function refreshRangeLabels() {
    const bf = document.getElementById('blueFilterIntensity');
    const ov = document.getElementById('overlayOpacity');
    document.getElementById('blueFilterVal').textContent = bf.value + '%';
    document.getElementById('overlayOpacityVal').textContent = ov.value + '%';
  }
  refreshRangeLabels();
  updateStatus('Ready');
}

function updateStatus(msg) {
  document.getElementById('status').textContent = msg + ' · ' + new Date().toLocaleTimeString();
}

init().catch((e) => {
  console.error(e);
  updateStatus('Error: ' + e.message);
});
