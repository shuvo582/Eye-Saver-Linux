const { app, BrowserWindow, Tray, Menu, ipcMain, dialog, Notification, powerSaveBlocker, screen, nativeImage, shell } = require('electron');
const path = require('path');
const fs = require('fs');
const { spawn, execFile } = require('child_process');

const singleInstance = app.requestSingleInstanceLock();
if (!singleInstance) {
  app.quit();
  process.exit(0);
}

let mainWindow = null;
let breakWindow = null;
let tray = null;
let breakTimer = null;
let microPauseTimer = null;
let idleCheckTimer = null;
let remainingBreakSeconds = 0;
let remainingMicroSeconds = 0;
let sleepBlockerId = null;
let applyingOverlay = false;

const DEFAULTS = {
  breaksEnabled: true,
  breakDurationMinutes: 20,
  breakIntervalMinutes: 20,
  enableSkipping: true,
  disableSkippingSeconds: 0,
  microPausesEnabled: true,
  microPauseIntervalMinutes: 5,
  microPauseDurationSeconds: 20,
  microPausesSound: true,
  blueFilterEnabled: false,
  blueFilterIntensity: 40,
  grayscaleEnabled: false,
  invertEnabled: false,
  overlayEnabled: false,
  overlayOpacity: 0.25,
  overlayColor: '#0b1e3a',
  autoSetMaxBrightness: false,
  preventSleeping: false,
  autostart: false,
  resetTimersAfterIdle: true,
  idleResetMinutes: 5,
  reminders: {
    stretch: true,
    water: true
  },
  lastBreakAt: Date.now(),
  lastMicroAt: Date.now()
};

let settings = loadSettings();

function settingsPath() {
  const dir = app.getPath('userData');
  fs.mkdirSync(dir, { recursive: true });
  return path.join(dir, 'settings.json');
}

function loadSettings() {
  try {
    const p = settingsPath();
    if (fs.existsSync(p)) {
      return { ...DEFAULTS, ...JSON.parse(fs.readFileSync(p, 'utf8')) };
    }
  } catch (e) {
    console.error('Failed to load settings', e);
  }
  return { ...DEFAULTS };
}

function saveSettings() {
  try {
    fs.writeFileSync(settingsPath(), JSON.stringify(settings, null, 2), 'utf8');
  } catch (e) {
    console.error('Failed to save settings', e);
  }
}

function pickReminder() {
  const options = [];
  if (settings.reminders.stretch) options.push('Sitting too long is not healthy. Do some stretching.');
  if (settings.reminders.water) options.push('Drink more water!');
  if (!options.length) options.push('Time for a break — rest your eyes.');
  return options[Math.floor(Math.random() * options.length)];
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 920,
    height: 680,
    minWidth: 720,
    minHeight: 520,
    title: 'Eye Saver',
    backgroundColor: '#0F172A',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    },
    show: false
  });

  mainWindow.loadFile(path.join(__dirname, 'index.html'));
  mainWindow.once('ready-to-show', () => mainWindow.show());
  mainWindow.on('close', (e) => {
    if (!app.isQuitting) {
      e.preventDefault();
      mainWindow.hide();
    }
  });
}

