import { invoke, isTauri as detectTauri } from '@tauri-apps/api/core';
import type { Settings, Snapshot, TransportPreference } from './types';

export const isTauri = detectTauri();
const browserSnapshot: Snapshot = {
  devices: [], savedDevices: [], session: null, logs: [],
  engineVersion: 'scrcpy', busy: false, lastError: null,
  settings: {
    maxFps: null, audio: true, alwaysOnTop: false,
    preferredTransport: 'auto', lastDeviceId: null,
  },
};

function desktopOnly<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) return Promise.reject(new Error('当前为界面预览，请打开桌面版连接设备。'));
  return invoke<T>(command, args);
}

export const api = {
  getSnapshot: () => isTauri ? invoke<Snapshot>('get_snapshot') : Promise.resolve(structuredClone(browserSnapshot)),
  refresh: () => desktopOnly<Snapshot>('refresh_devices'),
  saveSettings: (settings: Settings) => isTauri
    ? invoke<Snapshot>('save_settings', { settings })
    : Promise.resolve(structuredClone(Object.assign(browserSnapshot, { settings }))),
  selectDevice: (deviceId: string | null) => desktopOnly<Snapshot>('select_device', { deviceId }),
  startMirror: (deviceId: string, transport: TransportPreference) => desktopOnly<Snapshot>('start_mirror', { deviceId, transport }),
  stopMirror: () => desktopOnly<Snapshot>('stop_mirror'),
  configureWireless: (deviceId: string) => desktopOnly<Snapshot>('configure_wireless', { deviceId }),
  connectWireless: (address: string) => desktopOnly<Snapshot>('connect_wireless', { address }),
  pairWireless: (address: string, code: string, connectAddress: string | null) => desktopOnly<Snapshot>('pair_wireless', { pairingAddress: address, code, connectAddress }),
  hideWindow: () => desktopOnly<void>('hide_window'),
  minimizeWindow: () => desktopOnly<void>('minimize_window'),
  quitApp: () => desktopOnly<void>('quit_app'),
};
