import { useEffect, useRef, useState, type FormEvent, type ReactNode } from 'react';
import { api, isTauri } from './api';
import type { Device, Settings, Snapshot } from './types';
import SettingsPage from './SettingsPage';
import './styles.css';

type IconName = 'usb' | 'wifi' | 'refresh' | 'plus' | 'chevron' | 'check' | 'close' | 'minus' | 'help' | 'arrow' | 'play' | 'stop' | 'volume' | 'pin' | 'activity' | 'sliders' | 'spark' | 'log' | 'copy' | 'alert' | 'phone' | 'external' | 'settings';
function Icon({ name, size = 18, className = '' }: { name: IconName; size?: number; className?: string }) {
  const paths: Record<IconName, ReactNode> = {
    usb: <><path d="M12 21V3m0 0-3 3m3-3 3 3M12 16l-5-4V8m5 5 5-4V6"/><circle cx="7" cy="7" r="1.5"/><path d="M15.5 3.5h3v3h-3z"/><circle cx="12" cy="20" r="1.5"/></>,
    wifi: <><path d="M3 8.5a14 14 0 0 1 18 0M6 12a9.5 9.5 0 0 1 12 0m-9 3.5a4.7 4.7 0 0 1 6 0"/><circle cx="12" cy="19" r=".8" fill="currentColor" stroke="none"/></>,
    refresh: <><path d="M20 7v5h-5M4 17v-5h5"/><path d="M6.1 7a7 7 0 0 1 11.6-2L20 8M4 16l2.3 3A7 7 0 0 0 18 17"/></>,
    plus: <path d="M12 5v14M5 12h14"/>, chevron: <path d="m9 5 7 7-7 7"/>,
    check: <path d="m5 12 4 4L19 6"/>, close: <path d="m6 6 12 12M6 18 18 6"/>, minus: <path d="M5 12h14"/>,
    help: <><circle cx="12" cy="12" r="9"/><path d="M9.5 9a2.5 2.5 0 0 1 5 .4c0 1.8-2.5 1.9-2.5 3.6m0 3.2v.1"/></>,
    arrow: <path d="M5 12h14m-5-5 5 5-5 5"/>, play: <path d="m9 5 11 7-11 7z" fill="currentColor" stroke="none"/>, stop: <rect x="6" y="6" width="12" height="12" rx="2" fill="currentColor" stroke="none"/>,
    volume: <><path d="m11 4-5 4H3v8h3l5 4zm4 4a6 6 0 0 1 0 8m3-11a10 10 0 0 1 0 14"/></>,
    pin: <><path d="m16 3 5 5-3 2-2 5-3 1-5-5 1-3 5-2zM3 21l7-7"/></>,
    activity: <path d="M3 12h4l3-7 4 14 3-7h4"/>, sliders: <><path d="M4 7h4m4 0h8M4 17h8m4 0h4"/><circle cx="10" cy="7" r="2"/><circle cx="14" cy="17" r="2"/></>,
    spark: <><path d="m12 3 2.6 6.4L21 12l-6.4 2.6L12 21l-2.6-6.4L3 12l6.4-2.6z"/></>,
    log: <><rect x="4" y="3" width="16" height="18" rx="3"/><path d="M8 8h8M8 12h8m-8 4h5"/></>,
    copy: <><rect x="8" y="8" width="12" height="13" rx="2"/><path d="M15 8V3H3v13h5"/></>,
    alert: <><path d="m12 3 10 18H2z"/><path d="M12 9v5m0 3v.2"/></>,
    phone: <><rect x="6" y="2" width="12" height="20" rx="3"/><path d="M10 5h4m-3 14h2"/></>,
    external: <><path d="M14 3h7v7m0-7L11 13"/><path d="M10 3H3v18h18v-7"/></>,
    settings: <><path d="M18.837 9.168 19.210 10.335 21.087 10.561 21.087 13.439 19.210 13.665 18.837 14.832 18.276 15.921 19.443 17.408 17.408 19.443 15.921 18.276 14.832 18.837 13.665 19.210 13.439 21.087 10.561 21.087 10.335 19.210 9.168 18.837 8.079 18.276 6.592 19.443 4.557 17.408 5.724 15.921 5.163 14.832 4.790 13.665 2.913 13.439 2.913 10.561 4.790 10.335 5.163 9.168 5.724 8.079 4.557 6.592 6.592 4.557 8.079 5.724 9.168 5.163 10.335 4.790 10.561 2.913 13.439 2.913 13.665 4.790 14.832 5.163 15.921 5.724 17.408 4.557 19.443 6.592 18.276 8.079Z"/><circle cx="12" cy="12" r="3.1"/></>,
  };
  return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={name === 'settings' ? 1.7 : 1.65} strokeLinecap="round" strokeLinejoin="round" className={className} aria-hidden="true">{paths[name]}</svg>;
}

