use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Idle time in milliseconds via xprintidle (X11). Returns None if unavailable.
pub fn idle_ms() -> Option<u64> {
    let out = Command::new("xprintidle").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim().parse::<u64>().ok()
}

/// Managed idle flags read from the Tauri state.
pub struct IdleFlags {
    pub reset_enabled: AtomicBool,
    pub threshold_minutes: AtomicU32,
}

impl IdleFlags {
    pub fn new(reset: bool, minutes: u32) -> Self {
        Self {
            reset_enabled: AtomicBool::new(reset),
            threshold_minutes: AtomicU32::new(minutes),
        }
    }
    pub fn set_reset(&self, v: bool) {
        self.reset_enabled.store(v, Ordering::Relaxed);
    }
    pub fn load_reset(&self) -> bool {
        self.reset_enabled.load(Ordering::Relaxed)
    }
    pub fn set_threshold(&self, v: u32) {
        self.threshold_minutes.store(v, Ordering::Relaxed);
    }
    pub fn get_threshold(&self) -> u32 {
        self.threshold_minutes.load(Ordering::Relaxed)
    }
}
