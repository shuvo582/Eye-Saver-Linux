const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('eyeSaver', {
  getSettings: () => ipcRenderer.invoke('get-settings'),
  updateSettings: (patch) => ipcRenderer.invoke('update-settings', patch),
  resetSettings: () => ipcRenderer.invoke('reset-settings'),
  previewMicro: () => ipcRenderer.invoke('preview-micro'),
  previewBreak: () => ipcRenderer.invoke('preview-break'),
  delayBreak: (minutes) => ipcRenderer.invoke('delay-break', minutes),
  skipBreak: () => ipcRenderer.invoke('skip-break'),
  closeBreak: () => ipcRenderer.invoke('close-break'),
  onTick: (cb) => {
    const handler = (_e, remaining) => cb(remaining);
    ipcRenderer.on('tick', handler);
    return () => ipcRenderer.removeListener('tick', handler);
  },
  breakFinished: () => ipcRenderer.send('break-finished')
});