function createBreakWindow(reason = 'break') {
  if (breakWindow && !breakWindow.isDestroyed()) {
    breakWindow.focus();
    return;
  }

  remainingBreakSeconds = reason === 'micro'
    ? Math.max(1, settings.microPauseDurationSeconds)
    : Math.max(1, settings.breakDurationMinutes * 60);

  const display = screen.getPrimaryDisplay();
  const { width, height } = display.workAreaSize;

  breakWindow = new BrowserWindow({
    width,
    height,
    frame: false,
    resizable: false,
    movable: false,
    fullscreen: true,
    alwaysOnTop: true,
    skipTaskbar: true,
    backgroundColor: '#000000',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  breakWindow.loadFile(path.join(__dirname, 'break.html'), {
    query: {
      reason,
      total: String(remainingBreakSeconds),
      message: reason === 'micro' ? 'Micro pause — blink and relax your eyes' : pickReminder()
    }
  });

  breakWindow.setAlwaysOnTop(true, 'screen-saver');
  breakWindow.on('closed', () => {
    breakWindow = null;
    if (breakTimer) {
      clearInterval(breakTimer);
      breakTimer = null;
    }
    if (reason !== 'micro') {
      settings.lastBreakAt = Date.now();
    } else {
      settings.lastMicroAt = Date.now();
    }
    saveSettings();
  });

  if (breakTimer) clearInterval(breakTimer);
  breakTimer = setInterval(() => {
    remainingBreakSeconds -= 1;
    if (breakWindow && !breakWindow.isDestroyed()) {
      breakWindow.webContents.send('tick', remainingBreakSeconds);
    }
    if (remainingBreakSeconds <= 0) {
      if (breakWindow && !breakWindow.isDestroyed()) breakWindow.close();
    }
  }, 1000);

  if (settings.microPausesSound && reason === 'micro') {
    try {
      beep();
    } catch (_) {}
  }
}

function closeBreakWindow() {
  if (breakWindow && !breakWindow.isDestroyed()) {
    breakWindow.close();
  }
}

function delayBreak(minutes) {
  settings.lastBreakAt = Date.now() - (settings.breakIntervalMinutes - minutes) * 60 * 1000;
  saveSettings();
  closeBreakWindow();
  restartTimers();
}

function skipBreak() {
  if (!settings.enableSkipping) return;
  settings.lastBreakAt = Date.now();
  saveSettings();
  closeBreakWindow();
  restartTimers();
}

function beep() {
  // Simple terminal bell fallback; Electron can also use Audio
  try {
    process.stdout.write('\x07');
  } catch (_) {}
}

function scheduleTimers() {
  if (breakTimer && !breakWindow) {
    clearInterval(breakTimer);
    breakTimer = null;
  }

  if (settings.breaksEnabled) {
    const elapsed = Date.now() - settings.lastBreakAt;
    const intervalMs = settings.breakIntervalMinutes * 60 * 1000;
    if (elapsed >= intervalMs && (!breakWindow || breakWindow.isDestroyed())) {
      createBreakWindow('break');
    }
  }

  if (settings.microPausesEnabled && (!breakWindow || breakWindow.isDestroyed())) {
    const elapsed = Date.now() - settings.lastMicroAt;
    const intervalMs = settings.microPauseIntervalMinutes * 60 * 1000;
    if (elapsed >= intervalMs) {
      createBreakWindow('micro');
    }
  }
}

function restartTimers() {
  if (idleCheckTimer) clearInterval(idleCheckTimer);
  idleCheckTimer = setInterval(scheduleTimers, 1000);
  scheduleTimers();
}

function applyScreenEffects() {
  // Blue filter / grayscale / invert / overlay via a transparent fullscreen child window
  if (applyingOverlay) return;
  applyingOverlay = true;
  try {
    if (overlayWindow && !overlayWindow.isDestroyed()) {
      overlayWindow.close();
      overlayWindow = null;
    }

    const needsOverlay =
      settings.blueFilterEnabled ||
      settings.grayscaleEnabled ||
      settings.invertEnabled ||
      settings.overlayEnabled;

    if (!needsOverlay) return;

    const display = screen.getPrimaryDisplay();
    const { width, height } = display.bounds;
    const { x, y } = display.bounds;

    let css = '';
    if (settings.blueFilterEnabled) {
      const a = Math.min(0.7, Math.max(0.05, settings.blueFilterIntensity / 100));
      css += `background: rgba(255, 140, 0, ${a * 0.45}); mix-blend-mode: multiply;`;
      // warm amber overlay reduces blue
      css = `background: rgba(255, 120, 20, ${a});`;
    }
    if (settings.overlayEnabled && !settings.blueFilterEnabled) {
      const c = settings.overlayColor || '#0b1e3a';
      const o = Math.min(0.8, Math.max(0, settings.overlayOpacity));
      css = `background: ${hexToRgba(c, o)};`;
    }

    const filters = [];
    if (settings.grayscaleEnabled) filters.push('grayscale(1)');
    if (settings.invertEnabled) filters.push('invert(1)');

    overlayWindow = new BrowserWindow({
      x,
      y,
      width,
      height,
      frame: false,
      transparent: true,
      resizable: false,
      movable: false,
      hasShadow: false,
      focusable: false,
      skipTaskbar: true,
      alwaysOnTop: true,
      webPreferences: { contextIsolation: true, nodeIntegration: false }
    });

    overlayWindow.setAlwaysOnTop(true, 'screen-saver');
    overlayWindow.loadURL(
      'data:text/html;charset=utf-8,' +
        encodeURIComponent(`<!DOCTYPE html><html><head><style>
          html,body{margin:0;padding:0;width:100%;height:100%;overflow:hidden;pointer-events:none;}
          #fx{position:fixed;inset:0;${css}}
          #fx2{position:fixed;inset:0;backdrop-filter:${filters.join(' ')};pointer-events:none;}
        </style></head><body><div id="fx"></div><div id="fx2"></div></body></html>`)
    );
  } finally {
    applyingOverlay = false;
  }
}

function hexToRgba(hex, alpha) {
  const h = hex.replace('#', '');
  const full = h.length === 3 ? h.split('').map((c) => c + c).join('') : h;
  const n = parseInt(full, 16);
  const r = (n >> 16) & 255;
  const g = (n >> 8) & 255;
  const b = n & 255;
  return `rgba(${r},${g},${b},${alpha})`;
}

let overlayWindow = null;

function applyPreventSleep() {
  if (settings.preventSleeping) {
    if (sleepBlockerId == null) {
      sleepBlockerId = powerSaveBlocker.start('prevent-display-sleep');
    }
  } else if (sleepBlockerId != null) {
    powerSaveBlocker.stop(sleepBlockerId);
    sleepBlockerId = null;
  }
}

function applyAutostart() {
  app.setLoginItemSettings({
    openAtLogin: settings.autostart,
    path: process.execPath
  });
}

function applyAll() {
  saveSettings();
  applyScreenEffects();
  applyPreventSleep();
  applyAutostart();
  restartTimers();
}

function createTray() {
  // 16x16 simple eye icon as data URL PNG (blue circle)
  const iconPng = Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAmklEQVR42mNgGAWjYBSMglEwCkYB//8HFPD//x8Y////Hxj///8fGf///x8YGBgYgP///w8M////Hxj///8fGf///x8YGBgYgP///w8M////Hxj///8fGf///x8YGBgYgP///w8M////Hxj///8fGf///x8YGBgYgP///w8M////Hxj///8fGf///x8YGBgYgP///w8M//8HFPD//x8Y////Hxj///8fGf///x8YGBgYgP///w8M////Hxj///8fGf///x8YGBgYgP///w8M',
    'base64'
  );
  // Use a generated simple icon
  const size = 22;
  const buf = Buffer.alloc(size * size * 4);
  for (let i = 0; i < size * size; i++) {
    const x = i % size;
    const y = Math.floor(i / size);
    const dx = x - size / 2;
    const dy = y - size / 2;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const inEye = dist < size * 0.45;
    const inPupil = dist < size * 0.18;
    const o = i * 4;
    if (inPupil) {
      buf[o] = 15; buf[o + 1] = 23; buf[o + 2] = 42; buf[o + 3] = 255;
    } else if (inEye) {
      buf[o] = 6; buf[o + 1] = 182; buf[o + 2] = 212; buf[o + 3] = 255;
    } else {
      buf[o] = 0; buf[o + 1] = 0; buf[o + 2] = 0; buf[o + 3] = 0;
    }
  }
  const image = nativeImage.createFromBuffer(buf, { width: size, height: size });
  tray = new Tray(image);
  tray.setToolTip('Eye Saver');
  tray.setContextMenu(buildTrayMenu());
  tray.on('click', () => {
    if (mainWindow) {
      if (mainWindow.isVisible()) mainWindow.focus();
      else mainWindow.show();
    }
  });
}

function buildTrayMenu() {
  return Menu.buildFromTemplate([
    { label: 'Open Eye Saver', click: () => mainWindow && mainWindow.show() },
    { type: 'separator' },
    {
      label: settings.breaksEnabled ? 'Breaks: On' : 'Breaks: Off',
      click: () => {
        settings.breaksEnabled = !settings.breaksEnabled;
        applyAll();
        tray.setContextMenu(buildTrayMenu());
      }
    },
    {
      label: settings.microPausesEnabled ? 'Micro pauses: On' : 'Micro pauses: Off',
      click: () => {
        settings.microPausesEnabled = !settings.microPausesEnabled;
        applyAll();
        tray.setContextMenu(buildTrayMenu());
      }
    },
    {
      label: settings.blueFilterEnabled ? 'Blue filter: On' : 'Blue filter: Off',
      click: () => {
        settings.blueFilterEnabled = !settings.blueFilterEnabled;
        applyAll();
        tray.setContextMenu(buildTrayMenu());
      }
    },
    { type: 'separator' },
    { label: 'Take a break now', click: () => createBreakWindow('break') },
    { type: 'separator' },
    {
      label: 'Quit',
      click: () => {
        app.isQuitting = true;
        app.quit();
      }
    }
  ]);
}

// IPC
ipcMain.handle('get-settings', () => settings);

ipcMain.handle('update-settings', (_e, patch) => {
  settings = { ...settings, ...patch };
  if (patch.reminders) {
    settings.reminders = { ...DEFAULTS.reminders, ...patch.reminders };
  }
  applyAll();
  tray && tray.setContextMenu(buildTrayMenu());
  return settings;
});

ipcMain.handle('reset-settings', () => {
  settings = { ...DEFAULTS, lastBreakAt: Date.now(), lastMicroAt: Date.now() };
  applyAll();
  tray && tray.setContextMenu(buildTrayMenu());
  return settings;
});

ipcMain.handle('preview-micro', () => createBreakWindow('micro'));
ipcMain.handle('preview-break', () => createBreakWindow('break'));
ipcMain.handle('delay-break', (_e, minutes) => delayBreak(minutes));
ipcMain.handle('skip-break', () => skipBreak());
ipcMain.handle('close-break', () => closeBreakWindow());
ipcMain.handle('break-tick-info', () => ({
  remaining: remainingBreakSeconds,
  reason: breakWindow ? 'active' : 'idle'
}));

ipcMain.on('break-finished', () => closeBreakWindow());

app.whenReady().then(() => {
  createWindow();
  createTray();
  applyAll();
  restartTimers();

  // Idle detection via systemd-login / xprintidle if available; fallback simple heuristic
  setInterval(() => {
    if (!settings.resetTimersAfterIdle) return;
    // Optional: use xprintidle on X11
    execFile('xprintidle', (err, stdout) => {
      if (err) return;
      const idleMs = parseInt(stdout.trim(), 10);
      if (!Number.isNaN(idleMs) && idleMs > settings.idleResetMinutes * 60 * 1000) {
        settings.lastBreakAt = Date.now();
        settings.lastMicroAt = Date.now();
        saveSettings();
      }
    });
  }, 30000);
});

app.on('second-instance', () => {
  if (mainWindow) {
    if (mainWindow.isMinimized()) mainWindow.restore();
    mainWindow.show();
    mainWindow.focus();
  }
});

app.on('window-all-closed', () => {
  // keep running in tray
});

app.on('before-quit', () => {
  app.isQuitting = true;
});