function MirrorMark({ small = false }: { small?: boolean }) {
  return <span className={`mirror-mark ${small ? 'small' : ''}`} aria-hidden="true"><img src="/mirror-mark.svg" alt="" width="32" height="32" style={{ width: '100%', height: '100%', display: 'block' }}/></span>;
}

const emptySnapshot: Snapshot = {
  devices: [], savedDevices: [], session: null, logs: [], engineVersion: 'scrcpy', busy: false, lastError: null,
  settings: { maxFps: null, audio: true, alwaysOnTop: false, preferredTransport: 'auto', lastDeviceId: null },
};
const demoDevice: Device = { id: 'demo-it400i', name: 'IT-400i', model: 'IT-400i', androidVersion: '9', apiLevel: 28, transports: [{ serial: 'DEMO-USB-001', kind: 'usb', state: 'device' }, { serial: '192.0.2.10:5555', kind: 'wifi', state: 'device' }], wifiAddress: '192.0.2.10:5555', audioSupported: false };
const demoSnapshot: Snapshot = { ...emptySnapshot, devices: [demoDevice], savedDevices: [{ id: demoDevice.id, name: demoDevice.name, address: '192.0.2.10:5555' }], engineVersion: 'scrcpy 5.0.1', settings: { ...emptySnapshot.settings, lastDeviceId: demoDevice.id, audio: false }, logs: [{ time: Date.now(), level: 'info', message: '界面演示模式。这里的设备仅用于预览，不代表真实连接。' }] };
const hasTransport = (device: Device, kind: 'usb' | 'wifi') => device.transports.some(t => t.kind === kind && t.state === 'device');
const isReady = (device: Device) => device.transports.some(t => t.state === 'device');
const addressValid = (value: string) => /^[^\s:/]+:\d{1,5}$/.test(value.trim()) && Number(value.trim().split(':').pop()) > 0 && Number(value.trim().split(':').pop()) <= 65535;
const errorMessage = (error: unknown) => error instanceof Error ? error.message : typeof error === 'string' ? error : '操作没有完成，请重试。';

