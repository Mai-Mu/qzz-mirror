//! In-memory USB attachment policy. No timers, UI, ADB, or notification API.
//! A successful device refresh advances the baseline even when notifications
//! are suppressed, so minimizing later never replays an old attachment.
use std::{collections::{HashMap, HashSet}, time::{Duration, Instant}};

const ACTION_LIFETIME: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbNotification {
    pub token: String,
    pub device_id: String,
    pub device_name: String,
    pub usb_serial: String,
    pub adb_transport_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsbNotificationEvent {
    Show(UsbNotification),
    Remove { token: String },
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbPresence {
    pub serial: String,
    pub device_id: String,
    pub device_name: String,
    pub transport_id: Option<String>,
    pub authorized: bool,
}

impl UsbPresence {
    fn matches(&self, notice: &UsbNotification) -> bool {
        self.authorized && self.serial == notice.usb_serial
            && self.device_id == notice.device_id
            && self.transport_id.as_deref() == Some(notice.adb_transport_id.as_str())
    }
    fn ready(&self) -> bool { self.authorized && self.transport_id.is_some() }
    fn same_attachment(&self, other: &Self) -> bool {
        self.serial == other.serial && self.device_id == other.device_id
            && self.transport_id == other.transport_id && self.authorized == other.authorized
    }
}

struct Pending { notice: UsbNotification, created: Instant }

#[derive(Default)]
pub struct UsbNotificationPolicy {
    initialized: bool,
    present: HashMap<String, UsbPresence>,
    pending: HashMap<String, Pending>,
}

impl UsbNotificationPolicy {
    /// `allowed` is sampled for this discovery, not when the UI later drains
    /// its queue. A missing attachment identity fails closed for actions only.
    pub fn observe(&mut self, devices: Vec<UsbPresence>, allowed: bool, mut token: impl FnMut() -> Option<String>) -> Vec<UsbNotification> {
        let next: HashMap<_, _> = devices.into_iter().map(|device| (device.serial.clone(), device)).collect();
        self.pending.retain(|_, pending| pending.created.elapsed() < ACTION_LIFETIME
            && next.get(&pending.notice.usb_serial).is_some_and(|current| current.matches(&pending.notice)));
        let mut notified = HashSet::new();
        let mut notices = Vec::new();
        if self.initialized && allowed {
            // Sort makes simultaneous attachments deterministic; duplicate USB
            // transports for one physical identity still produce one toast.
            let mut candidates: Vec<_> = next.values().collect();
            candidates.sort_by(|left, right| left.serial.cmp(&right.serial));
            for current in candidates {
                let changed = self.present.get(&current.serial).is_none_or(|old| !current.same_attachment(old));
                if !current.ready() || !changed || !notified.insert(current.device_id.clone()) { continue; }
                let Some(token) = token().filter(|token| !token.is_empty() && !self.pending.contains_key(token)) else { continue; };
                let notice = UsbNotification {
                    token: token.clone(), device_id: current.device_id.clone(), device_name: current.device_name.clone(),
                    usb_serial: current.serial.clone(), adb_transport_id: current.transport_id.clone().unwrap(),
                };
                self.pending.insert(token, Pending { notice: notice.clone(), created: Instant::now() });
                notices.push(notice);
            }
        }
        self.present = next;
        self.initialized = true;
        notices
    }

    pub fn is_current(&self, token: &str) -> bool {
        self.pending.get(token).is_some_and(|pending| pending.created.elapsed() < ACTION_LIFETIME
            && self.present.get(&pending.notice.usb_serial).is_some_and(|current| current.matches(&pending.notice)))
    }

    pub fn attachment_is_current(&self, notice: &UsbNotification) -> bool {
        self.present.get(&notice.usb_serial).is_some_and(|current| current.matches(notice))
    }

    /// Remove before starting. A repeated activation cannot launch twice, even
    /// if a previous start fails and this same notification is clicked again.
    pub fn claim(&mut self, token: &str) -> Option<UsbNotification> {
        if !self.is_current(token) { self.pending.remove(token); return None; }
        self.pending.remove(token).map(|pending| pending.notice)
    }

    pub fn revoke_all(&mut self) { self.pending.clear(); }

    pub fn pending_tokens(&self) -> Vec<String> { self.pending.keys().cloned().collect() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn usb(serial: &str, id: &str, transport: &str, authorized: bool) -> UsbPresence {
        UsbPresence { serial: serial.into(), device_id: id.into(), device_name: id.into(), transport_id: Some(transport.into()), authorized }
    }
    fn token() -> Option<String> { Some("test-opaque-token".into()) }
    #[test]
    fn startup_and_suppressed_discovery_never_replay() {
        let mut policy = UsbNotificationPolicy::default();
        let a = usb("USB-A", "android:A", "1", true);
        assert!(policy.observe(vec![a.clone()], true, token).is_empty());
        assert!(policy.observe(vec![a.clone()], true, token).is_empty());
        policy.observe(vec![], true, token);
        assert!(policy.observe(vec![a.clone()], false, token).is_empty());
        assert!(policy.observe(vec![a], true, token).is_empty());
    }
    #[test]
    fn authorization_and_reinsert_have_one_shot_generation_bound_actions() {
        let mut policy = UsbNotificationPolicy::default();
        policy.observe(vec![], true, token);
        assert!(policy.observe(vec![usb("USB-A", "android:A", "1", false)], true, token).is_empty());
        let first = policy.observe(vec![usb("USB-A", "android:A", "1", true)], true, token).pop().unwrap();
        assert!(policy.observe(vec![usb("USB-A", "android:A", "1", true)], true, token).is_empty());
        assert!(policy.is_current(&first.token));
        let second = policy.observe(vec![usb("USB-A", "android:A", "2", true)], true, || Some("new-token".into())).pop().unwrap();
        assert!(!policy.is_current(&first.token));
        assert!(policy.claim(&first.token).is_none());
        assert_eq!(policy.claim(&second.token), Some(second.clone()));
        assert!(policy.claim(&second.token).is_none());
    }
    #[test]
    fn duplicate_identity_and_missing_identity_fail_closed() {
        let mut policy = UsbNotificationPolicy::default();
        policy.observe(vec![], true, token);
        let a = usb("USB-A", "android:A", "1", true);
        let b = usb("USB-B", "android:A", "2", true);
        assert_eq!(policy.observe(vec![a, b], true, token).len(), 1);
        policy.revoke_all();
        assert!(!policy.is_current("test-opaque-token"));
        let mut missing = usb("USB-C", "android:C", "3", true); missing.transport_id = None;
        assert!(policy.observe(vec![missing], true, token).is_empty());
    }
    #[test]
    fn offline_identity_change_and_unplug_revoke_old_actions() {
        for replacement in [vec![], vec![usb("USB-A", "android:A", "1", false)], vec![usb("USB-A", "android:B", "1", true)]] {
            let mut policy = UsbNotificationPolicy::default(); policy.observe(vec![], true, token);
            let first = policy.observe(vec![usb("USB-A", "android:A", "1", true)], true, token).pop().unwrap();
            policy.observe(replacement, false, token);
            assert!(policy.claim(&first.token).is_none());
        }
    }
    #[test]
    fn token_failure_and_expiry_do_not_replay_or_start() {
        let mut policy = UsbNotificationPolicy::default(); policy.observe(vec![], true, token);
        let a = usb("USB-A", "android:A", "1", true);
        assert!(policy.observe(vec![a.clone()], true, || None).is_empty());
        assert!(policy.observe(vec![a.clone()], true, token).is_empty());
        policy.observe(vec![], true, token);
        let notice = policy.observe(vec![a], true, token).pop().unwrap();
        policy.pending.get_mut(&notice.token).unwrap().created = Instant::now() - ACTION_LIFETIME;
        assert!(policy.claim(&notice.token).is_none());
    }
}
