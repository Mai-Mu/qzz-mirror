use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransportPreference {
    #[default]
    Auto,
    Usb,
    Wifi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// None keeps the device's own maximum frame rate (no scrcpy cap).
    pub max_fps: Option<u16>,
    pub audio: bool,
    pub always_on_top: bool,
    pub preferred_transport: TransportPreference,
    pub last_device_id: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_fps: None,
            audio: true,
            always_on_top: false,
            preferred_transport: TransportPreference::Auto,
            last_device_id: None,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_fps.is_some_and(|fps| !(1..=240).contains(&fps)) {
            return Err("帧率必须是 1～240 的整数，或选择「最高」。".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransportKind {
    Usb,
    Wifi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transport {
    pub serial: String,
    pub kind: TransportKind,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub name: String,
    pub model: String,
    pub android_version: Option<String>,
    pub api_level: Option<u32>,
    pub transports: Vec<Transport>,
    pub wifi_address: Option<String>,
    pub audio_supported: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedDevice {
    pub id: String,
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionPhase {
    Starting,
    Running,
    Reconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub device_id: String,
    pub device_name: String,
    pub serial: String,
    pub transport: TransportKind,
    pub phase: SessionPhase,
    pub started_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub time: u64,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub devices: Vec<Device>,
    pub saved_devices: Vec<SavedDevice>,
    pub settings: Settings,
    pub session: Option<Session>,
    pub logs: Vec<LogEntry>,
    pub engine_version: String,
    pub busy: bool,
    pub last_error: Option<String>,
}
