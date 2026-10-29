export type TransportPreference = 'auto' | 'usb' | 'wifi';
export interface Settings {
  maxFps: number | null;
  audio: boolean;
  alwaysOnTop: boolean;
  preferredTransport: TransportPreference;
  lastDeviceId: string | null;
}
export interface Transport { serial: string; kind: 'usb' | 'wifi'; state: string }
export interface Device {
  id: string;
  name: string;
  model: string;
  androidVersion: string | null;
  apiLevel: number | null;
  transports: Transport[];
  wifiAddress: string | null;
  audioSupported: boolean | null;
}
export interface SavedDevice { id: string; name: string; address: string }
export interface Session {
  deviceId: string;
  deviceName: string;
  serial: string;
  transport: 'usb' | 'wifi';
  phase: 'starting' | 'running' | 'reconnecting';
  startedAt: number;
}
export interface LogEntry { time: number; level: 'info' | 'success' | 'warning' | 'error'; message: string }
export interface Snapshot {
  devices: Device[];
  savedDevices: SavedDevice[];
  settings: Settings;
  session: Session | null;
  logs: LogEntry[];
  engineVersion: string;
  busy: boolean;
  lastError: string | null;
}
