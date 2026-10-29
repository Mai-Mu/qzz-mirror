import { useEffect, useRef, useState, type CSSProperties } from 'react';
import type { Settings } from './types';

interface Props {
  settings: Settings;
  audioUnsupported: boolean;
  disabled: boolean;
  sessionActive: boolean;
  saving: boolean;
  onSave: (patch: Partial<Settings>) => Promise<boolean>;
  onBack: () => void;
}

function SettingIcon({ kind }: { kind: 'screen' | 'fps' | 'volume' | 'pin' }) {
  return <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.65" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    {kind === 'screen' ? <><rect x="3" y="4" width="18" height="13" rx="2"/><path d="M8 21h8m-4-4v4"/></> : kind === 'fps' ? <path d="M3 12h4l3-7 4 14 3-7h4"/> : kind === 'volume' ? <><path d="m11 4-5 4H3v8h3l5 4zm4 4a6 6 0 0 1 0 8m3-11a10 10 0 0 1 0 14"/></> : <><path d="m16 3 5 5-3 2-2 5-3 1-5-5 1-3 5-2zM3 21l7-7"/></>}
  </svg>;
}

export default function SettingsPage({ settings, audioUnsupported, disabled, sessionActive, saving, onSave, onBack }: Props) {
  const [draft, setDraft] = useState(settings.maxFps === null ? '' : String(settings.maxFps));
  const [fpsError, setFpsError] = useState<string | null>(null);
  const [fpsSaving, setFpsSaving] = useState(false);
  const editing = useRef(false);
  const sliderChanged = useRef(false);
  const draftValue = useRef(draft);
  const confirmed = useRef(settings.maxFps);
  const submitted = useRef(settings.maxFps);
  const saveCount = useRef(0);
  const saveQueue = useRef<Promise<void>>(Promise.resolve());

  useEffect(() => {
    confirmed.current = settings.maxFps;
    if (!editing.current && saveCount.current === 0) {
      const next = settings.maxFps === null ? '' : String(settings.maxFps);
      draftValue.current = next; setDraft(next); submitted.current = settings.maxFps;
    }
  }, [settings.maxFps]);

  function changeDraft(value: string) {
    editing.current = true; draftValue.current = value; setDraft(value); setFpsError(null);
  }

  function commit(value: number | null) {
    if (value === submitted.current) return;
    submitted.current = value;
    saveCount.current += 1; setFpsSaving(true);
    saveQueue.current = saveQueue.current.then(async () => {
      const saved = await onSave({ maxFps: value });
      if (saved) confirmed.current = value;
      else {
        if (submitted.current === value) submitted.current = confirmed.current;
        setFpsError('帧率未保存，请重试。');
      }
    }).catch(() => { submitted.current = confirmed.current; setFpsError('帧率未保存，请重试。'); }).finally(() => {
      saveCount.current -= 1; setFpsSaving(saveCount.current > 0);
    });
  }

  function commitNumber() {
    editing.current = false;
    const value = draftValue.current.trim();
    if (!value) {
      if (submitted.current !== null) setFpsError('请输入 1–240 的整数，或点击「最高」。');
      return;
    }
    const parsed = Number(value);
    if (!Number.isInteger(parsed) || parsed < 1 || parsed > 240) { setFpsError('帧率应为 1–240 的整数。'); return; }
    setFpsError(null); commit(parsed);
  }

  function commitSlider() {
    editing.current = false;
    if (!sliderChanged.current) return;
    sliderChanged.current = false;
    const value = draftValue.current === '' ? null : Number(draftValue.current);
    if (value !== null && (!Number.isInteger(value) || value < 1 || value > 240)) return;
    commit(value);
  }

  function useHighest() {
    editing.current = false; sliderChanged.current = false; draftValue.current = ''; setDraft(''); setFpsError(null); commit(null);
  }

  const numericDraft = Number(draft);
  const sliderValue = draft === '' ? 241 : Number.isFinite(numericDraft) ? Math.max(1, Math.min(240, Math.round(numericDraft))) : 1;
  const rangeStyle = { '--range-progress': `${((sliderValue - 1) / 240) * 100}%` } as CSSProperties;

  return <main className="settings-content">
    <div className="settings-heading"><button className="back-button" onClick={onBack} aria-label="返回设备投屏"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M19 12H5m6-6-6 6 6 6"/></svg></button><h1>设置</h1>{(saving || fpsSaving) && <span className="saving-state"><span className="spinner"/>正在保存</span>}</div>
    {sessionActive && <p className="settings-session-note">设置将在下次开始投屏时生效。</p>}
    <section className="settings-panel" aria-label="投屏设置">
      <div className="setting-row resolution-row"><span className="setting-icon"><SettingIcon kind="screen"/></span><strong>分辨率</strong><span className="resolution-value">设备原始分辨率</span></div>
      <div className="fps-setting"><div className="setting-row fps-heading"><span className="setting-icon"><SettingIcon kind="fps"/></span><strong>帧率上限</strong><div className="fps-number-control"><input type="number" inputMode="numeric" min={1} max={240} step={1} aria-label="帧率上限" placeholder="最高" value={draft} disabled={disabled} onFocus={() => { editing.current = true; }} onChange={event => changeDraft(event.target.value)} onBlur={commitNumber} onKeyDown={event => { if (event.key === 'Enter') { event.preventDefault(); commitNumber(); event.currentTarget.blur(); } }}/><span>FPS</span></div><button className={`highest-button ${draft === '' ? 'selected' : ''}`} disabled={disabled} onClick={useHighest}>最高</button></div>
        <div className="fps-slider-area"><input className="fps-slider" type="range" min={1} max={241} step={1} value={sliderValue} style={rangeStyle} aria-label="调整帧率上限" aria-valuetext={draft === '' ? '最高，不设帧率上限' : `${draft} FPS`} disabled={disabled} onPointerDown={() => { editing.current = true; }} onChange={event => { sliderChanged.current = true; changeDraft(event.target.value === '241' ? '' : event.target.value); }} onPointerUp={commitSlider} onPointerCancel={commitSlider} onKeyUp={event => { if (['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home', 'End', 'PageUp', 'PageDown'].includes(event.key)) commitSlider(); }} onBlur={commitSlider}/><div className="range-labels"><span>1 FPS</span><span>120</span><span>最高</span></div><p className="fps-help">最高：不设帧率上限</p>{fpsError && <p className="fps-error" role="alert">{fpsError}</p>}</div>
      </div>
      <div className={`setting-row toggle-setting ${audioUnsupported ? 'unavailable' : ''}`}><span className="setting-icon"><SettingIcon kind="volume"/></span><div className="setting-copy"><strong>转发设备声音</strong>{audioUnsupported && <span>当前设备的 Android 版本不支持</span>}</div><button className={`toggle-switch ${settings.audio && !audioUnsupported ? 'on' : ''}`} role="switch" aria-checked={settings.audio && !audioUnsupported} aria-label="转发设备声音" disabled={disabled || fpsSaving || audioUnsupported} onClick={() => void onSave({ audio: !settings.audio })}><span/></button></div>
      <div className="setting-row toggle-setting"><span className="setting-icon"><SettingIcon kind="pin"/></span><strong>投屏窗口置顶</strong><button className={`toggle-switch ${settings.alwaysOnTop ? 'on' : ''}`} role="switch" aria-checked={settings.alwaysOnTop} aria-label="投屏窗口置顶" disabled={disabled || fpsSaving} onClick={() => void onSave({ alwaysOnTop: !settings.alwaysOnTop })}><span/></button></div>
    </section>
  </main>;
}