export default function App() {
  const demo = new URLSearchParams(window.location.search).get('preview') === 'demo';
  const [snapshot, setSnapshot] = useState<Snapshot>(demo ? demoSnapshot : emptySnapshot);
  const [page, setPage] = useState<'devices' | 'settings'>('devices');
  const settingsQueue = useRef<Promise<unknown>>(Promise.resolve());
  const latestSettings = useRef(snapshot.settings);
  const [loaded, setLoaded] = useState(demo);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const [action, setAction] = useState<string | null>(null);
  const actionVersion = useRef(0);
  const requestVersion = useRef(0);
  const appliedVersion = useRef(0);
  const [toast, setToast] = useState<{ message: string; kind: 'success' | 'error' | 'info' } | null>(null);
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [wirelessOpen, setWirelessOpen] = useState(false);
  const [wirelessTab, setWirelessTab] = useState<'usb' | 'pair' | 'manual'>('usb');
  const [wirelessDeviceId, setWirelessDeviceId] = useState('');
  const [pairAddress, setPairAddress] = useState('');
  const [pairCode, setPairCode] = useState('');
  const [pairConnectAddress, setPairConnectAddress] = useState('');
  const [manualAddress, setManualAddress] = useState('');
  const [wirelessResult, setWirelessResult] = useState<string | null>(null);
  const [helpOpen, setHelpOpen] = useState(false);
  const [logsOpen, setLogsOpen] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);
  const selectedDevice = snapshot.devices.find(d => d.id === snapshot.settings.lastDeviceId) ?? (snapshot.devices.length === 1 ? snapshot.devices[0] : null);
  const usbDevices = snapshot.devices.filter(d => hasTransport(d, 'usb'));
  const session = snapshot.session;
  const selectedSession = session?.deviceId === selectedDevice?.id ? session : null;
  const busy = Boolean(action) || snapshot.busy;
  const audioUnsupported = selectedDevice?.audioSupported === false || (selectedDevice?.apiLevel != null && selectedDevice.apiLevel < 30);
  const selectedReady = selectedDevice ? isReady(selectedDevice) : false;
  const unauthorized = selectedDevice?.transports.some(t => t.state === 'unauthorized');

  useEffect(() => { latestSettings.current = snapshot.settings; }, [snapshot.settings]);

  function showToast(message: string, kind: 'success' | 'error' | 'info' = 'info') {
    if (toastTimer.current) clearTimeout(toastTimer.current);
    setToast({ message, kind });
    toastTimer.current = setTimeout(() => setToast(null), 5200);
  }

  useEffect(() => {
    if (demo) return;
    let alive = true;
    async function load() {
      const version = ++requestVersion.current;
      try { const next = await api.getSnapshot(); if (alive && version >= appliedVersion.current) { appliedVersion.current = version; setSnapshot(next); setConnectionError(null); } }
      catch (error) { if (alive && version >= appliedVersion.current) setConnectionError(errorMessage(error)); }
      finally { if (alive) setLoaded(true); }
    }
    void load();
    const timer = setInterval(() => { void load(); }, 2000);
    return () => { alive = false; clearInterval(timer); };
  }, [demo]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => { if (event.key === 'Escape') { setWirelessOpen(false); setHelpOpen(false); setLogsOpen(false); } };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  useEffect(() => {
    if (!wirelessOpen && !helpOpen) return;
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const frame = requestAnimationFrame(() => {
      const modal = document.querySelector<HTMLElement>('.modal');
      const first = modal?.querySelector<HTMLElement>('.modal-body input:not(:disabled), .modal-body select:not(:disabled)') ?? modal?.querySelector<HTMLElement>('.modal-tabs button[aria-selected="true"]') ?? modal?.querySelector<HTMLElement>('.modal-heading button');
      first?.focus();
    });
    const trapFocus = (event: KeyboardEvent) => {
      if (event.key !== 'Tab') return;
      const modal = document.querySelector<HTMLElement>('.modal');
      const items = [...(modal?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), select:not(:disabled), [tabindex="0"]') ?? [])].filter(element => element.offsetParent !== null);
      const first = items[0]; const last = items[items.length - 1];
      if (!first || !last) return;
      if (!modal?.contains(document.activeElement)) { event.preventDefault(); first.focus(); }
      else if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    };
    document.addEventListener('keydown', trapFocus);
    return () => { cancelAnimationFrame(frame); document.removeEventListener('keydown', trapFocus); if (previous?.isConnected) previous.focus(); };
  }, [wirelessOpen, helpOpen]);

  async function perform(name: string, operation: () => Promise<Snapshot>, success?: string): Promise<Snapshot | null> {
    if (demo) { showToast('这是界面演示。请在桌面应用中连接真实设备。'); return null; }
    if (!isTauri) { showToast('当前为界面预览，请在轻镜桌面应用中使用设备功能。'); return null; }
    const operationVersion = ++actionVersion.current;
    appliedVersion.current = ++requestVersion.current;
    setAction(name); setFormError(null);
    try {
      const next = await operation();
      if (operationVersion !== actionVersion.current) return null;
      appliedVersion.current = ++requestVersion.current;
      setSnapshot(next); setConnectionError(null);
      if (next.lastError) { showToast(next.lastError, 'error'); setFormError(next.lastError); return null; }
      if (success) showToast(success, 'success');
      return next;
    } catch (error) { if (operationVersion !== actionVersion.current) return null; const message = errorMessage(error); showToast(message, 'error'); setFormError(message); return null; }
    finally { if (operationVersion === actionVersion.current) setAction(null); }
  }

  function updateSettings(patch: Partial<Settings>): Promise<boolean> {
    const save = settingsQueue.current.catch(() => undefined).then(async () => {
      const next = { ...latestSettings.current, ...patch };
      if (demo) { latestSettings.current = next; setSnapshot(current => ({ ...current, settings: next })); return true; }
      if (!isTauri) {
        try { const saved = await api.saveSettings(next); latestSettings.current = saved.settings; setSnapshot(saved); return true; }
        catch (error) { showToast(errorMessage(error), 'error'); return false; }
      }
      const saved = await perform('settings', () => api.saveSettings(next));
      if (saved) latestSettings.current = saved.settings;
      return Boolean(saved);
    });
    settingsQueue.current = save;
    return save;
  }

  function selectDevice(device: Device) {
    if (demo) { setSnapshot(current => ({ ...current, settings: { ...current.settings, lastDeviceId: device.id } })); return; }
    void perform('select', () => api.selectDevice(device.id));
  }

  function openWireless(tab: 'usb' | 'pair' | 'manual' = 'usb') {
    setWirelessTab(tab); setFormError(null); setWirelessResult(null);
    setWirelessDeviceId(selectedDevice && hasTransport(selectedDevice, 'usb') ? selectedDevice.id : usbDevices[0]?.id ?? '');
    const savedDevice = snapshot.savedDevices.find(device => device.id === (selectedDevice?.id ?? snapshot.settings.lastDeviceId));
    setManualAddress(selectedDevice?.wifiAddress ?? savedDevice?.address ?? snapshot.savedDevices[0]?.address ?? '');
    setWirelessOpen(true);
  }

  async function configureUsb() {
    const deviceId = wirelessDeviceId || usbDevices[0]?.id;
    if (!deviceId) return;
    const next = await perform('configure', () => api.configureWireless(deviceId));
    if (next) {
      const device = next.devices.find(d => d.id === deviceId);
      setWirelessResult(device && hasTransport(device, 'wifi') ? '无线连接已验证，可以拔掉数据线了。' : '无线配置已完成，请查看设备连接状态。');
    }
  }

  async function submitPair(event: FormEvent) {
    event.preventDefault();
    if (!addressValid(pairAddress)) { setFormError('请输入包含端口的配对地址，例如 192.168.1.8:37123。'); return; }
    if (!/^\d{6}$/.test(pairCode.trim())) { setFormError('请输入手机上显示的 6 位配对码。'); return; }
    if (pairConnectAddress.trim() && !addressValid(pairConnectAddress)) { setFormError('请检查连接地址，地址后需要填写端口。'); return; }
    const next = await perform('pair', () => api.pairWireless(pairAddress.trim(), pairCode.trim(), pairConnectAddress.trim() || null));
    if (next) setWirelessResult(next.devices.some(d => hasTransport(d, 'wifi')) ? '配对完成，设备已通过 Wi-Fi 连接。' : '配对已完成。如设备尚未出现，请填写无线调试主页上的连接地址。');
  }

  async function submitManual(event: FormEvent) {
    event.preventDefault();
    if (!addressValid(manualAddress)) { setFormError('请输入设备 IP 和端口，例如 192.168.1.8:5555。'); return; }
    const next = await perform('connect', () => api.connectWireless(manualAddress.trim()));
    if (next) {
      setWirelessResult(next.devices.some(d => d.transports.some(t => t.kind === 'wifi' && t.state === 'device' && t.serial === manualAddress.trim())) ? '无线设备已连接，现在可以开始投屏。' : '连接操作已完成，请查看设备列表中的状态。');
    }
  }

  async function windowAction(kind: 'minimize' | 'hide') {
    if (!isTauri) { showToast('当前为界面预览，窗口操作在桌面应用中可用。'); return; }
    try { await (kind === 'minimize' ? api.minimizeWindow() : api.hideWindow()); } catch (error) { showToast(errorMessage(error), 'error'); }
  }

  const cannotStartTransport = selectedDevice && snapshot.settings.preferredTransport !== 'auto' && !hasTransport(selectedDevice, snapshot.settings.preferredTransport);

  return <div className="app-shell">
    <header className="titlebar" data-tauri-drag-region>
      <div className="brand" data-tauri-drag-region><MirrorMark/><span data-tauri-drag-region>Qzz Mirror</span><span className="brand-divider"/><span className="brand-cn" data-tauri-drag-region>轻镜</span></div>
      <div className="titlebar-right">
        {(!isTauri || demo) && <span className="preview-label">{demo ? '演示预览 · 非真实设备' : '界面预览'}</span>}
        <button className={`icon-button settings-button ${page === 'settings' ? 'active' : ''}`} onClick={() => { setPage('settings'); setLogsOpen(false); }} aria-label="设置" title="设置"><Icon name="settings" size={18}/></button>
        <button className="icon-button help-button" onClick={() => setHelpOpen(true)} aria-label="使用帮助" title="使用帮助"><Icon name="help" size={18}/></button>
        <span className="window-separator"/>
        <button className="window-button" onClick={() => void windowAction('minimize')} aria-label="最小化" title="最小化"><Icon name="minus" size={16}/></button>
        <button className="window-button close-window" onClick={() => void windowAction('hide')} aria-label="关闭到托盘" title="关闭到托盘，投屏继续"><Icon name="close" size={16}/></button>
      </div>
    </header>

    {page === 'settings' ? <SettingsPage settings={snapshot.settings} audioUnsupported={audioUnsupported} disabled={busy && action !== 'settings'} sessionActive={Boolean(session)} saving={action === 'settings'} onSave={updateSettings} onBack={() => setPage('devices')}/> : <div className="workspace">
      <aside className="sidebar">
        <div className="sidebar-heading"><span>设备 <span className="count-badge">{snapshot.devices.length}</span></span><button className={`icon-button ${action === 'refresh' ? 'spinning' : ''}`} disabled={busy} onClick={() => void perform('refresh', () => api.refresh())} aria-label="刷新设备" title="刷新设备"><Icon name="refresh" size={16}/></button></div>
        <div className="device-list">
          {!loaded ? <div className="list-placeholder"><span className="spinner"/>查找设备中</div> : snapshot.devices.length === 0 ? <div className="sidebar-empty"><Icon name="phone" size={23}/><span>暂无设备</span></div> : snapshot.devices.map(device => <button key={device.id} className={`device-item ${selectedDevice?.id === device.id ? 'selected' : ''}`} disabled={busy && action !== 'settings'} onClick={() => selectDevice(device)}>
            <span className="device-item-icon"><Icon name="phone" size={24}/><i className={isReady(device) ? 'online' : 'offline'}/></span>
            <span className="device-item-info"><strong>{device.name || device.model}</strong><span>{device.androidVersion ? `Android ${device.androidVersion}` : 'Android 设备'}</span><span className="device-transport-tags">{hasTransport(device, 'usb') && <span><Icon name="usb" size={11}/>USB</span>}{hasTransport(device, 'wifi') && <span><Icon name="wifi" size={11}/>Wi-Fi</span>}{!isReady(device) && <span className="unavailable-tag">{device.transports.some(t => t.state === 'unauthorized') ? '等待授权' : '暂不可用'}</span>}</span></span>
            {selectedDevice?.id === device.id && <span className="selected-indicator"/>}
          </button>)}
        </div>
        {snapshot.savedDevices.length > 0 && <div className="saved-section"><span className="sidebar-section-label">已保存的无线设备</span>{snapshot.savedDevices.map(device => <button key={`${device.id}-${device.address}`} className="saved-device" onClick={() => { openWireless('manual'); setManualAddress(device.address); }}><Icon name="wifi" size={15}/><span><strong>{device.name}</strong><small>{device.address}</small></span><Icon name="chevron" size={12}/></button>)}</div>}
        <div className="sidebar-bottom"><button className="add-wireless" onClick={() => openWireless()}><Icon name="plus" size={17}/>连接无线设备</button></div>
      </aside>

      <main className="main-content">
        <div className="page-heading"><h1>设备投屏</h1><button className={`text-button log-trigger ${snapshot.logs.some(l => l.level === 'error') ? 'has-error' : ''}`} onClick={() => setLogsOpen(true)}><Icon name="log" size={16}/>连接日志</button></div>
        {connectionError && isTauri && <div className="inline-alert"><Icon name="alert" size={17}/><span>{connectionError}</span><button disabled={busy} onClick={() => void perform('refresh', () => api.refresh())}>重试</button></div>}
        {snapshot.lastError && <div className="inline-alert device-error" role="alert"><Icon name="alert" size={18}/><div className="device-error-copy"><strong>连接遇到问题</strong><span>{snapshot.lastError}</span></div><div className="device-error-actions"><button disabled={busy} onClick={() => selectedDevice && selectedReady && !session ? void perform('start', () => api.startMirror(selectedDevice.id, snapshot.settings.preferredTransport)) : void perform('refresh', () => api.refresh())}>{selectedDevice && selectedReady && !session ? '重试投屏' : '刷新设备'}</button><button className="reconnect-button" onClick={() => openWireless('manual')}><Icon name="wifi" size={13}/>重新连接</button></div></div>}
        {session && !selectedSession && <div className="session-notice"><span className="status-dot"/><span>{session.deviceName} 正在投屏</span><button disabled={action === 'stop'} onClick={() => void perform('stop', () => api.stopMirror())}>停止投屏</button></div>}
        {selectedDevice ? <section className={`mirror-panel ${selectedSession ? 'is-mirroring' : ''}`}>
          <div className="device-summary"><span className="summary-device-icon"><Icon name="phone" size={32}/></span><div className="summary-copy"><h2>{selectedDevice.name || selectedDevice.model}</h2><p className="device-meta">{selectedDevice.androidVersion ? `Android ${selectedDevice.androidVersion}` : 'Android 设备'}<span>·</span>{hasTransport(selectedDevice, 'usb') && hasTransport(selectedDevice, 'wifi') ? 'USB / Wi-Fi' : hasTransport(selectedDevice, 'usb') ? 'USB' : hasTransport(selectedDevice, 'wifi') ? 'Wi-Fi' : unauthorized ? '请允许 USB 调试' : '未连接'}</p></div><span className={`connection-pill ${selectedReady ? '' : 'pending'}`}><span className="status-dot"/>{selectedSession?.phase === 'running' ? `${selectedSession.transport === 'usb' ? 'USB' : 'Wi-Fi'} 投屏中` : selectedSession?.phase === 'reconnecting' ? '恢复连接中' : selectedSession?.phase === 'starting' ? '启动中' : selectedReady ? '已就绪' : unauthorized ? '等待授权' : '暂不可用'}</span></div>
          <div className="transport-control"><span className="control-label">连接方式</span><div className="segmented-control" aria-label="连接方式">{(['auto', 'usb', 'wifi'] as const).map(transport => <button key={transport} aria-pressed={snapshot.settings.preferredTransport === transport} className={snapshot.settings.preferredTransport === transport ? 'active' : ''} title={transport === 'auto' ? '优先使用 USB，断线后尝试已配置的 Wi-Fi' : undefined} disabled={busy} onClick={() => void updateSettings({ preferredTransport: transport })}>{transport === 'auto' ? '自动' : <><Icon name={transport} size={14}/>{transport === 'usb' ? 'USB' : 'Wi-Fi'}</>}</button>)}</div></div>
          {cannotStartTransport && <div className="transport-note unavailable"><Icon name="alert" size={14}/><span>{snapshot.settings.preferredTransport === 'usb' ? 'USB 未连接，请插入数据线。' : 'Wi-Fi 未连接，请先配置无线。'}</span></div>}
          <div className="mirror-actions"><button className="secondary-button wireless-action" onClick={() => openWireless()}><Icon name="wifi" size={16}/>配置无线</button>{selectedSession ? <button className="primary-button stop-button" disabled={action === 'stop'} onClick={() => void perform('stop', () => api.stopMirror())}>{action === 'stop' ? <span className="spinner"/> : <Icon name="stop" size={17}/>}停止投屏</button> : <button className="primary-button start-button" disabled={busy || !selectedReady || Boolean(session) || Boolean(cannotStartTransport)} onClick={() => void perform('start', () => api.startMirror(selectedDevice.id, snapshot.settings.preferredTransport))}>{action === 'start' ? <span className="spinner"/> : <Icon name="play" size={17}/>} {action === 'start' ? '正在启动' : '开始投屏'}<Icon name="arrow" size={16}/></button>}</div>
        </section> : <section className="empty-panel"><span className="empty-main-icon"><Icon name="phone" size={32}/></span><h2>{!loaded ? '正在查找设备' : snapshot.devices.length ? '选择一台设备' : '暂无连接的设备'}</h2><div className="empty-actions">{snapshot.devices.length === 0 && <><button className="secondary-button" onClick={() => setHelpOpen(true)}><Icon name="usb" size={16}/>USB 连接指南</button><button className="primary-button" onClick={() => openWireless('pair')}><Icon name="wifi" size={16}/>连接无线设备</button></>}</div></section>}
      </main>
    </div>}

    {wirelessOpen && <div className="modal-backdrop" onMouseDown={event => { if (event.target === event.currentTarget && !busy) setWirelessOpen(false); }}><section className="modal wireless-modal" role="dialog" aria-modal="true" aria-labelledby="wireless-title"><div className="modal-heading"><span className="modal-heading-icon"><Icon name="wifi" size={22}/></span><div><h2 id="wireless-title">连接无线设备</h2><p>让手机与电脑连接同一局域网</p></div><button className="icon-button" aria-label="关闭无线配置" disabled={busy} onClick={() => setWirelessOpen(false)}><Icon name="close" size={19}/></button></div><div className="modal-tabs" role="tablist">{([{ id: 'usb', label: 'USB 辅助配置' }, { id: 'pair', label: '配对码连接' }, { id: 'manual', label: '手动重连' }] as const).map(tab => <button key={tab.id} role="tab" aria-selected={wirelessTab === tab.id} disabled={busy} className={wirelessTab === tab.id ? 'active' : ''} onClick={() => { setWirelessTab(tab.id); setFormError(null); setWirelessResult(null); }}>{tab.label}</button>)}</div><div className="modal-body">
          {wirelessTab === 'usb' && <><div className="tab-intro"><h3>先插线，随后就能无线使用</h3><p>首次设置需要数据线，完成后可使用 Wi-Fi 投屏。</p></div><ol className="instruction-steps"><li><span>1</span><div><strong>用数据线连接设备</strong><p>开启「开发者选项 → USB 调试」，在手机上允许授权。</p></div></li><li><span>2</span><div><strong>确认已连接 Wi-Fi</strong><p>设备与电脑需在同一网络，保持设备屏幕解锁。</p></div></li><li><span>3</span><div><strong>配置成功后再拔线</strong><p>轻镜会自动获取地址，开启并验证无线连接。</p></div></li></ol>{usbDevices.length ? <label className="form-field"><span>选择 USB 设备</span><select value={wirelessDeviceId || usbDevices[0]?.id} onChange={e => setWirelessDeviceId(e.target.value)} disabled={busy}>{usbDevices.map(device => <option key={device.id} value={device.id}>{device.name} · USB 已连接</option>)}</select></label> : <div className="form-hint"><Icon name="usb" size={17}/><span>尚未检测到已授权的 USB 设备。连接后点击刷新。</span><button className="text-button" disabled={busy} onClick={() => void perform('refresh', () => api.refresh())}>刷新</button></div>}<button className="primary-button modal-submit" disabled={busy || usbDevices.length === 0} onClick={() => void configureUsb()}>{action === 'configure' ? <span className="spinner"/> : <Icon name="wifi" size={17}/>} {action === 'configure' ? '正在配置并验证…' : '配置并验证无线连接'}</button></>}
          {wirelessTab === 'pair' && <form onSubmit={event => void submitPair(event)}><div className="tab-intro"><h3>用配对码连接，无需插线</h3><p>适用于 Android 11 及以上。开启「开发者选项 → 无线调试」。</p></div><div className="pair-tip"><span className="step-number">1</span><p>在手机上点击<strong>「使用配对码配对设备」</strong><br/><span>将弹窗里的地址和配对码填在下方。</span></p></div><div className="form-row"><label className="form-field address-field"><span>配对地址</span><input autoComplete="off" placeholder="192.168.1.8:37123" value={pairAddress} onChange={e => setPairAddress(e.target.value)} disabled={busy}/></label><label className="form-field code-field"><span>6 位配对码</span><input inputMode="numeric" autoComplete="off" maxLength={6} placeholder="000000" value={pairCode} onChange={e => setPairCode(e.target.value.replace(/\D/g, ''))} disabled={busy}/></label></div><label className="form-field"><span>连接地址 <small>可选</small></span><input autoComplete="off" placeholder="无线调试主页显示的 IP 地址和端口" value={pairConnectAddress} onChange={e => setPairConnectAddress(e.target.value)} disabled={busy}/><small>配对端口与连接端口通常不同。留空时会尝试自动发现。</small></label><button type="submit" className="primary-button modal-submit" disabled={busy || !pairAddress || pairCode.length !== 6}>{action === 'pair' ? <span className="spinner"/> : <Icon name="wifi" size={17}/>} {action === 'pair' ? '正在配对…' : '配对并连接'}</button></form>}
          {wirelessTab === 'manual' && <form onSubmit={event => void submitManual(event)}><div className="tab-intro"><h3>重新连接已配置的设备</h3><p>输入无线调试地址，或选用之前保存的地址。</p></div>{snapshot.savedDevices.length > 0 && <div className="saved-addresses">{snapshot.savedDevices.map(device => <button type="button" key={device.address} className={manualAddress === device.address ? 'selected' : ''} onClick={() => setManualAddress(device.address)}><Icon name="phone" size={18}/><span><strong>{device.name}</strong><small>{device.address}</small></span>{manualAddress === device.address && <Icon name="check" size={15}/>}</button>)}</div>}<label className="form-field"><span>设备 IP 与端口</span><input autoComplete="off" placeholder="192.168.1.8:5555" value={manualAddress} onChange={e => setManualAddress(e.target.value)} disabled={busy}/><small>设备重启或网络变化后，可能需要重新配置无线连接。</small></label><button type="submit" className="primary-button modal-submit" disabled={busy || !manualAddress}>{action === 'connect' ? <span className="spinner"/> : <Icon name="wifi" size={17}/>} {action === 'connect' ? '正在连接…' : '连接设备'}</button></form>}
          {formError && <div className="modal-feedback error" role="alert"><Icon name="alert" size={17}/><span>{formError}</span></div>}{wirelessResult && <div className="modal-feedback success" role="status"><Icon name="check" size={17}/><span>{wirelessResult}</span></div>}
        </div><div className="modal-footer"><Icon name="help" size={14}/><span>连接问题？</span><button onClick={() => { setWirelessOpen(false); setHelpOpen(true); }}>查看连接指南</button><button className="modal-done" disabled={busy} onClick={() => setWirelessOpen(false)}>完成</button></div></section></div>}

    {helpOpen && <div className="modal-backdrop" onMouseDown={event => { if (event.target === event.currentTarget) setHelpOpen(false); }}><section className="modal help-modal" role="dialog" aria-modal="true" aria-labelledby="help-title"><div className="modal-heading"><span className="modal-heading-icon"><Icon name="help" size={22}/></span><div><h2 id="help-title">连接，其实很简单</h2><p>轻镜 · 快速使用指南</p></div><button className="icon-button" aria-label="关闭帮助" onClick={() => setHelpOpen(false)}><Icon name="close" size={19}/></button></div><div className="modal-body"><div className="help-section"><h3><Icon name="usb" size={18}/>第一次用 USB 连接</h3><ol><li>在 Android 设备中开启「开发者选项」及「USB 调试」。</li><li>用支持数据传输的数据线连接电脑，在设备上允许 USB 调试。</li><li>设备出现后，点击「开始投屏」。没有出现时，可点击设备列表右上角刷新。</li></ol></div><div className="help-section"><h3><Icon name="wifi" size={18}/>想要拔掉数据线</h3><p>先让手机和电脑接入同一局域网，再打开「配置无线」。旧版 Android 选择「USB 辅助配置」，看到连接验证成功后再拔线；Android 11 及以上也可以使用配对码。</p><p>重启设备或更换网络后，无线地址或调试状态可能改变，需要重新连接或配置。</p></div><div className="help-section"><h3><Icon name="phone" size={18}/>投屏与窗口</h3><p>电脑鼠标和键盘可直接操作设备。声音转发需要 Android 11 及以上，部分设备可能限制键鼠控制，需在开发者选项中另行允许。</p><p>关闭控制面板会缩到系统托盘，投屏继续运行。需要彻底关闭时，在托盘图标上选择「退出」。</p></div><div className="help-bottom"><MirrorMark small/><span>Qzz Mirror · 轻镜<small>基于 scrcpy，专注本地连接体验</small></span></div></div></section></div>}

    {logsOpen && <div className="drawer-backdrop" onMouseDown={event => { if (event.target === event.currentTarget) setLogsOpen(false); }}><aside className="log-drawer" aria-label="连接日志"><div className="drawer-heading"><div><h2>连接日志</h2><p>最近的设备与投屏活动</p></div><button className="icon-button" onClick={() => setLogsOpen(false)} aria-label="关闭日志"><Icon name="close" size={19}/></button></div><div className="log-list">{snapshot.logs.length === 0 ? <div className="logs-empty"><Icon name="log" size={32}/><strong>暂时没有连接活动</strong><span>开始连接设备后，日志会出现在这里。</span></div> : [...snapshot.logs].reverse().map((entry, index) => <div className={`log-entry ${entry.level}`} key={`${entry.time}-${index}`}><span className="log-level-dot"/><div><time>{new Date(entry.time).toLocaleTimeString('zh-CN', { hour12: false })}</time><p>{entry.message}</p></div></div>)}</div><div className="drawer-footer"><span>{snapshot.logs.length} 条记录</span><button className="secondary-button" onClick={() => { const text = snapshot.logs.map(log => `[${new Date(log.time).toLocaleString('zh-CN')}] [${log.level}] ${log.message}`).join('\n'); void navigator.clipboard.writeText(text).then(() => showToast('日志已复制', 'success')).catch(() => showToast('复制失败，请重试。', 'error')); }} disabled={!snapshot.logs.length}><Icon name="copy" size={15}/>复制日志</button></div></aside></div>}
    {toast && <div className={`toast ${toast.kind}`} role={toast.kind === 'error' ? 'alert' : 'status'}><span className="toast-icon"><Icon name={toast.kind === 'error' ? 'alert' : toast.kind === 'success' ? 'check' : 'help'} size={17}/></span><span>{toast.message}</span><button className="icon-button" onClick={() => setToast(null)} aria-label="关闭提示"><Icon name="close" size={14}/></button></div>}
  </div>;
}
